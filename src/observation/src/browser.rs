//! Observation Framework — Browser Provider
//!
//! Detects browser context: actual URL, page title, browser type.
//! Focuses on engineering portals (OpenShift, vCenter, Nagios, Checkmk, Grafana).
//!
//! Windows approach: Uses EnumChildWindows + GetWindowTextW to find the address bar
//! in browser process windows. Does NOT use UI Automation (too heavy for polling).
//!
//! Also includes HTML content analysis to detect error pages (500, 502, 503, etc.)
//! by scraping visible text from browser window text.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::event::{EventType, ObservationEvent, ObservationPayload, ProviderType};
use crate::provider::{ObservationProvider, ProviderConfig, ProviderLifecycle, ProviderState};

// ── Engineering portal patterns ─────────────────────────────────────
#[allow(dead_code)]
const ENGINEERING_PORTAL_PATTERNS: &[&str] = &[
    "openshift", "ocp", "okd", "vcenter", "vmware", "vsphere",
    "nagios", "checkmk", "grafana", "prometheus", "kubernetes", "k8s",
    "jenkins", "gitlab", "github", "zabbix", "splunk",
    "elastic", "kibana", "elasticsearch", "redhat", "rhel",
    "ansible", "tower", "satellite",
];

// ── HTML error page patterns ────────────────────────────────────────
#[allow(dead_code)]
const ERROR_PAGE_PATTERNS: &[(&str, &str)] = &[
    ("500", "Internal Server Error"),
    ("502", "Bad Gateway"),
    ("503", "Service Unavailable"),
    ("504", "Gateway Timeout"),
    ("403", "Forbidden"),
    ("404", "Not Found"),
    ("connection refused", "Connection Refused"),
    ("unable to connect", "Unable to Connect"),
    ("dns error", "DNS Error"),
    ("ssl certificate", "SSL Certificate Error"),
    ("maintenance mode", "Site Under Maintenance"),
    ("down for maintenance", "Site Under Maintenance"),
    ("timed out", "Connection Timed Out"),
    ("server unavailable", "Server Unavailable"),
    ("service unavailable", "Service Unavailable"),
    ("internal error", "Internal Error"),
    ("server error", "Server Error"),
    ("error", "Error Detected"),
    ("warning", "Warning Detected"),
    ("alert", "Alert Detected"),
];

