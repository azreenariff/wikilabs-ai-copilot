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

pub mod context_providers;
pub mod decision_engine;
pub mod evidence_framework;
pub mod recommendation_framework;
pub mod timeline;
pub mod workflow_framework;