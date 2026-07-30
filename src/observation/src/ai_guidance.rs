//! AI-Powered Guidance Provider
//!
//! Uses OpenRouter API to analyze observed context and generate intelligent,
//! conversational guidance suggestions. This provides reasoning that rule-based
//! heuristics can't achieve — connecting disparate observations into coherent advice.
//!
//! The AI guidance runs asynchronously and generates suggestions that complement
//! the rule-based GuidanceEngine output.

use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use tokio::sync::Semaphore;

use crate::correlation::{CorrelationEngine, CorrelationSet};
use crate::semantic_analyzer::SemanticAnalyzer;
use crate::error_detector::{ErrorDetector, DetectedError, ErrorSeverity};
use crate::browser::BrowserErrorSeverity;

/// AI configuration for OpenRouter integration.
#[derive(Debug, Clone)]
pub struct AiConfig {
    /// OpenRouter API key (base64 encoded).
    pub openrouter_api_key: String,
    /// Model to use for guidance.
    pub model: String,
    /// Maximum tokens in response.
    pub max_tokens: u32,
    /// Whether AI guidance is enabled.
    pub enabled: bool,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            openrouter_api_key: String::new(),
            model: "anthropic/claude-sonnet-4".to_string(),
            max_tokens: 512,
            enabled: false,
        }
    }
}

/// AI-generated guidance suggestion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiGuidanceSuggestion {
    /// Unique ID.
    pub id: String,
    /// When this was generated.
    pub generated_at: chrono::DateTime<chrono::Utc>,
    /// The AI-generated message (conversational tone).
    pub message: String,
    /// Category.
    pub category: AiGuidanceCategory,
    /// Whether this is actionable (has specific next steps).
    pub is_actionable: bool,
    /// Suggested actions (if applicable).
    pub suggested_actions: Vec<String>,
    /// Related context that triggered this suggestion.
    pub context_summary: String,
}

/// Category of AI guidance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AiGuidanceCategory {
    /// Proactive suggestion based on observed patterns.
    Proactive,
    /// Warning about potential issues.
    Warning,
    /// Explanation of what the user is doing.
    Explanation,
    /// Best practice reminder.
    BestPractice,
    /// Troubleshooting assistance.
    Troubleshooting,
}

/// AI guidance state.
#[derive(Debug, Clone)]
pub struct AiGuidanceState {
    /// Active AI suggestions.
    pub active_suggestions: Vec<AiGuidanceSuggestion>,
    /// Last generated timestamp.
    pub last_generated: Option<chrono::DateTime<chrono::Utc>>,
    /// Total suggestions generated.
    pub total_generated: u32,
    /// Whether the AI is currently processing.
    pub is_processing: bool,
}

/// AI guidance provider that uses OpenRouter for intelligent analysis.
pub struct AiGuidanceProvider {
    state: Arc<Mutex<AiGuidanceState>>,
    config: Arc<Mutex<AiConfig>>,
    correlation_engine: Arc<CorrelationEngine>,
    semantic_analyzer: SemanticAnalyzer,
    error_detector: ErrorDetector,
    /// Rate limiter — only one AI request at a time.
    rate_limiter: Arc<Semaphore>,
}

impl AiGuidanceProvider {
    /// Create a new AI guidance provider.
    pub fn new(correlation_engine: Arc<CorrelationEngine>) -> Self {
        Self {
            state: Arc::new(Mutex::new(AiGuidanceState {
                active_suggestions: Vec::new(),
                last_generated: None,
                total_generated: 0,
                is_processing: false,
            })),
            config: Arc::new(Mutex::new(AiConfig::default())),
            correlation_engine,
            semantic_analyzer: SemanticAnalyzer::new(),
            error_detector: ErrorDetector::new(),
            rate_limiter: Arc::new(Semaphore::new(1)),
        }
    }

    /// Configure the AI provider.
    pub fn configure(&self, config: AiConfig) {
        let mut state = self.config.lock().unwrap();
        *state = config;
    }

    /// Get current configuration.
    pub fn get_config(&self) -> AiConfig {
        self.config.lock().unwrap().clone()
    }

