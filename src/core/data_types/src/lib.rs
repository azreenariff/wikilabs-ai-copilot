//! Shared data types for the Wiki Labs AI Copilot.
//!
//! This crate defines the core domain models used across all modules:
//! - ChatMessage with role, content, timestamp, ID
//! - AiRequest / AiResponse with usage tracking
//! - ToolDefinition / ToolCall / ToolResult
//! - SkillMetadata
//! - Skill SDK data types: TechnologyDomain, DetectionRule, CommandDefinition, SkillManifest
//! - Engineering context: EngineeringContext, TechnologyInference, TimelineEntry
//! - Workspace with lifecycle timestamps
//! - KnowledgeDocument / KnowledgeChunk
//! - Intent enum

pub mod ai;
pub mod chat;
pub mod engineering_context;
pub mod tool;
pub mod skill;
pub mod workspace;
pub mod knowledge;
pub mod intent;
pub mod technology;
pub mod timeline;

// Re-export common types for convenience
pub use chat::ChatMessage;
pub use ai::AiRequest;
pub use ai::AiResponse;
pub use ai::TokenUsage;
pub use tool::ToolDefinition;
pub use tool::ToolCall;
pub use tool::ToolResult;
pub use skill::SkillManifest;
pub use skill::TechnologyDefinition;
pub use skill::IntentDefinition;
pub use skill::WorkflowState;
pub use skill::WorkflowTransition;
pub use skill::WorkflowDefinition;
pub use skill::DetectionRule;
pub use skill::DetectionType;
pub use skill::LoadedSkill;
pub use engineering_context::EngineeringContext;
pub use skill::CommandDefinition;
pub use skill::SafetyLevel;
pub use technology::TechnologyInference;
pub use timeline::TimelineEntry;
pub use workspace::Workspace;
pub use knowledge::KnowledgeDocument;
pub use knowledge::KnowledgeChunk;
pub use intent::Intent;