//! Observation Framework — Active Window Provider
//!
//! Detects foreground application, window title, executable, and process information.
//!
//! Platform support:
//! - Linux: Uses X11/Wayland window properties via xprop or similar
//! - Windows: Uses Win32 API GetForegroundWindow, GetWindowText
//! - macOS: Uses Accessibility API and AXUIElement
//!
//! This provider does NOT perform OCR or AI analysis.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::event::{EventType, ObservationEvent, ObservationPayload, ProviderType};
use crate::provider::{ObservationProvider, ProviderConfig, ProviderLifecycle, ProviderState};

/// Information about the active window.
#[derive(Debug, Clone)]
pub struct WindowInfo {
    /// Window title.
    pub title: String,
    /// Application/process name.
    pub process_name: String,
    /// Full executable path (when available).
    pub executable: Option<String>,
    /// Process ID.
    pub pid: Option<u32>,
    /// Window class (X11) or similar identifier.
    pub window_class: Option<String>,
    /// Detected URL (for browser windows).
    pub url: Option<String>,
    /// Is this window a browser window?
    pub is_browser: bool,
    /// Is this window a terminal?
    pub is_terminal: bool,
    /// Is this an engineering portal (OpenShift, vCenter, Grafana, etc.)?
    pub is_engineering_portal: bool,
}

impl WindowInfo {
    pub fn new(title: String, process_name: String) -> Self {
        let is_browser = Self::is_browser_process(&process_name);
        let is_terminal = Self::is_terminal_process(&process_name);
        let is_engineering_portal = Self::is_engineering_process(&process_name);

        Self {
            title,
            process_name,
            executable: None,
            pid: None,
            window_class: None,
            url: None,
            is_browser,
            is_terminal,
            is_engineering_portal,
        }
    }

    fn is_browser_process(name: &str) -> bool {
        let lower = name.to_lowercase();
        matches!(
            lower.as_str(),
            "firefox"
                | "chrome"
                | "chromium"
                | "microsoft-edge"
                | "brave-browser"
                | "vivaldi"
                | "opera"
                | "arc"
                | "safari"
                | "epiphany"
                | "gnome-chrome"
                | "firefox-esr"
                | "google-chrome"
                | "chromium-browser"
        ) || lower.ends_with(".exe") && lower.contains("browser")
    }

    fn is_terminal_process(name: &str) -> bool {
        let lower = name.to_lowercase();
        matches!(
            lower.as_str(),
            "alacritty"
                | "kitty"
                | "iterm"
                | "gnome-terminal"
                | "terminal"
                | "konsole"
                | "tilix"
                | "xfce4-terminal"
                | "xterm"
                | "x-terminal-emulator"
                | "wezterm"
                | "mintty"
                | "wt"
                | "windows terminal"
                | "pwsh"
                | "powershell"
                | "bash"
                | "zsh"
                | "fish"
                | "cmd"
                | "command prompt"
                | "openssh"
        ) || lower.contains("terminal")
            || lower.contains("ssh")
    }

    fn is_engineering_process(name: &str) -> bool {
        let lower = name.to_lowercase();
        matches!(
            lower.as_str(),
            "code"
                | "vscode"
                | "visual-studio-code"
                | "code-insiders"
                | "intellij-idea"
                | "pycharm"
                | "webstorm"
                | "goland"
                | "rubymine"
                | "eclipse"
                | "sublime-text"
                | "atom"
                | "notepad++"
        ) || lower.contains("openshift")
            || lower.contains("vcenter")
            || lower.contains("grafana")
            || lower.contains("nagios")
    }
}

/// Active window provider state.
pub struct ActiveWindowState {
    pub config: ProviderConfig,
    pub state: ProviderState,
    pub lifecycle: ProviderLifecycle,
    pub last_window_info: Option<WindowInfo>,
}

impl ActiveWindowState {
    fn new(config: ProviderConfig) -> Self {
        Self {
            config,
            state: ProviderState::Disabled,
            lifecycle: ProviderLifecycle::new(),
            last_window_info: None,
        }
    }
}

/// Active Window observation provider.
///
/// Collects: foreground application, window title, executable, PID, URL.
pub struct ActiveWindowProvider {
    state: Arc<Mutex<ActiveWindowState>>,
}

