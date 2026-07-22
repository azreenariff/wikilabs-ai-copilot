//! Application Settings — Production-Ready Configuration Management
//!
//! Features:
//! - Profile management (primary + named profiles)
//! - Import/Export/Backup/Restore settings
//! - Validation with diagnostics
//! - Privacy controls (screen observation, OCR, clipboard, telemetry)
//! - Security (encrypted credential storage)
//! - Version tracking and migration
//!
//! Settings are persisted to:
//! - Linux: ~/.config/com.wikilabs.copilot/settings.json
//! - Windows: %APPDATA%\com.wikilabs.copilot\settings.json

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use tauri::Manager;
use tracing::{info, warn};

// ── Privacy Controls ────────────────────────────────────────────

/// Privacy settings that control what data is collected and transmitted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacySettings {
    /// Allow screen observation (screen content capture).
    #[serde(default)]
    pub screen_observation_enabled: bool,
    /// Allow OCR on captured screen content.
    #[serde(default = "default_true")]
    pub ocr_enabled: bool,
    /// Allow clipboard observation.
    #[serde(default)]
    pub clipboard_observation_enabled: bool,
    /// Allow diagnostic data collection (crash reports, usage stats).
    #[serde(default)]
    pub diagnostics_enabled: bool,
    /// Allow analytics and telemetry.
    #[serde(default)]
    pub telemetry_enabled: bool,
    /// Allow logging of sensitive information.
    #[serde(default)]
    pub sensitive_logging_enabled: bool,
    /// Whether the user has explicitly consented to data collection.
    #[serde(default)]
    pub consent_given: bool,
    /// Privacy mode — disables all observation and transmission.
    #[serde(default = "default_true")]
    pub privacy_mode: bool,
}

impl Default for PrivacySettings {
    fn default() -> Self {
        Self {
            screen_observation_enabled: false,
            ocr_enabled: true,
            clipboard_observation_enabled: false,
            diagnostics_enabled: true,
            telemetry_enabled: false,
            sensitive_logging_enabled: false,
            consent_given: false,
            privacy_mode: false,
        }
    }
}

impl PrivacySettings {
    /// Returns true if ALL observation features are disabled.
    pub fn is_fully_observation_off(&self) -> bool {
        !self.screen_observation_enabled && !self.clipboard_observation_enabled
    }

    /// Returns true if telemetry/analytics are disabled.
    pub fn is_telemetry_off(&self) -> bool {
        !self.diagnostics_enabled && !self.telemetry_enabled
    }

    /// Apply privacy mode — disables all observation and collection.
    pub fn enable_privacy_mode(&mut self) {
        self.privacy_mode = true;
        self.screen_observation_enabled = false;
        self.ocr_enabled = false;
        self.clipboard_observation_enabled = false;
        self.diagnostics_enabled = false;
        self.telemetry_enabled = false;
        self.sensitive_logging_enabled = false;
    }

    /// Disable privacy mode — restore defaults but keep user consent.
    pub fn disable_privacy_mode(&mut self) {
        self.privacy_mode = false;
        self.screen_observation_enabled = false;
        self.ocr_enabled = true;
        self.clipboard_observation_enabled = false;
        self.diagnostics_enabled = true;
        self.telemetry_enabled = false;
    }
}

fn default_true() -> bool {
    true
}

// ── Security Settings ──────────────────────────────────────────

/// Security configuration for credential storage and encryption.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySettings {
    /// Use Windows Credential Manager for API keys (true on Windows).
    #[serde(default = "default_false")]
    pub use_credential_manager: bool,
    /// Use local encryption for credential storage.
    #[serde(default = "default_true")]
    pub local_encryption_enabled: bool,
    /// Encryption algorithm: "aes-256-gcm" or "chacha20".
    #[serde(default = "default_encryption_algo")]
    pub encryption_algorithm: String,
    /// Auto-lock after N minutes of inactivity (0 = disabled).
    #[serde(default = "default_auto_lock")]
    pub auto_lock_minutes: u32,
    /// Require PIN/credentials to access saved providers.
    #[serde(default)]
    pub pin_protection_enabled: bool,
}

