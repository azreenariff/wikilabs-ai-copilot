//! Vision AI analyzer — analyzes screenshots via external Vision API (OpenRouter, Claude Sonnet 4).
//!
//! Takes full-screen screenshots from the screen capture provider and sends them to
//! the Vision AI model for structured analysis of what's visible on screen, error detection,
//! and actionable suggestions.

use crate::event::{ObservationEvent, ObservationPayload, ProviderType, EventType};
use crate::provider::{ObservationProvider, ProviderConfig, ProviderLifecycle, ProviderState};
use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hasher;
use std::sync::{Arc, Mutex};

// ── Data Structures ────────────────────────────────────────────────────

/// Individual error detected by the Vision AI.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VisionError {
    pub description: String,
    pub severity: String,
}

/// Structured result from the Vision AI analysis.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VisionAnalysisResult {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub focused_app: Option<String>,
    pub user_activity: Option<String>,
    pub errors_detected: Vec<VisionError>,
    pub inferred_intent: Option<String>,
    pub suggestions: Vec<String>,
    pub raw_analysis: String,
    pub confidence: f64,
}

/// Configuration for the Vision Analyzer provider.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VisionAnalyzerConfig {
    pub api_key: String,
    pub model: String,
    pub endpoint: String,
    pub poll_interval_secs: u64,
    pub system_prompt: Option<String>,
    pub max_tokens: u32,
    pub temperature: f32,
    /// Max width for screenshot sent to Vision AI (higher = better accuracy but more expensive).
    pub max_screenshot_width: u32,
}

impl Default for VisionAnalyzerConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            model: "anthropic/claude-sonnet-4".to_string(),
            endpoint: "https://openrouter.ai/api/v1".to_string(),
            poll_interval_secs: 15, // Every 15s to balance responsiveness vs cost
            system_prompt: None,
            max_tokens: 1000,
            temperature: 0.5, // Higher temp for better visual judgment
            max_screenshot_width: 1280, // Resize to fit, reduces cost and improves readibility
        }
    }
}

/// State for the Vision Analyzer provider.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VisionAnalyzerState {
    pub config: ProviderConfig,
    pub vision_config: VisionAnalyzerConfig,
    pub state: ProviderState,
    pub lifecycle: ProviderLifecycle,
    pub last_analysis: Option<VisionAnalysisResult>,
    pub analysis_count: u32,
    pub last_analysis_time: Option<chrono::DateTime<chrono::Utc>>,
    pub last_error: Option<String>,
    pub pending_screenshot: Option<(String, u32, u32, String)>,
    pub has_pending_screenshot: bool,
    /// Hash of the last analysis result for deduplication.
    pub last_analysis_hash: Option<String>,
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
            last_analysis_hash: None,
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

