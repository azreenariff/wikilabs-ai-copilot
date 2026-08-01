//! Observation Engine — orchestrates providers, event bus, and downstream consumers.
//!
//! This is the central piece that was missing. Previously, providers existed
//! but nothing ever started them. Now:
//!
//! 1. Creates a ProviderRegistry and registers all providers.
//! 2. Starts all enabled providers.
//! 3. Runs a polling loop that collects observations from all providers.
//! 4. Feeds events to the event bus → session tracker → guidance panel.
//! 5. Produces heartbeat events periodically so the AI always has context
//!    even when no state changes are detected.
//! 6. When an error is detected or engineering context is found, generates
//!    a recommendation card and pushes it to the guidance panel.

use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Mutex;

use crate::correlation::CorrelationEngine;
use crate::error_detector::ErrorDetector;
use crate::event::{ObservationEvent, ObservationPayload, ProviderType};
use crate::event_bus::EventBus;
use crate::session_tracker::SessionTracker;
use crate::intent_analyzer::IntentAnalyzer;
use crate::provider::{ObservationProvider, ProviderRegistry, ProviderState};
use crate::vision_analyzer::VisionAnalyzerProvider;

/// Configuration for the observation engine.
#[derive(Debug, Clone)]
pub struct ObservationEngineConfig {
    /// Polling interval for providers that don't push.
    pub poll_interval_secs: u64,
    /// Whether to enable error detection.
    pub enable_error_detection: bool,
    /// Whether to enable session tracking.
    pub enable_session_tracking: bool,
    /// Whether to enable correlation engine.
    pub enable_correlation: bool,
    /// Delay in seconds before the first observation tick on startup.
    /// Gives the user's desktop time to settle — avoids capturing
    /// transient UI elements (splash screens, command palette, IME).
    pub startup_delay_secs: u64,
}

impl Default for ObservationEngineConfig {
    fn default() -> Self {
        Self {
            poll_interval_secs: 5,
            enable_error_detection: true,
            enable_session_tracking: true,
            enable_correlation: true,
            startup_delay_secs: 5,
        }
    }
}

/// The observation engine that orchestrates everything.
pub struct ObservationEngine {
    config: ObservationEngineConfig,
    registry: Arc<Mutex<ProviderRegistry>>,
    event_bus: Arc<EventBus>,
    correlation_engine: Arc<CorrelationEngine>,
    error_detector: Arc<ErrorDetector>,
    session_tracker: Arc<SessionTracker>,
    /// Intent analyzer for synthesizing structured intent summaries.
    intent_analyzer: Arc<IntentAnalyzer>,
    /// Whether the engine is running.
    running: Arc<Mutex<bool>>,
}

impl ObservationEngine {
    /// Create a new observation engine with all providers registered.
    pub fn new(config: ObservationEngineConfig) -> Self {
        let event_bus = EventBus::with_defaults();
        let correlation_engine = Arc::new(CorrelationEngine::new());
        let error_detector = Arc::new(ErrorDetector::new());
        let session_tracker = Arc::new(SessionTracker::new());

        let registry = Arc::new(Mutex::new(ProviderRegistry::new()));
        let intent_analyzer = Arc::new(IntentAnalyzer::new());

        Self {
            config,
            registry,
            event_bus: Arc::new(event_bus),
            correlation_engine,
            error_detector,
            session_tracker,
            intent_analyzer,
            running: Arc::new(Mutex::new(false)),
        }
    }

    /// Register a provider with the engine.
    pub async fn register_provider(&self, provider: Box<dyn ObservationProvider>) {
        let mut registry = self.registry.lock().await;
        registry.register(provider);
    }

    /// Start all registered providers.
    /// Returns the list of (provider_name, result) pairs.
    pub async fn start(&self) -> Vec<(String, Result<(), String>)> {
        tracing::info!("[ObservationEngine] Starting observation engine");

        let mut running = self.running.lock().await;
        *running = true;

        let mut registry = self.registry.lock().await;
        let results = registry.start_all().await;

        for (name, result) in &results {
            match result {
                Ok(_) => tracing::info!("[ObservationEngine] Provider {} started", name),
                Err(e) => tracing::error!("[ObservationEngine] Provider {} failed: {}", name, e),
            }
        }

        drop(registry);

        tracing::info!(
            "[ObservationEngine] Started {} providers, {} errors",
            results.len(),
            results.iter().filter(|(_, r)| r.is_err()).count()
        );

        results
    }