    /// Generate AI-powered guidance based on current observation state.
    pub fn generate_ai_guidance(&self) -> Vec<AiGuidanceSuggestion> {
        let config = self.config.lock().unwrap().clone();
        if !config.enabled || config.openrouter_api_key.is_empty() {
            return Vec::new();
        }

        // Rate limit — only one request at a time
        match self.rate_limiter.try_acquire() {
            Ok(_permit) => {}
            Err(_) => return Vec::new(),
        }

        // Gather context from all observation sources
        let correlation_set = self.correlation_engine.scan();
        let active_correlations = self.correlation_engine.get_engineering_correlations();
        let detected_errors = self.error_detector.get_errors();

        // Build context summary for the AI
        let context = self.build_context_summary(&correlation_set, &active_correlations, &detected_errors);

        // Call OpenRouter API
        let suggestions = match self.call_openrouter(&config, &context) {
            Ok(suggestions) => suggestions,
            Err(e) => {
                tracing::warn!("AI guidance generation failed: {}", e);
                return Vec::new();
            }
        };

        // Update state
        {
            let mut state = self.state.lock().unwrap();
            state.active_suggestions = suggestions.clone();
            state.last_generated = Some(chrono::Utc::now());
            state.total_generated += suggestions.len() as u32;
            state.is_processing = false;
        }

        suggestions
    }

    /// Build a context summary from all observation sources.
    /// 
    /// CRITICAL: Pass raw observation data to the AI alongside structured summaries.
    /// The AI needs to see actual page content, terminal output, and window titles —
    /// not just what hardcoded pattern detectors matched. It must be able to analyze
    /// whatever is on screen, not just pre-classified signals.
    fn build_context_summary(
        &self,
        correlation_set: &CorrelationSet,
        active_correlations: &CorrelationSet,
        detected_errors: &[DetectedError],
    ) -> String {
        let mut context = String::new();

        // ── Browser: raw data first, then structured summary ──
        // Use get_full_browser_context() which preserves visible_text and detected_errors
        // (get_browser_context() discards them — it's a lightweight view)
        if let Some(browser) = self.correlation_engine.get_full_browser_context() {
            context.push_str(&format!("Browser: {} - {} ({})\n",
                browser.browser_name,
                browser.url.as_deref().unwrap_or("unknown"),
                if browser.is_engineering_portal { "engineering portal" } else { "regular" }
            ));

            if let Some(text) = &browser.visible_text {
                if !text.is_empty() {
                    // Send more raw text so the AI can analyze actual page content,
                    // not just what pattern detectors flagged
                    let display = if text.len() > 3000 { &text[..3000] } else { text };
                    context.push_str(&format!("RAW PAGE CONTENT (full visible text on screen — analyze what's actually there):\n{}\n\n", display));
                }

                // Still include structured error classifications as a hint,
                // but the AI should also read the raw page content above
                if !browser.detected_errors.is_empty() {
                    context.push_str("Pattern-detected errors (for reference — the AI should also verify against raw page content):\n");
                    for error in &browser.detected_errors {
                        let severity_str = match error.severity {
                            BrowserErrorSeverity::Low => "LOW",
                            BrowserErrorSeverity::Medium => "MED",
                            BrowserErrorSeverity::High => "HIGH",
                            BrowserErrorSeverity::Critical => "CRITICAL",
                        };
                        context.push_str(&format!("- {} ({}) [severity: {}]\n", error.description, error.pattern, severity_str));
                    }
                }
            }
        }

        // ── Active window: raw title and process, not just classification ──
        if let Some(app) = self.correlation_engine.get_active_app() {
            context.push_str(&format!("Active window: {}\n", app));
        }

        // ── Terminal: raw output (full buffer), not just last command ──
        // Also get the last terminal command
        if let Some(cmd) = self.correlation_engine.get_terminal_command() {
            context.push_str(&format!("Last terminal command: {}\n", cmd));

            // Add semantic analysis
            if let Some(intent) = self.semantic_analyzer.analyze_command(&cmd) {
                context.push_str(&format!("Intent: {} ({})\n", intent.action, intent.target.as_deref().unwrap_or("unknown")));
            }
        }

        // Send raw terminal output so the AI can analyze whatever is in the terminal window
        // (errors, output, status — not just what the error detector matched)
        if let Some(term_output) = self.correlation_engine.get_terminal_output() {
            if !term_output.trim().is_empty() {
                // Terminal output can be large; send up to 5000 chars
                let display = if term_output.len() > 5000 { &term_output[..5000] } else { &term_output };
                context.push_str(&format!("RAW TERMINAL OUTPUT (full visible terminal buffer — analyze errors, status, any content):\n{}\n\n", display));
            }
        }

        // Add correlation information
        if !active_correlations.records.is_empty() {
            context.push_str("\nActive correlations:\n");
            for record in &active_correlations.records {
                context.push_str(&format!("- {} (confidence: {:.2})\n",
                    record.explanation,
                    record.confidence
                ));
            }
        }

        // Add error information
        if !detected_errors.is_empty() {
            context.push_str("\nDetected errors:\n");
            for error in detected_errors.iter().take(5) {
                let severity_str = match error.severity {
                    ErrorSeverity::Low => "LOW",
                    ErrorSeverity::Medium => "MED",
                    ErrorSeverity::High => "HIGH",
                    ErrorSeverity::Critical => "CRITICAL",
                };
                context.push_str(&format!("- [{}] {} - {}\n",
                    severity_str,
                    error.title,
                    error.description.chars().take(100).collect::<String>()
                ));
            }
        }

        // Add general summary
        context.push_str(&format!("\nSummary: {} correlations found, {} errors detected, {} engineering contexts.",
            correlation_set.records.len(),
            detected_errors.len(),
            if correlation_set.has_engineering_context { 1 } else { 0 }
        ));

        context
    }

