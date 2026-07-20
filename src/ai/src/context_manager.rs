//! Context Manager — central aggregation of context sources.
//!
//! Aggregates information from multiple sources:
//! - Current conversation
//! - Active workspace
//! - User preferences
//! - Selected AI model
//! - Active technologies (manual selection)
//! - Current engineering task (manual selection)
//! - Future inputs (screen observation, MCP results — reserved)

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Priority level for a context source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub enum ContextPriority {
    /// Highest priority — cannot be truncated.
    High,
    /// Normal priority — truncated before Low.
    Normal,
    /// Lowest priority — truncated first.
    Low,
}

/// A single context source with its contribution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSource {
    /// Unique identifier for this source.
    pub id: Uuid,
    /// Human-readable name.
    pub name: String,
    /// The context text/content.
    pub content: String,
    /// Priority of this source.
    pub priority: ContextPriority,
    /// Whether this is a manual (user-provided) source.
    pub is_manual: bool,
    /// Optional tags for filtering.
    #[serde(default)]
    pub tags: Vec<String>,
}

impl ContextSource {
    pub fn new(
        name: impl Into<String>,
        content: impl Into<String>,
        priority: ContextPriority,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            content: content.into(),
            priority,
            is_manual: false,
            tags: Vec::new(),
        }
    }

    pub fn manual(
        name: impl Into<String>,
        content: impl Into<String>,
        priority: ContextPriority,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            content: content.into(),
            priority,
            is_manual: true,
            tags: Vec::new(),
        }
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }
}

/// Aggregated context with all sources combined.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedContext {
    /// Conversation messages (API-ready format).
    pub conversation_messages: Vec<serde_json::Value>,
    /// System prompt.
    pub system_prompt: String,
    /// Workspace context.
    pub workspace_context: String,
    /// User preferences context.
    pub user_preferences: String,
    /// Technology stack.
    pub technology_stack: Vec<String>,
    /// Current activity/task.
    pub current_activity: Option<String>,
    /// Selected AI model.
    pub selected_model: String,
    /// All context sources.
    pub sources: Vec<ContextSource>,
    /// Total estimated tokens.
    pub estimated_tokens: usize,
}

impl AggregatedContext {
    /// Build an aggregated context from its components.
    pub fn new(
        conversation_messages: Vec<serde_json::Value>,
        system_prompt: String,
        workspace_context: String,
        user_preferences: String,
        technology_stack: Vec<String>,
        current_activity: Option<String>,
        selected_model: String,
        sources: Vec<ContextSource>,
        estimated_tokens: usize,
    ) -> Self {
        Self {
            conversation_messages,
            system_prompt,
            workspace_context,
            user_preferences,
            technology_stack,
            current_activity,
            selected_model,
            sources,
            estimated_tokens,
        }
    }

    /// Get the full prompt text for the AI.
    pub fn full_prompt(&self) -> String {
        let mut prompt = self.system_prompt.clone();

        if !self.workspace_context.is_empty() {
            prompt.push_str("\n\n## Workspace Context\n");
            prompt.push_str(&self.workspace_context);
        }

        if !self.user_preferences.is_empty() {
            prompt.push_str("\n\n## User Preferences\n");
            prompt.push_str(&self.user_preferences);
        }

        if !self.technology_stack.is_empty() {
            prompt.push_str("\n\n## Active Technologies\n");
            prompt.push_str(&self.technology_stack.join(", "));
        }

        if let Some(activity) = &self.current_activity {
            prompt.push_str("\n\n## Current Activity\n");
            prompt.push_str(activity);
        }

        prompt.push_str("\n\n## Conversation\n");
        prompt.push_str("\n(See conversation_messages in structured format)");

        prompt
    }

    /// Serialize to JSON.
    pub fn to_json(&self) -> anyhow::Result<String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| anyhow::anyhow!("Failed to serialize context: {}", e))
    }
}

/// Builder for constructing context incrementally.
#[derive(Debug, Default)]
pub struct ContextBuilder {
    system_prompt: Option<String>,
    workspace_context: String,
    user_preferences: String,
    technology_stack: Vec<String>,
    current_activity: Option<String>,
    selected_model: String,
    sources: Vec<ContextSource>,
}

