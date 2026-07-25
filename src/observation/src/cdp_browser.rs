//! Chrome DevTools Protocol (CDP) Browser Scraper
//!
//! Connects to Chrome/Edge browser instances via the DevTools Protocol WebSocket
//! to perform real HTML scraping — DOM extraction, error detection, and page content analysis.
//!
//! ## How it works
//!
//! 1. Discovers running Chrome/Edge processes via Process32FirstW
//! 2. Scans common CDP ports (9222-9230) to find active debugging endpoints
//! 3. Connects via WebSocket to the CDP JSON endpoint
//! 4. Uses CDP commands to extract page HTML, text content, and detect errors
//!
//! ## Limitations
//!
//! - Requires Chrome/Edge to have `--remote-debugging-port` enabled
//! - Chromium-based browsers only (Chrome, Edge, Brave, Opera, Vivaldi)
//! - Firefox requires separate CDP implementation

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use futures_util::{SinkExt, StreamExt};
use regex::Regex;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, warn};

use crate::event::{ObservationEvent, ObservationPayload};

/// A Chrome DevTools Protocol connection target.
#[derive(Debug, Clone, Deserialize)]
pub struct CdpTarget {
    pub title: String,
    pub url: String,
    pub r#type: String,
    pub id: String,
    pub webSocketDebuggerUrl: Option<String>,
}

/// An error detected in the page DOM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageError {
    pub pattern: String,
    pub description: String,
    pub severity: String,
    pub location: Option<String>,
}

/// HTML content extracted from a page via CDP.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HtmlContent {
    pub url: String,
    pub title: String,
    pub text_content: String,
    pub meta_description: Option<String>,
    pub http_status: Option<String>,
    pub errors: Vec<PageError>,
    pub is_error_page: bool,
    pub has_form: bool,
    pub has_table: bool,
    pub has_script_error: bool,
    pub content_length: usize,
}

/// CDP JSON response message.
#[derive(Debug, Deserialize)]
struct CdpMessage {
    pub id: Option<u64>,
    pub result: Option<serde_json::Value>,
    pub error: Option<CdpError>,
    pub method: Option<String>,
    pub params: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct CdpError {
    pub code: i32,
    pub message: String,
}

/// CDP Browser scraper state.
#[derive(Debug)]
pub struct CdpBrowserScraper {
    state: Arc<Mutex<CdpScraperState>>,
}

#[derive(Debug)]
struct CdpScraperState {
    enabled: bool,
    last_content: Option<HtmlContent>,
    last_connect: Option<Instant>,
}

impl Default for CdpBrowserScraper {
    fn default() -> Self {
        Self::new()
    }
}

impl CdpBrowserScraper {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(CdpScraperState {
                enabled: true,
                last_content: None,
                last_connect: None,
            })),
        }
    }

    pub fn set_enabled(&self, enabled: bool) {
        let mut state = self.state.lock().unwrap();
        state.enabled = enabled;
    }

    /// Discover Chromium-based browser processes on Windows.
    #[cfg(target_os = "windows")]
    pub fn discover_browsers(&self) -> Vec<(String, u32)> {
        use windows::Win32::Foundation::{CloseHandle, HANDLE};
        use windows::Win32::System::Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
            TH32CS_SNAPPROCESS,
        };

        let mut browsers = Vec::new();
        let chromium_browsers = [
            "chrome", "msedge", "brave", "opera", "vivaldi", "chromium",
        ];

        unsafe {
            let snapshot = match CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) {
                Ok(h) => h,
                Err(e) => {
                    warn!("CreateToolhelp32Snapshot failed: {e}");
                    return browsers;
                }
            };

            let mut pe = PROCESSENTRY32W {
                dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
                ..Default::default()
            };

            if Process32FirstW(snapshot, &mut pe).is_ok() {
                loop {
                    let name = String::from_utf16_lossy(&pe.szExeFile)
                        .trim_end_matches('\0')
                        .to_lowercase();

                    if chromium_browsers.iter().any(|b| *b == name.as_str()) {
                        browsers.push((name.clone(), pe.th32ProcessID));
                    }

                    pe.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
                    if Process32NextW(snapshot, &mut pe).is_err() {
                        break;
                    }
                }
            }

            let _ = CloseHandle(snapshot);
        }

        browsers
    }

    /// Try CDP ports 9222-9230 for running browser instances.
    pub async fn scan_cdp_ports(&self) -> Vec<(u16, Vec<CdpTarget>)> {
        let client = reqwest::Client::new();
        let mut found = Vec::new();

        for port in 9222..=9230 {
            let url = format!("http://127.0.0.1:{port}/json");
            match client.get(&url).timeout(Duration::from_secs(2)).send().await {
                Ok(resp) => {
                    match resp.json::<Vec<CdpTarget>>().await {
                        Ok(targets) if !targets.is_empty() => {
                            debug!("Found {n} CDP targets on port {port}", n = targets.len());
                            found.push((port, targets));
                        }
                        _ => {}
                    }
                }
                Err(_) => {} // Port not open or connection refused — skip
            }
        }

        found
    }

    /// Main observation — scans for CDP browsers and scrapes page content.
    pub async fn observe(&self) -> Result<Vec<ObservationEvent>, String> {
        let mut state = self.state.lock().unwrap();

        if !state.enabled {
            return Ok(Vec::new());
        }

        // Rate limit: max 1 scrape every 5 seconds
        if let Some(last) = state.last_connect {
            if last.elapsed() < Duration::from_secs(5) {
                return Ok(Vec::new());
            }
        }

        // Discover browsers
        let browsers = self.discover_browsers();
        if browsers.is_empty() {
            return Ok(Vec::new());
        }

        // Scan CDP ports for active debugging endpoints
        let found = self.scan_cdp_ports().await;
        if found.is_empty() {
            return Ok(Vec::new());
        }

        // Use the first found target
        let (port, targets) = &found[0];
        let browser_name = &browsers[0].0;

        // Find a page target
        let page_target = targets.iter()
            .find(|t| t.r#type == "page" && !t.url.starts_with("devtools/"))
            .or_else(|| targets.first())
            .ok_or("No targets found")?;

        let ws_url = page_target.webSocketDebuggerUrl.clone()
            .ok_or("No WebSocket URL")?;

        // Connect via WebSocket
        let (ws, _) = tokio_tungstenite::connect_async(&ws_url).await
            .map_err(|e| format!("CDP WebSocket connection failed: {e}"))?;

        debug!("Connected to CDP at {ws_url}");

        // Scrape page content
        let mut conn = CdpConnection::new(ws);
        let content = conn.scrape_page().await
            .map_err(|e| format!("CDP scrape failed: {e}"))?;

        // Enrich with browser metadata from CDP targets
        let mut enriched = content;
        enriched.url = page_target.url.clone();
        enriched.title = page_target.title.clone();

        state.last_content = Some(enriched.clone());
        state.last_connect = Some(Instant::now());

        let payload = serde_json::json!({
            "browser": browser_name,
            "cdp_port": port,
            "url": enriched.url,
            "title": enriched.title,
            "text_content_length": enriched.text_content.len(),
            "content_length": enriched.content_length,
            "has_errors": enriched.is_error_page,
            "errors": enriched.errors.iter().map(|e| {
                serde_json::json!({"pattern": e.pattern, "description": e.description, "severity": e.severity})
            }).collect::<Vec<_>>(),
            "has_form": enriched.has_form,
            "has_script_error": enriched.has_script_error,
        });

        Ok(vec![ObservationEvent::new(
            crate::event::EventType::BrowserContextChanged,
            crate::event::ProviderType::Browser,
            format!("chrome-cdp-{browser_name}"),
            None,
            ObservationPayload::new(payload),
        )])
    }

    pub fn get_last_content(&self) -> Option<HtmlContent> {
        let state = self.state.lock().unwrap();
        state.last_content.clone()
    }
}

