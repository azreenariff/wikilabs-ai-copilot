//! Observation Framework — Privacy Controls
//!
//! Implements comprehensive privacy controls for the observation framework.
//! The engineer must have full control over observation at all times.
//!
//! Controls:
//! - Master enable/disable
//! - Per-provider enable/disable
//! - Pause observation (temporarily stop all)
//! - Resume observation (restart after pause)
//! - Observation indicator (UI-visible state)
//! - Configurable retention policies

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::event::ProviderType;

/// Master privacy control state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObservationMode {
    /// Observation is disabled globally.
    Disabled,
    /// Observation is active.
    Enabled,
    /// Observation is paused (temporarily suspended).
    Paused,
}

impl std::fmt::Display for ObservationMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ObservationMode::Disabled => write!(f, "disabled"),
            ObservationMode::Enabled => write!(f, "enabled"),
            ObservationMode::Paused => write!(f, "paused"),
        }
    }
}

/// Privacy configuration for the observation framework.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyConfig {
    /// Global master control.
    pub mode: ObservationMode,
    /// Per-provider overrides (when None, use mode default).
    pub provider_overrides: HashMap<ProviderType, bool>,
    /// Maximum age of event data (0 = unlimited).
    pub retention_days: u64,
    /// Whether to store clipboard content (default: false).
    pub store_clipboard_content: bool,
    /// Whether to store file content (default: false, metadata only).
    pub store_file_content: bool,
    /// Maximum screenshot retention days (0 = don't store).
    pub screenshot_retention_days: u64,
}

impl Default for PrivacyConfig {
    fn default() -> Self {
        Self {
            mode: ObservationMode::Enabled,
            provider_overrides: HashMap::new(),
            retention_days: 7,
            store_clipboard_content: false,
            store_file_content: false,
            screenshot_retention_days: 1,
        }
    }
}

impl PrivacyConfig {
    /// Check if a specific provider is allowed to observe.
    pub fn is_provider_allowed(&self, provider: &ProviderType) -> bool {
        match self.mode {
            ObservationMode::Disabled => false,
            ObservationMode::Paused => false,
            ObservationMode::Enabled => {
                match self.provider_overrides.get(provider) {
                    Some(&enabled) => enabled,
                    None => true, // Default: enabled
                }
            }
        }
    }

    /// Check if clipboard observation is allowed.
    pub fn allows_clipboard(&self) -> bool {
        matches!(self.mode, ObservationMode::Enabled) && self.store_clipboard_content
    }

    /// Check if file content observation is allowed.
    pub fn allows_file_content(&self) -> bool {
        matches!(self.mode, ObservationMode::Enabled) && self.store_file_content
    }

    /// Check if screenshots should be stored.
    pub fn allows_screenshots(&self) -> bool {
        self.screenshot_retention_days > 0
    }
}

/// Privacy indicator state — for UI display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyIndicator {
    /// Current observation mode.
    pub mode: ObservationMode,
    /// List of providers currently collecting data.
    pub active_providers: Vec<String>,
    /// Whether the observation indicator should be visible.
    pub indicator_visible: bool,
    /// Timestamp of last observation activity.
    pub last_activity: Option<DateTime<Utc>>,
}

impl PrivacyIndicator {
    pub fn new(
        mode: ObservationMode,
        active_providers: Vec<String>,
        indicator_visible: bool,
        last_activity: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            mode,
            active_providers,
            indicator_visible,
            last_activity,
        }
    }

    /// Create an indicator for a disabled state.
    pub fn disabled() -> Self {
        Self {
            mode: ObservationMode::Disabled,
            active_providers: Vec::new(),
            indicator_visible: false,
            last_activity: None,
        }
    }

    /// Check if any observation is currently happening.
    pub fn is_active(&self) -> bool {
        matches!(self.mode, ObservationMode::Enabled) && !self.active_providers.is_empty()
    }
}

/// Privacy control manager — manages all privacy settings.
pub struct PrivacyManager {
    config: Arc<Mutex<PrivacyConfig>>,
    indicator: Arc<Mutex<PrivacyIndicator>>,
}

impl PrivacyManager {
    pub fn new() -> Self {
        let config = Arc::new(Mutex::new(PrivacyConfig::default()));
        let indicator = Arc::new(Mutex::new(PrivacyIndicator::disabled()));
        Self { config, indicator }
    }

    /// Get the current privacy configuration.
    pub fn get_config(&self) -> PrivacyConfig {
        self.config.lock().unwrap().clone()
    }

    /// Update the privacy configuration.
    pub fn set_config(&self, config: PrivacyConfig) {
        {
            let mut cfg = self.config.lock().unwrap();
            *cfg = config.clone();
        }
        self.update_indicator();
    }

    /// Set the global observation mode.
    pub fn set_mode(&self, mode: ObservationMode) {
        {
            let mut cfg = self.config.lock().unwrap();
            cfg.mode = mode.clone();
        }
        self.update_indicator();
    }

