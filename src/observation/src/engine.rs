//! AI Engine Integration — Observation Context Provider
//!
//! Translates observation events into structured context for the AI engine.
//! This module does NOT analyze content — it provides metadata-based context.
//!
//! Supported event types for AI context:
//! - ApplicationChanged → Current application context
//! - BrowserContextChanged → Browser context (URL, title)
//! - TerminalCommand → Active terminal context
//! - ClipboardChanged → Recent clipboard type (error, stack trace, log)
//! - ConfigurationFileOpened → Active config files
//! - ScreenCapture → Screen capture metadata

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::event::{ObservationEvent, EventType, ProviderType};

/// A structured context entry for the AI engine.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ContextEntry {
    /// The observation event type that generated this context.
    pub event_type: EventType,
    /// The provider that generated this context.
    pub provider: ProviderType,
    /// A summary of the current context (no raw content).
    pub summary: String,
    /// Metadata from the observation event.
    pub metadata: HashMap<String, serde_json::Value>,
    /// When this context was generated.
    pub timestamp: String,
}

/// AI context that is sent to the engine alongside LLM prompts.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AiContext {
    /// List of active applications.
    pub applications: Vec<String>,
    /// Current browser context (if any).
    pub browser_context: Option<BrowserContextSummary>,
    /// Current terminal context (if any).
    pub terminal_context: Option<TerminalContextSummary>,
    /// Recently opened configuration files.
    pub config_files: Vec<ConfigFileSummary>,
    /// Recent clipboard indicators (not content).
    pub clipboard_indicators: Vec<ClipboardIndicator>,
    /// Screen capture metadata.
    pub screen_capture: Option<ScreenCaptureMetadata>,
    /// Total number of observations since session start.
    pub total_observations: u64,
    /// Timestamp when this context was generated.
    pub generated_at: String,
}

/// Browser context summary for AI.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BrowserContextSummary {
    pub browser: String,
    pub url: Option<String>,
    pub title: Option<String>,
    pub is_engineering_portal: bool,
}

/// Terminal context summary for AI.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TerminalContextSummary {
    pub terminal: String,
    pub shell: String,
    pub session_id: String,
    pub is_ssh: bool,
    pub is_engineering: bool,
}

/// Config file summary for AI.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ConfigFileSummary {
    pub path: String,
    pub extension: String,
    pub size_bytes: Option<u64>,
}

/// Clipboard indicator for AI.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ClipboardIndicator {
    pub looks_like_error: bool,
    pub looks_like_stack_trace: bool,
    pub looks_like_log: bool,
    pub text_length: u64,
}

/// Screen capture metadata for AI.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ScreenCaptureMetadata {
    pub width: u32,
    pub height: u32,
    pub screen_index: u32,
    pub total_screens: u32,
}

/// AI engine integration context manager.
///
/// Translates observation events into structured context entries
/// that can be injected into LLM prompts.
pub struct AiContextManager {
    /// Latest context snapshot.
    pub latest_context: Arc<Mutex<AiContext>>,
    /// Number of observations processed.
    pub observation_count: Arc<Mutex<u64>>,
    /// Recently seen provider types (for deduplication).
    pub recent_providers: Arc<Mutex<Vec<ProviderType>>>,
}

