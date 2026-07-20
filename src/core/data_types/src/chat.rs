//! Chat message types with timestamps and IDs.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A single chat message in the conversation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatMessage {
    /// Unique identifier for this message.
    pub id: Uuid,
    /// Role: "user", "assistant", or "system".
    pub role: String,
    /// Message content (text).
    pub content: String,
    /// When this message was created.
    pub created_at: DateTime<Utc>,
    /// Optional tool call results associated with this message.
    #[serde(default)]
    pub tool_calls: Vec<String>,
}

impl ChatMessage {
    /// Create a new user message.
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            role: "user".to_string(),
            content: content.into(),
            created_at: Utc::now(),
            tool_calls: Vec::new(),
        }
    }

    /// Create a new assistant message.
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            role: "assistant".to_string(),
            content: content.into(),
            created_at: Utc::now(),
            tool_calls: Vec::new(),
        }
    }

    /// Create a new system message.
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            role: "system".to_string(),
            content: content.into(),
            created_at: Utc::now(),
            tool_calls: Vec::new(),
        }
    }

    /// Format as a string suitable for API requests.
    pub fn format_for_api(&self) -> serde_json::Value {
        serde_json::json!({
            "role": self.role,
            "content": self.content
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_message_creation() {
        let msg = ChatMessage::user("Hello");
        assert_eq!(msg.role, "user");
        assert_eq!(msg.content, "Hello");
        assert_eq!(msg.tool_calls.len(), 0);
    }

    #[test]
    fn test_system_message_creation() {
        let msg = ChatMessage::system("System prompt");
        assert_eq!(msg.role, "system");
        assert_eq!(msg.content, "System prompt");
    }

    #[test]
    fn test_message_serialization() {
        let msg = ChatMessage::user("Test message");
        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: ChatMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.role, "user");
        assert_eq!(deserialized.content, "Test message");
    }
}
