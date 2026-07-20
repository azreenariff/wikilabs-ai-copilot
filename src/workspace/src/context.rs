//! Workspace context — knowledge association and skill selection.

use serde_json::Value;

/// Builder for constructing a workspace context document.
#[derive(Debug, Default)]
pub struct ContextBuilder {
    tech_stack: Vec<String>,
    active_skills: Vec<String>,
    recent_knowledge: Vec<String>,
}

impl ContextBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_technologies(mut self, techs: &[&str]) -> Self {
        self.tech_stack = techs.iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn with_skills(mut self, skills: &[&str]) -> Self {
        self.active_skills = skills.iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn with_recent_knowledge(mut self, knowledge: &[&str]) -> Self {
        self.recent_knowledge = knowledge.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Build a JSON context object.
    pub fn build(&self) -> Value {
        serde_json::json!({
            "technology_stack": self.tech_stack,
            "active_skills": self.active_skills,
            "recent_knowledge": self.recent_knowledge,
        })
    }
}

/// Context manager — builds and caches context per workspace.
pub struct ContextManager {
    contexts: std::collections::HashMap<uuid::Uuid, Value>,
}

impl ContextManager {
    pub fn new() -> Self {
        Self {
            contexts: std::collections::HashMap::new(),
        }
    }

    pub fn get_or_build_context(
        &mut self,
        workspace_id: uuid::Uuid,
        builder: ContextBuilder,
    ) -> Value {
        self.contexts
            .entry(workspace_id)
            .or_insert_with(|| builder.build())
            .clone()
    }

    pub fn update_context(&mut self, workspace_id: uuid::Uuid, builder: ContextBuilder) -> Value {
        let ctx = builder.build();
        self.contexts.insert(workspace_id, ctx.clone());
        ctx
    }

    pub fn get_context(&self, workspace_id: &uuid::Uuid) -> Option<&Value> {
        self.contexts.get(workspace_id)
    }

    pub fn remove_context(&mut self, workspace_id: &uuid::Uuid) {
        self.contexts.remove(workspace_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_builder_default() {
        let builder = ContextBuilder::new();
        let ctx = builder.build();
        assert!(ctx["technology_stack"].is_array());
        assert!(ctx["active_skills"].is_array());
        assert!(ctx["recent_knowledge"].is_array());
    }

    #[test]
    fn test_context_builder_with_data() {
        let builder = ContextBuilder::new()
            .with_technologies(&["rust", "kubernetes"])
            .with_skills(&["knowledge-retrieval"])
            .with_recent_knowledge(&["pod crash", "restart"]);

        let ctx = builder.build();
        assert_eq!(ctx["technology_stack"].as_array().unwrap().len(), 2);
        assert_eq!(ctx["active_skills"].as_array().unwrap().len(), 1);
        assert_eq!(ctx["recent_knowledge"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_context_manager_new() {
        let cm = ContextManager::new();
        let id = uuid::Uuid::new_v4();
        assert!(cm.get_context(&id).is_none());
    }

    #[test]
    fn test_context_manager_or_build() {
        let mut cm = ContextManager::new();
        let id = uuid::Uuid::new_v4();
        let builder = ContextBuilder::new().with_technologies(&["rust"]);

        let ctx = cm.get_or_build_context(id, builder);
        assert_eq!(ctx["technology_stack"].as_array().unwrap().len(), 1);

        // Calling again should return cached value
        let ctx2 = cm.get_or_build_context(id, ContextBuilder::new());
        assert_eq!(ctx2["technology_stack"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_context_manager_update() {
        let mut cm = ContextManager::new();
        let id = uuid::Uuid::new_v4();

        let ctx1 = cm.get_or_build_context(id, ContextBuilder::new());
        assert_eq!(ctx1["technology_stack"].as_array().unwrap().len(), 0);

        let ctx2 = cm.update_context(id, ContextBuilder::new().with_technologies(&["go", "java"]));
        assert_eq!(ctx2["technology_stack"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_context_manager_remove() {
        let mut cm = ContextManager::new();
        let id = uuid::Uuid::new_v4();
        cm.get_or_build_context(id, ContextBuilder::new());
        cm.remove_context(&id);
        assert!(cm.get_context(&id).is_none());
    }
}
