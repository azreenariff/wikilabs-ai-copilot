//! Vision Analysis Provider
//!
//! Takes screenshots from the screen capture buffer and sends them to a Vision AI model
//! (GPT-4o, Claude, Gemini) via an OpenRouter-compatible API.
//! Returns structured analysis: what's on screen, what the user is doing, any errors,
//! and the user's likely intent.
//!
//! This provider receives screenshot data through the observe() call which is triggered
//! by the engine's polling loop. The screen_capture provider emits ScreenshotCaptured events
//! with the screenshot data; the engine's feed_event() routes them. The vision analyzer
//! stores the latest screenshot and analyzes it on its next observe() cycle.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::event::{EventType, ObservationEvent, ObservationPayload, ProviderType};
use crate::provider::{ObservationProvider, ProviderConfig, ProviderLifecycle, ProviderState};

/// Configuration for the vision analyzer.
#[derive(Debug, Clone)]
pub struct VisionAnalyzerConfig {
    /// Vision model name (e.g., "anthropic/claude-sonnet-4", "openai/gpt-4o").
    pub model: String,
    /// API endpoint (OpenRouter or direct).
    pub endpoint: String,
    /// API key for the vision provider.
    pub api_key: String,
    /// Poll interval in seconds (minimum time between Vision API calls).
    pub poll_interval_secs: u64,
    /// System prompt for the vision analysis.
    pub system_prompt: Option<String>,
    /// Maximum tokens for the Vision response.
    pub max_tokens: u32,
    /// Temperature for the Vision model.
    pub temperature: f32,
}

impl Default for VisionAnalyzerConfig {
    fn default() -> Self {
        Self {
            model: "anthropic/claude-sonnet-4".to_string(),
            endpoint: "https://openrouter.ai/api/v1".to_string(),
            api_key: String::new(),
            poll_interval_secs: 30, // Every 30s to control costs
            system_prompt: None,
            max_tokens: 1000,
            temperature: 0.3,
        }
    }
}

/// Result of a Vision AI analysis.
#[derive(Debug, Clone)]
pub struct VisionAnalysisResult {
    /// Timestamp of analysis.
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// What application/process is in focus.
    pub focused_app: Option<String>,
    /// What the user appears to be doing.
    pub user_activity: Option<String>,
    /// Any errors or problems detected on screen.
    pub errors_detected: Vec<VisionError>,
    /// The user's likely intent/goal.
    pub inferred_intent: Option<String>,
    /// Any suggestions or guidance the Vision model provided.
    pub suggestions: Vec<String>,
    /// Raw text response from the Vision model for debugging.
    pub raw_analysis: String,
    /// Confidence in the analysis (0.0-1.0).
    pub confidence: f32,
}

/// An error detected by the Vision model.
#[derive(Debug, Clone)]
pub struct VisionError {
    pub description: String,
    pub severity: String, // "low", "medium", "high", "critical"
}

/// Vision analyzer provider state.
pub struct VisionAnalyzerState {
    pub config: ProviderConfig,
    pub vision_config: VisionAnalyzerConfig,
    pub state: ProviderState,
    pub lifecycle: ProviderLifecycle,
    pub last_analysis: Option<VisionAnalysisResult>,
    pub analysis_count: u64,
    pub last_analysis_time: Option<chrono::DateTime<chrono::Utc>>,
    pub last_error: Option<String>,
    /// Latest screenshot data (base64 PNG), width, height, focused window.
    /// Set by the engine when a ScreenshotCaptured event is observed.
    pub pending_screenshot: Option<(String, u32, u32, String)>,
    /// Whether a ScreenshotCaptured event has been queued since last analyze.
    pub has_pending_screenshot: bool,
}

