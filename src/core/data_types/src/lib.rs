//! Shared data types for the Wiki Labs AI Copilot.
//!
//! This crate defines the core domain models used across all modules:
//! - ChatMessage with role, content, timestamp, ID
//! - AiRequest / AiResponse with usage tracking
//! - ToolDefinition / ToolCall / ToolResult
//! - SkillMetadata
//! - Workspace with lifecycle timestamps
//! - KnowledgeDocument / KnowledgeChunk
//! - Intent enum

pub mod ai;
pub mod chat;
pub mod tool;
pub mod skill;
pub mod workspace;
pub mod knowledge;
pub mod intent;

// Re-export common types for convenience
pub use chat::ChatMessage;
pub use ai::AiRequest;
pub use ai::AiResponse;
pub use ai::TokenUsage;
pub use tool::ToolDefinition;
pub use tool::ToolCall;
pub use tool::ToolResult;
pub use skill::SkillMetadata;
pub use workspace::Workspace;
pub use knowledge::KnowledgeDocument;
pub use knowledge::KnowledgeChunk;
pub use intent::Intent;