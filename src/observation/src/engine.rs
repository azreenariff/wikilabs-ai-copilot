//! Observation Engine — orchestrates providers, event bus, and downstream consumers.
//!
//! This is the central piece that was missing. Previously, providers existed
//! but nothing ever started them. Now:
//!
//! 1. Creates a ProviderRegistry and registers all providers.
//! 2. Starts all enabled providers.
//! 3. Runs a polling loop that collects observations from all providers.
//! 4. Feeds events to the event bus → session tracker → guidance panel.
//! 5. When an error is detected or engineering context is found, generates
//!    a recommendation card and pushes it to the guidance panel.

use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Mutex;

use crate::correlation::CorrelationEngine;
use crate::error_detector::ErrorDetector;
use crate::event::{ObservationEvent, ObservationPayload, ProviderType};
use crate::event_bus::EventBus;
use crate::session_tracker::SessionTracker;
use crate::provider::{ObservationProvider, ProviderRegistry, ProviderState};

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
}

impl Default for ObservationEngineConfig {
    fn default() -> Self {
        Self {
            poll_interval_secs: 5,
            enable_error_detection: true,
            enable_session_tracking: true,
            enable_correlation: true,
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

        Self {
            config,
            registry,
            event_bus: Arc::new(event_bus),
            correlation_engine,
            error_detector,
            session_tracker,
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
    pub async fn run_loop(&self) {
        let interval = Duration::from_secs(self.config.poll_interval_secs);
        let mut tick = 0u64;

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

            // Poll all providers
            let registry = self.registry.lock().await;
            let mut event_count = 0usize;

            tracing::info!(
                "[ObservationEngine] Poll tick #{} — polling {} providers",
                tick,
                registry.all_providers().len()
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
                            event_count += 1;

                            // Publish to event bus
                            if let Err(e) = self.event_bus.publish(event.clone()) {
                                tracing::warn!(
                                    "[ObservationEngine] Failed to publish event: {}",
                                    e
                                );
                            }

                            // Feed to downstream consumers
                            self.feed_event(event);
                        }
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

            drop(registry);

            if event_count > 0 {
                tracing::info!(
                    "[ObservationEngine] Poll tick #{} returned {} events — publishing to event bus",
                    tick,
                    event_count
                );
            } else {
                tracing::debug!(
                    "[ObservationEngine] Poll tick #{} returned 0 events (nothing changed)",
                    tick
                );
            }

            // Wait for next poll cycle
            tokio::time::sleep(interval).await;
        }

        tracing::info!("[ObservationEngine] Polling loop ended");
    }

    /// Feed an observation event to downstream consumers.
    fn feed_event(&self, event: &ObservationEvent) {
        // 1. Update correlation engine based on provider type
        if self.config.enable_correlation {
            match &event.provider {
                ProviderType::ActiveWindow => {
                    self.correlation_engine
                        .update_active_app(Some(event.source.clone()));
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
                        self.correlation_engine
                            .update_browser_context(Some(url.to_string()), title, is_portal);
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
                        self.correlation_engine
                            .update_terminal_context(Some(session_id.to_string()), shell);
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

    /// Get the event bus reference for subscribing to events.
    pub fn event_bus(&self) -> &Arc<EventBus> {
        &self.event_bus
    }

    /// Check if the engine is running.
    pub async fn is_running(&self) -> bool {
        *self.running.lock().await
    }
}