impl VisionAnalyzerState {
    fn new(config: ProviderConfig, vision_config: VisionAnalyzerConfig) -> Self {
        Self {
            config,
            vision_config,
            state: ProviderState::Disabled,
            lifecycle: ProviderLifecycle::new(),
            last_analysis: None,
            analysis_count: 0,
            last_analysis_time: None,
            last_error: None,
            pending_screenshot: None,
            has_pending_screenshot: false,
        }
    }

    /// Queue a new screenshot for analysis. Called by the engine when
    /// a ScreenshotCaptured event is received.
    pub fn queue_screenshot(&mut self, data_base64: String, width: u32, height: u32, focused_window: String) {
        self.pending_screenshot = Some((data_base64, width, height, focused_window));
        self.has_pending_screenshot = true;
    }
}

/// External queue interface for the vision analyzer state.
pub struct VisionAnalyzerQueue(pub Arc<Mutex<VisionAnalyzerState>>);

impl VisionAnalyzerQueue {
    pub fn queue_screenshot(&self, data_base64: String, width: u32, height: u32, focused_window: String) {
        let mut state = self.0.lock().unwrap();
        state.pending_screenshot = Some((data_base64, width, height, focused_window));
        state.has_pending_screenshot = true;
    }
}

/// Vision analysis provider.
pub struct VisionAnalyzerProvider {
    state: Arc<Mutex<VisionAnalyzerState>>,
}

impl VisionAnalyzerProvider {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(VisionAnalyzerState::new(
                ProviderConfig::default(),
                VisionAnalyzerConfig::default(),
            ))),
        }
    }

    pub fn with_config(config: ProviderConfig, vision_config: VisionAnalyzerConfig) -> Self {
        Self {
            state: Arc::new(Mutex::new(VisionAnalyzerState::new(
                config,
                vision_config,
            ))),
        }
    }

    /// Queue a screenshot for analysis from external code (engine).
    pub fn queue_screenshot(&self, data_base64: String, width: u32, height: u32, focused_window: String) {
        self.state.lock().unwrap().queue_screenshot(data_base64, width, height, focused_window);
    }

    /// Analyze a screenshot using the Vision AI model.
    async fn analyze_screenshot(&self, screenshot_base64: &str, _width: u32, _height: u32, focused_window: &str) -> Option<VisionAnalysisResult> {
        let config = self.state.lock().unwrap().vision_config.clone();

        if config.api_key.is_empty() {
            tracing::warn!("[Vision] No API key configured");
            return None;
        }

        if config.endpoint.is_empty() || config.model.is_empty() {
            tracing::warn!("[Vision] Endpoint or model not configured");
            return None;
        }

        // Rate limiting
        let now = chrono::Utc::now();
        {
            let state = self.state.lock().unwrap();
            if let Some(last_time) = state.last_analysis_time {
                let elapsed = (now - last_time).num_seconds();
                if elapsed < config.poll_interval_secs as i64 {
                    tracing::debug!(
                        "[Vision] Rate limited — {}s since last analysis (interval: {}s)",
                        elapsed,
                        config.poll_interval_secs
                    );
                    return None;
                }
            }
        }

        let system_prompt = config.system_prompt.as_deref().unwrap_or(DEFAULT_VISION_PROMPT);

        // Build the message with the screenshot
        let content = vec![
            serde_json::json!({
                "type": "text",
                "text": format!("{}\n\nFocused window: {}", system_prompt, focused_window)
            }),
            serde_json::json!({
                "type": "image_url",
                "image_url": {
                    "url": format!("data:image/png;base64,{}", screenshot_base64),
                }
            }),
        ];

        let body = serde_json::json!({
            "model": config.model,
            "messages": [
                {
                    "role": "user",
                    "content": content
                }
            ],
            "max_tokens": config.max_tokens,
            "temperature": config.temperature,
        });

        // Send to the Vision API
        let client = reqwest::Client::new();
        let response = match client
            .post(&config.endpoint)
            .header("Authorization", format!("Bearer {}", config.api_key))
            .header("Content-Type", "application/json")
            .header("HTTP-Referer", "https://wikilabs.ai")
            .header("X-Title", "Wiki Labs AI Copilot")
            .json(&body)
            .send()
            .await
        {
            Ok(resp) => resp,
            Err(e) => {
                tracing::error!("[Vision] Failed to send request: {}", e);
                return None;
            }
        };

        if !response.status().is_success() {
            let status = response.status();
            let body_text = response.text().await.unwrap_or_default();
            tracing::error!("[Vision] API error: {} — {}", status, body_text.chars().take(200).collect::<String>());
            return None;
        }

        let api_response: serde_json::Value = match response.json().await {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("[Vision] Failed to parse response: {}", e);
                return None;
            }
        };

        // Extract the response text
        let response_text = api_response
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .unwrap_or("");

        // Parse the analysis result from the Vision model response
        let analysis = parse_vision_response(response_text, focused_window);

        Some(analysis)
    }
}