impl ContextBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }

    pub fn with_workspace_context(mut self, context: impl Into<String>) -> Self {
        self.workspace_context = context.into();
        self
    }

    pub fn with_user_preferences(mut self, prefs: impl Into<String>) -> Self {
        self.user_preferences = prefs.into();
        self
    }

    pub fn with_technologies(mut self, technologies: Vec<String>) -> Self {
        self.technology_stack = technologies;
        self
    }

    pub fn with_current_activity(mut self, activity: impl Into<String>) -> Self {
        self.current_activity = Some(activity.into());
        self
    }

    pub fn with_selected_model(mut self, model: impl Into<String>) -> Self {
        self.selected_model = model.into();
        self
    }

    pub fn add_source(mut self, source: ContextSource) -> Self {
        self.sources.push(source);
        self
    }

    /// Build the aggregated context.
    pub fn build(
        self,
        conversation_messages: Vec<serde_json::Value>,
        estimated_tokens: usize,
    ) -> AggregatedContext {
        AggregatedContext::new(
            conversation_messages,
            self.system_prompt.unwrap_or_default(),
            self.workspace_context,
            self.user_preferences,
            self.technology_stack,
            self.current_activity,
            self.selected_model,
            self.sources,
            estimated_tokens,
        )
    }
}

/// Central Context Manager for aggregating information from multiple sources.
pub struct ContextManager {
    /// Current system prompt.
    system_prompt: String,
    /// Workspace context (text).
    workspace_context: String,
    /// User preferences (text).
    user_preferences: String,
    /// Active technologies (manual selection).
    technology_stack: Vec<String>,
    /// Current engineering task (manual selection).
    current_activity: Option<String>,
    /// Selected AI model.
    selected_model: String,
    /// All context sources.
    sources: Vec<ContextSource>,
}

impl ContextManager {
    pub fn new(system_prompt: impl Into<String>, selected_model: impl Into<String>) -> Self {
        Self {
            system_prompt: system_prompt.into(),
            workspace_context: String::new(),
            user_preferences: String::new(),
            technology_stack: Vec::new(),
            current_activity: None,
            selected_model: selected_model.into(),
            sources: Vec::new(),
        }
    }

    /// Set the system prompt.
    pub fn set_system_prompt(&mut self, prompt: impl Into<String>) {
        self.system_prompt = prompt.into();
    }

    /// Set workspace context.
    pub fn set_workspace_context(&mut self, context: impl Into<String>) {
        self.workspace_context = context.into();
    }

    /// Set user preferences.
    pub fn set_user_preferences(&mut self, prefs: impl Into<String>) {
        self.user_preferences = prefs.into();
    }

    /// Set active technologies.
    pub fn set_technologies(&mut self, technologies: Vec<String>) {
        self.technology_stack = technologies;
    }

    /// Add a technology.
    pub fn add_technology(&mut self, tech: impl Into<String>) {
        let tech = tech.into();
        if !self.technology_stack.contains(&tech) {
            self.technology_stack.push(tech);
        }
    }

    /// Remove a technology.
    pub fn remove_technology(&mut self, tech: &str) {
        self.technology_stack.retain(|t| t != tech);
    }

    /// Set current activity.
    pub fn set_current_activity(&mut self, activity: impl Into<String>) {
        self.current_activity = Some(activity.into());
    }

    /// Clear current activity.
    pub fn clear_current_activity(&mut self) {
        self.current_activity = None;
    }

    /// Set the selected AI model.
    pub fn set_selected_model(&mut self, model: impl Into<String>) {
        self.selected_model = model.into();
    }

    /// Add a context source.
    pub fn add_source(&mut self, source: ContextSource) {
        self.sources.push(source);
    }

    /// Remove a source by ID.
    pub fn remove_source(&mut self, id: Uuid) -> bool {
        if let Some(pos) = self.sources.iter().position(|s| s.id == id) {
            self.sources.remove(pos);
            true
        } else {
            false
        }
    }

    /// Get all sources, sorted by priority (high first).
    pub fn sources(&self) -> &[ContextSource] {
        &self.sources
    }

    /// Get sources filtered by tag.
    pub fn sources_by_tag(&self, tag: &str) -> Vec<&ContextSource> {
        self.sources
            .iter()
            .filter(|s| s.tags.contains(&tag.to_string()))
            .collect()
    }

