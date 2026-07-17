//! AI Response types and streaming.

pub struct StreamItem {
    pub content: String,
    pub is_complete: bool,
}

pub struct ToolCall {
    pub name: String,
    pub arguments: serde_json::Value,
}