//! Conversation Manager — structured conversation lifecycle.
//!
//! Manages multiple conversations with CRUD operations,
//! history tracking, and export/restore capabilities.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Role for a message in a conversation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConversationRole {
    User,
    Assistant,
    System,
}

impl ToString for ConversationRole {
    fn to_string(&self) -> String {
        match self {
            ConversationRole::User => "user".to_string(),
            ConversationRole::Assistant => "assistant".to_string(),
            ConversationRole::System => "system".to_string(),
        }
    }
}

/// A single message within a conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    /// Unique identifier for this message.
    pub id: Uuid,
    /// Message role.
    pub role: ConversationRole,
    /// Message content.
    pub content: String,
    /// When the message was created.
    pub created_at: DateTime<Utc>,
    /// Optional tool calls from the assistant.
    #[serde(default)]
    pub tool_calls: Vec<serde_json::Value>,
}

impl ConversationMessage {
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            role: ConversationRole::User,
            content: content.into(),
            created_at: Utc::now(),
            tool_calls: Vec::new(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            role: ConversationRole::Assistant,
            content: content.into(),
            created_at: Utc::now(),
            tool_calls: Vec::new(),
        }
    }

    pub fn system(content: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            role: ConversationRole::System,
            content: content.into(),
            created_at: Utc::now(),
            tool_calls: Vec::new(),
        }
    }

    pub fn with_tool_calls(mut self, tool_calls: Vec<serde_json::Value>) -> Self {
        self.tool_calls = tool_calls;
        self
    }
}

/// Summary of a conversation for listing purposes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationSummary {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub message_count: usize,
    pub first_message_preview: String,
}

/// A structured conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    /// Unique identifier.
    pub id: Uuid,
    /// Human-readable name.
    pub name: String,
    /// Messages in this conversation.
    pub messages: Vec<ConversationMessage>,
    /// When the conversation was created.
    pub created_at: DateTime<Utc>,
    /// When the conversation was last updated.
    pub updated_at: DateTime<Utc>,
    /// Optional system override for this conversation.
    pub system_prompt: Option<String>,
    /// Tags for categorization.
    #[serde(default)]
    pub tags: Vec<String>,
}

impl Conversation {
    pub fn new(name: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            messages: Vec::new(),
            created_at: now,
            updated_at: now,
            system_prompt: None,
            tags: Vec::new(),
        }
    }

    /// Add a message to the conversation.
    pub fn add_message(&mut self, message: ConversationMessage) {
        self.messages.push(message);
        self.updated_at = Utc::now();
    }

    /// Add a user message.
    pub fn add_user_message(&mut self, content: impl Into<String>) {
        self.add_message(ConversationMessage::user(content));
    }

    /// Add an assistant message.
    pub fn add_assistant_message(&mut self, content: impl Into<String>) {
        self.add_message(ConversationMessage::assistant(content));
    }

    /// Add a system message.
    pub fn add_system_message(&mut self, content: impl Into<String>) {
        self.add_message(ConversationMessage::system(content));
    }

    /// Get messages formatted for API request.
    pub fn messages_for_api(&self) -> Vec<serde_json::Value> {
        self.messages
            .iter()
            .map(|m| {
                serde_json::json!({
                    "role": m.role.to_string(),
                    "content": m.content,
                })
            })
            .collect()
    }

    /// Get the most recent message, if any.
    pub fn last_message(&self) -> Option<&ConversationMessage> {
        self.messages.last()
    }

    /// Count messages by role.
    pub fn count_by_role(&self, role: &ConversationRole) -> usize {
        self.messages.iter().filter(|m| &m.role == role).count()
    }

    /// Generate a preview for listing.
    pub fn first_message_preview(&self) -> String {
        self.messages
            .first()
            .map(|m| {
                let max_len = 80;
                if m.content.len() > max_len {
                    format!("{}...", &m.content[..max_len])
                } else {
                    m.content.clone()
                }
            })
            .unwrap_or_default()
    }

    /// Rename the conversation.
    pub fn rename(&mut self, new_name: impl Into<String>) {
        self.name = new_name.into();
        self.updated_at = Utc::now();
    }

    /// Add a tag.
    pub fn add_tag(&mut self, tag: impl Into<String>) {
        let tag = tag.into();
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
        self.updated_at = Utc::now();
    }

    /// Remove a tag.
    pub fn remove_tag(&mut self, tag: &str) {
        self.tags.retain(|t| t != tag);
        self.updated_at = Utc::now();
    }

    /// Export conversation as formatted text.
    pub fn export_text(&self) -> String {
        let mut output = format!(
            "Conversation: {}\nCreated: {}\n\n",
            self.name, self.created_at
        );
        for msg in &self.messages {
            output.push_str(&format!(
                "[{}] {}:\n{}\n\n",
                msg.role.to_string(),
                msg.created_at,
                msg.content
            ));
        }
        output
    }

    /// Serialize to JSON string.
    pub fn export_json(&self) -> anyhow::Result<String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| anyhow::anyhow!("Failed to serialize conversation: {}", e))
    }

    /// Deserialize from JSON string.
    pub fn from_json(json: &str) -> anyhow::Result<Self> {
        serde_json::from_str(json)
            .map_err(|e| anyhow::anyhow!("Failed to deserialize conversation: {}", e))
    }

    /// Restore messages from JSON (for recovery).
    pub fn restore_from(json: &str) -> anyhow::Result<Self> {
        Self::from_json(json)
    }
}