    /// Stop all providers.
    pub async fn stop(&self) {
        tracing::info!("[ObservationEngine] Stopping observation engine");

        {
            let mut running = self.running.lock().await;
            *running = false;
        }

        let mut registry = self.registry.lock().await;
        let results = registry.stop_all().await;

        for (name, result) in &results {
            match result {
                Ok(_) => tracing::info!("[ObservationEngine] Provider {} stopped", name),
                Err(e) => tracing::error!("[ObservationEngine] Provider {} stop failed: {}", name, e),
            }
        }
    }

    /// Run the observation polling loop (blocking). This runs in a background thread.
    /// 
    /// On startup, waits `startup_delay` before the first poll tick to give the user's
    /// desktop time to settle — avoiding captures of transient UI elements like splash
    /// screens, command palette, IME windows, etc. that appear briefly at launch.
    pub async fn run_loop(&self) {
        let interval = Duration::from_secs(self.config.poll_interval_secs);
        let mut tick = 0u64;

        // Startup delay: wait before the first observation tick
        let startup_delay = Duration::from_secs(self.config.startup_delay_secs);
        if startup_delay.as_secs() > 0 {
            tracing::info!(
                "[ObservationEngine] Waiting {}s startup delay before first observation tick",
                startup_delay.as_secs()
            );
            tokio::time::sleep(startup_delay).await;
        }

        loop {
            tick += 1;

            // Check if we should stop
            {
                let running = self.running.lock().await;
                if !*running {
                    break;
                }
                drop(running);
            }

            // Poll all providers — collect events first, then process them
            // after releasing the registry lock to avoid reentrant deadlocks.
            let registry = self.registry.lock().await;
            let provider_count = registry.all_providers().len();
            let mut all_events: Vec<ObservationEvent> = Vec::new();
            let mut _heartbeat_count = 0usize;

            tracing::info!(
                "[ObservationEngine] Poll tick #{} — polling {} providers",
                tick,
                provider_count
            );

            for provider in registry.all_providers() {
                // Skip disabled providers
                let config = provider.config();
                if !config.enabled {
                    continue;
                }

                // Check provider state
                match provider.state() {
                    ProviderState::Active | ProviderState::Paused => {}
                    ProviderState::Disabled => continue,
                    ProviderState::Error(_) => continue,
                }

                // Try to observe
                match provider.observe().await {
                    Ok(events) => {
                        tracing::debug!(
                            provider = %provider.name(),
                            events = events.len(),
                            "[ObservationEngine] Provider returned events"
                        );

                        for event in &events {
                            // Publish to event bus (safe while holding lock)
                            if let Err(e) = self.event_bus.publish(event.clone()) {
                                tracing::warn!(
                                    "[ObservationEngine] Failed to publish event: {}",
                                    e
                                );
                            }
                        }
                        all_events.extend(events);
                    }
                    Err(e) => {
                        tracing::debug!(
                            provider = %provider.name(),
                            error = %e,
                            "[ObservationEngine] Observe error"
                        );
                    }
                }
            }

            // Release the registry lock BEFORE feeding events downstream
            drop(registry);

            // Now feed all collected events (safe — no registry lock held)
            for event in &all_events {
                self.feed_event(event).await;
            }

            let event_count = all_events.len();
            if event_count > 0 {
                tracing::info!(
                    "[ObservationEngine] Poll tick #{} returned {} events — publishing to event bus",
                    tick,
                    event_count
                );
            } else {
                // No state-change events — produce a heartbeat so the AI always
                // has recent context. This prevents the "no guidance appears"
                // problem when the user is working in a stable window (e.g.
                // browsing an engineering portal without switching windows).
                tracing::debug!(
                    "[ObservationEngine] Poll tick #{} returned 0 events — producing heartbeat",
                    tick
                );

                let heartbeat_event = crate::event::ObservationEvent::new(
                    crate::event::EventType::ApplicationChanged,
                    crate::event::ProviderType::ActiveWindow,
                    "engine_heartbeat".to_string(),
                    None,
                    crate::event::ObservationPayload::new(serde_json::json!({
                        "type": "heartbeat",
                        "poll_tick": tick,
                        "provider_count": provider_count,
                        "message": "Observation engine is active — AI can use current context for guidance"
                    })),
                );

                if let Err(e) = self.event_bus.publish(heartbeat_event.clone()) {
                    tracing::warn!(
                        "[ObservationEngine] Failed to publish heartbeat event: {}",
                        e
                    );
                } else {
                    _heartbeat_count += 1;
                }
            }

            // Wait for next poll cycle
            tokio::time::sleep(interval).await;
        }

        tracing::info!("[ObservationEngine] Polling loop ended");
    }