// ── Browser context ─────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct BrowserContext {
    pub browser_name: String,
    pub url: Option<String>,
    pub title: Option<String>,
    pub is_engineering_portal: bool,
    pub visible_text: Option<String>,
    pub detected_errors: Vec<BrowserError>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserError {
    pub pattern: String,
    pub description: String,
    pub severity: BrowserErrorSeverity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BrowserErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[allow(dead_code)]
impl BrowserContext {
    #[allow(dead_code)]
    fn from_title(browser: &str, title: &str, url: &str) -> Self {
        let is_engineering = ENGINEERING_PORTAL_PATTERNS.iter().any(|pattern| {
            title.to_lowercase().contains(pattern) || url.to_lowercase().contains(pattern)
        });

        Self {
            browser_name: browser.to_string(),
            url: if url.is_empty() { None } else { Some(url.to_string()) },
            title: if title.is_empty() { None } else { Some(title.to_string()) },
            is_engineering_portal: is_engineering,
            visible_text: None,
            detected_errors: Vec::new(),
        }
    }
}

// ── Browser state ───────────────────────────────────────────────────

pub struct BrowserState {
    pub config: ProviderConfig,
    pub state: ProviderState,
    pub lifecycle: ProviderLifecycle,
    pub last_context: Option<BrowserContext>,
}

impl BrowserState {
    fn new(config: ProviderConfig) -> Self {
        Self {
            config,
            state: ProviderState::Disabled,
            lifecycle: ProviderLifecycle::new(),
            last_context: None,
        }
    }
}

// ── Browser provider ────────────────────────────────────────────────

pub struct BrowserProvider {
    state: Arc<Mutex<BrowserState>>,
}

impl BrowserProvider {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(BrowserState::new(ProviderConfig::default()))),
        }
    }

    fn detect_browser_context(&self) -> Option<BrowserContext> {
        #[cfg(target_os = "windows")]
        {
            use windows::Win32::Foundation::{CloseHandle, HWND};
            use windows::Win32::System::ProcessStatus::GetModuleFileNameExW;
            use windows::Win32::System::Threading::{
                OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ,
            };
            use windows::Win32::UI::WindowsAndMessaging::{
                GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW,
                GetWindowThreadProcessId,
            };

            unsafe {
                let hwnd: HWND = GetForegroundWindow();
                if hwnd.0.is_null() { return None; }

                let len = GetWindowTextLengthW(hwnd);
                if len == 0 { return None; }
                let mut buf = vec![0u16; (len + 1) as usize];
                GetWindowTextW(hwnd, &mut buf);
                let title = String::from_utf16_lossy(&buf[..len as usize]).trim().to_string();
                if title.is_empty() { return None; }

                let mut pid: u32 = 0;
                let _ = GetWindowThreadProcessId(hwnd, Some(&mut pid));
                if pid == 0 { return None; }

                let mut process_name = String::new();
                let handle = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, pid);
                if let Ok(proc_handle) = handle {
                    let mut exe_buf = [0u16; 260];
                    let exe_len = GetModuleFileNameExW(proc_handle, None, &mut exe_buf);
                    let _ = CloseHandle(proc_handle);
                    if exe_len > 0 {
                        let exe_path = String::from_utf16_lossy(&exe_buf[..exe_len as usize]);
                        let path = std::path::Path::new(&exe_path);
                        process_name = path.file_stem()
                            .and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
                    }
                }

                let is_browser = matches!(process_name.as_str(),
                    "firefox" | "firefox-esr" | "chrome" | "chromium" | "msedge"
                    | "brave" | "opera" | "vivaldi" | "safari" | "arc");
                if !is_browser { return None; }

                let url = extract_browser_url(hwnd, &process_name);
                let visible_text = collect_visible_text(hwnd);
                let is_engineering = ENGINEERING_PORTAL_PATTERNS.iter().any(|pattern| {
                    title.to_lowercase().contains(pattern)
                        || url.as_deref().map(|u| u.contains(pattern)).unwrap_or(false)
                        || visible_text.to_lowercase().contains(pattern)
                });
                let detected_errors = analyze_visible_text(&visible_text, url.as_deref());

                Some(BrowserContext {
                    browser_name: process_name,
                    url: url.clone(),
                    title: Some(title),
                    is_engineering_portal: is_engineering,
                    visible_text: if visible_text.is_empty() { None } else { Some(visible_text) },
                    detected_errors,
                })
            }
        }

        #[cfg(not(target_os = "windows"))]
        None
    }

    #[allow(dead_code)]
    fn looks_like_url(s: &str) -> bool {
        s.starts_with("http://") || s.starts_with("https://") || s.contains('.')
    }
}

// ── HTML content analysis ───────────────────────────────────────────

#[allow(dead_code)]
fn analyze_visible_text(visible_text: &str, url: Option<&str>) -> Vec<BrowserError> {
    if visible_text.is_empty() { return Vec::new(); }

    let text_lower = visible_text.to_lowercase();
    let url_lower = url.unwrap_or("").to_lowercase();
    let mut errors = Vec::new();

    for (pattern, description) in ERROR_PAGE_PATTERNS {
        let url_matches = url_lower.contains(*pattern);
        let text_matches = text_lower.contains(*pattern);

        if text_matches || url_matches {
            let severity = if text_lower.contains("503") || url_lower.contains("503") {
                BrowserErrorSeverity::Critical
            } else if text_lower.contains("500") || url_lower.contains("500")
                || text_lower.contains("502") || url_lower.contains("502")
                || text_lower.contains("504") || url_lower.contains("504")
                || text_lower.contains("refused") || text_lower.contains("unable to connect") {
                BrowserErrorSeverity::High
            } else if text_lower.contains("403") || url_lower.contains("403")
                || text_lower.contains("404") || url_lower.contains("404")
                || text_lower.contains("timeout") || text_lower.contains("timed out") {
                BrowserErrorSeverity::Medium
            } else {
                BrowserErrorSeverity::Low
            };

            errors.push(BrowserError {
                pattern: pattern.to_string(),
                description: description.to_string(),
                severity,
            });
        }
    }

    errors.dedup_by(|a, b| a.pattern == b.pattern);
    errors
}

