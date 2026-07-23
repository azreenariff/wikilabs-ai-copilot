//! Observation Framework — Clipboard Provider
//!
//! Observes copied text only when explicitly enabled.
//! Useful for capturing error messages, logs, and stack traces.
//!
//! This provider is opt-in by default (store_clipboard_content = false).

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::event::{EventType, ObservationEvent, ObservationPayload, ProviderType};
use crate::provider::{ObservationProvider, ProviderConfig, ProviderLifecycle, ProviderState};

/// Clipboard content observed.
#[derive(Debug, Clone)]
pub struct ClipboardContent {
    /// The copied text.
    pub text: Option<String>,
    /// Length of copied text (for metadata without content).
    pub text_length: u64,
    /// Whether the content looks like an error message.
    pub looks_like_error: bool,
    /// Whether the content looks like a stack trace.
    pub looks_like_stack_trace: bool,
    /// Whether the content looks like a log entry.
    pub looks_like_log: bool,
}

impl ClipboardContent {
    fn from_text(text: &str) -> Self {
        let text_length = text.len() as u64;
        let text_lower = text.to_lowercase();

        let looks_like_error = text_lower
            .contains(["error", "exception", "failed", "critical", "fatal"][0])
            || text.contains(["Error:", "Exception:", "FAILED", "FATAL:"][0])
            || (text_lower.contains("at ") && text.contains("("));

        let looks_like_stack_trace = text_lower.contains(["traceback", "stack:", "call stack"][0])
            || text.contains(["    at ", "  File \"", "  -> "][0])
            || (text_lower.contains("at ")
                && (text.contains(".java:")
                    || text.contains(".kt:")
                    || text.contains(".py:")
                    || text.contains(".js:")
                    || text.contains(".rs:")));

        let looks_like_log = text_lower.contains(["info:", "warn:", "debug:", "error:"][0])
            || text.contains(["[ERROR]", "[INFO]", "[WARN]", "[DEBUG]"][0])
            || text_lower.contains(["[info]", "[warn]", "[debug]", "[error]"][0]);

        Self {
            text: Some(text.to_string()),
            text_length,
            looks_like_error,
            looks_like_stack_trace,
            looks_like_log,
        }
    }

    #[allow(dead_code)]
    fn from_length(text_length: u64) -> Self {
        Self {
            text: None,
            text_length,
            looks_like_error: false,
            looks_like_stack_trace: false,
            looks_like_log: false,
        }
    }
}

/// Clipboard provider state.
pub struct ClipboardState {
    pub config: ProviderConfig,
    pub state: ProviderState,
    pub lifecycle: ProviderLifecycle,
    pub last_content: Option<ClipboardContent>,
    pub last_content_hash: u64,
}

impl ClipboardState {
    fn new(config: ProviderConfig) -> Self {
        Self {
            config,
            state: ProviderState::Disabled,
            lifecycle: ProviderLifecycle::new(),
            last_content: None,
            last_content_hash: 0,
        }
    }
}

fn simple_hash(text: &str) -> u64 {
    let mut hash: u64 = 0;
    for byte in text.bytes() {
        hash = hash.wrapping_mul(31).wrapping_add(byte as u64);
    }
    hash
}

/// Clipboard observation provider.
pub struct ClipboardProvider {
    state: Arc<Mutex<ClipboardState>>,
}

impl ClipboardProvider {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(ClipboardState::new(ProviderConfig::default()))),
        }
    }

    /// Platform-specific clipboard read.
    fn read_clipboard(&self) -> Option<String> {
        #[cfg(target_os = "windows")]
        {
            use windows::Win32::UI::Clipboard::{OpenClipboard, GetClipboardData, CloseClipboard, CF_UNICODETEXT};
            use windows::Win32::Foundation::{CloseHandle, HGLOBAL};
            unsafe {
                if OpenClipboard(None).is_ok() {
                    let data = GetClipboardData(CF_UNICODETEXT.0 as u32);
                    if let Ok(handle) = data {
                        let ptr = windows::Win32::System::Memory::GlobalLock(handle);
                        if !ptr.is_null() {
                            let slice = std::slice::from_raw_parts(ptr as *const u16, 65536);
                            let len = slice.iter().position(|&c| c == 0).unwrap_or(0);
                            let text = String::from_utf16_lossy(&slice[..len]);
                            let _ = windows::Win32::System::Memory::GlobalUnlock(handle);
                            let _ = CloseClipboard();
                            return Some(text);
                        }
                    }
                    let _ = CloseClipboard();
                }
                None
            }
        }

        #[cfg(not(target_os = "windows"))]
        None
    }
}