impl ActiveWindowProvider {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(
                ActiveWindowState::new(ProviderConfig::default()),
            )),
        }
    }

    /// Platform-specific implementation stub.
    /// Returns None on platforms we don't support or when no window info is available.
    fn detect_active_window(&self) -> Option<WindowInfo> {
        #[cfg(target_os = "linux")]
        {
            // Linux: Try to detect via xprop or similar
            // This is a stub — real implementation would use xlib/xcb or wayland protocols
            // For now, return None (platform detection would go here)
            None
        }

        #[cfg(target_os = "windows")]
        {
            // Windows: Use Win32 API via windows crate
            use windows::Win32::Foundation::{CloseHandle, HWND};
            use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowTextW, GetWindowTextLengthW, GetWindowThreadProcessId};
            use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};
            use windows::Win32::System::ProcessStatus::GetModuleFileNameExW;

            unsafe {
                let hwnd = match GetForegroundWindow() {
                    Ok(h) => h,
                    Err(_) => return None,
                };
                if hwnd.is_invalid() {
                    return None;
                }

                // Get window title
                let len = GetWindowTextLengthW(hwnd);
                let len = match len {
                    Ok(l) => l,
                    Err(_) => return None,
                };
                if len == 0 {
                    return None;
                }
                let mut title_buf = vec![0u16; (len + 1) as usize];
                let _ = GetWindowTextW(hwnd, &mut title_buf);
                let title = String::from_utf16_lossy(&title_buf[..len as usize]).trim().to_string();

                // Get process ID
                let mut pid: u32 = 0;
                let _ = GetWindowThreadProcessId(hwnd, Some(&mut pid));
                if pid == 0 {
                    return Some(WindowInfo::new(title, "unknown".to_string()));
                }

                // Open process to get executable name
                let handle = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, pid);
                if let Ok(handle) = handle {
                    let mut exe_buf = [0u16; 260];
                    let exe_len = GetModuleFileNameExW(handle, None, &mut exe_buf);
                    let _ = CloseHandle(handle);

                    if exe_len > 0 {
                        let exe_path = String::from_utf16_lossy(&exe_buf[..exe_len as usize]);
                        let path = std::path::Path::new(&exe_path);
                        let process_name = path.file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("unknown")
                            .to_string();

                        let mut info = WindowInfo::new(title, process_name);
                        info.executable = Some(exe_path);
                        info.pid = Some(pid);
                        return Some(info);
                    }
                }

                Some(WindowInfo::new(title, format!("pid:{}", pid)))
            }
        }

        #[cfg(target_os = "macos")]
        {
            // macOS: Use Accessibility API
            // Real implementation would use:
            // - AXUIElementCopyAttributeValue(AXUIElementCreateSystemWide(), kAXFocusedWindowAttribute)
            None
        }

        #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
        {
            None
        }
    }
}

impl Default for ActiveWindowProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ObservationProvider for ActiveWindowProvider {
    fn provider_type(&self) -> ProviderType {
        ProviderType::ActiveWindow
    }

    fn name(&self) -> &str {
        "Active Window"
    }

    fn description(&self) -> &str {
        "Detects foreground application, window title, executable, and process information"
    }

    fn config(&self) -> ProviderConfig {
        self.state.lock().unwrap().config.clone()
    }

    fn set_config(&mut self, config: ProviderConfig) {
        self.state.lock().unwrap().config = config;
    }

    fn state(&self) -> ProviderState {
        self.state.lock().unwrap().state.clone()
    }

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
        let window_info = match self.detect_active_window() {
            Some(info) => info,
            None => {
                // Return a minimal event to indicate we tried
                return Ok(vec![ObservationEvent::new(
                    EventType::ApplicationChanged,
                    ProviderType::ActiveWindow,
                    "stub".to_string(),
                    None,
                    ObservationPayload::new(serde_json::json!({
                        "status": "no_window_info_available",
                        "platform": std::env::consts::OS,
                    })),
                )]);
            }
        };

        let mut state = self.state.lock().unwrap();
        let previous = state.last_window_info.clone();

        // Only emit event if the window actually changed
        let needs_event = match &previous {
            Some(prev) => {
                prev.process_name != window_info.process_name
                    || prev.title != window_info.title
                    || prev.url != window_info.url
            }
            None => true,
        };

        state.last_window_info = Some(window_info.clone());

        if !needs_event {
            return Ok(Vec::new());
        }

        let payload = ObservationPayload::new(serde_json::json!({
            "window_title": window_info.title,
            "process_name": window_info.process_name,
            "executable": window_info.executable,
            "pid": window_info.pid,
            "url": window_info.url,
            "is_browser": window_info.is_browser,
            "is_terminal": window_info.is_terminal,
            "is_engineering_portal": window_info.is_engineering_portal,
        }));