    /// Call OpenRouter API to generate guidance suggestions.
        fn call_openrouter(&self, config: &AiConfig, context: &str) -> Result<Vec<AiGuidanceSuggestion>, String> {
            let prompt = format!(
                "You are a senior DevOps engineer providing proactive, conversational guidance to a user. \
                Be helpful, technical but approachable — like a teammate giving you advice.\n\n\
                Current observation context:\n{}\n\n\
                CRITICAL RULES — follow ALL of them:\n\
                1. The RAW PAGE CONTENT and RAW TERMINAL OUTPUT sections contain unfiltered, raw observation data from the user's screen. Read and analyze this data directly — do NOT rely only on the structured summaries or error classifications.\n\
                2. The AI should look at what's actually on the page, in the terminal, and in the window — not just what pattern detectors matched.\n\
                3. Do NOT guess about past activity. Only give advice based on what is IN THIS context block above.\n\
                4. Do NOT reference things the user may have done in previous sessions or windows.\n\
                5. If you cannot see something in the context, do NOT claim to see it. Do NOT hallucinate.\n\
                6. If the user is troubleshooting (you see errors + debug commands), focus on troubleshooting advice.\n\
                7. If nothing specific is visible in context, say so — do NOT generate generic advice.\n\n\
                Based on this context, provide up to 3 specific, actionable suggestions. \
                Focus on:\n\
                - Proactive advice (what the user should do next)\n\
                - Warning about potential issues they might not see\n\
                - Best practices for the infrastructure they're working with\n\
                - Troubleshooting steps if errors are detected\n\n\
                Format each suggestion as a JSON object with these fields:\n\
                {{\\\"message\\\": \\\"conversational advice\\\", \\\"category\\\": \\\"Proactive|Warning|Explanation|BestPractice|Troubleshooting\\\", \\\"is_actionable\\\": true/false, \\\"suggested_actions\\\": [\\\"action1\\\", \\\"action2\\\"]}}\n\
            Return ONLY a JSON array of objects, no other text.",
            context
        );

        let client = reqwest::blocking::Client::new();
        let response = client
            .post("https://openrouter.ai/api/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", config.openrouter_api_key))
            .header("Content-Type", "application/json")
            .header("HTTP-Referer", "https://wikilabs.ai")
            .header("X-Title", "Wiki Labs AI Copilot")
            .json(&serde_json::json!({
                "model": config.model,
                "messages": [
                    {"role": "system", "content": "You are a helpful DevOps engineer providing real-time, conversational guidance. Keep messages short and actionable like a teammate. NEVER hallucinate or guess about things not visible in the context."},
                    {"role": "user", "content": prompt}
                ],
                "max_tokens": config.max_tokens,
                "temperature": 0.3,
            }))
            .send()
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().map_err(|e| format!("Failed to read response: {}", e))?;
            return Err(format!("API error {}: {}", status, body));
        }

        let api_response: OpenRouterResponse = serde_json::from_str(
            &response.text().map_err(|e| format!("Failed to parse response: {}", e))?
        ).map_err(|e| format!("Invalid JSON response: {}", e))?;

        // Extract suggestions from the AI response
        let content = api_response.choices.first()
            .ok_or("No choices in response")?
            .message.content.clone();

        // Parse the JSON array from the response
        let suggestions = Self::parse_ai_suggestions(&content)?;

        Ok(suggestions)
    }

    /// Parse AI-generated JSON suggestions.
    fn parse_ai_suggestions(content: &str) -> Result<Vec<AiGuidanceSuggestion>, String> {
        // Try to find JSON array in the response
        let json_start = content.find('[').ok_or("No JSON array found")?;
        let json_end = content.rfind(']').ok_or("No JSON array end found")? + 1;
        let json_str = &content[json_start..json_end];

        let suggestions: Vec<RawAiSuggestion> = serde_json::from_str(json_str)
            .map_err(|e| format!("Failed to parse suggestions JSON: {}", e))?;

        let now = chrono::Utc::now();
        let mut result = Vec::new();

        for (i, raw) in suggestions.iter().enumerate() {
            let category = match raw.category.as_str() {
                "Proactive" => AiGuidanceCategory::Proactive,
                "Warning" => AiGuidanceCategory::Warning,
                "Explanation" => AiGuidanceCategory::Explanation,
                "BestPractice" => AiGuidanceCategory::BestPractice,
                "Troubleshooting" => AiGuidanceCategory::Troubleshooting,
                _ => AiGuidanceCategory::Proactive,
            };

            result.push(AiGuidanceSuggestion {
                id: format!("ai-{}-{}", now.timestamp_micros(), i),
                generated_at: now,
                message: raw.message.clone(),
                category,
                is_actionable: raw.is_actionable,
                suggested_actions: raw.suggested_actions.clone(),
                context_summary: "AI analysis of observation context".to_string(),
            });
        }

        Ok(result)
    }

    /// Dismiss an AI suggestion.
    pub fn dismiss(&self, id: &str) {
        let mut state = self.state.lock().unwrap();
        if let Some(pos) = state.active_suggestions.iter().position(|s| s.id == id) {
            state.active_suggestions.remove(pos);
        }
    }

    /// Get current active AI suggestions.
    pub fn get_active_suggestions(&self) -> Vec<AiGuidanceSuggestion> {
        self.state.lock().unwrap().active_suggestions.clone()
    }

    /// Get all state information.
    pub fn get_state(&self) -> AiGuidanceState {
        self.state.lock().unwrap().clone()
    }

    /// Check if AI guidance is enabled and configured.
    pub fn is_available(&self) -> bool {
        let config = self.config.lock().unwrap();
        config.enabled && !config.openrouter_api_key.is_empty()
    }
}

