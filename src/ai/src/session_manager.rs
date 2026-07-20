//! Session Manager — lifecycle management of AI sessions.
//!
//! Manages session configuration, state transitions,
//! persistence, and cleanup.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// State of an AI session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionState {
    /// Session is active and ready.
    Active,
    /// Session is paused (e.g., user switched context).
    Paused,
    /// Session is suspended (e.g., workspace switched away).
    Suspended,
    /// Session has ended (conversation closed).
    Ended,
}

impl ToString for SessionState {
    fn to_string(&self) -> String {
        match self {
            SessionState::Active => "active".to_string(),
            SessionState::Paused => "paused".to_string(),
            SessionState::Suspended => "suspended".to_string(),
            SessionState::Ended => "ended".to_string(),
        }
    }
}

/// Configuration for a new session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Session name.
    pub name: String,
    /// System prompt for this session.
    pub system_prompt: String,
    /// Selected AI model.
    pub model: String,
    /// Temperature setting.
    pub temperature: f32,
    /// Maximum output tokens.
    pub max_tokens: usize,
    /// Active workspace ID.
    pub workspace_id: Option<Uuid>,
    /// Active technologies.
    pub technologies: Vec<String>,
    /// Tags for the session.
    #[serde(default)]
    pub tags: Vec<String>,
}

impl SessionConfig {
    pub fn new(
        name: impl Into<String>,
        system_prompt: impl Into<String>,
        model: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            system_prompt: system_prompt.into(),
            model: model.into(),
            temperature: 0.7,
            max_tokens: 4096,
            workspace_id: None,
            technologies: Vec::new(),
            tags: Vec::new(),
        }
    }

    pub fn with_temperature(mut self, temp: f32) -> Self {
        self.temperature = temp;
        self
    }

    pub fn with_max_tokens(mut self, max: usize) -> Self {
        self.max_tokens = max;
        self
    }

    pub fn with_workspace(mut self, workspace_id: Uuid) -> Self {
        self.workspace_id = Some(workspace_id);
        self
    }

    pub fn with_technologies(mut self, technologies: Vec<String>) -> Self {
        self.technologies = technologies;
        self
    }

    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        let tag = tag.into();
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
        self
    }
}

/// An AI session with lifecycle tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Unique identifier.
    pub id: Uuid,
    /// Session configuration.
    pub config: SessionConfig,
    /// Current state.
    pub state: SessionState,
    /// When the session was created.
    pub created_at: DateTime<Utc>,
    /// When the session was last updated.
    pub updated_at: DateTime<Utc>,
    /// When the session was last used (for idle tracking).
    pub last_activity: DateTime<Utc>,
    /// Number of messages exchanged.
    pub message_count: usize,
    /// Tokens consumed so far.
    pub tokens_consumed: usize,
    /// Optional notes about the session.
    pub notes: Option<String>,
}

impl Session {
    pub fn new(config: SessionConfig) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            config,
            state: SessionState::Active,
            created_at: now,
            updated_at: now,
            last_activity: now,
            message_count: 0,
            tokens_consumed: 0,
            notes: None,
        }
    }

    /// Transition to a new state.
    pub fn transition(&mut self, new_state: SessionState) {
        self.state = new_state;
        self.updated_at = Utc::now();
    }

    /// Record a message exchange.
    pub fn record_message(&mut self, prompt_tokens: usize, completion_tokens: usize) {
        self.message_count += 2; // user + assistant
        self.tokens_consumed += prompt_tokens + completion_tokens;
        self.last_activity = Utc::now();
        self.updated_at = Utc::now();
    }

    /// Record just tokens (e.g., for context assembly).
    pub fn record_tokens(&mut self, tokens: usize) {
        self.tokens_consumed += tokens;
        self.updated_at = Utc::now();
    }

    /// Check if the session is idle (no activity for N seconds).
    pub fn is_idle(&self, idle_seconds: u64) -> bool {
        let elapsed = Utc::now()
            .signed_duration_since(self.last_activity)
            .num_seconds() as u64;
        elapsed > idle_seconds
    }

    /// Get the session duration in seconds.
    pub fn duration_seconds(&self) -> i64 {
        Utc::now()
            .signed_duration_since(self.created_at)
            .num_seconds()
    }

    /// Set session notes.
    pub fn set_notes(&mut self, notes: impl Into<String>) {
        self.notes = Some(notes.into());
        self.updated_at = Utc::now();
    }

    /// Clear session notes.
    pub fn clear_notes(&mut self) {
        self.notes = None;
        self.updated_at = Utc::now();
    }

    /// Export session summary as JSON.
    pub fn summary_json(&self) -> anyhow::Result<String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| anyhow::anyhow!("Failed to serialize session: {}", e))
    }
}

