//! AI runtime abstraction.
//!
//! - AiProvider trait (OpenAI, vLLM, Ollama)
//! - AiRequest / AiResponse types
//! - ContextWindow manager
//! - Token counting
//! - Streaming support

pub mod provider;
pub mod context;
pub mod token_counter;
pub mod response;

#[cfg(test)]
mod tests;