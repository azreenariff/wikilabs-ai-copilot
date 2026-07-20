/// Wiki Labs AI Copilot — Guidance Engine
///
/// Provides AI-driven guidance features for engineers:
///
/// - **Decision Engine**: Structured decision-making with rules and reasoning
/// - **Recommendation Framework**: Generate, rank, and track recommendations
/// - **Evidence Framework**: Collect, evaluate, and chain evidence
/// - **Workflow Framework**: Map actions to workflows and track progress
/// - **Timeline**: Track guidance history, engineer actions, and outcomes
/// - **Context Providers**: Read-only providers for external systems (MCP)
/// - **Command Recommendation**: CLI command suggestions with risk classification
/// - **Safety Framework**: Command risk classification and warning generation
/// - **Feedback System**: Engineer feedback for adaptive session behavior
///
/// Core principle: The AI assists engineers but never replaces them.
/// The engineer performs all actions. The AI observes, explains, recommends, and guides.

pub mod command_recommendation;
pub mod context_providers;
pub mod decision_engine;
pub mod evidence_framework;
pub mod feedback_system;
pub mod recommendation_framework;
pub mod safety_framework;
pub mod timeline;
pub mod workflow_framework;