/// Manager for AI sessions.
pub struct SessionManager {
    sessions: std::collections::HashMap<Uuid, Session>,
    active_id: Option<Uuid>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: std::collections::HashMap::new(),
            active_id: None,
        }
    }

    /// Create a new session.
    pub fn create(&mut self, config: SessionConfig) -> Uuid {
        let session = Session::new(config);
        let id = session.id;
        self.sessions.insert(id, session);
        self.active_id = Some(id);
        id
    }

    /// Get the active session ID.
    pub fn active_id(&self) -> Option<Uuid> {
        self.active_id
    }

    /// Switch to a session by ID.
    pub fn switch(&mut self, id: Uuid) -> anyhow::Result<()> {
        if !self.sessions.contains_key(&id) {
            anyhow::bail!("Session not found: {}", id);
        }
        self.active_id = Some(id);
        Ok(())
    }

    /// Get the active session (mutable).
    pub fn active_mut(&mut self) -> Option<&mut Session> {
        self.active_id
            .as_ref()
            .and_then(|id| self.sessions.get_mut(id))
    }

    /// Get the active session (immutable).
    pub fn active(&self) -> Option<&Session> {
        self.active_id.as_ref().and_then(|id| self.sessions.get(id))
    }

    /// Get a session by ID.
    pub fn get(&self, id: Uuid) -> Option<&Session> {
        self.sessions.get(&id)
    }

    /// Get a mutable session by ID.
    pub fn get_mut(&mut self, id: Uuid) -> Option<&mut Session> {
        self.sessions.get_mut(&id)
    }

    /// End the active session.
    pub fn end_active(&mut self) -> anyhow::Result<()> {
        if let Some(id) = self.active_id {
            if let Some(session) = self.sessions.get_mut(&id) {
                session.transition(SessionState::Ended);
            }
            // Pick next active session
            self.active_id = self
                .sessions
                .iter()
                .find(|(_, s)| s.state == SessionState::Active)
                .map(|(id, _)| *id);
            Ok(())
        } else {
            anyhow::bail!("No active session to end")
        }
    }

    /// Pause the active session.
    pub fn pause_active(&mut self) -> anyhow::Result<()> {
        if let Some(session) = self.active_mut() {
            session.transition(SessionState::Paused);
            Ok(())
        } else {
            anyhow::bail!("No active session to pause")
        }
    }

    /// Resume the active session.
    pub fn resume_active(&mut self) -> anyhow::Result<()> {
        if let Some(session) = self.active_mut() {
            session.transition(SessionState::Active);
            Ok(())
        } else {
            anyhow::bail!("No active session to resume")
        }
    }

    /// Suspend the active session.
    pub fn suspend_active(&mut self) -> anyhow::Result<()> {
        if let Some(session) = self.active_mut() {
            session.transition(SessionState::Suspended);
            Ok(())
        } else {
            anyhow::bail!("No active session to suspend")
        }
    }

    /// Delete a session.
    pub fn delete(&mut self, id: Uuid) -> anyhow::Result<()> {
        if self.sessions.remove(&id).is_none() {
            anyhow::bail!("Session not found: {}", id);
        }
        if self.active_id == Some(id) {
            self.active_id = self
                .sessions
                .iter()
                .find(|(_, s)| s.state != SessionState::Ended)
                .map(|(id, _)| *id);
        }
        Ok(())
    }

    /// Record tokens for the active session.
    pub fn record_tokens_active(&mut self, tokens: usize) -> anyhow::Result<()> {
        self.active_mut()
            .ok_or_else(|| anyhow::anyhow!("No active session"))?
            .record_tokens(tokens);
        Ok(())
    }

    /// Record a message exchange for the active session.
    pub fn record_message_active(
        &mut self,
        prompt_tokens: usize,
        completion_tokens: usize,
    ) -> anyhow::Result<()> {
        self.active_mut()
            .ok_or_else(|| anyhow::anyhow!("No active session"))?
            .record_message(prompt_tokens, completion_tokens);
        Ok(())
    }

    /// List all sessions.
    pub fn list(&self) -> Vec<&Session> {
        self.sessions.values().collect()
    }

    /// List sessions filtered by state.
    pub fn list_by_state(&self, state: SessionState) -> Vec<&Session> {
        self.sessions
            .values()
            .filter(|s| s.state == state)
            .collect()
    }

    /// Count sessions.
    pub fn count(&self) -> usize {
        self.sessions.len()
    }

    /// Clean up ended sessions (remove them).
    pub fn cleanup_ended(&mut self) -> usize {
        let before = self.sessions.len();
        self.sessions.retain(|_, s| s.state != SessionState::Ended);
        let removed = before - self.sessions.len();

        // If we removed the active session, set a new one
        if self
            .active_id
            .map_or(false, |id| !self.sessions.contains_key(&id))
        {
            self.active_id = self
                .sessions
                .iter()
                .find(|(_, s)| s.state == SessionState::Active)
                .map(|(id, _)| *id);
        }

        removed
    }

    /// Get total tokens across all sessions.
    pub fn total_tokens_consumed(&self) -> usize {
        self.sessions.values().map(|s| s.tokens_consumed).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_new() {
        let config = SessionConfig::new("Test", "System prompt", "gpt-4");
        let session = Session::new(config);
        assert_eq!(session.state, SessionState::Active);
        assert_eq!(session.message_count, 0);
        assert_eq!(session.tokens_consumed, 0);
    }

    #[test]
    fn test_session_config_defaults() {
        let config = SessionConfig::new("T", "S", "M");
        assert_eq!(config.temperature, 0.7);
        assert_eq!(config.max_tokens, 4096);
        assert!(config.workspace_id.is_none());
        assert!(config.technologies.is_empty());
    }

    #[test]
    fn test_session_config_builder() {
        let config = SessionConfig::new("T", "S", "M")
            .with_temperature(0.5)
            .with_max_tokens(8192)
            .with_tag("test");
        assert_eq!(config.temperature, 0.5);
        assert_eq!(config.max_tokens, 8192);
        assert_eq!(config.tags.len(), 1);
    }

    #[test]
    fn test_session_transition() {
        let config = SessionConfig::new("T", "S", "M");
        let mut session = Session::new(config);

        session.transition(SessionState::Paused);
        assert_eq!(session.state, SessionState::Paused);

        session.transition(SessionState::Active);
        assert_eq!(session.state, SessionState::Active);
    }

    #[test]
    fn test_session_record_message() {
        let config = SessionConfig::new("T", "S", "M");
        let mut session = Session::new(config);

        session.record_message(100, 200);
        assert_eq!(session.message_count, 2);
        assert_eq!(session.tokens_consumed, 300);
    }

    #[test]
    fn test_session_record_tokens() {
        let config = SessionConfig::new("T", "S", "M");
        let mut session = Session::new(config);

        session.record_tokens(500);
        assert_eq!(session.tokens_consumed, 500);
    }

    #[test]
    fn test_session_idle() {
        let config = SessionConfig::new("T", "S", "M");
        let session = Session::new(config);
        // Just created, not idle
        assert!(!session.is_idle(60));
    }

    #[test]
    fn test_session_set_notes() {
        let config = SessionConfig::new("T", "S", "M");
        let mut session = Session::new(config);
        session.set_notes("Important info");
        assert_eq!(session.notes, Some("Important info".to_string()));
        session.clear_notes();
        assert!(session.notes.is_none());
    }

    #[test]
    fn test_session_summary_json() {
        let config = SessionConfig::new("T", "S", "M");
        let session = Session::new(config);
        let json = session.summary_json().unwrap();
        assert!(json.contains("T"));
    }

    #[test]
    fn test_session_manager_create() {
        let mut sm = SessionManager::new();
        let config = SessionConfig::new("Test", "System", "gpt-4");
        let id = sm.create(config);
        assert_eq!(sm.active_id(), Some(id));
        assert_eq!(sm.count(), 1);
    }

    #[test]
    fn test_session_manager_switch() {
        let mut sm = SessionManager::new();
        let id1 = sm.create(SessionConfig::new("S1", "S", "M"));
        let id2 = sm.create(SessionConfig::new("S2", "S", "M"));

        sm.switch(id1).unwrap();
        assert_eq!(sm.active_id(), Some(id1));

        sm.switch(id2).unwrap();
        assert_eq!(sm.active_id(), Some(id2));

        assert!(sm.switch(Uuid::new_v4()).is_err());
    }

    #[test]
    fn test_session_manager_end_active() {
        let mut sm = SessionManager::new();
        let id1 = sm.create(SessionConfig::new("S1", "S", "M"));
        let id2 = sm.create(SessionConfig::new("S2", "S", "M"));

        sm.switch(id1).unwrap();
        sm.end_active().unwrap();

        // id1 is ended, should switch to id2
        assert_eq!(sm.active_id(), Some(id2));

        let session = sm.get(id1).unwrap();
        assert_eq!(session.state, SessionState::Ended);
    }

    #[test]
    fn test_session_manager_pause_resume() {
        let mut sm = SessionManager::new();
        sm.create(SessionConfig::new("S1", "S", "M"));

        sm.pause_active().unwrap();
        assert_eq!(sm.active().unwrap().state, SessionState::Paused);

        sm.resume_active().unwrap();
        assert_eq!(sm.active().unwrap().state, SessionState::Active);
    }

    #[test]
    fn test_session_manager_suspend() {
        let mut sm = SessionManager::new();
        sm.create(SessionConfig::new("S1", "S", "M"));

        sm.suspend_active().unwrap();
        assert_eq!(sm.active().unwrap().state, SessionState::Suspended);
    }

    #[test]
    fn test_session_manager_delete() {
        let mut sm = SessionManager::new();
        let id = sm.create(SessionConfig::new("S1", "S", "M"));
        assert_eq!(sm.count(), 1);

        sm.delete(id).unwrap();
        assert_eq!(sm.count(), 0);
        assert!(sm.active_id().is_none());
    }

    #[test]
    fn test_session_manager_delete_nonexistent() {
        let mut sm = SessionManager::new();
        assert!(sm.delete(Uuid::new_v4()).is_err());
    }

    #[test]
    fn test_session_manager_record_tokens() {
        let mut sm = SessionManager::new();
        sm.create(SessionConfig::new("S1", "S", "M"));

        sm.record_tokens_active(100).unwrap();
        assert_eq!(sm.active().unwrap().tokens_consumed, 100);
    }

    #[test]
    fn test_session_manager_list() {
        let mut sm = SessionManager::new();
        sm.create(SessionConfig::new("S1", "S", "M"));
        sm.create(SessionConfig::new("S2", "S", "M"));
        sm.create(SessionConfig::new("S3", "S", "M"));

        let sessions = sm.list();
        assert_eq!(sessions.len(), 3);
    }

    #[test]
    fn test_session_manager_cleanup_ended() {
        let mut sm = SessionManager::new();
        let s1 = sm.create(SessionConfig::new("S1", "S", "M"));
        let s2 = sm.create(SessionConfig::new("S2", "S", "M"));

        // End first session
        if let Some(session) = sm.get_mut(s1) {
            session.transition(SessionState::Ended);
        }

        let cleaned = sm.cleanup_ended();
        assert_eq!(cleaned, 1);
        assert_eq!(sm.count(), 1);
    }

    #[test]
    fn test_session_manager_total_tokens() {
        let mut sm = SessionManager::new();
        let _id1 = sm.create(SessionConfig::new("S1", "S", "M"));
        let id2 = sm.create(SessionConfig::new("S2", "S", "M"));

        sm.record_tokens_active(100).unwrap(); // S1
        sm.switch(id2).unwrap();
        sm.record_tokens_active(200).unwrap(); // S2

        assert_eq!(sm.total_tokens_consumed(), 300);
    }

    #[test]
    fn test_session_manager_record_message_active_no_session() {
        let mut sm = SessionManager::new();
        assert!(sm.record_message_active(100, 200).is_err());
    }

    #[test]
    fn test_session_list_by_state() {
        let mut sm = SessionManager::new();
        sm.create(SessionConfig::new("S1", "S", "M"));
        sm.create(SessionConfig::new("S2", "S", "M"));
        sm.create(SessionConfig::new("S3", "S", "M"));

        // End one session
        let s3_id = sm.list()[2].id;
        if let Some(session) = sm.get_mut(s3_id) {
            session.transition(SessionState::Ended);
        }

        let active = sm.list_by_state(SessionState::Active);
        assert_eq!(active.len(), 2);

        let ended = sm.list_by_state(SessionState::Ended);
        assert_eq!(ended.len(), 1);
    }

    #[test]
    fn test_session_config_with_workspace() {
        let ws_id = Uuid::new_v4();
        let config = SessionConfig::new("T", "S", "M").with_workspace(ws_id);
        assert_eq!(config.workspace_id, Some(ws_id));
    }

    #[test]
    fn test_session_config_with_technologies() {
        let config = SessionConfig::new("T", "S", "M")
            .with_technologies(vec!["rust".to_string(), "k8s".to_string()]);
        assert_eq!(config.technologies.len(), 2);
    }

    #[test]
    fn test_session_config_duplicate_tags() {
        let config = SessionConfig::new("T", "S", "M")
            .with_tag("a")
            .with_tag("a"); // duplicate
        assert_eq!(config.tags.len(), 1);
    }
}