impl Default for VisionAnalyzerProvider {
    fn default() -> Self { Self::new() }
}

#[async_trait]
impl ObservationProvider for VisionAnalyzerProvider {
    fn provider_type(&self) -> ProviderType { ProviderType::VisionAnalysis }
    fn name(&self) -> &str { "VisionAnalyzer" }
    fn description(&self) -> &str {
        "Analyzes screenshots using Vision AI (Claude Sonnet 4) for full visual awareness and intent inference"
    }
    fn config(&self) -> ProviderConfig { self.state.lock().unwrap().config.clone() }
    fn set_config(&mut self, config: ProviderConfig) { self.state.lock().unwrap().config = config; }
    fn state(&self) -> ProviderState { self.state.lock().unwrap().state.clone() }

    async fn start(&mut self) -> Result<(), String> {
        let mut state = self.state.lock().unwrap();
        state.lifecycle.start();
        state.state = ProviderState::Active;
        tracing::info!("[Vision] VisionAnalyzer provider started");
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), String> {
        let mut state = self.state.lock().unwrap();
        state.lifecycle.stop();
        state.state = ProviderState::Disabled;
        tracing::info!("[Vision] VisionAnalyzer provider stopped");
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
        // Check if we have a pending screenshot to analyze
        let pending = {
            let state = self.state.lock().unwrap();
            if !state.has_pending_screenshot || state.pending_screenshot.is_none() {
                return Ok(Vec::new());
            }
            state.pending_screenshot.as_ref().unwrap().clone()
        };

        let (screenshot_data, width, height, focused_window) = pending.clone();

        // Analyze the screenshot
        match self.analyze_screenshot(&screenshot_data, width, height, &focused_window).await {
            Some(analysis) => {
                // Update state
                {
                    let mut state = self.state.lock().unwrap();
                    state.last_analysis = Some(analysis.clone());
                    state.analysis_count += 1;
                    state.last_analysis_time = Some(chrono::Utc::now());
                    state.has_pending_screenshot = false;
                    state.pending_screenshot = None;
                }

                tracing::info!(
                    "[Vision] Analysis complete — confidence: {:.2}, errors: {}, suggestions: {}",
                    analysis.confidence,
                    analysis.errors_detected.len(),
                    analysis.suggestions.len()
                );

                // Emit VisionAnalysisResult event
                Ok(vec![ObservationEvent::new(
                    EventType::VisionAnalysisResult,
                    ProviderType::VisionAnalysis,
                    focused_window.clone(),
                    None,
                    ObservationPayload::new(serde_json::json!({
                        "focused_app": analysis.focused_app,
                        "user_activity": analysis.user_activity,
                        "errors_detected": analysis.errors_detected.iter().map(|e| {
                            serde_json::json!({"description": e.description, "severity": e.severity})
                        }).collect::<Vec<_>>(),
                        "inferred_intent": analysis.inferred_intent,
                        "suggestions": analysis.suggestions,
                        "confidence": analysis.confidence,
                        "raw_analysis": analysis.raw_analysis,
                        "analysis_count": self.state.lock().unwrap().analysis_count,
                    })),
                )])
            }
            None => {
                // No analysis produced (rate limited, API error, etc.)
                // Clear the pending screenshot so we don't keep trying
                {
                    let mut state = self.state.lock().unwrap();
                    state.has_pending_screenshot = false;
                    state.pending_screenshot = None;
                }
                // Don't emit an event — just return empty
                Ok(Vec::new())
            }
        }
    }

