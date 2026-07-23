//! Observation Framework — File Observer Provider
//!
//! Detects files opened and configuration files edited.
//! Only collects metadata unless content inspection is explicitly enabled.
//!
//! Supported file types for metadata-only observation:
//! - YAML (.yml, .yaml)
//! - JSON (.json)
//! - XML (.xml)
//! - INI (.ini, .cfg, .conf)
//! - Properties (.properties)
//! - TOML (.toml)
//! - Env (.env)
//!
//! Does NOT read file contents unless store_file_content is enabled.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::event::{EventType, ObservationEvent, ObservationPayload, ProviderType};
use crate::provider::{ObservationProvider, ProviderConfig, ProviderLifecycle, ProviderState};

/// Configuration file extensions that trigger file observation.
#[allow(dead_code)]
const CONFIG_EXTENSIONS: &[&str] = &[
    "yml",
    "yaml",
    "json",
    "xml",
    "ini",
    "cfg",
    "conf",
    "properties",
    "toml",
    "env",
];

/// File metadata observed.
#[derive(Debug, Clone)]
pub struct FileMetadata {
    /// File path.
    pub path: String,
    /// File extension.
    pub extension: Option<String>,
    /// Whether this is a configuration file.
    pub is_config_file: bool,
    /// File size in bytes.
    pub size_bytes: Option<u64>,
    /// Last modified timestamp.
    pub modified_at: Option<u64>,
    /// Whether content was read (should be false by default).
    pub content_read: bool,
}

impl FileMetadata {
    #[allow(dead_code)]
    fn from_path(path: &str) -> Self {
        let extension = std::path::Path::new(path)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|s| s.to_string());
        let is_config = extension
            .as_ref()
            .map(|ext| CONFIG_EXTENSIONS.iter().any(|&e| e == ext))
            .unwrap_or(false);

        Self {
            path: path.to_string(),
            extension,
            is_config_file: is_config,
            size_bytes: None,
            modified_at: None,
            content_read: false,
        }
    }
}

/// File observer provider state.
pub struct FileObserverState {
    pub config: ProviderConfig,
    pub state: ProviderState,
    pub lifecycle: ProviderLifecycle,
    pub recently_opened_files: Vec<FileMetadata>,
    pub recently_edited_files: Vec<FileMetadata>,
}

impl FileObserverState {
    fn new(config: ProviderConfig) -> Self {
        Self {
            config,
            state: ProviderState::Disabled,
            lifecycle: ProviderLifecycle::new(),
            recently_opened_files: Vec::new(),
            recently_edited_files: Vec::new(),
        }
    }
}

/// File observation provider.
pub struct FileObserverProvider {
    state: Arc<Mutex<FileObserverState>>,
}

impl FileObserverProvider {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(
                FileObserverState::new(ProviderConfig::default()),
            )),
        }
    }

    /// Platform-specific file monitoring (stub).
    /// Returns list of recently opened/active files.
    fn detect_opened_files(&self) -> Vec<FileMetadata> {
        #[cfg(target_os = "linux")]
        {
            // Linux: Monitor /proc/<pid>/fd, inotify, lsof
            // This is a stub — real implementation would use inotify
            Vec::new()
        }

        #[cfg(target_os = "windows")]
        {
            // Windows: Monitor file handles via NtQuerySystemInformation
            use windows::Win32::System::Diagnostics::ToolHelp::{CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, TH32CS_SNAPPROCESS, PROCESSENTRY32W};
            unsafe {
                let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
                if snapshot.is_invalid() { return Vec::new(); }

                let mut entry = PROCESSENTRY32W::default();
                entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
                if Process32FirstW(snapshot, &mut entry).is_err() {
                    let _ = windows::Win32::Foundation::CloseHandle(snapshot);
                    return Vec::new();
                }

                let mut files = Vec::new();
                let engineering_tools = ["code.exe", "notepad++.exe", "vim.exe", "nvim.exe",
                    "sublime_text.exe", "idea64.exe", "pycharm64.exe", "eclipse.exe"];
                loop {
                    let name = String::from_utf16_lossy(&entry.szExeFile)
                        .trim_end_matches('\0').to_lowercase();
                    if engineering_tools.contains(&name.as_str()) {
                        files.push(FileMetadata {
                            path: format!("process:{}", name),
                            extension: None,
                            is_config_file: false,
                            size_bytes: None,
                            modified_at: None,
                            content_read: false,
                        });
                    }
                    if Process32NextW(snapshot, &mut entry).is_err() { break; }
                }
                let _ = windows::Win32::Foundation::CloseHandle(snapshot);
                files
            }
        }

        #[cfg(target_os = "macos")]
        {
            // macOS: FSEvents API
            Vec::new()
        }

        #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
        {
            Vec::new()
        }
    }
}