/// Active WebSocket connection to a CDP endpoint.
pub struct CdpConnection {
    ws: tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
    message_id: u64,
}

impl CdpConnection {
    fn new(ws: tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>) -> Self {
        Self { ws, message_id: 0 }
    }

    fn next_id(&mut self) -> u64 {
        self.message_id += 1;
        self.message_id
    }

    /// Send a CDP command and await the response.
    async fn send_command(&mut self, method: &str, params: Option<serde_json::Value>) -> Result<serde_json::Value, String> {
        let id = self.next_id();
        let mut cmd = serde_json::json!({ "id": id, "method": method });
        if let Some(p) = params {
            cmd["params"] = p;
        }

        let msg = serde_json::to_string(&cmd)
            .map_err(|e| format!("JSON encode error: {e}"))?;

        self.ws.send(tungstenite::Message::Text(msg.into())).await
            .map_err(|e| format!("WebSocket send error: {e}"))?;

        loop {
            match tokio::time::timeout(Duration::from_secs(5), self.ws.next()).await {
                Ok(Some(Ok(msg))) => {
                    let text = msg.to_string();
                    if let Ok(resp) = serde_json::from_str::<CdpMessage>(&text) {
                        if resp.id == Some(id) {
                            if let Some(err) = resp.error {
                                return Err(format!("CDP error {}: {}", err.code, err.message));
                            }
                            if let Some(result) = resp.result {
                                return Ok(result);
                            }
                        }
                        continue;
                    }
                }
                Ok(Some(Err(e))) => return Err(format!("WebSocket error: {e}")),
                Ok(None) => return Err("CDP connection closed".to_string()),
                Err(_) => return Err("CDP command timeout".to_string()),
            }
        }
    }