#[cfg(target_os = "windows")]
use windows::Win32::Foundation::{BOOL, FALSE, HWND, LPARAM, TRUE};

// ── Visible text collection ─────────────────────────────────────────

#[cfg(target_os = "windows")]
fn collect_visible_text(hwnd: HWND) -> String {
    use windows::Win32::UI::WindowsAndMessaging::EnumChildWindows;

    let mut children: Vec<HWND> = Vec::new();
    unsafe {
        let _ = EnumChildWindows(hwnd, Some(child_window_text_callback), LPARAM(&mut children as *mut _ as _));
    }

    let mut child_texts: Vec<String> = Vec::new();
    for child in &children {
        if let Some(t) = get_window_text_safe(*child) {
            if !t.is_empty() { child_texts.push(t); }
        }
    }

    let all_text = child_texts.join(" ");
    if all_text.len() > 5000 { all_text[..5000].to_string() } else { all_text }
}

#[cfg(not(target_os = "windows"))]
#[allow(dead_code)]
fn collect_visible_text(_hwnd: isize) -> String {
    String::new()
}

#[cfg(target_os = "windows")]
unsafe extern "system" fn child_window_text_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    if lparam.0 == 0 { return FALSE; }
    let ptr = lparam.0 as *mut Vec<HWND>;
    unsafe { (*ptr).push(hwnd); }
    TRUE
}

#[cfg(target_os = "windows")]
fn get_window_text_safe(hwnd: HWND) -> Option<String> {
    unsafe {
        let len = windows::Win32::UI::WindowsAndMessaging::GetWindowTextLengthW(hwnd);
        if len == 0 { return None; }
        let mut buf = vec![0u16; (len + 1) as usize];
        if windows::Win32::UI::WindowsAndMessaging::GetWindowTextW(hwnd, &mut buf) == 0 { return None; }
        let text = String::from_utf16_lossy(&buf[..len as usize]).trim().to_string();
        if text.is_empty() { return None; }
        Some(text)
    }
}

// ── Browser URL extraction ──────────────────────────────────────────

#[cfg(target_os = "windows")]
mod browser_url_windows {
    use windows::Win32::Foundation::{BOOL, FALSE, HWND, LPARAM, TRUE};
    use windows::Win32::UI::WindowsAndMessaging::{EnumChildWindows, GetClassNameW, GetWindowTextW};

    pub(super) fn extract_browser_url(hwnd: HWND, process_name: &str) -> Option<String> {
        match process_name {
            "firefox" | "firefox-esr" => extract_firefox_url(hwnd),
            _ => extract_chromium_url(hwnd),
        }
    }

    fn extract_chromium_url(hwnd: HWND) -> Option<String> {
        unsafe {
            let mut children: Vec<HWND> = Vec::new();
            let _ = EnumChildWindows(hwnd, Some(callback), LPARAM(&mut children as *mut _ as _));
            if children.is_empty() { return None; }

            for child in &children {
                let cls = get_class_name(*child);
                if cls.contains("Chrome_AutocompleteEditView") || cls.contains("Edit") {
                    if let Some(txt) = get_window_text(*child) {
                        if txt.starts_with("http://") || txt.starts_with("https://") || txt.contains('.') {
                            return Some(txt);
                        }
                    }
                }
            }
            // Check grand-children
            for child in &children {
                let child_class = get_class_name(*child);
                if child_class.contains("Shell Embedding") || child_class.contains("Chrome") {
                    let mut gc: Vec<HWND> = Vec::new();
                    let _ = EnumChildWindows(*child, Some(callback), LPARAM(&mut gc as *mut _ as _));
                    for g in &gc {
                        let gc_cls = get_class_name(*g);
                        if gc_cls.contains("Edit") || gc_cls.contains("Chrome_AutocompleteEditView") {
                            if let Some(txt) = get_window_text(*g) {
                                if txt.starts_with("http://") || txt.starts_with("https://") || txt.contains('.') {
                                    return Some(txt);
                                }
                            }
                        }
                    }
                }
            }
            None
        }
    }