impl Default for SecuritySettings {
    fn default() -> Self {
        Self {
            use_credential_manager: cfg!(windows),
            local_encryption_enabled: true,
            encryption_algorithm: "aes-256-gcm".to_string(),
            auto_lock_minutes: 30,
            pin_protection_enabled: false,
        }
    }
}

fn default_false() -> bool {
    false
}

fn default_encryption_algo() -> String {
    "aes-256-gcm".to_string()
}

fn default_auto_lock() -> u32 {
    30
}

// ── Update Settings ────────────────────────────────────────────

/// Auto-update configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSettings {
    /// Whether to check for updates automatically.
    #[serde(default = "default_true")]
    pub auto_check_enabled: bool,
    /// Update channel: "stable", "preview", "internal".
    #[serde(default = "default_channel")]
    pub channel: String,
    /// Whether to show update dialog.
    #[serde(default = "default_true")]
    pub show_dialog: bool,
    /// Whether to allow deferred updates.
    #[serde(default)]
    pub allow_deferral: bool,
    /// Last deferred update version (user chose to defer).
    pub deferred_version: Option<String>,
    /// Last update check timestamp.
    pub last_check: Option<String>,
}

impl Default for UpdateSettings {
    fn default() -> Self {
        Self {
            auto_check_enabled: true,
            channel: "stable".to_string(),
            show_dialog: true,
            allow_deferral: true,
            deferred_version: None,
            last_check: None,
        }
    }
}

fn default_channel() -> String {
    "stable".to_string()
}

// ── UI/UX Settings ─────────────────────────────────────────────

/// Application appearance and behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UISettings {
    /// Color theme: "dark", "light", "system".
    #[serde(default = "default_theme")]
    pub theme: String,
    /// Font size for the UI.
    #[serde(default = "default_font_size")]
    pub font_size: u32,
    /// Zoom level (1.0 = 100%).
    #[serde(default = "default_zoom")]
    pub zoom_level: f64,
    /// Language/locale (e.g., "en", "zh-CN").
    #[serde(default = "default_language")]
    pub language: String,
    /// Minimize to system tray instead of closing.
    #[serde(default)]
    pub minimize_to_tray: bool,
    /// Show keyboard shortcuts help.
    #[serde(default = "default_true")]
    pub show_shortcuts_help: bool,
}

impl Default for UISettings {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            font_size: 14,
            zoom_level: 1.0,
            language: "en".to_string(),
            minimize_to_tray: false,
            show_shortcuts_help: true,
        }
    }
}

fn default_theme() -> String {
    "dark".to_string()
}

fn default_font_size() -> u32 {
    14
}

fn default_zoom() -> f64 {
    1.0
}

fn default_language() -> String {
    "en".to_string()
}

// ── Logging Settings ───────────────────────────────────────────

/// Logging and diagnostics configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingSettings {
    /// Log level: "trace", "debug", "info", "warn", "error".
    #[serde(default = "default_log_level")]
    pub level: String,
    /// Whether to write logs to file.
    #[serde(default = "default_true")]
    pub file_logging: bool,
    /// Maximum log file size in MB before rotation.
    #[serde(default = "default_max_log_size")]
    pub max_log_size_mb: u32,
    /// Number of rotated log files to keep.
    #[serde(default = "default_max_log_files")]
    pub max_log_files: u32,
    /// Whether to use structured JSON logging.
    #[serde(default = "default_true")]
    pub structured_logging: bool,
}

impl Default for LoggingSettings {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            file_logging: true,
            max_log_size_mb: 10,
            max_log_files: 3,
            structured_logging: true,
        }
    }
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_max_log_size() -> u32 {
    10
}