    /// Enable or disable a specific provider.
    pub fn set_provider_enabled(&self, provider: ProviderType, enabled: bool) {
        {
            let mut cfg = self.config.lock().unwrap();
            if enabled {
                cfg.provider_overrides.remove(&provider);
            } else {
                cfg.provider_overrides.insert(provider, false);
            }
        }
        self.update_indicator();
    }

    /// Check if observation is allowed.
    pub fn is_observing(&self) -> bool {
        let cfg = self.config.lock().unwrap();
        cfg.mode == ObservationMode::Enabled
    }

    /// Check if a specific provider is allowed.
    pub fn is_provider_allowed(&self, provider: &ProviderType) -> bool {
        let cfg = self.config.lock().unwrap();
        cfg.is_provider_allowed(provider)
    }

    /// Update the privacy indicator state.
    fn update_indicator(&self) {
        let cfg = self.config.lock().unwrap();
        let (mode, active) = match &cfg.mode {
            ObservationMode::Disabled => (ObservationMode::Disabled, Vec::new()),
            ObservationMode::Paused => (ObservationMode::Paused, Vec::new()),
            ObservationMode::Enabled => {
                let active: Vec<String> = cfg
                    .provider_overrides
                    .iter()
                    .filter(|(_, &enabled)| enabled)
                    .map(|(p, _)| p.to_string())
                    .collect();
                if active.is_empty() {
                    // All providers enabled by default in enabled mode
                    (ObservationMode::Enabled, vec!["active_window".to_string(), "terminal".to_string(), "browser".to_string()])
                } else {
                    (ObservationMode::Enabled, active)
                }
            }
        };

        let mut indicator = self.indicator.lock().unwrap();
        indicator.mode = mode;
        indicator.active_providers = active.clone();
        indicator.indicator_visible = matches!(cfg.mode, ObservationMode::Enabled) && !active.is_empty();
    }

    /// Get the current privacy indicator.
    pub fn get_indicator(&self) -> PrivacyIndicator {
        self.indicator.lock().unwrap().clone()
    }

    /// Record last activity timestamp.
    pub fn record_activity(&self, timestamp: DateTime<Utc>) {
        let mut indicator = self.indicator.lock().unwrap();
        indicator.last_activity = Some(timestamp);
    }

    /// Enable clipboard observation.
    pub fn enable_clipboard(&self) {
        let mut cfg = self.config.lock().unwrap();
        cfg.store_clipboard_content = true;
    }

    /// Disable clipboard observation.
    pub fn disable_clipboard(&self) {
        let mut cfg = self.config.lock().unwrap();
        cfg.store_clipboard_content = false;
    }

    /// Enable file content observation.
    pub fn enable_file_content(&self) {
        let mut cfg = self.config.lock().unwrap();
        cfg.store_file_content = true;
    }

    /// Disable file content observation.
    pub fn disable_file_content(&self) {
        let mut cfg = self.config.lock().unwrap();
        cfg.store_file_content = false;
    }

    /// Set maximum retention days.
    pub fn set_retention_days(&self, days: u64) {
        let mut cfg = self.config.lock().unwrap();
        cfg.retention_days = days;
    }

    /// Set screenshot retention days.
    pub fn set_screenshot_retention(&self, days: u64) {
        let mut cfg = self.config.lock().unwrap();
        cfg.screenshot_retention_days = days;
    }

    /// Pause all observation.
    pub fn pause(&self) {
        self.set_mode(ObservationMode::Paused);
    }

    /// Resume observation.
    pub fn resume(&self) {
        self.set_mode(ObservationMode::Enabled);
    }

    /// Disable all observation.
    pub fn disable_all(&self) {
        self.set_mode(ObservationMode::Disabled);
    }

    /// Enable all observation (respecting per-provider overrides).
    pub fn enable_all(&self) {
        self.set_mode(ObservationMode::Enabled);
    }
}