    fn extract_firefox_url(hwnd: HWND) -> Option<String> {
        unsafe {
            let mut children: Vec<HWND> = Vec::new();
            let _ = EnumChildWindows(hwnd, Some(callback), LPARAM(&mut children as *mut _ as _));

            for child in &children {
                let cls = get_class_name(*child);
                if cls.contains("Mozilla") || cls.contains("urlbar") {
                    if let Some(txt) = get_window_text(*child) {
                        if txt.starts_with("http://") || txt.starts_with("https://") || txt.contains('.') {
                            return Some(txt);
                        }
                    }
                }
                let mut gc: Vec<HWND> = Vec::new();
                let _ = EnumChildWindows(*child, Some(callback), LPARAM(&mut gc as *mut _ as _));
                for g in &gc {
                    let gc_cls = get_class_name(*g);
                    if gc_cls.contains("urlbar") || gc_cls.contains("Mozilla") {
                        if let Some(txt) = get_window_text(*g) {
                            if txt.starts_with("http://") || txt.starts_with("https://") || txt.contains('.') {
                                return Some(txt);
                            }
                        }
                    }
                }
            }
            None
        }
    }

    fn get_class_name(hwnd: HWND) -> String {
        unsafe {
            let mut buf = [0u16; 256];
            let len = GetClassNameW(hwnd, &mut buf);
            if len > 0 { String::from_utf16_lossy(&buf[..len as usize]) } else { String::new() }
        }
    }

    fn get_window_text(hwnd: HWND) -> Option<String> {
        unsafe {
            let len = windows::Win32::UI::WindowsAndMessaging::GetWindowTextLengthW(hwnd);
            if len == 0 { return None; }
            let mut buf = vec![0u16; (len + 1) as usize];
            if GetWindowTextW(hwnd, &mut buf) == 0 { return None; }
            let text = String::from_utf16_lossy(&buf[..len as usize]).trim().to_string();
            if text.is_empty() { return None; }
            Some(text)
        }
    }

    unsafe extern "system" fn callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
        if lparam.0 == 0 { return FALSE; }
        let ptr = lparam.0 as *mut Vec<HWND>;
        unsafe { (*ptr).push(hwnd); }
        TRUE
    }
}

#[cfg(target_os = "windows")]
use browser_url_windows::extract_browser_url;

#[cfg(not(target_os = "windows"))]
#[allow(dead_code)]
fn extract_browser_url(_: isize, _: &str) -> Option<String> { None }

// ── ObservationProvider impl ────────────────────────────────────────

impl Default for BrowserProvider {
    fn default() -> Self { Self::new() }
}

#[async_trait]
impl ObservationProvider for BrowserProvider {
    fn provider_type(&self) -> ProviderType { ProviderType::Browser }
    fn name(&self) -> &str { "Browser" }
    fn description(&self) -> &str {
        "Detects browser context: URL, page title, error pages, engineering portal detection"
    }
    fn config(&self) -> ProviderConfig { self.state.lock().unwrap().config.clone() }
    fn set_config(&mut self, config: ProviderConfig) { self.state.lock().unwrap().config = config; }
    fn state(&self) -> ProviderState { self.state.lock().unwrap().state.clone() }