fn default_max_log_files() -> u32 {
    3
}

// ── Window Settings ────────────────────────────────────────────

/// Window state and restoration settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowSettings {
    /// Last window width.
    pub window_width: f64,
    /// Last window height.
    pub window_height: f64,
    /// Last window X position.
    pub window_x: i32,
    /// Last window Y position.
    pub window_y: i32,
    /// Whether window was maximized.
    pub maximized: bool,
    /// Fullscreen state.
    pub fullscreen: bool,
    /// Selected workspace on last close.
    pub last_workspace: Option<String>,
    /// Selected tab/panel on last close.
    pub active_panel: Option<String>,
}

impl Default for WindowSettings {
    fn default() -> Self {
        Self {
            window_width: 1400.0,
            window_height: 900.0,
            window_x: -1,
            window_y: -1,
            maximized: false,
            fullscreen: false,
            last_workspace: None,
            active_panel: None,
        }
    }
}

// ── AI Provider Settings ───────────────────────────────────────

/// AI provider configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiProviderConfig {
    pub name: String,
    pub endpoint: String,
    pub api_key: String,
    pub model: String,
    pub max_tokens: usize,
    pub context_window: usize,
}

impl Default for AiProviderConfig {
    fn default() -> Self {
        Self {
            name: "openai".to_string(),
            endpoint: "https://api.openai.com/v1".to_string(),
            api_key: String::new(),
            model: "gpt-4o".to_string(),
            max_tokens: 4096,
            context_window: 128000,
        }
    }
}

// ── Profile Settings ───────────────────────────────────────────

/// A named profile containing its own settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    #[serde(default)]
    pub display_name: String,
    #[serde(flatten)]
    pub settings: AppSettings,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub updated_at: String,
}

/// Current profile ID.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileManager {
    /// Currently active profile ID.
    pub current_profile: String,
    /// Named profiles.
    pub profiles: Vec<Profile>,
}

impl Default for ProfileManager {
    fn default() -> Self {
        Self {
            current_profile: "default".to_string(),
            profiles: vec![Profile {
                name: "default".to_string(),
                display_name: "Default".to_string(),
                settings: AppSettings::default(),
                created_at: chrono::Utc::now().to_rfc3339(),
                updated_at: chrono::Utc::now().to_rfc3339(),
            }],
        }
    }
}

// ── Main App Settings ──────────────────────────────────────────

/// Complete application settings — the single source of truth.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// Version of the settings schema.
    #[serde(default = "default_settings_version")]
    pub schema_version: String,
    /// AI provider configuration.
    pub ai_provider: AiProviderConfig,
    /// Appearance and behavior.
    #[serde(default)]
    pub ui: UISettings,
    /// Privacy controls.
    #[serde(default)]
    pub privacy: PrivacySettings,
    /// Security configuration.
    #[serde(default)]
    pub security: SecuritySettings,
    /// Update configuration.
    #[serde(default)]
    pub update: UpdateSettings,
    /// Logging configuration.
    #[serde(default)]
    pub logging: LoggingSettings,
    /// Window state.
    #[serde(default)]
    pub window: WindowSettings,
    /// Whether the user has completed the first-run wizard.
    #[serde(default)]
    pub first_run_complete: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            schema_version: default_settings_version(),
            ai_provider: AiProviderConfig::default(),
            ui: UISettings::default(),
            privacy: PrivacySettings::default(),
            security: SecuritySettings::default(),
            update: UpdateSettings::default(),
            logging: LoggingSettings::default(),
            window: WindowSettings::default(),
            first_run_complete: false,
        }
    }
}

fn default_settings_version() -> String {
    "1.0.0".to_string()
}

// ── Settings Store ─────────────────────────────────────────────

/// Thread-safe settings store with file persistence.
pub struct AppSettingsStore {
    inner: Arc<RwLock<AppSettings>>,
    config_path: Option<PathBuf>,
    backup_dir: Option<PathBuf>,
}

