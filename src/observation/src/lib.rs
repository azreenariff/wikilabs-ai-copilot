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
//! - **Browser Provider** (`browser`): Browser context detection
//! - **Clipboard Provider** (`clipboard`): Clipboard content observation
//! - **File Provider** (`file_observer`): File open/edit observation
//! - **Screen Capture Provider** (`screen_capture`): Periodic screenshot capture
//! - **Observation Engine** (`engine`): Orchestrates all providers

pub mod app_monitor;
pub mod browser;
pub mod clipboard;
pub mod engine;
pub mod event;
pub mod event_bus;
pub mod file_observer;
pub mod privacy;
pub mod provider;
pub mod screen_capture;
pub mod terminal;

#[cfg(test)]
mod tests;

// Re-export key types at crate level for convenience
pub use event::{EventType, ObservationEvent, ObservationPayload, ObservationStats, ProviderType};
pub use event_bus::EventBus;
pub use privacy::{ObservationMode, PrivacyConfig, PrivacyManager};
pub use provider::{
    ObservationProvider, ProviderConfig, ProviderRegistry, ProviderState, ProviderStatus,
};
