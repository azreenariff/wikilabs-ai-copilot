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

pub mod context;
pub mod context_manager;
pub mod conversation_manager;
pub mod persona;
pub mod prompt_manager;
pub mod provider;
pub mod response;
pub mod session_manager;
pub mod token_budget;
pub mod token_counter;

// Re-export common types for convenience
pub use context::{ContextAllocation, ContextEntry, ContextWindow};
pub use context_manager::{
    AggregatedContext, ContextManager as AiContextManager, ContextPriority, ContextSource,
};
pub use conversation_manager::{
    Conversation, ConversationManager, ConversationRole, ConversationSummary,
};
pub use persona::EngineeringPersona;
pub use prompt_manager::{PromptAssembly, PromptManager, PromptTemplate, PromptVersion};
pub use provider::{
    AiMessage, AiProvider, AiRequest, AiResponse, EmbedRequest, EmbedResponse, EmbeddingData,
    ModelInfo, OpenAICompatibleProvider, ProviderInfo, TokenUsage, ToolCall,
};
pub use session_manager::{SessionConfig, SessionManager, SessionState};
pub use token_budget::{BudgetAction, BudgetPolicy, TokenBudgetManager};