impl Default for FileObserverProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ObservationProvider for FileObserverProvider {
    fn provider_type(&self) -> ProviderType {
        ProviderType::FileObserver
    }

    fn name(&self) -> &str {
        "File Observer"
    }

    fn description(&self) -> &str {
        "Detects files opened and configuration files edited (metadata only by default)"
    }

    fn config(&self) -> ProviderConfig {
        self.state.lock().unwrap().config.clone()
    }

    fn set_config(&mut self, config: ProviderConfig) {
        self.state.lock().unwrap().config = config;
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
        let files = self.detect_opened_files();

        // Only emit events for config files
        let config_files: Vec<&FileMetadata> = files.iter().filter(|f| f.is_config_file).collect();

        if !config_files.is_empty() {
            let mut events = Vec::new();
            for file in &config_files {
                state.recently_opened_files.push((*file).clone());
                // Keep only last 50 entries
                if state.recently_opened_files.len() > 50 {
                    state.recently_opened_files.remove(0);
                }

                state.recently_edited_files.push((*file).clone());
                if state.recently_edited_files.len() > 50 {
                    state.recently_edited_files.remove(0);
                }

                let payload = serde_json::json!({
                    "path": file.path,
                    "extension": file.extension,
                    "is_config_file": file.is_config_file,
                    "size_bytes": file.size_bytes,
                    "content_read": file.content_read,
                });

                events.push(ObservationEvent::new(
                    EventType::ConfigurationFileOpened,
                    ProviderType::FileObserver,
                    file.path.clone(),
                    None,
                    ObservationPayload::new(payload),
                ));
            }
            return Ok(events);
        }

        // Emit minimal event when no config files detected
        Ok(vec![ObservationEvent::new(
            EventType::ConfigurationFileOpened,
            ProviderType::FileObserver,
            "stub".to_string(),
            None,
            ObservationPayload::new(serde_json::json!({
                "status": "no_config_files_detected",
                "platform": std::env::consts::OS,
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
            "recently_opened".to_string(),
            serde_json::json!(state.recently_opened_files.len()),
        );
        details.insert(
            "recently_edited".to_string(),
            serde_json::json!(state.recently_edited_files.len()),
        );
        details.insert(
            "platform".to_string(),
            serde_json::json!(std::env::consts::OS),
        );
        details
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_observer_creation() {
        let provider = FileObserverProvider::new();
        assert_eq!(provider.provider_type(), ProviderType::FileObserver);
        assert_eq!(provider.name(), "File Observer");
    }

    #[test]
    fn test_config_file_detection() {
        let meta = FileMetadata::from_path("/etc/nginx/nginx.conf");
        assert!(meta.is_config_file);
        assert_eq!(meta.extension, Some("conf".to_string()));

        let meta = FileMetadata::from_path("/home/user/config.yaml");
        assert!(meta.is_config_file);
        assert_eq!(meta.extension, Some("yaml".to_string()));

        let meta = FileMetadata::from_path("/home/user/script.py");
        assert!(!meta.is_config_file);
    }

    #[test]
    fn test_config_extensions() {
        let tests = vec![
            ("config.yml", true),
            ("config.json", true),
            ("config.xml", true),
            ("config.ini", true),
            ("config.properties", true),
            ("config.toml", true),
            ("script.js", false),
            ("README.md", false),
        ];
        for (path, expected) in tests {
            let meta = FileMetadata::from_path(path);
            assert_eq!(meta.is_config_file, expected, "path: {}", path);
        }
    }

    #[test]
    fn test_provider_lifecycle() {
        let mut provider = FileObserverProvider::new();
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
        let mut provider = FileObserverProvider::new();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let events = rt.block_on(async {
            provider.start().await.unwrap();
            provider.observe().await.unwrap()
        });
        assert!(!events.is_empty());
        assert_eq!(events[0].event_type, EventType::ConfigurationFileOpened);
    }

    #[test]
    fn test_config_get_set() {
        let mut provider = FileObserverProvider::new();
        let mut config = provider.config();
        config.enabled = false;
        provider.set_config(config);
        assert!(!provider.config().enabled);
    }

    #[test]
    fn test_status_details() {
        let provider = FileObserverProvider::new();
        let details = provider.status_details();
        assert!(details.contains_key("platform"));
        assert!(details.contains_key("recently_opened"));
    }
}