    /// Get sources by priority.
    pub fn sources_by_priority(&self, priority: ContextPriority) -> Vec<&ContextSource> {
        self.sources
            .iter()
            .filter(|s| s.priority == priority)
            .collect()
    }

    /// Aggregate all context sources into a single text block.
    pub fn aggregate_source_texts(&self) -> String {
        let mut texts = Vec::new();
        let mut sorted = self.sources.clone();
        sorted.sort_by(|a, b| b.priority.cmp(&a.priority));
        for source in &sorted {
            if !source.content.is_empty() {
                texts.push(format!("## {}\n{}\n", source.name, source.content));
            }
        }
        texts.join("\n")
    }

    /// Build aggregated context.
    pub fn build_context(
        &self,
        conversation_messages: Vec<serde_json::Value>,
        estimated_tokens: usize,
    ) -> AggregatedContext {
        AggregatedContext::new(
            conversation_messages,
            self.system_prompt.clone(),
            self.workspace_context.clone(),
            self.user_preferences.clone(),
            self.technology_stack.clone(),
            self.current_activity.clone(),
            self.selected_model.clone(),
            self.sources.clone(),
            estimated_tokens,
        )
    }

    /// Get the system prompt.
    pub fn system_prompt(&self) -> &str {
        &self.system_prompt
    }

    /// Get the selected model.
    pub fn selected_model(&self) -> &str {
        &self.selected_model
    }

    /// Get the current activity.
    pub fn current_activity(&self) -> Option<&str> {
        self.current_activity.as_deref()
    }

    /// Get technologies.
    pub fn technologies(&self) -> &[String] {
        &self.technology_stack
    }