/// Raw suggestion from OpenRouter API.
#[derive(Debug, Deserialize)]
struct RawAiSuggestion {
    message: String,
    category: String,
    is_actionable: bool,
    suggested_actions: Vec<String>,
}

/// OpenRouter API response structure.
#[derive(Debug, Deserialize)]
struct OpenRouterResponse {
    choices: Vec<ApiChoice>,
}

#[derive(Debug, Deserialize)]
struct ApiChoice {
    message: ApiMessage,
}

#[derive(Debug, Deserialize)]
struct ApiMessage {
    content: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_guidance_creation() {
        let engine = Arc::new(CorrelationEngine::new());
        let provider = AiGuidanceProvider::new(engine);
        assert!(!provider.is_available());
    }

    #[test]
    fn test_ai_guidance_disabled() {
        let engine = Arc::new(CorrelationEngine::new());
        let provider = AiGuidanceProvider::new(engine);

        // Should return empty when not configured
        let suggestions = provider.generate_ai_guidance();
        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_ai_guidance_parse() {
        let json_content = r#"Here are some suggestions: [{"message": "Check service status", "category": "Proactive", "is_actionable": true, "suggested_actions": ["systemctl status"]}]"#;
        let suggestions = AiGuidanceProvider::parse_ai_suggestions(json_content);
        assert!(suggestions.is_ok());
        let suggestions = suggestions.unwrap();
        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].message, "Check service status");
    }
}