impl Default for PrivacyManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for PrivacyManager {
    fn clone(&self) -> Self {
        Self {
            config: Arc::clone(&self.config),
            indicator: Arc::clone(&self.indicator),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_observation_mode_display() {
        assert_eq!(ObservationMode::Disabled.to_string(), "disabled");
        assert_eq!(ObservationMode::Enabled.to_string(), "enabled");
        assert_eq!(ObservationMode::Paused.to_string(), "paused");
    }

    #[test]
    fn test_privacy_config_defaults() {
        let config = PrivacyConfig::default();
        assert_eq!(config.mode, ObservationMode::Enabled);
        assert_eq!(config.retention_days, 7);
        assert!(!config.store_clipboard_content);
        assert!(!config.store_file_content);
        assert_eq!(config.screenshot_retention_days, 1);
    }

    #[test]
    fn test_provider_allowed_default() {
        let config = PrivacyConfig::default();
        assert!(config.is_provider_allowed(&ProviderType::ActiveWindow));
        assert!(config.is_provider_allowed(&ProviderType::Terminal));
        assert!(config.is_provider_allowed(&ProviderType::Browser));
    }

    #[test]
    fn test_provider_disabled_by_config() {
        let mut config = PrivacyConfig::default();
        config.provider_overrides.insert(ProviderType::Clipboard, false);
        assert!(config.is_provider_allowed(&ProviderType::ActiveWindow));
        assert!(!config.is_provider_allowed(&ProviderType::Clipboard));
    }

    #[test]
    fn test_mode_disabled_disallows_all() {
        let mut config = PrivacyConfig::default();
        config.mode = ObservationMode::Disabled;
        assert!(!config.is_provider_allowed(&ProviderType::ActiveWindow));
        assert!(!config.is_provider_allowed(&ProviderType::Clipboard));
    }

    #[test]
    fn test_mode_paused_disallows_all() {
        let mut config = PrivacyConfig::default();
        config.mode = ObservationMode::Paused;
        assert!(!config.is_provider_allowed(&ProviderType::ActiveWindow));
    }

    #[test]
    fn test_clipboard_allowed() {
        let config = PrivacyConfig::default();
        assert!(!config.allows_clipboard());

        let mut config = PrivacyConfig::default();
        config.store_clipboard_content = true;
        assert!(config.allows_clipboard());
    }

    #[test]
    fn test_file_content_allowed() {
        let config = PrivacyConfig::default();
        assert!(!config.allows_file_content());

        let mut config = PrivacyConfig::default();
        config.store_file_content = true;
        assert!(config.allows_file_content());
    }

    #[test]
    fn test_screenshots_allowed() {
        let config = PrivacyConfig::default();
        assert!(config.allows_screenshots());

        let mut config = PrivacyConfig::default();
        config.screenshot_retention_days = 0;
        assert!(!config.allows_screenshots());
    }

    #[test]
    fn test_privacy_indicator_disabled() {
        let indicator = PrivacyIndicator::disabled();
        assert!(!indicator.is_active());
        assert!(indicator.active_providers.is_empty());
        assert!(!indicator.indicator_visible);
    }

    #[test]
    fn test_privacy_indicator_active() {
        let indicator = PrivacyIndicator::new(
            ObservationMode::Enabled,
            vec!["active_window".to_string()],
            true,
            Some(Utc::now()),
        );
        assert!(indicator.is_active());
        assert!(indicator.indicator_visible);
        assert!(!indicator.active_providers.is_empty());
    }

    #[test]
    fn test_privacy_manager_defaults() {
        let manager = PrivacyManager::new();
        assert!(manager.is_observing());
        assert!(manager.is_provider_allowed(&ProviderType::ActiveWindow));
    }

    #[test]
    fn test_privacy_manager_disable() {
        let manager = PrivacyManager::new();
        manager.disable_all();
        assert!(!manager.is_observing());
        assert!(!manager.is_provider_allowed(&ProviderType::ActiveWindow));
    }

    #[test]
    fn test_privacy_manager_pause_resume() {
        let manager = PrivacyManager::new();
        assert!(manager.is_observing());

        manager.pause();
        assert!(!manager.is_observing());

        manager.resume();
        assert!(manager.is_observing());
    }

    #[test]
    fn test_privacy_manager_provider_toggle() {
        let manager = PrivacyManager::new();

        // Initially all providers allowed
        assert!(manager.is_provider_allowed(&ProviderType::Clipboard));

        // Disable clipboard
        manager.set_provider_enabled(ProviderType::Clipboard, false);
        assert!(!manager.is_provider_allowed(&ProviderType::Clipboard));

        // Other providers still allowed
        assert!(manager.is_provider_allowed(&ProviderType::ActiveWindow));
    }

    #[test]
    fn test_privacy_manager_clipboard_toggle() {
        let manager = PrivacyManager::new();
        assert!(!manager.get_config().allows_clipboard());

        manager.enable_clipboard();
        assert!(manager.get_config().allows_clipboard());

        manager.disable_clipboard();
        assert!(!manager.get_config().allows_clipboard());
    }

    #[test]
    fn test_privacy_manager_retention() {
        let manager = PrivacyManager::new();
        assert_eq!(manager.get_config().retention_days, 7);

        manager.set_retention_days(30);
        assert_eq!(manager.get_config().retention_days, 30);
    }

    #[test]
    fn test_privacy_manager_config_serialization() {
        let manager = PrivacyManager::new();
        let config = manager.get_config();

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: PrivacyConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.mode, ObservationMode::Enabled);
        assert_eq!(deserialized.retention_days, 7);
    }

    #[test]
    fn test_privacy_manager_clone() {
        let manager1 = PrivacyManager::new();
        manager1.disable_all();

        let manager2 = manager1.clone();
        // Clone shares state
        assert!(!manager2.is_observing());

        manager1.resume();
        // State is shared via Arc
        assert!(manager2.is_observing());
    }

    #[test]
    fn test_custom_provider_allowed() {
        let mut config = PrivacyConfig::default();
        assert!(config.is_provider_allowed(&ProviderType::Custom("my_provider".to_string())));

        config.provider_overrides.insert(
            ProviderType::Custom("my_provider".to_string()),
            false,
        );
        assert!(!config.is_provider_allowed(&ProviderType::Custom("my_provider".to_string())));
    }
}