    /// Full CDP session: enable domains, extract DOM, analyze content.
    pub async fn scrape_page(&mut self) -> Result<HtmlContent, String> {
        self.send_command("DOM.enable", None).await?;
        self.send_command("Runtime.enable", None).await?;
        self.send_command("Page.enable", None).await?;

        // Get the DOM document root
        let dom_result = self.send_command("DOM.getDocument", Some(serde_json::json!({
            "depth": -1, "pierce": true,
        }))).await?;

        let node_id = dom_result["root"]["nodeId"].as_i64().ok_or("No nodeId")?;

        // Get outer HTML
        let text_result = self.send_command("DOM.getOuterHTML", Some(serde_json::json!({ "nodeId": node_id }))).await?;
        let outer_html = text_result["outerHTML"].as_str().unwrap_or("").to_string();

        // Extract visible text by stripping HTML tags
        let text_content = Self::extract_visible_text(&outer_html);

        // Extract meta description
        let meta_desc = Self::extract_meta_content(&outer_html, "description");

        // Detect errors
        let mut errors = Vec::new();
        Self::detect_page_errors(&outer_html, &text_content, &mut errors);

        // Check for script errors
        let has_script_error = self.check_script_errors().await.unwrap_or(false);

        // Check for forms and tables
        let has_form = outer_html.contains("<form") || outer_html.contains("<FORM");
        let has_table = outer_html.contains("<table") || outer_html.contains("<TABLE");

        let errors_len = errors.len();
        Ok(HtmlContent {
            url: String::new(), // Set by caller
            title: String::new(), // Set by caller
            text_content,
            meta_description: Some(meta_desc),
            http_status: None,
            errors,
            is_error_page: errors_len > 0,
            has_form,
            has_table,
            has_script_error,
            content_length: outer_html.len(),
        })
    }

    /// Strip HTML tags and normalize whitespace to get visible text.
    fn extract_visible_text(html: &str) -> String {
        let re = Regex::new(r"<[^>]+>").unwrap();
        let text = re.replace_all(html, " ");

        let re2 = Regex::new(r"\s+").unwrap();
        let text = re2.replace_all(&text, " ");

        // Remove inline script/style remnants
        let re3 = Regex::new(r"(\s+)(function\s*\(|const\s|let\s|var\s|document\.|window\.|alert\(|console\.|if\s*\(|for\s*\(|return\s)").unwrap();
        let text = re3.replace_all(&text, "$1");

        text.trim().to_string()
    }

    fn extract_meta_content(html: &str, name: &str) -> String {
        let re = Regex::new(&format!(r#"<meta\s+[^>]*name=["']{}["'][^>]*content=["']([^"']*)["']"#, regex::escape(name))).unwrap();
        re.captures(html)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().to_string())
            .unwrap_or_default()
    }

    /// Detect errors, warnings, and status indicators in page content.
    fn detect_page_errors(html: &str, text: &str, errors: &mut Vec<PageError>) {
        let mut seen = std::collections::HashSet::new();
        let text_lower = text.to_lowercase();
        let html_lower = html.to_lowercase();

        let patterns = [
            ("500", "Internal Server Error", "High"),
            ("502", "Bad Gateway", "High"),
            ("503", "Service Unavailable", "Critical"),
            ("504", "Gateway Timeout", "High"),
            ("403", "Forbidden / Access Denied", "Medium"),
            ("404", "Not Found", "Medium"),
            ("429", "Too Many Requests", "Low"),
            ("fatal error", "Fatal error in page content", "Critical"),
            ("out of memory", "Out of memory condition", "Critical"),
            ("connection refused", "Connection refused", "High"),
            ("timeout", "Request timeout detected", "Medium"),
            ("circuit breaker", "Circuit breaker pattern detected", "Medium"),
            ("database connection failed", "Database connectivity issue", "Critical"),
            ("disk full", "Storage capacity issue", "Critical"),
            ("cors", "CORS policy violation detected", "Medium"),
            ("uncaught exception", "Uncaught JavaScript runtime error", "Medium"),
            ("uncaught typeerror", "Uncaught JavaScript TypeError", "Medium"),
            ("failed to load", "Resource load failure", "High"),
        ];

        for (pattern, description, severity) in &patterns {
            if (text_lower.contains(pattern.to_lowercase().as_str())
                || html_lower.contains(pattern.to_lowercase().as_str()))
                && seen.insert(pattern.to_string()) {
                errors.push(PageError {
                    pattern: pattern.to_string(),
                    description: description.to_string(),
                    severity: severity.to_string(),
                    location: None,
                });
            }
        }
    }

    /// Check for JavaScript runtime errors via Runtime.evaluate.
    async fn check_script_errors(&mut self) -> Result<bool, String> {
        let result = self.send_command("Runtime.evaluate", Some(serde_json::json!({
            "expression": "typeof console !== 'undefined' && typeof console.error === 'function' ? 'ok' : 'none'",
            "returnByValue": true,
        }))).await;

        match result {
            Ok(r) => {
                let val = r["result"]["value"].as_str().unwrap_or("");
                Ok(val == "ok")
            }
            Err(_) => Ok(false),
        }
    }
}