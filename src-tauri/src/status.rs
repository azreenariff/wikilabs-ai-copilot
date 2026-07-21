//! Status Indicators — Connection Status, Engine Status, Skill Pack Status
//!
//! # Overview
//!
//! This module provides real-time status indicators for the application's
//! key subsystems. Status information is exposed to the frontend via events
//! so the UI can display live indicators (e.g., green dot for connected,
//! red dot for disconnected).
//!
//! # Monitored Statuses
//!
//! | Indicator                  | Purpose                                            |
//! |---------------------------|----------------------------------------------------|
//! | AI Backend Connection     | Whether the AI provider endpoint is reachable      |
//! | Observation Engine        | Whether screen/clipboard observation is running    |
//! | Skill Pack Loader         | Whether skill packs are loaded and active          |
//! | Database                  | Whether the local database is connected            |
//! | Knowledge Index           | Whether knowledge packs are indexed                |
//! | Auto-Update               | Whether the updater is checking for updates        |
//! | Crash Recovery            | Whether a previous crash was detected              |
//!
//! # Integration
//!
//! The status manager integrates with:
//! - [`config::AppSettingsStore`] — reads privacy/provider settings.
//! - [`error_handling::ErrorHandler`] — gets crash detection info.
//!
//! # Frontend Integration
//!
//! Events are emitted on the `status-changed` channel with a JSON payload.
//! The frontend should listen for these events and update the status bar.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tracing::{info, warn};

/// Connection status for a service.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConnectionStatus {
    /// Service is connected and operational.
    Connected,
    /// Service is disconnected.
    Disconnected,
    /// Service is connecting (in progress).
    Connecting,
    /// Service is in an error state.
    Error(String),
}

impl ConnectionStatus {
    /// Get a color code for the status indicator (CSS color).
    pub fn color(&self) -> &'static str {
        match self {
            Self::Connected => "#4ade80",   // green
            Self::Disconnected => "#9ca3af", // gray
            Self::Connecting => "#facc15",   // yellow
            Self::Error(_) => "#f87171",     // red
        }
    }

    /// Get a human-readable label for the status.
    pub fn label(&self) -> &str {
        match self {
            Self::Connected => "Connected",
            Self::Disconnected => "Disconnected",
            Self::Connecting => "Connecting...",
            Self::Error(msg) => &format!("Error: {}", msg),
        }
    }

    /// Check if the service is operational (connected or connecting).
    pub fn is_operational(&self) -> bool {
        matches!(self, Self::Connected | Self::Connecting)
    }
}

/// Status of the AI backend connection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiBackendStatus {
    /// Whether the AI provider is reachable.
    pub connected: ConnectionStatus,
    /// The configured provider name (if available).
    pub provider_name: Option<String>,
    /// The configured model (if available).
    pub model: Option<String>,
    /// Last checked timestamp.
    pub last_checked: Option<String>,
    /// Average response time in milliseconds.
    pub avg_response_time_ms: Option<u64>,
}

/// Status of the observation engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservationEngineStatus {
    /// Whether the observation engine is running.
    pub running: bool,
    /// Current screen capture status.
    pub screen_capture: bool,
    /// Current clipboard monitoring status.
    pub clipboard_monitoring: bool,
    /// Number of observations captured since start.
    pub observation_count: u64,
}

/// Status of a loaded skill pack.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillPackStatus {
    /// Number of loaded skill packs.
    pub loaded_count: usize,
    /// Number of active skill packs.
    pub active_count: usize,
    /// Whether all skill packs loaded successfully.
    pub all_loaded: bool,
    /// List of skill packs with their status.
    pub packs: Vec<SkillPackEntry>,
}

/// Individual skill pack status entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillPackEntry {
    /// Pack name.
    pub name: String,
    /// Pack version.
    pub version: String,
    /// Whether the pack is enabled.
    pub enabled: bool,
    /// Whether the pack is active.
    pub active: bool,
    /// Whether the pack loaded successfully.
    pub loaded: bool,
}

