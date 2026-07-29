//! Observation Framework for Wiki Labs AI Copilot
//!
//! Phase 6 — Observation infrastructure only.
//! This crate does NOT interpret intent, perform AI reasoning, or provide recommendations.
//! It only observes activity and produces structured events for downstream consumers.
//!
//! ## Architecture
//!
//! - **Event Model** (`event`): Common schema for all observation events
//! - **Event Bus** (`event_bus`): Central pub/sub system
//! - **Provider Plugin Architecture** (`provider`): Trait-based pluggable providers
//! - **Privacy Controls** (`privacy`): Master enable/disable, per-provider toggle, pause/resume
//! - **Active Window Provider** (`app_monitor`): Foreground app/window detection
//! - **Terminal Provider** (`terminal`): Shell command observation
//! - **Browser Provider** (`browser`): Browser context detection (URL extraction via Win32)
//! - **Clipboard Provider** (`clipboard`): Clipboard content observation
//! - **File Provider** (`file_observer`): File open/edit observation
//! - **Screen Capture Provider** (`screen_capture`): Periodic screenshot capture
//! - **Shell Integration** (`shell`): Shell command capture via polling
//! - **Semantic Analyzer** (`semantic_analyzer`): Understands what commands mean
//! - **Session Tracker** (`session_tracker`): Tracks troubleshooting narratives
//! - **Cross-Context Correlation** (`correlation`): Links browser+terminal+app context
//! - **AI Guidance** (`guidance`): Proactive suggestions based on correlated context
//! - **CDP Browser Scraper** (`cdp_browser`): Chrome DevTools Protocol WebSocket scraping for real HTML analysis
//! - **Console Output Capture** (`console_output`): Windows Console API for real terminal I/O capture
//! - **Observation Engine** (`engine`): Orchestrates all providers and consumers

pub mod app_monitor;
pub mod browser;
pub mod clipboard;
pub mod correlation;
pub mod engine;
pub mod event;
pub mod event_bus;
pub mod file_observer;
pub mod guidance;
pub mod privacy;
pub mod provider;
pub mod screen_capture;
pub mod error_detector;
pub mod semantic_analyzer;
pub mod session_tracker;
pub mod shell;
pub mod terminal;
pub mod ai_guidance;
pub mod intent_analyzer;
#[cfg(windows)]
pub mod cdp_browser;
#[cfg(windows)]
pub mod console_output;
pub mod vision_analyzer;

#[cfg(test)]
mod tests;

// Re-export key types at crate level for convenience
pub use event::{EventType, ObservationEvent, ObservationPayload, ObservationStats, ProviderType};
pub use event_bus::EventBus;
pub use privacy::{ObservationMode, PrivacyConfig, PrivacyManager};
pub use provider::{
    ObservationProvider, ProviderConfig, ProviderRegistry, ProviderState, ProviderStatus,
};

// Re-export new public types
pub use correlation::{CorrelationEngine, CorrelationSet, CorrelationType};
pub use guidance::{GuidanceEngine, GuidanceSuggestion, GuidanceSeverity, GuidanceCategory};
pub use semantic_analyzer::{SemanticAnalyzer, CommandIntent, IntentCategory, AnalysisResult};
pub use session_tracker::{SessionTracker, SessionState, Suggestion};
pub use shell::{ShellObserver, ShellStatus, ShellCommand};
pub use intent_analyzer::{
    ActivityCategory, IntentAnalyzer, IntentSummary, IssueReport, IssueSeverity,
    UserActivity, UserIntent,
};

use engine::{ObservationEngine, ObservationEngineConfig};
use std::sync::OnceLock;

/// Global observation engine instance.
static OBSERVATION_ENGINE: OnceLock<std::sync::Arc<ObservationEngine>> = OnceLock::new();

/// Initialize the observation engine. Returns a reference to the global engine.
pub fn init_observation_engine() -> std::sync::Arc<ObservationEngine> {
    OBSERVATION_ENGINE.get_or_init(|| {
        std::sync::Arc::new(ObservationEngine::new(ObservationEngineConfig::default()))
    })
    .clone()
}

/// Get the global observation engine instance.
/// Returns `Some` if the engine has been initialized, `None` otherwise.
pub fn get_observation_engine() -> Option<std::sync::Arc<ObservationEngine>> {
    OBSERVATION_ENGINE.get().cloned()
}

/// Get the latest Vision analysis result from the global observation engine.
/// Returns `None` if the engine hasn't been initialized or no Vision result is available.
pub fn get_vision_result() -> Option<crate::vision_analyzer::VisionAnalysisResult> {
    get_observation_engine().and_then(|engine| engine.get_vision_result())
}