/// Manages multiple conversations.
pub struct ConversationManager {
    conversations: std::collections::HashMap<Uuid, Conversation>,
    active_id: Option<Uuid>,
}

impl ConversationManager {
    pub fn new() -> Self {
        Self {
            conversations: std::collections::HashMap::new(),
            active_id: None,
        }
    }

    /// Create a new conversation.
    pub fn create(&mut self, name: impl Into<String>) -> Uuid {
        let conv = Conversation::new(name);
        let id = conv.id;
        self.conversations.insert(id, conv);
        self.active_id = Some(id);
        id
    }

    /// Get the active conversation ID.
    pub fn active_id(&self) -> Option<Uuid> {
        self.active_id
    }

    /// Switch to a conversation by ID.
    pub fn switch(&mut self, id: Uuid) -> anyhow::Result<()> {
        if !self.conversations.contains_key(&id) {
            anyhow::bail!("Conversation not found: {}", id);
        }
        self.active_id = Some(id);
        Ok(())
    }

    /// Get the active conversation (mutable).
    pub fn active_mut(&mut self) -> Option<&mut Conversation> {
        self.active_id
            .as_ref()
            .and_then(|id| self.conversations.get_mut(id))
    }

    /// Get the active conversation (immutable).
    pub fn active(&self) -> Option<&Conversation> {
        self.active_id
            .as_ref()
            .and_then(|id| self.conversations.get(id))
    }

    /// Add a message to the active conversation.
    pub fn add_message(&mut self, message: ConversationMessage) -> anyhow::Result<()> {
        self.active_mut()
            .ok_or_else(|| anyhow::anyhow!("No active conversation"))?
            .add_message(message);
        Ok(())
    }

    /// Get a conversation by ID.
    pub fn get(&self, id: Uuid) -> Option<&Conversation> {
        self.conversations.get(&id)
    }

    /// Get a mutable conversation by ID.
    pub fn get_mut(&mut self, id: Uuid) -> Option<&mut Conversation> {
        self.conversations.get_mut(&id)
    }

    /// List all conversations as summaries.
    pub fn list(&self) -> Vec<ConversationSummary> {
        self.conversations
            .values()
            .map(|c| ConversationSummary {
                id: c.id,
                name: c.name.clone(),
                created_at: c.created_at,
                updated_at: c.updated_at,
                message_count: c.messages.len(),
                first_message_preview: c.first_message_preview(),
            })
            .collect()
    }