/// The overall application status snapshot.
#[derive(Debug, Clone, Serialize)]
pub struct StatusSnapshot {
    /// AI backend connection status.
    pub ai_backend: AiBackendStatus,
    /// Observation engine status.
    pub observation_engine: ObservationEngineStatus,
    /// Skill pack loader status.
    pub skill_packs: SkillPackStatus,
    /// Database connection status.
    pub database: ConnectionStatus,
    /// Knowledge indexing status.
    pub knowledge_indexing: ConnectionStatus,
    /// Auto-update status.
    pub auto_update: ConnectionStatus,
    /// Whether a previous crash was detected.
    pub has_previous_crash: bool,
    /// Application uptime in seconds.
    pub uptime_seconds: u64,
    /// Whether the overall application status is healthy.
    pub is_healthy: bool,
}

impl StatusSnapshot {
    /// Determine if the overall application is healthy.
    pub fn is_healthy(&self) -> bool {
        self.ai_backend.connected.is_operational()
            || !self.ai_backend.connected.is_operational() // AI is optional — app can work offline
            && self.database.is_operational()
    }
}

/// The status manager — tracks and reports subsystem statuses.
pub struct StatusManager {
    app_handle: AppHandle,
    /// Start time for uptime calculation.
    start_time: std::time::Instant,
    /// AI backend status.
    ai_status: Arc<std::sync::RwLock<AiBackendStatus>>,
    /// Observation engine status.
    observation_status: Arc<std::sync::RwLock<ObservationEngineStatus>>,
    /// Skill pack status.
    skill_status: Arc<std::sync::RwLock<SkillPackStatus>>,
    /// Database status.
    db_status: Arc<std::sync::RwLock<ConnectionStatus>>,
    /// Knowledge indexing status.
    knowledge_status: Arc<std::sync::RwLock<ConnectionStatus>>,
    /// Whether a previous crash was detected.
    has_previous_crash: Arc<std::sync::RwLock<bool>>,
    /// Whether auto-update is enabled.
    auto_update_enabled: Arc<std::sync::RwLock<bool>>,
}

impl Clone for StatusManager {
    fn clone(&self) -> Self {
        Self {
            app_handle: self.app_handle.clone(),
            start_time: self.start_time,
            ai_status: Arc::clone(&self.ai_status),
            observation_status: Arc::clone(&self.observation_status),
            skill_status: Arc::clone(&self.skill_status),
            db_status: Arc::clone(&self.db_status),
            knowledge_status: Arc::clone(&self.knowledge_status),
            has_previous_crash: Arc::clone(&self.has_previous_crash),
            auto_update_enabled: Arc::clone(&self.auto_update_enabled),
        }
    }
}

impl StatusManager {
    /// Create a new status manager.
    pub fn new(app_handle: AppHandle) -> Self {
        let now = std::time::Instant::now();
        Self {
            app_handle,
            start_time: now,
            ai_status: Arc::new(std::sync::RwLock::new(AiBackendStatus {
                connected: ConnectionStatus::Disconnected,
                provider_name: None,
                model: None,
                last_checked: None,
                avg_response_time_ms: None,
            })),
            observation_status: Arc::new(std::sync::RwLock::new(ObservationEngineStatus {
                running: false,
                screen_capture: false,
                clipboard_monitoring: false,
                observation_count: 0,
            })),
            skill_status: Arc::new(std::sync::RwLock::new(SkillPackStatus {
                loaded_count: 0,
                active_count: 0,
                all_loaded: true,
                packs: Vec::new(),
            })),
            db_status: Arc::new(std::sync::RwLock::new(ConnectionStatus::Disconnected)),
            knowledge_status: Arc::new(std::sync::RwLock::new(ConnectionStatus::Disconnected)),
            has_previous_crash: Arc::new(std::sync::RwLock::new(false)),
            auto_update_enabled: Arc::new(std::sync::RwLock::new(false)),
        }
    }

    /// Get the application uptime in seconds.
    pub fn uptime_seconds(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }

    // ── AI Backend Status ───────────────────────────────────

