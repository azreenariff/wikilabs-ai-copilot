//! AI Runtime — Phase 5 implementation.
//!
//! This crate provides the AI intelligence layer for Wiki Labs AI Copilot.
//!
//! ## Features Implemented
//!
//! 1. **AI Runtime** — Provider abstraction with streaming, retry, timeout, cancellation, health checks
//! 2. **Conversation Manager** — Structured conversations with CRUD operations
//! 3. **Context Manager** — Central context aggregation from multiple sources
//! 4. **Prompt Manager** — Assembling prompts from templates with versioning
//! 5. **Engineering Persona** — Default system behavior specification
//! 6. **Workspace Context** — Per-workspace context with session history
//! 7. **Memory Architecture** — Short-term session memory, conversation memory, workspace memory
//! 8. **AI Streaming** — Progressive response display with cancellation support
//! 9. **Token Budget Manager** — Token estimation, intelligent trimming, summarization
//! 10. **Manual Context Selection** — User-configurable context influence

pub mod provider;
pub mod context;
pub mod token_counter;
pub mod response;
pub mod conversation_manager;
pub mod context_manager;
pub mod prompt_manager;
pub mod persona;
pub mod session_manager;
pub mod token_budget;

// Re-export common types for convenience
pub use provider::{AiProvider, ProviderInfo, ModelInfo, AiMessage, AiRequest, AiResponse, TokenUsage, ToolCall, EmbedRequest, EmbedResponse, EmbeddingData, OpenAICompatibleProvider};
pub use context::{ContextWindow, ContextAllocation, ContextEntry};
pub use conversation_manager::{ConversationManager, Conversation, ConversationRole, ConversationSummary};
pub use context_manager::{ContextManager as AiContextManager, ContextSource, ContextPriority, AggregatedContext};
pub use prompt_manager::{PromptManager, PromptTemplate, PromptAssembly, PromptVersion};
pub use persona::EngineeringPersona;
pub use session_manager::{SessionManager, SessionConfig, SessionState};
pub use token_budget::{TokenBudgetManager, BudgetAction, BudgetPolicy};