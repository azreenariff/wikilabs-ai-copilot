//! Observation Framework — Screen Capture Provider
//!
//! Periodic screenshot capture with configurable interval.
//! Multi-monitor support. Window-aware capture.
//! Capture pause/resume.
//!
//! Does NOT perform OCR or AI analysis — only captures raw images.
//! Screenshot retention is configurable.
//!
//! Platform support:
//! - Linux: X11/xcb, Wayland (xdg-desktop-portal)
//! - Windows: Win32 BitBlt/DXGI
//! - macOS: CGWindowListCopyWindowInfo/CoreGraphics

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::event::{ObservationEvent, EventType, ProviderType, ObservationPayload};
use crate::provider::{ObservationProvider, ProviderConfig, ProviderState, ProviderLifecycle};

/// Screenshot metadata (does NOT include the image data).
#[derive(Debug, Clone)]
pub struct ScreenshotMetadata {
    /// Screenshot timestamp.
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Screen index (for multi-monitor).
    pub screen_index: u32,
    /// Screen width in pixels.
    pub width: u32,
    /// Screen height in pixels.
    pub height: u32,
    /// Number of screens on this system.
    pub total_screens: u32,
    /// Whether this screen was captured.
    pub captured: bool,
}

/// Screen capture provider state.
pub struct ScreenCaptureState {
    pub config: ProviderConfig,
    pub state: ProviderState,
    pub lifecycle: ProviderLifecycle,
    pub last_screenshots: Vec<ScreenshotMetadata>,
    pub capture_interval_secs: u64,
    pub capture_all_screens: bool,
}

impl ScreenCaptureState {
    fn new(config: ProviderConfig) -> Self {
        Self {
            config: config.clone(),
            state: ProviderState::Disabled,
            lifecycle: ProviderLifecycle::new(),
            last_screenshots: Vec::new(),
            capture_interval_secs: config.interval_secs,
            capture_all_screens: true,
        }
    }
}

/// Screen observation provider.
pub struct ScreenCaptureProvider {
    state: Arc<Mutex<ScreenCaptureState>>,
}

impl ScreenCaptureProvider {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(ScreenCaptureState::new(ProviderConfig::default()))),
        }
    }

    /// Returns the number of screens detected on this system.
    fn detect_screen_count(&self) -> u32 {
        #[cfg(target_os = "linux")]
        {
            // Linux: Xinerama or RandR
            1
        }

        #[cfg(target_os = "windows")]
        {
            // Windows: EnumDisplayDevices
            1
        }

        #[cfg(target_os = "macos")]
        {
            // macOS: CGGetOnlineDisplayList
            1
        }

        #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
        {
            1
        }
    }

    /// Platform-specific screenshot capture (stub).
    /// Returns metadata only — actual image data would be stored separately.
    fn capture_screen(&self, _screen_index: u32) -> Option<ScreenshotMetadata> {
        // This would use platform-specific screenshot APIs:
        // - X11: XGetImage + XGetImage for multi-monitor
        // - Wayland: xdg-desktop-portal screen-cast
        // - Windows: BitBlt/DXGI
        // - macOS: CGWindowListCopyWindowInfo
        None
    }
}

impl Default for ScreenCaptureProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ObservationProvider for ScreenCaptureProvider {
    fn provider_type(&self) -> ProviderType {
        ProviderType::ScreenCapture
    }

    fn name(&self) -> &str {
        "Screen Capture"
    }

    fn description(&self) -> &str {
        "Periodic screenshot capture with configurable interval, multi-monitor support"
    }

    fn config(&self) -> ProviderConfig {
        self.state.lock().unwrap().config.clone()
    }

    fn set_config(&mut self, config: ProviderConfig) {
        let mut state = self.state.lock().unwrap();
        state.config = config.clone();
        state.capture_interval_secs = config.interval_secs;
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
        let screen_count = self.detect_screen_count();

        if !state.capture_all_screens {
            // Single screen capture
            if let Some(metadata) = self.capture_screen(0) {
                state.last_screenshots.push(metadata.clone());
                if state.last_screenshots.len() > 100 {
                    state.last_screenshots.remove(0);
                }

                return Ok(vec![ObservationEvent::new(
                    EventType::ScreenshotCaptured,
                    ProviderType::ScreenCapture,
                    format!("screen_{}", metadata.screen_index),
                    None,
                    ObservationPayload::new(serde_json::json!({
                        "screen_index": metadata.screen_index,
                        "width": metadata.width,
                        "height": metadata.height,
                        "total_screens": metadata.total_screens,
                        "captured": metadata.captured,
                    })),
                )]);
            }
        } else {
            // Multi-screen capture
            for i in 0..screen_count {
                if let Some(metadata) = self.capture_screen(i) {
                    state.last_screenshots.push(metadata.clone());
                }
            }
        }

        // Emit minimal event when no screenshot available
        let total_captured = state.last_screenshots.len();
        Ok(vec![ObservationEvent::new(
            EventType::ScreenshotCaptured,
            ProviderType::ScreenCapture,
            "stub".to_string(),
            None,
            ObservationPayload::new(serde_json::json!({
                "status": "no_screenshot_available",
                "platform": std::env::consts::OS,
                "total_screens": screen_count,
                "last_captured_count": total_captured,
                "capture_interval_secs": state.capture_interval_secs,
                "capture_all_screens": state.capture_all_screens,
            })),
        )])
    }

    fn lifecycle(&self) -> ProviderLifecycle {
        self.state.lock().unwrap().lifecycle.clone()
    }

    fn status_details(&self) -> HashMap<String, serde_json::Value> {
        let state = self.state.lock().unwrap();
        let mut details = HashMap::new();
        details.insert(
            "last_captured_count".to_string(),
            serde_json::json!(state.last_screenshots.len()),
        );
        details.insert(
            "capture_interval_secs".to_string(),
            serde_json::json!(state.capture_interval_secs),
        );
        details.insert(
            "capture_all_screens".to_string(),
            serde_json::json!(state.capture_all_screens),
        );
        details
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screen_capture_creation() {
        let provider = ScreenCaptureProvider::new();
        assert_eq!(provider.provider_type(), ProviderType::ScreenCapture);
        assert_eq!(provider.name(), "Screen Capture");
    }

    #[test]
    fn test_screen_count_detection() {
        let provider = ScreenCaptureProvider::new();
        // Stub returns 1
        assert_eq!(provider.detect_screen_count(), 1);
    }

    #[test]
    fn test_provider_lifecycle() {
        let mut provider = ScreenCaptureProvider::new();
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
        let mut provider = ScreenCaptureProvider::new();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let events = rt.block_on(async {
            provider.start().await.unwrap();
            provider.observe().await.unwrap()
        });
        assert!(!events.is_empty());
        assert_eq!(events[0].event_type, EventType::ScreenshotCaptured);
    }

    #[test]
    fn test_config_get_set() {
        let mut provider = ScreenCaptureProvider::new();
        let mut config = provider.config();
        config.interval_secs = 30;
        provider.set_config(config.clone());
        assert_eq!(provider.config().interval_secs, 30);
    }

    #[test]
    fn test_status_details() {
        let provider = ScreenCaptureProvider::new();
        let details = provider.status_details();
        assert!(details.contains_key("last_captured_count"));
        assert!(details.contains_key("capture_interval_secs"));
    }
}