    fn lifecycle(&self) -> crate::provider::ProviderLifecycle {
        self.state.lock().unwrap().lifecycle.clone()
    }

    fn status_details(&self) -> HashMap<String, serde_json::Value> {
        let state = self.state.lock().unwrap();
        let mut details = HashMap::new();
        details.insert("analysis_count".to_string(), serde_json::json!(state.analysis_count));
        details.insert("poll_interval_secs".to_string(), serde_json::json!(state.vision_config.poll_interval_secs));
        details.insert("model".to_string(), serde_json::json!(state.vision_config.model));
        details.insert("api_key_configured".to_string(), serde_json::json!(!state.vision_config.api_key.is_empty()));
        details.insert("has_pending_screenshot".to_string(), serde_json::json!(state.has_pending_screenshot));
        if let Some(ref a) = state.last_analysis {
            details.insert("last_analysis_timestamp".to_string(), serde_json::json!(a.timestamp.to_rfc3339()));
            details.insert("last_confidence".to_string(), serde_json::json!(a.confidence));
            details.insert("errors_found".to_string(), serde_json::json!(a.errors_detected.len()));
        }
        if let Some(ref e) = state.last_error {
            details.insert("last_error".to_string(), serde_json::json!(e.chars().take(200).collect::<String>()));
        }
        details
    }

    fn as_any(&self) -> &dyn std::any::Any { self }
}

/// Parse the Vision model's response into structured data.
fn parse_vision_response(response: &str, focused_window: &str) -> VisionAnalysisResult {
    let raw_analysis = response.to_string();

    let mut errors_detected = Vec::new();
    let mut suggestions = Vec::new();

    let lower = response.to_lowercase();

    // Detect errors from response
    let error_patterns = [
        ("error", "Error detected"),
        ("warning", "Warning detected"),
        ("fail", "Failure detected"),
        ("crash", "Crash detected"),
        ("exception", "Exception detected"),
        ("500", "HTTP 500 error"),
        ("502", "HTTP 502 Bad Gateway"),
        ("503", "HTTP 503 Service Unavailable"),
        ("404", "HTTP 404 Not Found"),
        ("timeout", "Timeout detected"),
        ("out of memory", "Out of memory condition"),
        ("circuit breaker", "Circuit breaker triggered"),
        ("connection refused", "Connection refused"),
        ("disk full", "Disk full"),
        ("fatal", "Fatal error"),
    ];

    for (pattern, description) in &error_patterns {
        if lower.contains(pattern) {
            errors_detected.push(VisionError {
                description: description.to_string(),
                severity: "high".to_string(),
            });
        }
    }

    // Extract suggestions (look for sentences with "should", "try", "check", "verify")
    let sentences: Vec<&str> = response.split(|c: char| c.is_ascii_punctuation()).collect();
    for sentence in &sentences {
        let s_lower = sentence.to_lowercase();
        if s_lower.contains("should") || s_lower.contains("try") || s_lower.contains("check")
            || s_lower.contains("verify") || s_lower.contains("consider") || s_lower.contains("run") {
            let trimmed = sentence.trim();
            if !trimmed.is_empty() && trimmed.len() > 5 {
                suggestions.push(trimmed.to_string());
            }
        }
    }

    // Infer user activity from focused window and response content
    let user_activity = if errors_detected.is_empty() {
        Some(format!("User is interacting with {}", focused_window))
    } else {
        Some(format!("User appears to be troubleshooting issues in {}", focused_window))
    };

    let inferred_intent = if errors_detected.is_empty() {
        Some(format!("Viewing or working in {}", focused_window))
    } else {
        Some("Troubleshooting — user is experiencing errors or issues".to_string())
    };

    let confidence = if !errors_detected.is_empty() { 0.85 } else { 0.75 };

    VisionAnalysisResult {
        timestamp: chrono::Utc::now(),
        focused_app: Some(focused_window.to_string()),
        user_activity,
        errors_detected,
        inferred_intent,
        suggestions,
        raw_analysis,
        confidence,
    }
}