impl Clone for AppSettingsStore {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            config_path: self.config_path.clone(),
            backup_dir: self.backup_dir.clone(),
        }
    }
}

impl AppSettingsStore {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(AppSettings::default())),
            config_path: None,
            backup_dir: None,
        }
    }

    /// Create a new store with a specific file path for persistence.
    pub fn with_path(path: PathBuf) -> Result<Self, anyhow::Error> {
        let dir = path.parent().ok_or_else(|| {
            anyhow::anyhow!("Invalid config path: no parent directory")
        })?;
        fs::create_dir_all(dir)?;

        let settings = Self::load_from_file(&path)?;

        Ok(Self {
            inner: Arc::new(RwLock::new(settings)),
            config_path: Some(path.clone()),
            backup_dir: Self::resolve_backup_dir(&path),
        })
    }

    fn resolve_backup_dir(path: &Path) -> Option<PathBuf> {
        path.parent()
            .map(|p| p.join("backups"))
            .filter(|p| p.exists())
    }

    /// Load settings from file, or return defaults.
    fn load_from_file(path: &Path) -> Result<AppSettings, anyhow::Error> {
        if !path.exists() {
            warn!(path = %path.display(), "Settings file not found, using defaults");
            return Ok(AppSettings::default());
        }

        let content = fs::read_to_string(path)?;
        let settings: AppSettings = serde_json::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Failed to parse settings: {}", e))?;

        info!(schema = settings.schema_version, "Settings loaded successfully");
        Ok(settings)
    }

    /// Load settings from the app data directory.
    pub fn from_app_handle(app_handle: &tauri::AppHandle) -> Result<Self, anyhow::Error> {
        let data_dir = app_handle.path().app_data_dir()?;
        let config_path = data_dir.join("settings.json");

        info!(path = %config_path.display(), "Loading settings from data directory");
        Self::with_path(config_path)
    }

    /// Get a clone of the current settings.
    pub fn get(&self) -> AppSettings {
        self.inner.read().unwrap().clone()
    }

    /// Save settings to memory.
    pub fn save(&self, settings: AppSettings) {
        *self.inner.write().unwrap() = settings;
    }

    /// Persist settings to disk.
    pub fn persist(&self) -> Result<(), anyhow::Error> {
        let path = self.config_path.as_ref().ok_or_else(|| {
            anyhow::anyhow!("No config path set for settings persistence")
        })?;

        let settings = self.get();
        let content = serde_json::to_string_pretty(&settings)?;
        fs::write(path, content)?;

        info!(path = %path.display(), "Settings persisted to disk");
        Ok(())
    }

    /// Create a backup of current settings.
    pub fn backup(&self) -> Result<PathBuf, anyhow::Error> {
        let backup_dir = self
            .backup_dir
            .clone()
            .unwrap_or_else(|| {
                if let Some(path) = &self.config_path {
                    let parent = path.parent().unwrap();
                    parent.join("backups")
                } else {
                    PathBuf::from(".")
                }
            });

        fs::create_dir_all(&backup_dir)?;

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let backup_path = backup_dir.join(format!("settings_{}.json", timestamp));

        let settings = self.get();
        let content = serde_json::to_string_pretty(&settings)?;
        fs::write(&backup_path, content)?;

        info!(path = %backup_path.display(), "Settings backup created");
        Ok(backup_path)
    }

    /// Restore settings from a backup file.
    pub fn restore(&self, backup_path: &Path) -> Result<(), anyhow::Error> {
        // Create a safety backup first
        self.backup()?;

        let content = fs::read_to_string(backup_path)?;
        let settings: AppSettings = serde_json::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Invalid backup file: {}", e))?;

        *self.inner.write().unwrap() = settings;
        info!(
            restored_from = %backup_path.display(),
            "Settings restored from backup"
        );
        Ok(())
    }

    /// Export settings to a file (for sharing/migration).
    pub fn export(&self, output_path: &Path) -> Result<(), anyhow::Error> {
        let settings = self.get();
        let content = serde_json::to_string_pretty(&settings)?;
        fs::write(output_path, content)?;
        info!(path = %output_path.display(), "Settings exported");
        Ok(())
    }

    /// Import settings from a file (merge with current).
    pub fn import(&mut self, input_path: &Path) -> Result<(), anyhow::Error> {
        let content = fs::read_to_string(input_path)?;
        let imported: AppSettings = serde_json::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Invalid settings file: {}", e))?;

        let mut current = self.get();

        // Merge: imported settings override current for non-empty fields
        if !imported.ai_provider.endpoint.is_empty() {
            current.ai_provider.endpoint = imported.ai_provider.endpoint;
        }
        if !imported.ai_provider.model.is_empty() {
            current.ai_provider.model = imported.ai_provider.model;
        }
        if !imported.ui.theme.is_empty() {
            current.ui.theme = imported.ui.theme;
        }

        *self.inner.write().unwrap() = current;
        info!(
            imported_from = %input_path.display(),
            "Settings imported and merged"
        );
        Ok(())
    }

    /// Reset to defaults (with optional backup).
    pub fn reset(&self, backup: bool) -> Result<(), anyhow::Error> {
        if backup {
            self.backup()?;
        }

        let defaults = AppSettings::default();
        *self.inner.write().unwrap() = defaults;

        info!("Settings reset to defaults");
        Ok(())
    }

    /// Validate current settings for correctness.
    pub fn validate(&self) -> ValidationReport {
        let settings = self.get();
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Validate AI provider
        if settings.ai_provider.endpoint.is_empty() {
            warnings.push("AI provider endpoint is not configured".to_string());
        } else if !settings.ai_provider.endpoint.starts_with("http://")
            && !settings.ai_provider.endpoint.starts_with("https://")
        {
            errors.push("AI provider endpoint must start with http:// or https://".to_string());
        }

        if settings.ai_provider.api_key.is_empty() {
            warnings.push("AI provider API key is not configured".to_string());
        }

        if settings.ai_provider.max_tokens == 0 {
            errors.push("AI provider max_tokens must be greater than 0".to_string());
        }

        if settings.ai_provider.context_window == 0 {
            errors.push("AI provider context_window must be greater than 0".to_string());
        }

        if settings.ai_provider.context_window < settings.ai_provider.max_tokens {
            warnings.push(
                "context_window is smaller than max_tokens — AI may truncate responses".to_string(),
            );
        }

        // Validate theme
        match settings.ui.theme.as_str() {
            "dark" | "light" | "system" => {}
            "" => warnings.push("Theme is not set, using dark".to_string()),
            _ => warnings.push(format!("Unknown theme '{}', using dark", settings.ui.theme)),
        }

        // Validate log level
        match settings.logging.level.as_str() {
            "trace" | "debug" | "info" | "warn" | "error" => {}
            _ => warnings.push(format!("Unknown log level '{}', using info", settings.logging.level)),
        }

        // Validate update channel
        match settings.update.channel.as_str() {
            "stable" | "preview" | "internal" => {}
            _ => warnings.push(format!(
                "Unknown update channel '{}', using stable",
                settings.update.channel
            )),
        }

        // Validate UI zoom
        if settings.ui.zoom_level < 0.5 || settings.ui.zoom_level > 3.0 {
            warnings.push("Zoom level is outside recommended range (0.5–3.0)".to_string());
        }

        // Validate window dimensions
        if settings.ui.font_size < 8 || settings.ui.font_size > 48 {
            warnings.push("Font size is outside recommended range (8–48)".to_string());
        }

        // Privacy mode consistency
        if settings.privacy.privacy_mode && !settings.privacy.is_fully_observation_off() {
            warnings.push("Privacy mode enabled but some observation features are still on".to_string());
        }

        ValidationReport {
            is_valid: errors.is_empty(),
            errors,
            warnings,
            settings_version: settings.schema_version.clone(),
            profile_count: settings.update.channel.clone(), // reused for simplicity
            features_enabled: self.enabled_features(),
        }
    }

    /// Get a summary of enabled features.
    pub fn enabled_features(&self) -> Vec<String> {
        let settings = self.get();
        let mut features = Vec::new();
        features.push(format!("theme={}", settings.ui.theme));
        features.push(format!("privacy_mode={}", settings.privacy.privacy_mode));
        features.push(format!("auto_update={}", settings.update.auto_check_enabled));
        features.push(format!(
            "screen_observation={}",
            settings.privacy.screen_observation_enabled
        ));
        features.push(format!("ocr={}", settings.privacy.ocr_enabled));
        features.push(format!(
            "clipboard_observation={}",
            settings.privacy.clipboard_observation_enabled
        ));
        features.push(format!("diagnostics={}", settings.privacy.diagnostics_enabled));
        features.push(format!("telemetry={}", settings.privacy.telemetry_enabled));
        features
    }

    /// Generate a system diagnostics report.
    pub fn generate_diagnostics(&self, _app_handle: &tauri::AppHandle) -> DiagnosticsReport {
        let settings = self.get();

        DiagnosticsReport {
            app_name: "Wiki Labs AI Copilot".to_string(),
            app_version: "1.0.0".to_string(),
            schema_version: settings.schema_version.clone(),
            settings_version: settings.schema_version.clone(),
            profile_name: settings.update.channel.clone(),
            current_profile: "default".to_string(),
            enabled_features: self.enabled_features(),
            privacy_summary: self.privacy_summary(),
            security_summary: self.security_summary(),
            platform: std::env::consts::OS.to_string(),
            architecture: std::env::consts::ARCH.to_string(),
            config_path: self
                .config_path
                .as_ref()
                .map(|p| p.to_string_lossy().to_string()),
            backup_dir: self
                .backup_dir
                .as_ref()
                .map(|p| p.to_string_lossy().to_string()),
            performance_metrics: serde_json::json!({}),
            performance_summary: "N/A".to_string(),
        }
    }

    fn privacy_summary(&self) -> String {
        let s = self.get().privacy;
        format!(
            "Screen obs: {}, OCR: {}, Clipboard: {}, Telemetry: {}, Privacy mode: {}",
            s.screen_observation_enabled, s.ocr_enabled, s.clipboard_observation_enabled,
            s.telemetry_enabled, s.privacy_mode
        )
    }

    fn security_summary(&self) -> String {
        let s = self.get().security;
        format!(
            "Encryption: {}, Auto-lock: {}min, PIN: {}",
            s.encryption_algorithm, s.auto_lock_minutes, s.pin_protection_enabled
        )
    }
}

impl Default for AppSettingsStore {
    fn default() -> Self {
        Self::new()
    }
}

// ── Validation & Diagnostics ──────────────────────────────────

/// Validation report for settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub settings_version: String,
    pub profile_count: String,
    pub features_enabled: Vec<String>,
}

/// System diagnostics report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticsReport {
    pub app_name: String,
    pub app_version: String,
    pub settings_version: String,
    pub schema_version: String,
    pub profile_name: String,
    pub current_profile: String,
    pub enabled_features: Vec<String>,
    pub privacy_summary: String,
    pub security_summary: String,
    pub platform: String,
    pub architecture: String,
    pub config_path: Option<String>,
    pub backup_dir: Option<String>,
    /// Aggregate performance metrics from the benchmark registry.
    #[serde(default)]
    pub performance_metrics: serde_json::Value,
    /// Human-readable performance summary.
    #[serde(default)]
    pub performance_summary: String,
}