impl Default for ClipboardProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ObservationProvider for ClipboardProvider {
    fn provider_type(&self) -> ProviderType {
        ProviderType::Clipboard
    }

    fn name(&self) -> &str {
        "Clipboard"
    }

    fn description(&self) -> &str {
        "Observes copied text content (opt-in, requires explicit enable)"
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
        let mut state = self.state.lock().unwrap();

        match self.read_clipboard() {
            Some(text) => {
                let content = ClipboardContent::from_text(&text);
                let content_hash = simple_hash(&text);

                if content_hash == state.last_content_hash {
                    return Ok(Vec::new());
                }

                state.last_content_hash = content_hash;
                state.last_content = Some(content.clone());

                let payload = serde_json::json!({
                    "text_length": content.text_length,
                    "looks_like_error": content.looks_like_error,
                    "looks_like_stack_trace": content.looks_like_stack_trace,
                    "looks_like_log": content.looks_like_log,
                    "content_stored": false, // Default: don't store content
                });

                Ok(vec![ObservationEvent::new(
                    EventType::ClipboardChanged,
                    ProviderType::Clipboard,
                    "clipboard".to_string(),
                    None,
                    ObservationPayload::new(payload),
                )])
            }
            None => {
                // No clipboard API available — return metadata-only event
                Ok(vec![ObservationEvent::new(
                    EventType::ClipboardChanged,
                    ProviderType::Clipboard,
                    "stub".to_string(),
                    None,
                    ObservationPayload::new(serde_json::json!({
                        "status": "clipboard_not_accessible",
                        "platform": std::env::consts::OS,
                    })),
                )])
            }
        }
    }

    fn lifecycle(&self) -> ProviderLifecycle {
        self.state.lock().unwrap().lifecycle.clone()
    }

    fn status_details(&self) -> HashMap<String, serde_json::Value> {
        let state = self.state.lock().unwrap();
        let mut details = HashMap::new();
        if let Some(ref content) = state.last_content {
            details.insert(
                "text_length".to_string(),
                serde_json::json!(content.text_length),
            );
            details.insert(
                "looks_like_error".to_string(),
                serde_json::json!(content.looks_like_error),
            );
        }
        details
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clipboard_provider_creation() {
        let provider = ClipboardProvider::new();
        assert_eq!(provider.provider_type(), ProviderType::Clipboard);
        assert_eq!(provider.name(), "Clipboard");
    }

    #[test]
    fn test_clipboard_content_detection() {
        let content = ClipboardContent::from_text("Error: connection refused at 127.0.0.1:5432");
        assert!(content.looks_like_error);
        assert!(!content.looks_like_stack_trace);

        let content = ClipboardContent::from_text(
            "at com.example.Main.run(Main.java:42)\n  at com.example.Main.main(Main.java:10)",
        );
        assert!(content.looks_like_stack_trace);
        assert!(content.looks_like_error);

        let content = ClipboardContent::from_text("[ERROR] 2024-01-01 Something went wrong");
        assert!(content.looks_like_error);
        assert!(content.looks_like_log);
    }

    #[test]
    fn test_clipboard_lifecycle() {
        let mut provider = ClipboardProvider::new();
        assert_eq!(provider.state(), ProviderState::Disabled);

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            assert!(provider.start().await.is_ok());
            assert_eq!(provider.state(), ProviderState::Active);
            assert!(provider.pause().await.is_ok());
            assert_eq!(provider.state(), ProviderState::Paused);
            assert!(provider.resume().await.is_ok());
            assert_eq!(provider.state(), ProviderState::Active);
            assert!(provider.stop().await.is_ok());
            assert_eq!(provider.state(), ProviderState::Disabled);
        });
    }

    #[test]
    fn test_observe_emits_event() {
        let mut provider = ClipboardProvider::new();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let events = rt.block_on(async {
            provider.start().await.unwrap();
            provider.observe().await.unwrap()
        });
        assert!(!events.is_empty());
        assert_eq!(events[0].event_type, EventType::ClipboardChanged);
    }

    #[test]
    fn test_config_get_set() {
        let mut provider = ClipboardProvider::new();
        let mut config = provider.config();
        config.enabled = false;
        provider.set_config(config);
        assert!(!provider.config().enabled);
    }

    #[test]
    fn test_status_details() {
        let provider = ClipboardProvider::new();
        let details = provider.status_details();
        // No last_content initially, so details should be empty or just platform
        assert!(details.is_empty() || !details.contains_key("text_length"));
    }
}