    /// Set the AI backend connection status.
    pub fn set_ai_connected(&self, connected: ConnectionStatus, provider: Option<&str>, model: Option<&str>) {
        let mut status = self.ai_status.write().unwrap();
        status.connected = connected;
        status.provider_name = provider.map(|s| s.to_string());
        status.model = model.map(|s| s.to_string());
        status.last_checked = Some(chrono::Utc::now().to_rfc3339());
        info!(
            connected = ?status.connected,
            provider = ?provider,
            "AI backend status updated"
        );
        self.emit_status_change();
    }

    /// Set the AI backend's average response time.
    #[allow(dead_code)]
    pub fn set_ai_response_time(&self, ms: u64) {
        let mut status = self.ai_status.write().unwrap();
        status.avg_response_time_ms = Some(ms);
        self.emit_status_change();
    }

    /// Set the AI backend as connecting.
    #[allow(dead_code)]
    pub fn set_ai_connecting(&self) {
        let mut status = self.ai_status.write().unwrap();
        status.connected = ConnectionStatus::Connecting;
        status.last_checked = Some(chrono::Utc::now().to_rfc3339());
        self.emit_status_change();
    }

    // ── Observation Engine Status ───────────────────────────

    /// Set the observation engine status.
    #[allow(dead_code)]
    pub fn set_observation_status(
        &self,
        running: bool,
        screen_capture: bool,
        clipboard: bool,
        count: u64,
    ) {
        let mut status = self.observation_status.write().unwrap();
        status.running = running;
        status.screen_capture = screen_capture;
        status.clipboard_monitoring = clipboard;
        status.observation_count = count;
        self.emit_status_change();
    }

    // ── Skill Pack Status ───────────────────────────────────

    /// Set the skill pack status.
    #[allow(dead_code)]
    pub fn set_skill_packs(&self, packs: Vec<SkillPackEntry>) {
        let mut status = self.skill_status.write().unwrap();
        status.packs = packs.clone();
        status.loaded_count = packs.len();
        status.active_count = packs.iter().filter(|p| p.active).count();
        status.all_loaded = packs.iter().all(|p| p.loaded);
        info!(
            loaded = status.loaded_count,
            active = status.active_count,
            "Skill pack status updated"
        );
        self.emit_status_change();
    }

    // ── Database Status ─────────────────────────────────────

    /// Set the database connection status.
    pub fn set_database_status(&self, status: ConnectionStatus) {
        let mut db = self.db_status.write().unwrap();
        *db = status;
        info!("Database status updated: {:?}", status);
        self.emit_status_change();
    }

    /// Set the database as connected.
    pub fn set_database_connected(&self) {
        self.set_database_status(ConnectionStatus::Connected);
    }

    // ── Knowledge Indexing Status ───────────────────────────

    /// Set the knowledge indexing status.
    #[allow(dead_code)]
    pub fn set_knowledge_status(&self, status: ConnectionStatus) {
        let mut ks = self.knowledge_status.write().unwrap();
        *ks = status;
        info!("Knowledge indexing status updated: {:?}", status);
        self.emit_status_change();
    }

    // ── Crash Detection ─────────────────────────────────────

    /// Set whether a previous crash was detected.
    #[allow(dead_code)]
    pub fn set_previous_crash(&self, detected: bool) {
        let mut crash = self.has_previous_crash.write().unwrap();
        *crash = detected;
        info!(detected, "Previous crash detection status updated");
    }

    /// Check if a previous crash was detected.
    #[allow(dead_code)]
    pub fn has_previous_crash(&self) -> bool {
        *self.has_previous_crash.read().unwrap()
    }

    // ── Auto-Update ─────────────────────────────────────────

    /// Set the auto-update status.
    #[allow(dead_code)]
    pub fn set_auto_update(&self, enabled: bool) {
        let mut au = self.auto_update_enabled.write().unwrap();
        *au = enabled;
        let status = if enabled {
            ConnectionStatus::Connected
        } else {
            ConnectionStatus::Disconnected
        };
        info!(enabled, "Auto-update status updated");
        self.emit_status_change();
    }

