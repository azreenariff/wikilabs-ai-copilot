# Conversation Manager — Wiki Labs AI Copilot

## Overview

The Conversation Manager (`conversation_manager.rs`) provides structured conversation lifecycle management for the AI Copilot. It handles multiple concurrent conversations with CRUD operations, history tracking, tagging, and export/restore capabilities.

**Module:** `src/ai/src/conversation_manager.rs`  
**Lines of code:** ~546 (including tests)  
**Tests:** 22 unit tests

## Architecture

### Core Types

```rust
// An individual message
ConversationMessage {
    id: Uuid,
    role: ConversationRole,    // User | Assistant | System
    content: String,
    created_at: DateTime<Utc>,
    tool_calls: Vec<serde_json::Value>,  // Optional tool calls
}

// A named conversation
Conversation {
    id: Uuid,
    name: String,
    messages: Vec<ConversationMessage>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    system_prompt: Option<String>,  // Conversation-level override
    tags: Vec<String>,              // Categorization tags
}

// Manager for multiple conversations
ConversationManager {
    conversations: HashMap<Uuid, Conversation>,
    active_id: Option<Uuid>,
}
```

## API

### Creating Conversations

```rust
let mut mgr = ConversationManager::new();
let id = mgr.create("Database Debug");
```

When a new conversation is created, it automatically becomes the active conversation.

### Adding Messages

```rust
// Direct message construction
mgr.add_message(ConversationMessage::user("Why is this query slow?")).unwrap();

// Helper methods on Conversation
conv.add_user_message("Help me debug this");
conv.add_assistant_message("Let me check...");
conv.add_system_message("You are helpful.");

// With tool calls
let msg = ConversationMessage::assistant("Processing...")
    .with_tool_calls(vec![json!({
        "name": "read_file",
        "arguments": {"path": "/app/config.yml"}
    })]);
```

### Switching Conversations

```rust
let id1 = mgr.create("Chat 1");
let id2 = mgr.create("Chat 2");

mgr.switch(id1).unwrap();  // Now id1 is active
assert_eq!(mgr.active_id(), Some(id1));
```

### CRUD Operations

```rust
// Get by ID
let conv = mgr.get(id).unwrap();

// Rename
mgr.rename(id, "New Name").unwrap();

// Add/remove tags
conv.add_tag("bugfix");
conv.remove_tag("urgent");

// Export/Import
let json = mgr.export(id).unwrap();
let restored_id = mgr.restore(&json).unwrap();
```

### Listing Conversations

```rust
let summaries = mgr.list();
// Returns Vec<ConversationSummary> with:
//   id, name, created_at, updated_at,
//   message_count, first_message_preview
```

### Message Queries

```rust
// Last message
let last = conv.last_message();

// Count by role
let user_count = conv.count_by_role(&ConversationRole::User);

// Format for API
let api_msgs = conv.messages_for_api();
// Returns Vec<serde_json::Value> with {role, content}
```

### Lifecycle

```rust
// Delete a conversation
mgr.delete(id).unwrap();
// Auto-selects another conversation as active if the deleted one was active

// Clear all conversations
mgr.clear();
// Resets active_id to None
```

## Features

### Conversation Summary

Each conversation provides a summary for listing:

```rust
ConversationSummary {
    id: Uuid,
    name: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    message_count: usize,
    first_message_preview: String,  // First 80 chars
}
```

### Tag Management

Tags provide categorization with deduplication:

```rust
conv.add_tag("production");
conv.add_tag("production");  // Duplicate, ignored
assert_eq!(conv.tags.len(), 1);
```

### Export Formats

- **JSON** — Full state preservation (messages, tags, system prompt, timestamps)
- **Text** — Human-readable export with role annotations:
  ```
  Conversation: Database Debug
  Created: 2024-01-15T10:30:00Z
  
  [2024-01-15T10:31:00Z] user:
  Why is this query slow?
  
  [2024-01-15T10:31:05Z] assistant:
  Let me check the execution plan...
  ```

### Message Tools

Assistant messages can carry tool calls for multi-turn reasoning:

```rust
ConversationMessage {
    role: Assistant,
    content: "I'll check the file...",
    tool_calls: [
        {
            "name": "read_file",
            "arguments": {"path": "/app/config.yml"}
        }
    ]
}
```

## Tests

22 unit tests covering:

| Test | What It Verifies |
|---|---|
| `test_create_conversation` | Creation with default state |
| `test_active_conversation` | Auto-selection of active conversation |
| `test_switch_conversation` | Switching between conversations |
| `test_add_message` | Adding user/assistant/system messages |
| `test_add_message_no_active` | Error when no active conversation |
| `test_delete_conversation` | Deletion with auto-selection |
| `test_delete_nonexistent` | Error handling for invalid ID |
| `test_rename_conversation` | Renaming with timestamp update |
| `test_export_import` | JSON round-trip preservation |
| `test_list_conversations` | Listing multiple conversations |
| `test_conversation_tags` | Tag add/remove/dedup |
| `test_conversation_export_text` | Human-readable export format |
| `test_messages_for_api` | JSON formatting for API requests |
| `test_message_roles` | Role string serialization |
| `test_message_with_tool_calls` | Tool call attachment |
| `test_count_by_role` | Role-based counting |
| `test_last_message` | Last message retrieval |
| `test_clear` | Clear all + active reset |

## Usage in the Copilot

The Conversation Manager integrates with:

- **Context Manager** — Provides `messages_for_api()` as conversation context
- **Session Manager** — Records token consumption per conversation session
- **Tauri Commands** — `send_message`, `get_history`, `stream_message` commands use it
- **Persistence Layer** — Conversations can be saved to SQLite via `Repository` traits

## Design Notes

1. **Active conversation model** — Like a text editor, there's always one "active" conversation, making message operations straightforward without always specifying IDs.

2. **UUID-based IDs** — Every message and conversation gets a unique UUID, enabling reliable export/import and distributed synchronization.

3. **No persistence built-in** — The manager is in-memory only. Persistence is handled by the `persistence` crate via repository traits, keeping the AI crate focused on conversation logic.

4. **JSON API format** — `messages_for_api()` returns `Vec<serde_json::Value>` for maximum flexibility with different provider APIs (OpenAI, vLLM, Ollama all use slightly different formats).