    /// Feed an observation event to downstream consumers.
    async fn feed_event(&self, event: &ObservationEvent) {
        // 1. Update correlation engine based on provider type
        if self.config.enable_correlation {
            match &event.provider {
                ProviderType::ActiveWindow => {
                    // Filter out UI chrome noise before feeding to correlation engine.
                    // The source here is the window title captured from the ActiveWindow provider.
                    let source = event.source.clone();
                    let filtered = if Self::is_active_window_noise(&source) {
                        None
                    } else {
                        Some(source)
                    };
                    self.correlation_engine
                        .update_active_app(filtered);
                }
                ProviderType::Browser => {
                    if let Some(url) = event
                        .payload
                        .data
                        .get("url")
                        .and_then(|v| v.as_str())
                    {
                        let title = event
                            .payload
                            .data
                            .get("title")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        let is_portal = event
                            .payload
                            .data
                            .get("is_engineering_portal")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);
                        let browser_name = event
                            .payload
                            .data
                            .get("browser")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| "Browser".to_string());
                        let visible_text = event
                            .payload
                            .data
                            .get("visible_text")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        let detected_errors: Vec<crate::browser::BrowserError> = event
                            .payload
                            .data
                            .get("detected_errors")
                            .and_then(|v| v.as_array())
                            .map(|arr| {
                                arr.iter().filter_map(|e| {
                                    e.as_object().and_then(|obj| {
                                        let pattern = obj.get("pattern").and_then(|p| p.as_str()).map(|s| s.to_string())?;
                                        let description = obj.get("description").and_then(|d| d.as_str()).map(|s| s.to_string())?;
                                        let severity_str = obj.get("severity").and_then(|s| s.as_str()).unwrap_or("Low");
                                        let severity = match severity_str {
                                            "High" => crate::browser::BrowserErrorSeverity::High,
                                            "Critical" => crate::browser::BrowserErrorSeverity::Critical,
                                            "Medium" => crate::browser::BrowserErrorSeverity::Medium,
                                            _ => crate::browser::BrowserErrorSeverity::Low,
                                        };
                                        Some(crate::browser::BrowserError { pattern, description, severity })
                                    })
                                }).collect()
                            })
                            .unwrap_or_default();

                        // Update minimal state for quick access
                        self.correlation_engine
                            .update_browser_context(Some(url.to_string()), title.clone(), is_portal);

                        // Also update full context for intent analysis
                        let full_ctx = crate::browser::BrowserContext {
                            browser_name,
                            url: Some(url.to_string()),
                            title,
                            is_engineering_portal: is_portal,
                            visible_text,
                            detected_errors,
                        };
                        self.correlation_engine.update_browser_context_full(full_ctx);
                    }
                }
                ProviderType::Terminal => {
                    if let Some(session_id) = event
                        .payload
                        .data
                        .get("session_id")
                        .and_then(|v| v.as_str())
                    {
                        let shell = event
                            .payload
                            .data
                            .get("shell")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        let output = event
                            .payload
                            .data
                            .get("output")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        self.correlation_engine
                            .update_terminal_context(Some(session_id.to_string()), shell);
                        if let Some(out) = output {
                            self.correlation_engine.update_terminal_output(Some(out));
                        }
                    }
                }
                ProviderType::ScreenCapture => {
                    // Feed screenshot to vision analyzer.
                    // Safe to acquire the registry lock here because the polling loop
                    // released it before calling feed_event (see run_loop: drop then feed).
                    // We queue the screenshot directly to avoid the reentrant lock
                    // that would happen if we called feed_screenshot_to_vision_analyzer.
                    let screen_registry = self.registry.lock().await;
                    for provider in screen_registry.all_providers() {
                        if provider.provider_type() == ProviderType::ScreenCapture {
                            if let Some(screen) = provider.as_any().downcast_ref::<crate::screen_capture::ScreenCaptureProvider>() {
                                if let Some(screenshot) = screen.get_last_screenshot() {
                                    let focused_window = screenshot.focused_window.clone().unwrap_or_else(|| "unknown".to_string());
                                    let data = screenshot.data_base64.clone();
                                    let w = screenshot.width;
                                    let h = screenshot.height;
                                    // Release registry lock before queuing to VisionAnalyzer
                                    drop(screen_registry);
                                    // Queue the screenshot directly
                                    self.feed_screenshot_to_vision_analyzer(data, w, h, focused_window).await;
                                    break; // found it, done
                                }
                            }
                        }
                    }
                }
                ProviderType::VisionAnalysis if event.event_type == crate::event::EventType::VisionAnalysisResult => {
                    // Phase 3: Capture VisionAnalysisResult event and feed to IntentAnalyzer
                        if let Some(data_obj) = event.payload.data.as_object() {
                            // Parse the VisionAnalysisResult from the event payload
                            if let Some(inferred_intent) = data_obj.get("inferred_intent").and_then(|v| v.as_str()).map(|s| s.to_string()) {
                                let errors_detected: Vec<crate::vision_analyzer::VisionError> = data_obj.get("errors_detected")
                                    .and_then(|v| v.as_array())
                                    .map(|arr| arr.iter().filter_map(|e| {
                                        e.as_object().and_then(|obj| {
                                            let description = obj.get("description").and_then(|d| d.as_str()).map(|s| s.to_string())?;
                                            let severity = obj.get("severity").and_then(|s| s.as_str()).unwrap_or("low").to_string();
                                            Some(crate::vision_analyzer::VisionError { description, severity })
                                        })
                                    }).collect())
                                    .unwrap_or_default();
                                let suggestions: Vec<String> = data_obj.get("suggestions")
                                    .and_then(|v| v.as_array())
                                    .map(|arr| arr.iter().filter_map(|e| e.as_str().map(|s| s.to_string())).collect())
                                    .unwrap_or_default();
                                let focused_app = data_obj.get("focused_app").and_then(|v| v.as_str()).map(|s| s.to_string());
                                let user_activity = data_obj.get("user_activity").and_then(|v| v.as_str()).map(|s| s.to_string());
                                let confidence = data_obj.get("confidence").and_then(|v| v.as_f64()).unwrap_or(0.0);
                                let result = crate::vision_analyzer::VisionAnalysisResult {
                                    timestamp: chrono::Utc::now(),
                                    focused_app,
                                    user_activity,
                                    errors_detected,
                                    inferred_intent: Some(inferred_intent),
                                    suggestions,
                                    raw_analysis: String::new(),
                                    confidence,
                                };
                                self.intent_analyzer.set_vision_result(result);
                            }
                        }
                }
                _ => {}
            }
        }

        // 2. Run error detection via analyze_tick
        if self.config.enable_error_detection {
            let browser_url = event
                .payload
                .data
                .get("url")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let browser_content = event
                .payload
                .data
                .get("content")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let window_title = event
                .payload
                .data
                .get("title")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let window_content = event
                .payload
                .data
                .get("content")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let errors = self.error_detector.analyze_tick(
                browser_url.as_deref(),
                browser_content.as_deref(),
                None,  // terminal_cmd
                None,  // terminal_output
                window_title.as_deref(),
                window_content.as_deref(),
            );

            for detected in &errors {
                tracing::info!(
                    error_id = detected.id,
                    severity = ?detected.severity,
                    title = %detected.title,
                    "[ObservationEngine] Error detected"
                );
            }

            if !errors.is_empty() {
                let _ = self.event_bus.publish(
                    ObservationEvent::new(
                        crate::event::EventType::ApplicationChanged,
                        ProviderType::ActiveWindow,
                        "error_detector".to_string(),
                        None,
                        ObservationPayload::new(serde_json::json!({
                            "errors": errors.iter().map(|e| {
                                serde_json::json!({
                                    "id": e.id,
                                    "severity": format!("{:?}", e.severity),
                                    "title": e.title,
                                    "description": e.description,
                                })
                            }).collect::<Vec<_>>(),
                        })),
                    ),
                );
            }
        }

        // 3. Update session tracker
        if self.config.enable_session_tracking {
            let browser_url = event
                .payload
                .data
                .get("url")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let browser_error = event
                .payload
                .data
                .get("error")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let command = event
                .payload
                .data
                .get("command_text")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let command_output = event
                .payload
                .data
                .get("output")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let (session, suggestions) = self.session_tracker.process_tick(
                browser_url.as_deref(),
                browser_error.as_deref(),
                command.as_deref(),
                command_output.as_deref(),
            );

            // If session tracker generated suggestions, push to event bus
            if !suggestions.is_empty() {
                let sugg = &suggestions[0];
                tracing::info!(
                    suggestion = %sugg.message,
                    target = ?sugg.related_target,
                    confidence = sugg.confidence,
                    "[ObservationEngine] Session tracker suggestion"
                );

                // Record evidence from suggestions
                if let Some(ref target) = sugg.related_target {
                    let _ = self.event_bus.publish(
                                        ObservationEvent::new(
                                            crate::event::EventType::ConfigurationFileOpened,
                                            ProviderType::ActiveWindow,
                                            format!("session_tracker_suggestion_{}", target),
                                            None,
                                            ObservationPayload::new(serde_json::json!({
                                                "suggestion": sugg.message,
                                                "target": target,
                                                "session_state": format!("{:?}", session.state),
                                                "hypothesis": session.current_hypothesis,
                                                "target_system": session.target_system,
                                            })),
                                        ),
                                    );
                }
            }

            // Update session state in the event bus for tracking
            if session.state != crate::session_tracker::SessionState::Idle {
                let _ = self.event_bus.publish(
                    ObservationEvent::new(
                        crate::event::EventType::ApplicationChanged,
                        ProviderType::ActiveWindow,
                        "session_tracker_state".to_string(),
                        None,
                        ObservationPayload::new(serde_json::json!({
                            "session_state": format!("{:?}", session.state),
                            "steps": session.steps.len(),
                            "hypothesis": session.current_hypothesis,
                            "target_system": session.target_system,
                            "suggested_next_step": session.suggested_next_step,
                        })),
                    ),
                );
            }
        }
    }

    /// Get detected errors.
    pub fn get_errors(&self) -> Vec<crate::error_detector::DetectedError> {
        self.error_detector.get_errors()
    }

    /// Get current session state.
    pub fn get_session_state(&self) -> Option<crate::session_tracker::TroubleshootingSession> {
        self.session_tracker.get_session()
    }

    /// Get provider status.
    pub async fn get_provider_status(&self) -> Vec<crate::provider::ProviderStatus> {
        let registry = self.registry.lock().await;
        registry.all_status()
    }

    /// Filter noise window titles from ActiveWindow events.
    /// Returns true if the title is UI chrome noise and should be filtered out.
    fn is_active_window_noise(title: &str) -> bool {
        let title = title.trim();
        if title.is_empty() {
            return true;
        }
        let lower = title.to_lowercase();

        // Matches the noise patterns from app_monitor.rs (Windows-side filter)
        // but also catches any noise that slips through to the event bus
        if lower.contains("command palette")
            || lower.contains("start menu")
            || lower.contains("windows key")
        {
            return true;
        }
        if lower.contains("msctfime")
            || lower.contains("ime ui")
            || lower.contains("ime")
            || lower.contains("msctf_inputpane")
        {
            return true;
        }
        if lower.contains("clipboard")
            || lower.contains("clipbrd")
            || lower.contains("toast")
            || lower.contains("notification")
            || lower.contains("action center")
            || lower.contains("notification area")
        {
            return true;
        }
        if lower.contains("settings") && !lower.contains("configuration") {
            return true;
        }
        if lower.contains("about ")
            || lower.contains("uninstall")
            || lower.contains("sign in")
            || lower.contains("login")
            || lower.contains("updating")
            || lower.contains("installing")
            || lower.contains("progress")
            || lower.contains("error report")
            || lower.contains("crash")
        {
            return true;
        }
        if title.len() < 15 {
            let noise_kws = [
                "properties", "security", "options", "preferences", "help",
                "message", "warning", "confirm", "close", "cancel", "apply",
                "ok", "save", "open", "file", "edit", "view", "tools",
                "format", "window", "about",
            ];
            if noise_kws.iter().any(|k| lower.contains(k)) {
                return true;
            }
        }

        false
    }

    /// Get the event bus reference for subscribing to events.
    pub fn event_bus(&self) -> &Arc<EventBus> {
        &self.event_bus
    }

    /// Get the correlation engine reference for external use (e.g., AI guidance).
    pub fn correlation_engine(&self) -> &Arc<CorrelationEngine> {
        &self.correlation_engine
    }

    /// Get the error detector reference for external use.
    pub fn error_detector(&self) -> &Arc<ErrorDetector> {
        &self.error_detector
    }

    /// Run the intent analyzer on current observation state.
    /// Produces a structured IntentSummary that the AI can use for proactive guidance.
    pub fn analyze_intent(&self) -> crate::intent_analyzer::IntentSummary {
        let errors = self.error_detector.get_errors();
        let session = self.session_tracker.get_session();
        let correlation = &self.correlation_engine;

        // Get full browser context (includes visible_text and detected_errors)
        let browser_ctx = correlation.get_full_browser_context();
        let terminal_cmd = correlation.get_terminal_command();
        let terminal_output = correlation.get_terminal_output();
        let active_app = correlation.get_active_app();

        // Get session state for intent analysis
        let session_state = session.clone();

        self.intent_analyzer.analyze(
            browser_ctx.as_ref(),
            terminal_cmd.as_deref(),
            terminal_output.as_deref(),
            active_app.as_deref(),
            &errors,
            session_state.as_ref(),
            correlation,
            // Phase 3: pass the latest Vision analysis result for cross-context correlation
            self.intent_analyzer.get_vision_result().as_ref(),
        )
    }

    /// Get the latest Vision analysis result from the intent analyzer.
    pub fn get_vision_result(&self) -> Option<crate::vision_analyzer::VisionAnalysisResult> {
        self.intent_analyzer.get_vision_result()
    }

    /// Get the last analyzed intent (highest-confidence UserIntent from recent analysis).
    pub fn get_last_intent(&self) -> Option<crate::intent_analyzer::UserIntent> {
        self.intent_analyzer.get_last_intent()
    }

    /// Check if the engine is running.
    pub async fn is_running(&self) -> bool {
        *self.running.lock().await
    }

    /// Feed a screenshot from the screen capture provider to the vision analyzer.
    /// This is called by feed_event() when a ScreenshotCaptured event is received.
    async fn feed_screenshot_to_vision_analyzer(
        &self,
        data_base64: String,
        width: u32,
        height: u32,
        focused_window: String,
    ) {
        // Queue the screenshot directly to the VisionAnalyzer provider.
        // We get the registry lock briefly to find the provider, queue,
        // and release — no reentrant deadlock since we only lock once.
        let registry = self.registry.lock().await;
        for provider in registry.all_providers() {
            if provider.provider_type() == ProviderType::VisionAnalysis {
                if let Some(vision) = provider.as_any().downcast_ref::<VisionAnalyzerProvider>() {
                    vision.queue_screenshot(data_base64.clone(), width, height, focused_window.clone());
                    tracing::debug!("[Engine] Screenshot queued for vision analysis");
                }
            }
        }
        // Registry lock released here
    }

    /// Get the most recently captured screenshot from the screen capture provider.
    /// Returns None if no screenshot is available yet.
    pub fn get_last_screenshot(&self) -> Option<crate::screen_capture::CapturedScreenshot> {
        let registry = self.registry.blocking_lock();
        for provider in registry.all_providers() {
            if provider.provider_type() == ProviderType::ScreenCapture {
                if let Some(screen) = provider.as_any().downcast_ref::<crate::screen_capture::ScreenCaptureProvider>() {
                    return screen.get_last_screenshot();
                }
            }
        }
        None
    }
}