impl AiContextManager {
    pub fn new() -> Self {
        Self {
            latest_context: Arc::new(Mutex::new(AiContext::default())),
            observation_count: Arc::new(Mutex::new(0)),
            recent_providers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Process an observation event and update context if relevant.
    pub fn process_event(&self, event: &ObservationEvent) {
        // Increment observation count
        let mut count = self.observation_count.lock().unwrap();
        *count += 1;

        // Skip non-relevant events
        let relevant = matches!(
            event.event_type,
            EventType::ApplicationChanged
                | EventType::BrowserContextChanged
                | EventType::TerminalCommand
                | EventType::ClipboardChanged
                | EventType::ConfigurationFileOpened
                | EventType::ScreenshotCaptured
        );

        if !relevant {
            return;
        }

        // Track which provider types we've seen
        let mut recent = self.recent_providers.lock().unwrap();
        if !recent.contains(&event.provider) {
            recent.push(event.provider.clone());
            // Keep only last 100 unique providers
            if recent.len() > 100 {
                recent.remove(0);
            }
        }

        // Update context based on provider type
        let mut ctx = self.latest_context.lock().unwrap();
        match &event.provider {
            ProviderType::ActiveWindow => {
                let app = event.source.clone();
                if !ctx.applications.contains(&app) {
                    ctx.applications.push(app);
                }
            }
            ProviderType::Browser => {
                if let Some(url) = event.payload.data.get("url").and_then(|v| v.as_str()) {
                    ctx.browser_context = Some(BrowserContextSummary {
                        browser: event.source.clone(),
                        url: Some(url.to_string()),
                        title: event.payload.data.get("title").and_then(|v| v.as_str()).map(|s| s.to_string()),
                        is_engineering_portal: event.payload.data.get("is_engineering_portal").and_then(|v| v.as_bool()).unwrap_or(false),
                    });
                }
            }
            ProviderType::Terminal => {
                if let Some(session) = event.payload.data.get("session_id").and_then(|v| v.as_str()) {
                    ctx.terminal_context = Some(TerminalContextSummary {
                        terminal: event.source.clone(),
                        shell: event.payload.data.get("shell").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        session_id: session.to_string(),
                        is_ssh: event.payload.data.get("is_ssh").and_then(|v| v.as_bool()).unwrap_or(false),
                        is_engineering: event.payload.data.get("is_engineering").and_then(|v| v.as_bool()).unwrap_or(false),
                    });
                }
            }
            ProviderType::Clipboard => {
                if let (Some(error), Some(trace), Some(log), Some(len)) = (
                    event.payload.data.get("looks_like_error").and_then(|v| v.as_bool()),
                    event.payload.data.get("looks_like_stack_trace").and_then(|v| v.as_bool()),
                    event.payload.data.get("looks_like_log").and_then(|v| v.as_bool()),
                    event.payload.data.get("text_length").and_then(|v| v.as_u64()),
                ) {
                    ctx.clipboard_indicators.push(ClipboardIndicator {
                        looks_like_error: error,
                        looks_like_stack_trace: trace,
                        looks_like_log: log,
                        text_length: len,
                    });
                    // Keep only last 10 indicators
                    if ctx.clipboard_indicators.len() > 10 {
                        ctx.clipboard_indicators.remove(0);
                    }
                }
            }
            ProviderType::FileObserver => {
                if let (Some(path), Some(ext), Some(size)) = (
                    event.payload.data.get("path").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    event.payload.data.get("extension").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    event.payload.data.get("size_bytes").and_then(|v| v.as_u64()),
                ) {
                    ctx.config_files.push(ConfigFileSummary { path, extension: ext, size_bytes: Some(size) });
                    if ctx.config_files.len() > 10 {
                        ctx.config_files.remove(0);
                    }
                }
            }
            ProviderType::ScreenCapture => {
                if let (Some(w), Some(h), Some(s), Some(t)) = (
                    event.payload.data.get("width").and_then(|v| v.as_u64()).map(|v| v as u32),
                    event.payload.data.get("height").and_then(|v| v.as_u64()).map(|v| v as u32),
                    event.payload.data.get("screen_index").and_then(|v| v.as_u64()).map(|v| v as u32),
                    event.payload.data.get("total_screens").and_then(|v| v.as_u64()).map(|v| v as u32),
                ) {
                    ctx.screen_capture = Some(ScreenCaptureMetadata { width: w, height: h, screen_index: s, total_screens: t });
                }
            }
            _ => {}
        }

        ctx.total_observations = *count;
    }

    /// Get the current AI context.
    pub fn get_context(&self) -> AiContext {
        self.latest_context.lock().unwrap().clone()
    }

    /// Get the observation count.
    pub fn get_observation_count(&self) -> u64 {
        *self.observation_count.lock().unwrap()
    }

    /// Get the list of unique providers seen.
    pub fn get_unique_providers(&self) -> Vec<ProviderType> {
        self.recent_providers.lock().unwrap().clone()
    }
}

impl Default for AiContextManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for AiContext {
    fn default() -> Self {
        Self {
            applications: Vec::new(),
            browser_context: None,
            terminal_context: None,
            config_files: Vec::new(),
            clipboard_indicators: Vec::new(),
            screen_capture: None,
            total_observations: 0,
            generated_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// Format the AI context as a human-readable summary for prompts.
pub fn format_context_for_prompt(ctx: &AiContext) -> String {
    let mut parts: Vec<String> = Vec::new();

    if !ctx.applications.is_empty() {
        parts.push(format!("Current applications: {}", ctx.applications.join(", ")));
    }

    if let Some(ref browser) = ctx.browser_context {
        if browser.is_engineering_portal {
            parts.push(format!(
                "Engineering portal: {} - {} ({})",
                browser.browser,
                browser.url.as_deref().unwrap_or("unknown"),
                browser.title.as_deref().unwrap_or("no title")
            ));
        }
    }

    if let Some(ref terminal) = ctx.terminal_context {
        if terminal.is_engineering {
            parts.push(format!(
                "Engineering terminal: {} (shell: {})",
                terminal.terminal, terminal.shell
            ));
        }
    }

    if !ctx.config_files.is_empty() {
        let files: Vec<String> = ctx
            .config_files
            .iter()
            .filter_map(|f| {
                if f.size_bytes.unwrap_or(0) > 0 {
                    Some(f.path.clone())
                } else {
                    None
                }
            })
            .collect();
        if !files.is_empty() {
            parts.push(format!("Active config files: {}", files.join(", ")));
        }
    }

    if !ctx.clipboard_indicators.is_empty() {
        let errors: u32 = ctx
            .clipboard_indicators
            .iter()
            .filter(|c| c.looks_like_error)
            .count() as u32;
        let traces: u32 = ctx
            .clipboard_indicators
            .iter()
            .filter(|c| c.looks_like_stack_trace)
            .count() as u32;
        if errors > 0 || traces > 0 {
            parts.push(format!("Clipboard: {} errors, {} stack traces detected", errors, traces));
        }
    }

    if parts.is_empty() {
        parts.push("No relevant observation context available".to_string());
    }

    format!(
        "## Observation Context\n{}\n\nObservations processed: {}",
        parts.join("\n"),
        ctx.total_observations
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::ObservationPayload;

    #[test]
    fn test_context_manager_creation() {
        let manager = AiContextManager::new();
        assert_eq!(manager.get_observation_count(), 0);
        assert!(manager.get_unique_providers().is_empty());
    }

    #[test]
    fn test_process_application_event() {
        let manager = AiContextManager::new();
        let event = ObservationEvent::new(
            EventType::ApplicationChanged,
            ProviderType::ActiveWindow,
            "vscode".to_string(),
            None,
            ObservationPayload::new(serde_json::json!({"app": "vscode"})),
        );
        manager.process_event(&event);
        assert_eq!(manager.get_observation_count(), 1);

        let ctx = manager.get_context();
        assert!(ctx.applications.contains(&"vscode".to_string()));
    }

    #[test]
    fn test_process_browser_event() {
        let manager = AiContextManager::new();
        let event = ObservationEvent::new(
            EventType::BrowserContextChanged,
            ProviderType::Browser,
            "firefox".to_string(),
            None,
            ObservationPayload::new(serde_json::json!({
                "url": "https://openshift.example.com",
                "title": "OpenShift Console",
                "is_engineering_portal": true,
            })),
        );
        manager.process_event(&event);

        let ctx = manager.get_context();
        assert!(ctx.browser_context.is_some());
        let browser = ctx.browser_context.unwrap();
        assert!(browser.is_engineering_portal);
        assert!(browser.url.as_ref().unwrap().contains("openshift"));
    }

    #[test]
    fn test_process_clipboard_error_event() {
        let manager = AiContextManager::new();
        let event = ObservationEvent::new(
            EventType::ClipboardChanged,
            ProviderType::Clipboard,
            "clipboard".to_string(),
            None,
            ObservationPayload::new(serde_json::json!({
                "looks_like_error": true,
                "looks_like_stack_trace": false,
                "looks_like_log": false,
                "text_length": 256,
            })),
        );
        manager.process_event(&event);

        let ctx = manager.get_context();
        assert!(!ctx.clipboard_indicators.is_empty());
        assert!(ctx.clipboard_indicators[0].looks_like_error);
    }

    #[test]
    fn test_format_context_for_prompt() {
        let manager = AiContextManager::new();
        manager.process_event(&ObservationEvent::new(
            EventType::ApplicationChanged,
            ProviderType::ActiveWindow,
            "vscode".to_string(),
            None,
            ObservationPayload::new(serde_json::json!({})),
        ));

        let ctx = manager.get_context();
        let formatted = format_context_for_prompt(&ctx);
        assert!(formatted.contains("vscode"));
        assert!(formatted.contains("Observations processed: 1"));
    }

    #[test]
    fn test_format_empty_context() {
        let manager = AiContextManager::new();
        let ctx = manager.get_context();
        let formatted = format_context_for_prompt(&ctx);
        assert!(formatted.contains("No relevant observation context available"));
    }

    #[test]
    fn test_deduplication() {
        let manager = AiContextManager::new();
        // Send multiple events from the same provider
        for _ in 0..5 {
            manager.process_event(&ObservationEvent::new(
                EventType::ApplicationChanged,
                ProviderType::ActiveWindow,
                "vscode".to_string(),
                None,
                ObservationPayload::new(serde_json::json!({})),
            ));
        }
        assert_eq!(manager.get_observation_count(), 5);
        // Only one unique provider
        assert_eq!(manager.get_unique_providers().len(), 1);
    }

    #[test]
    fn test_clipboard_deduplication() {
        let manager = AiContextManager::new();
        // Send 15 identical clipboard events
        for i in 0..15 {
            manager.process_event(&ObservationEvent::new(
                EventType::ClipboardChanged,
                ProviderType::Clipboard,
                "clipboard".to_string(),
                None,
                ObservationPayload::new(serde_json::json!({
                    "looks_like_error": true,
                    "looks_like_stack_trace": false,
                    "looks_like_log": false,
                    "text_length": i,
                })),
            ));
        }

        let ctx = manager.get_context();
        // Should only keep last 10
        assert!(ctx.clipboard_indicators.len() <= 10);
    }
}