    async fn start(&mut self) -> Result<(), String> {
        let mut state = self.state.lock().unwrap();
        state.lifecycle.start();
        state.state = ProviderState::Active;
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), String> {
        let mut state = self.state.lock().unwrap();
        state.lifecycle.stop();
        state.state = ProviderState::Disabled;
        Ok(())
    }

    async fn pause(&mut self) -> Result<(), String> {
        let mut state = self.state.lock().unwrap();
        if matches!(state.state, ProviderState::Active) {
            state.state = ProviderState::Paused;
            Ok(())
        } else {
            Err("Provider is not currently active".to_string())
        }
    }

    async fn resume(&mut self) -> Result<(), String> {
        let mut state = self.state.lock().unwrap();
        if matches!(state.state, ProviderState::Paused) {
            state.state = ProviderState::Active;
            Ok(())
        } else {
            Err("Provider is not currently paused".to_string())
        }
    }

    async fn observe(&self) -> Result<Vec<ObservationEvent>, String> {
        let mut state = self.state.lock().unwrap();

        match self.detect_browser_context() {
            Some(context) => {
                let changed = state.last_context.as_ref().map(|prev| {
                    prev.browser_name != context.browser_name
                        || prev.url != context.url
                        || prev.title != context.title
                        || prev.detected_errors != context.detected_errors
                }).unwrap_or(true);

                state.last_context = Some(context.clone());

                if !changed { return Ok(Vec::new()); }

                let payload = serde_json::json!({
                    "browser": context.browser_name,
                    "url": context.url,
                    "title": context.title,
                    "is_engineering_portal": context.is_engineering_portal,
                    "detected_errors": context.detected_errors.iter().map(|e| {
                        serde_json::json!({"pattern": e.pattern, "description": e.description, "severity": format!("{:?}", e.severity)})
                    }).collect::<Vec<_>>(),
                    "visible_text_length": context.visible_text.as_ref().map(|t| t.len()),
                });

                Ok(vec![ObservationEvent::new(
                    EventType::BrowserContextChanged,
                    ProviderType::Browser,
                    context.browser_name.clone(),
                    None,
                    ObservationPayload::new(payload),
                )])
            }
            None => Ok(vec![ObservationEvent::new(
                EventType::BrowserContextChanged,
                ProviderType::Browser,
                "inactive".to_string(),
                None,
                ObservationPayload::new(serde_json::json!({"status": "no_browser_context_detected", "platform": std::env::consts::OS})),
            )]),
        }
    }

    fn lifecycle(&self) -> crate::provider::ProviderLifecycle {
        self.state.lock().unwrap().lifecycle.clone()
    }

    fn status_details(&self) -> HashMap<String, serde_json::Value> {
        let state = self.state.lock().unwrap();
        let mut details = HashMap::new();
        if let Some(ref ctx) = state.last_context {
            details.insert("last_browser".to_string(), serde_json::json!(ctx.browser_name));
            details.insert("last_url".to_string(), serde_json::json!(ctx.url));
            details.insert("is_portal".to_string(), serde_json::json!(ctx.is_engineering_portal));
            details.insert("errors_detected".to_string(), serde_json::json!(ctx.detected_errors.len()));
        }
        details.insert("platform".to_string(), serde_json::json!(std::env::consts::OS));
        details
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_browser_provider_creation() {
        let provider = BrowserProvider::new();
        assert_eq!(provider.provider_type(), ProviderType::Browser);
        assert_eq!(provider.name(), "Browser");
    }

    #[test]
    fn test_error_detection_500() {
        let errors = analyze_visible_text("500 Internal Server Error", Some("https://nagios/"));
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.pattern == "500"));
    }

    #[test]
    fn test_error_detection_503() {
        let errors = analyze_visible_text("Service Unavailable", Some("https://app/503"));
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.severity == BrowserErrorSeverity::Critical));
    }

    #[test]
    fn test_no_errors_clean_page() {
        let errors = analyze_visible_text("Welcome to Grafana Dashboard", Some("https://grafana/dashboard"));
        assert!(!errors.iter().any(|e| e.severity == BrowserErrorSeverity::Critical));
    }
}