    /// Delete a conversation.
    pub fn delete(&mut self, id: Uuid) -> anyhow::Result<()> {
        if self.conversations.remove(&id).is_none() {
            anyhow::bail!("Conversation not found: {}", id);
        }
        // If we deleted the active conversation, reset active_id
        if self.active_id == Some(id) {
            self.active_id = self.conversations.keys().next().copied();
        }
        Ok(())
    }

    /// Rename a conversation.
    pub fn rename(&mut self, id: Uuid, new_name: &str) -> anyhow::Result<()> {
        self.conversations
            .get_mut(&id)
            .ok_or_else(|| anyhow::anyhow!("Conversation not found: {}", id))?
            .rename(new_name);
        Ok(())
    }

    /// Export a conversation as JSON.
    pub fn export(&self, id: Uuid) -> anyhow::Result<String> {
        self.conversations
            .get(&id)
            .ok_or_else(|| anyhow::anyhow!("Conversation not found: {}", id))?
            .export_json()
    }

    /// Restore a conversation from JSON.
    pub fn restore(&mut self, json: &str) -> anyhow::Result<Uuid> {
        let conv = Conversation::from_json(json)?;
        let id = conv.id;
        self.conversations.insert(id, conv);
        Ok(id)
    }

    /// Count conversations.
    pub fn count(&self) -> usize {
        self.conversations.len()
    }

