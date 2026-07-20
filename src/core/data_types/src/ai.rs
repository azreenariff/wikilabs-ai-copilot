//! AI request/response data types with tool support and usage tracking.

use crate::chat::ChatMessage;
use crate::tool::{ToolCall, ToolDefinition};
use serde::{Deserialize, Serialize};

/// AI request payload sent to the provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiRequest {
    /// System prompt that sets the AI's behavior.
    pub system_prompt: String,
    /// Conversation messages (user, assistant, system).
    pub messages: Vec<ChatMessage>,
    /// Available tools the AI can use.
    #[serde(default)]
    pub tools: Vec<ToolDefinition>,
    /// Sampling temperature (0.0–2.0).
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    /// Maximum tokens to generate.
    #[serde(default = "default_max_tokens")]
    pub max_tokens: usize,
}

/// AI response payload from the provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiResponse {
    /// The assistant's response message.
    pub message: ChatMessage,
    /// Tool calls the AI requested.
    #[serde(default)]
    pub tool_calls: Vec<ToolCall>,
    /// Token usage for cost tracking.
    pub usage: TokenUsage,
}

/// Token usage tracking.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenUsage {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
}

fn default_temperature() -> f32 {
    0.7
}

fn default_max_tokens() -> usize {
    4096
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_request_serialization() {
        let req = AiRequest {
            system_prompt: "You are a helpful assistant.".to_string(),
            messages: vec![ChatMessage::user("How are you?".to_string())],
            tools: vec![],
            temperature: 0.5,
            max_tokens: 1024,
        };
        let json = serde_json::to_string(&req).unwrap();
        let deserialized: AiRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.system_prompt, "You are a helpful assistant.");
        assert_eq!(deserialized.temperature, 0.5);
    }
}