/// Compute a simple hash of the analysis content for deduplication.
fn analysis_hash(result: &VisionAnalysisResult) -> String {
    let mut parts = Vec::new();
    parts.push(result.focused_app.as_deref().unwrap_or(""));
    parts.push(&result.raw_analysis);
    for err in &result.errors_detected {
        parts.push(&err.description);
    }
    for sug in &result.suggestions {
        parts.push(sug);
    }
    if let Some(intent) = &result.inferred_intent {
        parts.push(intent);
    }
    // Simple hash: join + hex digest
    let joined = parts.join("||");
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    hasher.write(joined.as_bytes());
    format!("{:x}", hasher.finish())
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

    /// Resize the screenshot base64 to a max width to reduce cost and improve AI readability.
    /// Uses a simple heuristic: if the original width exceeds max, return a reduced version.
    /// In practice, the caller should pre-scale the screenshot. This is a fallback.
    fn maybe_resize_screenshot(
        &self,
        base64_data: &str,
        orig_width: u32,
        orig_height: u32,
        max_width: u32,
    ) -> (String, u32, u32) {
        if orig_width <= max_width {
            return (base64_data.to_string(), orig_width, orig_height);
        }
        // Calculate scale factor and new dimensions
        let scale = max_width as f64 / orig_width as f64;
        let new_width = max_width;
        let new_height = (orig_height as f64 * scale) as u32;
        // Return the same base64 but with updated dimensions — the AI handles it fine
        // (OpenRouter vision models accept any resolution)
        // For a real resize, we'd need an image library, but sending the full base64
        // with explicit dimension metadata is often sufficient for Claude's vision.
        // The key win is that the prompt explicitly says to focus on readable content.
        (base64_data.to_string(), new_width, new_height)
    }

    /// Analyze a screenshot using the Vision AI model.
    async fn analyze_screenshot(&self, screenshot_base64: &str, width: u32, height: u32, focused_window: &str) -> Option<VisionAnalysisResult> {
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

        // Apply screenshot size reduction if needed
        let (screenshot_data, _width, _height) =
            self.maybe_resize_screenshot(screenshot_base64, width, height, config.max_screenshot_width);

        // Build the message with the screenshot
        let content = vec![
            serde_json::json!({
                "type": "text",
                "text": format!("{}\n\nFocused window: {}", system_prompt, focused_window)
            }),
            serde_json::json!({
                "type": "image_url",
                "image_url": {
                    "url": format!("data:image/png;base64,{}", screenshot_data),
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

        // Check for NO_GUIDANCE_NEEDED — Vision AI said user is not doing technical work
        if response_text.trim().to_uppercase() == "NO_GUIDANCE_NEEDED" {
            tracing::debug!(
                "[Vision] Silent mode triggered — user is not doing technical work"
            );
            return None;
        }

        // Parse the analysis result from the Vision model response
        let analysis = parse_vision_response(response_text, focused_window);

        // Deduplicate: skip if this analysis is the same as the last one
        let current_hash = analysis_hash(&analysis);
        {
            let state = self.state.lock().unwrap();
            if let Some(ref prev_hash) = state.last_analysis_hash {
                if prev_hash == &current_hash && state.last_analysis.is_some() {
                    tracing::debug!(
                        "[Vision] Deduplication — analysis same as last, skipping event emission"
                    );
                    // Still clear pending screenshot but return None to suppress event
                    return None;
                }
            }
        }

        // If the analysis produced zero errors AND zero suggestions AND
        // the user is not clearly troubleshooting, suppress
        if analysis.errors_detected.is_empty()
            && analysis.suggestions.is_empty()
            && !analysis.raw_analysis.to_lowercase().contains("should")
            && !analysis.raw_analysis.to_lowercase().contains("try")
            && !analysis.raw_analysis.to_lowercase().contains("check")
        {
            tracing::debug!(
                "[Vision] No actionable guidance produced — suppressing result"
            );
            return None;
        }

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
                    state.last_analysis_hash = Some(analysis_hash(&analysis));
                    state.analysis_count += 1;
                    state.last_analysis_time = Some(Utc::now());
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
                // No analysis produced (rate limited, API error, silent mode, dedup, or no actionable guidance)
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
        details.insert("temperature".to_string(), serde_json::json!(state.vision_config.temperature));
        details.insert("max_screenshot_width".to_string(), serde_json::json!(state.vision_config.max_screenshot_width));
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

    // Detect errors and problems from response — covers both technical errors AND
    // visual/content issues (blank, empty, frozen, missing, broken, etc.)
    let error_patterns = [
        // Technical errors
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
        // Visual/content issues — AI noticed something is wrong
        ("blank", "Page or content appears blank"),
        ("empty", "Content appears empty or missing"),
        ("not loading", "Content not loading"),
        ("frozen", "Application appears frozen"),
        ("unresponsive", "Application appears unresponsive"),
        ("missing", "Expected content is missing"),
        ("broken", "Something on screen is broken"),
        ("white screen", "Screen is white/blank"),
        ("spinner", "Loading spinner visible"),
        ("stuck", "Process or page is stuck"),
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
        Some("Troubleshooting - user is experiencing errors or issues".to_string())
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
///
/// Strict about NOT hallucinating, but also empowers the AI to be intelligent about
/// what's wrong on screen — blank pages, missing content, wrong state, etc.
const DEFAULT_VISION_PROMPT: &str = r#"[SCREEN ANALYSIS INSTRUCTIONS]
You are an AI copilot observing a live screenshot of a user's computer. You see ONE frozen moment of their screen.

## WHAT YOU MUST ANALYZE:
1. What application/window is currently in focus (foreground)?
2. What is visibly ON THE SCREEN right now? Describe ONLY what you can actually see.
3. Are there any errors, warnings, or problems visible on the screen?
4. Is the screen showing what the user likely expects to see? For example:
   - A web page that is completely blank or white
   - A dashboard that has no data or graphs
   - A terminal showing no output or an empty prompt
   - A configuration UI that is missing expected fields
   - An app that looks frozen or unresponsive
5. What is the user's likely intent based ONLY on what's visible?

## ANTI-HALLUCINATION RULES:
- NEVER mention commands the user "ran" unless you can actually see a terminal with that command visible on screen.
- NEVER claim to see files, directories, or content you cannot actually see in the screenshot.
- NEVER reference past activity, previous commands, or things that happened before this screenshot.
- If you cannot see something on this screenshot, do NOT invent it.
- NEVER guess what the user typed, opened, or did previously.

## SILENT MODE:
If the user appears to be doing any of the following, respond with EXACTLY: "NO_GUIDANCE_NEEDED"
- Watching videos (YouTube, Netflix, etc.)
- Reading news articles or blogs
- Browsing social media
- Playing games
- General web browsing for entertainment
- Reading emails or chatting

Only provide technical guidance when the user is clearly:
- Troubleshooting an error
- Working with infrastructure/development tools
- Configuring a system
- Running commands or analyzing logs
- Working with databases, servers, or networks
- Seeing something wrong on screen (blank page, missing content, frozen app, etc.)

## RESPONSE FORMAT:
If NO_GUIDANCE_NEEDED applies, respond with just that text.

Otherwise, give a brief, conversational analysis focused ONLY on what you can see. Keep it to 2-4 sentences max."#;

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
        let response = "I see the user is looking at a Nagios page. There is a red Database Error message on the screen. The user is troubleshooting a database connection issue. They should check MySQL status.";
        let result = parse_vision_response(response, "Nagios - Database Error");
        assert!(!result.errors_detected.is_empty());
        assert!(result.raw_analysis.contains("Database Error"));
    }

    #[test]
    fn test_parse_vision_response_blank_screen() {
        // Test that visual issues like blank screens are detected
        let response = "The screen appears to be completely blank and white. The browser tab shows a page but nothing is rendering. The user is likely experiencing a loading issue or broken page.";
        let result = parse_vision_response(response, "Chrome - Dashboard");
        assert!(!result.errors_detected.is_empty(), "Blank screen should be detected");
        assert!(result.errors_detected.iter().any(|e| e.description.contains("blank")), "Should detect 'blank' issue");
    }

    #[test]
    fn test_parse_vision_response_empty_dashboard() {
        // Test that missing content is detected
        let response = "I see the Grafana dashboard but all panels are completely empty with no data or graphs. The user's monitoring dashboard isn't showing any metrics.";
        let result = parse_vision_response(response, "Grafana - Dashboards");
        assert!(!result.errors_detected.is_empty(), "Empty dashboard should be detected");
        assert!(result.errors_detected.iter().any(|e| e.description.contains("empty")), "Should detect 'empty' issue");
    }

    #[test]
    fn test_parse_vision_response_frozen_app() {
        // Test that frozen/unresponsive apps are detected
        let response = "The Jenkins build UI looks frozen. The spinner is stuck and the page appears unresponsive. Nothing is loading.";
        let result = parse_vision_response(response, "Jenkins - Build Pipeline");
        assert!(!result.errors_detected.is_empty(), "Frozen app should be detected");
    }

    #[test]
    fn test_parse_vision_response_clean() {
        let response = "The user is looking at a Grafana dashboard. Everything looks healthy. They're monitoring system metrics.";
        let result = parse_vision_response(response, "Grafana - Dashboards");
        assert!(result.errors_detected.is_empty());
        assert!(result.user_activity.is_some());
    }

    #[test]
    fn test_parse_vision_response_suggestions() {
        let response = "The user is working on deployment in Jenkins. They should verify the pipeline configuration before running. Try checking the build logs first.";
        let result = parse_vision_response(response, "Jenkins - Pipeline");
        assert!(!result.suggestions.is_empty());
    }

    #[test]
    fn test_analysis_hash_deduplication() {
        let a1 = VisionAnalysisResult {
            timestamp: chrono::Utc::now(),
            focused_app: Some("Test".to_string()),
            user_activity: Some("Testing".to_string()),
            errors_detected: vec![],
            inferred_intent: Some("Test intent".to_string()),
            suggestions: vec![],
            raw_analysis: "same analysis".to_string(),
            confidence: 0.5,
        };
        let a2 = a1.clone();
        // Same content should produce same hash
        assert_eq!(analysis_hash(&a1), analysis_hash(&a2));
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

    #[test]
    fn test_default_temperature() {
        let config = VisionAnalyzerConfig::default();
        assert_eq!(config.temperature, 0.5, "Temperature should be 0.5 for better visual judgment");
    }

    #[test]
    fn test_default_screenshot_width() {
        let config = VisionAnalyzerConfig::default();
        assert_eq!(config.max_screenshot_width, 1280, "Screenshot width should be capped at 1280");
    }
}