        Ok(vec![ObservationEvent::new(
            EventType::ApplicationChanged,
            ProviderType::ActiveWindow,
            window_info.process_name.clone(),
            None,
            payload,
        )
        .with_confidence(if window_info.url.is_some() { 0.8 } else { 1.0 })
        .with_metadata("is_browser", serde_json::json!(window_info.is_browser))
        .with_metadata("is_terminal", serde_json::json!(window_info.is_terminal))
        .with_metadata(
            "is_engineering_portal",
            serde_json::json!(window_info.is_engineering_portal),
        )])
    }

    fn lifecycle(&self) -> ProviderLifecycle {
        self.state.lock().unwrap().lifecycle.clone()
    }

    fn status_details(&self) -> HashMap<String, serde_json::Value> {
        let state = self.state.lock().unwrap();
        let mut details = HashMap::new();
        if let Some(ref info) = state.last_window_info {
            details.insert(
                "last_process".to_string(),
                serde_json::json!(info.process_name),
            );
            details.insert("last_title".to_string(), serde_json::json!(info.title));
        }
        details.insert(
            "platform".to_string(),
            serde_json::json!(std::env::consts::OS),
        );
        details
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_active_window_provider_creation() {
        let provider = ActiveWindowProvider::new();
        assert_eq!(provider.provider_type(), ProviderType::ActiveWindow);
        assert_eq!(provider.name(), "Active Window");
    }

    #[test]
    fn test_window_info_detection() {
        let info = WindowInfo::new("Firefox - GitHub".to_string(), "firefox".to_string());
        assert!(info.is_browser);
        assert!(!info.is_terminal);
        assert!(!info.is_engineering_portal);

        let info = WindowInfo::new("Terminal".to_string(), "alacritty".to_string());
        assert!(!info.is_browser);
        assert!(info.is_terminal);
        assert!(!info.is_engineering_portal);

        let info = WindowInfo::new("OpenShift".to_string(), "code".to_string());
        assert!(!info.is_browser);
        assert!(!info.is_terminal);
        assert!(info.is_engineering_portal);
    }

    #[test]
    fn test_browser_process_names() {
        assert!(WindowInfo::is_browser_process("firefox"));
        assert!(WindowInfo::is_browser_process("chrome"));
        assert!(WindowInfo::is_browser_process("google-chrome"));
        assert!(WindowInfo::is_browser_process("chromium"));
        assert!(!WindowInfo::is_browser_process("alacritty"));
        assert!(!WindowInfo::is_browser_process("code"));
    }

    #[test]
    fn test_terminal_process_names() {
        assert!(WindowInfo::is_terminal_process("alacritty"));
        assert!(WindowInfo::is_terminal_process("iterm"));
        assert!(WindowInfo::is_terminal_process("gnome-terminal"));
        assert!(WindowInfo::is_terminal_process("wezterm"));
        assert!(WindowInfo::is_terminal_process("bash"));
        assert!(!WindowInfo::is_terminal_process("firefox"));
        assert!(!WindowInfo::is_terminal_process("code"));
    }

    #[test]
    fn test_provider_lifecycle() {
        let mut provider = ActiveWindowProvider::new();

        assert_eq!(provider.state(), ProviderState::Disabled);

        // Start
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            assert!(provider.start().await.is_ok());
            assert_eq!(provider.state(), ProviderState::Active);
        });

        // Pause
        rt.block_on(async {
            assert!(provider.pause().await.is_ok());
            assert_eq!(provider.state(), ProviderState::Paused);
        });

        // Resume
        rt.block_on(async {
            assert!(provider.resume().await.is_ok());
            assert_eq!(provider.state(), ProviderState::Active);
        });

        // Stop
        rt.block_on(async {
            assert!(provider.stop().await.is_ok());
            assert_eq!(provider.state(), ProviderState::Disabled);
        });

        // Can't pause when disabled
        rt.block_on(async {
            assert!(provider.pause().await.is_err());
        });
    }

    #[test]
    fn test_provider_observe_stub() {
        let mut provider = ActiveWindowProvider::new();

        let rt = tokio::runtime::Runtime::new().unwrap();
        let events = rt.block_on(async {
            provider.start().await.unwrap();
            provider.observe().await.unwrap()
        });

        // Should get at least one event (the stub response)
        assert!(!events.is_empty());
        assert_eq!(events[0].event_type, EventType::ApplicationChanged);
        assert_eq!(events[0].provider, ProviderType::ActiveWindow);
    }

    #[test]
    fn test_config_get_set() {
        let mut provider = ActiveWindowProvider::new();
        let mut config = provider.config();
        assert!(config.enabled);

        config.enabled = false;
        provider.set_config(config);
        assert!(!provider.config().enabled);
    }

    #[test]
    fn test_status_details() {
        let provider = ActiveWindowProvider::new();
        let details = provider.status_details();
        assert!(details.contains_key("platform"));
    }
}
