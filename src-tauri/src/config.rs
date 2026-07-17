use serde::{Deserialize, Serialize};
use std::sync::RwLock;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub ai_provider: AiProviderConfig,
    pub theme: String,
    pub log_level: String,
    pub privacy_mode: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            ai_provider: AiProviderConfig::default(),
            theme: "dark".to_string(),
            log_level: "info".to_string(),
            privacy_mode: false,
        }
    }
}

/// Thread-safe settings store using interior mutability.
pub struct AppSettingsStore {
    inner: RwLock<AppSettings>,
}

impl Clone for AppSettingsStore {
    fn clone(&self) -> Self {
        Self {
            inner: RwLock::new(self.inner.read().unwrap().clone()),
        }
    }
}

impl AppSettingsStore {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(AppSettings::default()),
        }
    }

    pub fn load(&self) -> AppSettings {
        self.inner.read().unwrap().clone()
    }

    pub fn save(&self, settings: AppSettings) {
        *self.inner.write().unwrap() = settings;
    }
}

impl Default for AppSettingsStore {
    fn default() -> Self {
        Self::new()
    }
}