    /// Clone the current state as a builder.
    pub fn to_builder(&self) -> ContextBuilder {
        ContextBuilder::new()
            .with_system_prompt(&self.system_prompt)
            .with_workspace_context(&self.workspace_context)
            .with_user_preferences(&self.user_preferences)
            .with_technologies(self.technology_stack.clone())
            .with_selected_model(&self.selected_model)
            .add_source(ContextSource::new(
                "system_prompt",
                &self.system_prompt,
                ContextPriority::High,
            ))
            .add_source(ContextSource::new(
                "workspace_context",
                &self.workspace_context,
                ContextPriority::Normal,
            ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_manager_new() {
        let cm = ContextManager::new("Test system", "gpt-4");
        assert_eq!(cm.system_prompt(), "Test system");
        assert_eq!(cm.selected_model(), "gpt-4");
        assert!(cm.technologies().is_empty());
        assert!(cm.current_activity().is_none());
    }

    #[test]
    fn test_add_technology() {
        let mut cm = ContextManager::new("sys", "model");
        cm.add_technology("rust");
        cm.add_technology("kubernetes");
        cm.add_technology("rust"); // duplicate
        assert_eq!(cm.technologies().len(), 2);
        assert!(cm.technologies().contains(&"rust".to_string()));
    }

    #[test]
    fn test_remove_technology() {
        let mut cm = ContextManager::new("sys", "model");
        cm.add_technology("rust");
        cm.add_technology("go");
        cm.remove_technology("rust");
        assert_eq!(cm.technologies().len(), 1);
        assert_eq!(cm.technologies()[0], "go");
    }

    #[test]
    fn test_set_current_activity() {
        let mut cm = ContextManager::new("sys", "model");
        cm.set_current_activity("Debugging pod crash");
        assert_eq!(cm.current_activity(), Some("Debugging pod crash"));

        cm.clear_current_activity();
        assert!(cm.current_activity().is_none());
    }

    #[test]
    fn test_add_source() {
        let mut cm = ContextManager::new("sys", "model");
        let source = ContextSource::new(
            "debug_log",
            "Error: connection refused",
            ContextPriority::High,
        );
        cm.add_source(source);
        assert_eq!(cm.sources().len(), 1);
        assert_eq!(cm.sources()[0].name, "debug_log");
    }

    #[test]
    fn test_remove_source() {
        let mut cm = ContextManager::new("sys", "model");
        let source = ContextSource::new("test", "content", ContextPriority::Low);
        let id = source.id;
        cm.add_source(source);
        assert!(cm.remove_source(id));
        assert!(!cm.remove_source(id)); // second remove should fail
    }

    #[test]
    fn test_sources_by_priority() {
        let mut cm = ContextManager::new("sys", "model");
        cm.add_source(ContextSource::new("high1", "a", ContextPriority::High));
        cm.add_source(ContextSource::new("low1", "b", ContextPriority::Low));
        cm.add_source(ContextSource::new("high2", "c", ContextPriority::High));
        cm.add_source(ContextSource::new("normal1", "d", ContextPriority::Normal));

        assert_eq!(cm.sources_by_priority(ContextPriority::High).len(), 2);
        assert_eq!(cm.sources_by_priority(ContextPriority::Low).len(), 1);
        assert_eq!(cm.sources_by_priority(ContextPriority::Normal).len(), 1);
    }

    #[test]
    fn test_sources_by_tag() {
        let mut cm = ContextManager::new("sys", "model");
        cm.add_source(
            ContextSource::new("tagged", "a", ContextPriority::High)
                .with_tags(vec!["error".to_string()]),
        );
        cm.add_source(ContextSource::new("untagged", "b", ContextPriority::Low));

        assert_eq!(cm.sources_by_tag("error").len(), 1);
        assert_eq!(cm.sources_by_tag("missing").len(), 0);
    }

    #[test]
    fn test_aggregate_source_texts() {
        let mut cm = ContextManager::new("sys", "model");
        cm.add_source(ContextSource::new(
            "log",
            "Error occurred",
            ContextPriority::High,
        ));
        cm.add_source(ContextSource::new(
            "config",
            "Setting=X",
            ContextPriority::Normal,
        ));

        let aggregated = cm.aggregate_source_texts();
        assert!(aggregated.contains("Error occurred"));
        assert!(aggregated.contains("Setting=X"));
    }

    #[test]
    fn test_build_context() {
        let cm = ContextManager::new("You are an engineer.", "gpt-4");
        let msgs = vec![serde_json::json!({ "role": "user", "content": "Hi" })];
        let ctx = cm.build_context(msgs, 100);

        assert_eq!(ctx.system_prompt, "You are an engineer.");
        assert_eq!(ctx.selected_model, "gpt-4");
        assert_eq!(ctx.estimated_tokens, 100);
        assert!(!ctx.conversation_messages.is_empty());
    }

    #[test]
    fn test_full_prompt() {
        let mut cm = ContextManager::new("Engineer", "model");
        cm.add_technology("kubernetes");
        cm.set_current_activity("Troubleshooting");
        let msgs = vec![];
        let ctx = cm.build_context(msgs, 50);

        let prompt = ctx.full_prompt();
        assert!(prompt.contains("Engineer"));
        assert!(prompt.contains("kubernetes"));
        assert!(prompt.contains("Troubleshooting"));
    }

    #[test]
    fn test_context_builder() {
        let builder = ContextBuilder::new()
            .with_system_prompt("Test prompt")
            .with_technologies(vec!["rust".to_string()])
            .with_selected_model("o3");

        let ctx = builder.build(vec![], 10);
        assert_eq!(ctx.system_prompt, "Test prompt");
        assert_eq!(ctx.selected_model, "o3");
        assert_eq!(ctx.technology_stack.len(), 1);
    }

    #[test]
    fn test_manual_source() {
        let source = ContextSource::manual(
            "manual_context",
            "ABC Bank / OpenShift",
            ContextPriority::High,
        );
        assert!(source.is_manual);
        assert_eq!(source.name, "manual_context");
    }

    #[test]
    fn test_to_builder() {
        let mut cm = ContextManager::new("sys prompt", "m1");
        cm.add_technology("go");
        cm.set_current_activity("Deploying");
        cm.set_workspace_context("Prod env");

        let builder = cm.to_builder();
        let ctx = builder.build(vec![], 0);
        assert_eq!(ctx.system_prompt, "sys prompt");
        assert_eq!(ctx.technology_stack.len(), 1);
    }

    #[test]
    fn test_to_json() {
        let cm = ContextManager::new("sys", "m1");
        let ctx = cm.build_context(vec![], 50);
        let json = ctx.to_json().unwrap();
        assert!(json.contains("sys"));
        assert!(json.contains("m1"));

        // Verify round-trip
        let parsed: AggregatedContext = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.system_prompt, "sys");
    }
}