    // ── Snapshot & Emission ─────────────────────────────────

    /// Get a complete status snapshot of all subsystems.
    #[allow(dead_code)]
    pub fn get_snapshot(&self) -> StatusSnapshot {
        let ai = self.ai_status.read().unwrap();
        let obs = self.observation_status.read().unwrap();
        let skills = self.skill_status.read().unwrap();
        let db = self.db_status.read().unwrap();
        let knowledge = self.knowledge_status.read().unwrap();
        let crash = *self.has_previous_crash.read().unwrap();

        let is_healthy = ai.connected.is_operational() || !ai.connected.is_operational(); // App is healthy regardless of AI
        let db_healthy = db.is_operational();
        let skills_healthy = skills.all_loaded;

        StatusSnapshot {
            ai_backend: ai.clone(),
            observation_engine: obs.clone(),
            skill_packs: skills.clone(),
            database: db.clone(),
            knowledge_indexing: knowledge.clone(),
            auto_update: if *self.auto_update_enabled.read().unwrap() {
                ConnectionStatus::Connected
            } else {
                ConnectionStatus::Disconnected
            },
            has_previous_crash: crash,
            uptime_seconds: self.uptime_seconds(),
            is_healthy: is_healthy && db_healthy && skills_healthy,
        }
    }

    /// Emit the current status snapshot to the frontend.
    #[allow(dead_code)]
    fn emit_status_change(&self) {
        let snapshot = self.get_snapshot();
        if let Ok(payload) = serde_json::to_value(&snapshot) {
            if let Err(e) = self.app_handle.emit("status-changed", payload) {
                warn!(error = %e, "Failed to emit status change event");
            }
        }
    }

    /// Emit status change event with a specific label.
    #[allow(dead_code)]
    pub fn emit_event(&self, event_name: &str, payload: serde_json::Value) {
        if let Err(e) = self.app_handle.emit(event_name, payload) {
            warn!(event = event_name, error = %e, "Failed to emit status event");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_status_colors() {
        assert_eq!(ConnectionStatus::Connected.color(), "#4ade80");
        assert_eq!(ConnectionStatus::Disconnected.color(), "#9ca3af");
        assert_eq!(ConnectionStatus::Connecting.color(), "#facc15");
        assert!(ConnectionStatus::Error("test".to_string()).color() == "#f87171");
    }

    #[test]
    fn test_connection_status_labels() {
        assert_eq!(ConnectionStatus::Connected.label(), "Connected");
        assert_eq!(ConnectionStatus::Disconnected.label(), "Disconnected");
        assert!(ConnectionStatus::Connecting.label().contains("Connecting"));
        assert!(ConnectionStatus::Error("oops".to_string()).contains("oops"));
    }

    #[test]
    fn test_status_manager_snapshot() {
        let mgr = StatusManager::new(
            tauri::test::mock_app_handle(
                &tauri::test::MockWindowServer::new(),
                tauri::test::MockRuntime::new(),
            ),
        );

        mgr.set_database_connected();

        let snapshot = mgr.get_snapshot();
        assert_eq!(snapshot.database, ConnectionStatus::Connected);
        assert_eq!(snapshot.uptime_seconds(), 0); // Should be 0 on immediate call
        assert!(snapshot.is_healthy); // AI is optional, so healthy
    }

    #[test]
    fn test_ai_status_update() {
        let mgr = StatusManager::new(
            tauri::test::mock_app_handle(
                &tauri::test::MockWindowServer::new(),
                tauri::test::MockRuntime::new(),
            ),
        );

        mgr.set_ai_connected(ConnectionStatus::Connected, Some("OpenAI"), Some("gpt-4o"));

        let snapshot = mgr.get_snapshot();
        assert!(snapshot.ai_backend.connected.is_operational());
        assert_eq!(snapshot.ai_backend.provider_name, Some("OpenAI".to_string()));
        assert_eq!(snapshot.ai_backend.model, Some("gpt-4o".to_string()));
    }
}