/// Default prompt for Vision AI analysis.
const DEFAULT_VISION_PROMPT: &str = r#"You are an AI copilot observing a user's screen. Analyze this screenshot and tell me:

1. What application is in focus?
2. What is the user likely doing RIGHT NOW based ONLY on what is visible in this screenshot?
3. Are there any errors, warnings, or problems visible on the screen?
4. What is the user's likely intent or goal?
5. Are there any specific, actionable suggestions you would give the user?

CRITICAL RULES — follow ALL:
- ONLY describe what you can SEE in this screenshot. Do NOT guess about past activity.
- Do NOT reference things the user may have done in other windows or previous sessions.
- If you cannot see something in the screenshot, do NOT claim to see it. Do NOT hallucinate.
- If the user is doing something wrong, point it out based on what is visible.
- Keep it concise (3-5 sentences).

Be conversational — like a helpful teammate sitting next to the user."#;

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vision_analyzer_creation() {
        let provider = VisionAnalyzerProvider::new();
        assert_eq!(provider.provider_type(), ProviderType::VisionAnalysis);
        assert_eq!(provider.name(), "VisionAnalyzer");
    }

    #[test]
    fn test_parse_vision_response_with_error() {
        let response = "I see the user is looking at a Nagios page. There's a red 'Database Error' message on the screen. The user is troubleshooting a database connection issue. They should check MySQL status with `systemctl status mysqld`.";
        let result = parse_vision_response(response, "Nagios — Database Error");
        assert!(!result.errors_detected.is_empty());
        assert!(result.raw_analysis.contains("Database Error"));
    }

    #[test]
    fn test_parse_vision_response_clean() {
        let response = "The user is looking at a Grafana dashboard. Everything looks healthy. They're monitoring system metrics.";
        let result = parse_vision_response(response, "Grafana — Dashboards");
        assert!(result.errors_detected.is_empty());
        assert!(result.user_activity.is_some());
    }

    #[test]
    fn test_parse_vision_response_suggestions() {
        let response = "The user is working on deployment in Jenkins. They should verify the pipeline configuration before running. Try checking the build logs first.";
        let result = parse_vision_response(response, "Jenkins — Pipeline");
        assert!(!result.suggestions.is_empty());
    }

    #[test]
    fn test_provider_lifecycle() {
        use crate::provider::ProviderState;
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut provider = VisionAnalyzerProvider::new();
        assert_eq!(provider.state(), ProviderState::Disabled);

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
    fn test_queue_screenshot() {
        let state = Arc::new(Mutex::new(VisionAnalyzerState::new(
            ProviderConfig::default(),
            VisionAnalyzerConfig::default(),
        )));

        {
            let mut s = state.lock().unwrap();
            assert!(!s.has_pending_screenshot);

            s.queue_screenshot(
                "dGVzdA==".to_string(), // "test" in base64
                1920, 1080,
                "TestApp".to_string(),
            );

            assert!(s.has_pending_screenshot);
            assert!(s.pending_screenshot.is_some());
            let (data, w, h, fw) = s.pending_screenshot.as_ref().unwrap();
            assert_eq!(data, "dGVzdA==");
            assert_eq!(*w, 1920);
            assert_eq!(*h, 1080);
            assert_eq!(fw, "TestApp");
        }
    }
}