    /// Clear all conversations.
    pub fn clear(&mut self) {
        self.conversations.clear();
        self.active_id = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_conversation() {
        let mut cm = ConversationManager::new();
        let id = cm.create("Test Chat");
        let conv = cm.get(id).unwrap();
        assert_eq!(conv.name, "Test Chat");
        assert_eq!(conv.messages.len(), 0);
    }

    #[test]
    fn test_active_conversation() {
        let mut cm = ConversationManager::new();
        let id1 = cm.create("Chat 1");
        assert_eq!(cm.active_id(), Some(id1));

        let _id2 = cm.create("Chat 2");
        assert_eq!(cm.active_id(), Some(_id2));
    }

    #[test]
    fn test_switch_conversation() {
        let mut cm = ConversationManager::new();
        let _id1 = cm.create("Chat 1");
        let _id2 = cm.create("Chat 2");

        cm.switch(_id1).unwrap();
        assert_eq!(cm.active_id(), Some(_id1));

        assert!(cm.switch(Uuid::new_v4()).is_err());
    }

    #[test]
    fn test_add_message() {
        let mut cm = ConversationManager::new();
        cm.create("Chat 1");
        cm.add_message(ConversationMessage::user("Hello")).unwrap();

        let conv = cm.active().unwrap();
        assert_eq!(conv.messages.len(), 1);
        assert_eq!(conv.messages[0].role, ConversationRole::User);
        assert_eq!(conv.messages[0].content, "Hello");
    }

    #[test]
    fn test_add_message_no_active() {
        let mut cm = ConversationManager::new();
        assert!(cm.add_message(ConversationMessage::user("test")).is_err());
    }

    #[test]
    fn test_delete_conversation() {
        let mut cm = ConversationManager::new();
        let id = cm.create("Chat 1");
        assert_eq!(cm.count(), 1);

        cm.delete(id).unwrap();
        assert_eq!(cm.count(), 0);
        assert!(cm.active_id().is_none());
    }

    #[test]
    fn test_delete_nonexistent() {
        let mut cm = ConversationManager::new();
        assert!(cm.delete(Uuid::new_v4()).is_err());
    }

    #[test]
    fn test_rename_conversation() {
        let mut cm = ConversationManager::new();
        let id = cm.create("Old Name");
        cm.rename(id, "New Name").unwrap();

        let conv = cm.get(id).unwrap();
        assert_eq!(conv.name, "New Name");
    }

    #[test]
    fn test_export_import() {
        let mut cm = ConversationManager::new();
        let id = cm.create("Chat 1");
        cm.add_message(ConversationMessage::user("Hi")).unwrap();
        cm.add_message(ConversationMessage::assistant("Hello!"))
            .unwrap();

        let json = cm.export(id).unwrap();
        let restored_id = cm.restore(&json).unwrap();

        let restored = cm.get(restored_id).unwrap();
        assert_eq!(restored.name, "Chat 1");
        assert_eq!(restored.messages.len(), 2);
    }

    #[test]
    fn test_list_conversations() {
        let mut cm = ConversationManager::new();
        cm.create("Alpha");
        cm.create("Beta");
        cm.create("Gamma");

        let list = cm.list();
        assert_eq!(list.len(), 3);
        assert!(list.iter().any(|c| c.name == "Alpha"));
    }

    #[test]
    fn test_conversation_tags() {
        let mut conv = Conversation::new("Tagged");
        conv.add_tag("bugfix");
        conv.add_tag("urgent");
        conv.add_tag("bugfix"); // duplicate
        assert_eq!(conv.tags.len(), 2);
        assert!(conv.tags.contains(&"bugfix".to_string()));

        conv.remove_tag("urgent");
        assert_eq!(conv.tags.len(), 1);
    }

    #[test]
    fn test_conversation_export_text() {
        let mut conv = Conversation::new("Export Test");
        conv.add_user_message("Question");
        conv.add_assistant_message("Answer");

        let text = conv.export_text();
        assert!(text.contains("Export Test"));
        assert!(text.contains("Question"));
        assert!(text.contains("Answer"));
    }

    #[test]
    fn test_messages_for_api() {
        let mut conv = Conversation::new("API Test");
        conv.add_system_message("You are helpful.");
        conv.add_user_message("Hi");

        let api_msgs = conv.messages_for_api();
        assert_eq!(api_msgs.len(), 2);
        assert_eq!(api_msgs[0]["role"], "system");
        assert_eq!(api_msgs[1]["role"], "user");
    }

    #[test]
    fn test_message_roles() {
        assert_eq!(ConversationRole::User.to_string(), "user");
        assert_eq!(ConversationRole::Assistant.to_string(), "assistant");
        assert_eq!(ConversationRole::System.to_string(), "system");
    }

    #[test]
    fn test_message_with_tool_calls() {
        let msg = ConversationMessage::assistant("test").with_tool_calls(vec![serde_json::json!({
            "name": "read_file",
            "arguments": {"path": "/test.txt"}
        })]);
        assert_eq!(msg.tool_calls.len(), 1);
        assert_eq!(msg.tool_calls[0]["name"], "read_file");
    }

    #[test]
    fn test_count_by_role() {
        let mut conv = Conversation::new("Role Count");
        conv.add_user_message("1");
        conv.add_user_message("2");
        conv.add_assistant_message("a");

        assert_eq!(conv.count_by_role(&ConversationRole::User), 2);
        assert_eq!(conv.count_by_role(&ConversationRole::Assistant), 1);
        assert_eq!(conv.count_by_role(&ConversationRole::System), 0);
    }

    #[test]
    fn test_last_message() {
        let mut conv = Conversation::new("Last Msg");
        assert!(conv.last_message().is_none());

        conv.add_user_message("Hi");
        assert!(conv.last_message().is_some());
        assert_eq!(conv.last_message().unwrap().content, "Hi");

        conv.add_assistant_message("Hello");
        assert_eq!(conv.last_message().unwrap().content, "Hello");
    }

    #[test]
    fn test_clear() {
        let mut cm = ConversationManager::new();
        cm.create("A");
        cm.create("B");
        cm.clear();
        assert_eq!(cm.count(), 0);
        assert!(cm.active_id().is_none());
    }
}
