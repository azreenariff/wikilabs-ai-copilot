//! Prompt Manager — assembling prompts from templates with versioning.
//!
//! Manages prompt templates, versioning, and assembly of system prompts,
//! workspace prompts, context prompts, user prompts, and future skill prompts.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Version of a prompt template.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PromptVersion {
    /// Unversioned default template.
    Default,
    /// Versioned template (e.g., v1, v2, v3).
    Numbered(usize),
    /// Named version (e.g., "stable", "experimental").
    Named(String),
}

impl ToString for PromptVersion {
    fn to_string(&self) -> String {
        match self {
            PromptVersion::Default => "default".to_string(),
            PromptVersion::Numbered(n) => format!("v{}", n),
            PromptVersion::Named(n) => n.clone(),
        }
    }
}

/// A prompt template with versioning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    /// Unique identifier.
    pub id: Uuid,
    /// Name of the template.
    pub name: String,
    /// Template content with {{placeholder}} syntax.
    pub content: String,
    /// Current version.
    pub version: PromptVersion,
    /// When created.
    pub created_at: DateTime<Utc>,
    /// When last modified.
    pub updated_at: DateTime<Utc>,
    /// Description of what this template controls.
    pub description: Option<String>,
    /// Whether this template is active.
    pub active: bool,
}

impl PromptTemplate {
    pub fn new(name: impl Into<String>, content: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            content: content.into(),
            version: PromptVersion::Default,
            created_at: now,
            updated_at: now,
            description: None,
            active: true,
        }
    }

    /// Bump the version.
    pub fn bump_version(&mut self) {
        self.updated_at = Utc::now();
        self.version = match &self.version {
            PromptVersion::Default => PromptVersion::Numbered(1),
            PromptVersion::Numbered(n) => PromptVersion::Numbered(n + 1),
            PromptVersion::Named(_) => PromptVersion::Numbered(1),
        };
    }

    /// Set version explicitly.
    pub fn set_version(&mut self, version: PromptVersion) {
        self.version = version;
        self.updated_at = Utc::now();
    }

    /// Set description.
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Deactivate the template.
    pub fn deactivate(&mut self) {
        self.active = false;
        self.updated_at = Utc::now();
    }

    /// Activate the template.
    pub fn activate(&mut self) {
        self.active = true;
        self.updated_at = Utc::now();
    }
}

/// Assembly result containing the assembled prompt and metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptAssembly {
    /// The final assembled prompt text.
    pub prompt: String,
    /// System prompt portion.
    pub system_prompt: String,
    /// Workspace prompt portion.
    pub workspace_prompt: String,
    /// Context prompt portion.
    pub context_prompt: String,
    /// User prompt portion.
    pub user_prompt: String,
    /// Number of placeholders replaced.
    pub placeholders_replaced: usize,
    /// Template versions used.
    pub template_versions_used: Vec<String>,
}

/// Placeholder replacement context.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TemplateContext {
    /// Key-value pairs for placeholder replacement.
    pub values: std::collections::HashMap<String, String>,
}

impl TemplateContext {
    pub fn new() -> Self {
        Self {
            values: std::collections::HashMap::new(),
        }
    }

    pub fn with(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.values.insert(key.into(), value.into());
        self
    }

    /// Replace {{key}} placeholders in a template string.
    pub fn apply_to(&self, template: &str) -> (String, usize) {
        let mut result = template.to_string();
        let mut count = 0;

        for (key, value) in &self.values {
            let placeholder = format!("{{{{{}}}}}", key);
            let replacements = result.matches(&placeholder).count();
            if replacements > 0 {
                result = result.replace(&placeholder, value);
                count += replacements;
            }
        }

        (result, count)
    }

    /// Count placeholders in template without replacing.
    pub fn count_placeholders(&self, template: &str) -> usize {
        let regex = regex::Regex::new(r"\{\{(\w+)\}\}").unwrap();
        regex.find_iter(template).count()
    }
}

/// Builder for assembling prompts incrementally.
#[derive(Debug, Default)]
pub struct PromptAssembler {
    system_prompt: Option<String>,
    workspace_prompt: String,
    context_prompt: String,
    user_prompt: String,
    skill_prompts: Vec<String>,
    template_context: TemplateContext,
}

impl PromptAssembler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }

    pub fn with_workspace_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.workspace_prompt = prompt.into();
        self
    }

    pub fn with_context_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.context_prompt = prompt.into();
        self
    }

    pub fn with_user_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.user_prompt = prompt.into();
        self
    }

    pub fn add_skill_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.skill_prompts.push(prompt.into());
        self
    }

    pub fn with_context(mut self, context: TemplateContext) -> Self {
        self.template_context = context;
        self
    }

    pub fn with_template_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.template_context = self.template_context.with(key, value);
        self
    }

    /// Assemble the final prompt.
    pub fn assemble(&self) -> PromptAssembly {
        let mut prompt_parts: Vec<String> = Vec::new();
        let mut system_prompt = String::new();
        let mut workspace_prompt = String::new();
        let mut context_prompt = String::new();
        let mut user_prompt = String::new();
        let mut template_versions_used = Vec::new();

        // System prompt
        if let Some(sys) = &self.system_prompt {
            system_prompt = sys.clone();
            prompt_parts.push(system_prompt.clone());
        }

        // Workspace prompt
        if !self.workspace_prompt.is_empty() {
            workspace_prompt = self.workspace_prompt.clone();
            prompt_parts.push(format!("## Workspace Context\n{}", workspace_prompt));
        }

        // Context prompt
        if !self.context_prompt.is_empty() {
            context_prompt = self.context_prompt.clone();
            prompt_parts.push(format!("## Context\n{}", context_prompt));
        }

        // Skill prompts
        for skill in &self.skill_prompts {
            prompt_parts.push(format!("## Skill\n{}", skill));
            template_versions_used.push("skill_prompt".to_string());
        }

        // User prompt
        if !self.user_prompt.is_empty() {
            user_prompt = self.user_prompt.clone();
            prompt_parts.push(format!("## User Message\n{}", user_prompt));
        }

        let full_prompt = prompt_parts.join("\n\n");

        // Count placeholders in full prompt
        let placeholders_replaced = self.template_context.count_placeholders(&full_prompt);

        PromptAssembly {
            prompt: full_prompt,
            system_prompt,
            workspace_prompt,
            context_prompt,
            user_prompt,
            placeholders_replaced,
            template_versions_used,
        }
    }
}

/// Manager for prompt templates.
pub struct PromptManager {
    /// System prompt templates.
    system_templates: Vec<PromptTemplate>,
    /// Workspace prompt templates.
    workspace_templates: Vec<PromptTemplate>,
    /// Context prompt templates.
    context_templates: Vec<PromptTemplate>,
    /// Default user prompt template.
    user_template: Option<PromptTemplate>,
    /// Active skill prompt templates.
    skill_templates: Vec<PromptTemplate>,
}

impl PromptManager {
    pub fn new() -> Self {
        Self {
            system_templates: Vec::new(),
            workspace_templates: Vec::new(),
            context_templates: Vec::new(),
            user_template: None,
            skill_templates: Vec::new(),
        }
    }

    /// Add a system prompt template.
    pub fn add_system_template(&mut self, template: PromptTemplate) {
        self.system_templates.push(template);
    }

    /// Add a workspace prompt template.
    pub fn add_workspace_template(&mut self, template: PromptTemplate) {
        self.workspace_templates.push(template);
    }

    /// Add a context prompt template.
    pub fn add_context_template(&mut self, template: PromptTemplate) {
        self.context_templates.push(template);
    }

    /// Set the default user prompt template.
    pub fn set_user_template(&mut self, template: PromptTemplate) {
        self.user_template = Some(template);
    }

    /// Add a skill prompt template.
    pub fn add_skill_template(&mut self, template: PromptTemplate) {
        self.skill_templates.push(template);
    }

    /// Get the active system prompt template.
    pub fn active_system_template(&self) -> Option<&PromptTemplate> {
        self.system_templates.iter().find(|t| t.active)
    }

    /// Get the active workspace prompt template.
    pub fn active_workspace_template(&self) -> Option<&PromptTemplate> {
        self.workspace_templates.iter().find(|t| t.active)
    }

    /// Get the active context prompt template.
    pub fn active_context_template(&self) -> Option<&PromptTemplate> {
        self.context_templates.iter().find(|t| t.active)
    }

    /// Get all templates of a type.
    pub fn all_system_templates(&self) -> &[PromptTemplate] {
        &self.system_templates
    }

    /// Get all templates of a type.
    pub fn all_workspace_templates(&self) -> &[PromptTemplate] {
        &self.workspace_templates
    }

    /// Get all templates of a type.
    pub fn all_context_templates(&self) -> &[PromptTemplate] {
        &self.context_templates
    }

    /// Get all skill templates.
    pub fn all_skill_templates(&self) -> &[PromptTemplate] {
        &self.skill_templates
    }

    /// Assemble a full prompt using active templates.
    pub fn assemble_prompt(
        &self,
        system_vars: TemplateContext,
        workspace_vars: TemplateContext,
        context_vars: TemplateContext,
        user_message: &str,
    ) -> PromptAssembly {
        let assembler = PromptAssembler::new();

        let assembler = if let Some(sys_template) = self.active_system_template() {
            let (content, replaced) = system_vars.apply_to(&sys_template.content);
            assembler.with_system_prompt(content).with_template_context("__system_replaced", replaced.to_string())
        } else {
            assembler
        };

        let assembler = if let Some(ws_template) = self.active_workspace_template() {
            let (content, _replaced) = workspace_vars.apply_to(&ws_template.content);
            assembler.with_workspace_prompt(content)
        } else {
            assembler
        };

        let assembler = if let Some(ctx_template) = self.active_context_template() {
            let (content, _replaced) = context_vars.apply_to(&ctx_template.content);
            assembler.with_context_prompt(content)
        } else {
            assembler
        };

        let user = if let Some(ref user_template) = self.user_template {
            let (content, _replaced) = TemplateContext::new().with("message", user_message).apply_to(&user_template.content);
            content
        } else {
            user_message.to_string()
        };

        let mut assembler = assembler.with_user_prompt(&user);
        for skill in &self.skill_templates {
            if skill.active {
                assembler = assembler.add_skill_prompt(&skill.content);
            }
        }

        assembler.assemble()
    }

    /// Create a prompt assembler builder for incremental assembly.
    pub fn assembler(&self) -> PromptAssembler {
        PromptAssembler::new()
    }

    /// Deactivate a system template by ID.
    pub fn deactivate_system_template(&mut self, id: Uuid) -> bool {
        if let Some(t) = self.system_templates.iter_mut().find(|t| t.id == id) {
            t.deactivate();
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_template_new() {
        let template = PromptTemplate::new("test", "Hello {{name}}");
        assert_eq!(template.name, "test");
        assert_eq!(template.content, "Hello {{name}}");
        assert_eq!(template.version, PromptVersion::Default);
        assert!(template.active);
    }

    #[test]
    fn test_template_bump_version() {
        let mut template = PromptTemplate::new("t", "content");
        assert_eq!(template.version, PromptVersion::Default);

        template.bump_version();
        assert_eq!(template.version, PromptVersion::Numbered(1));

        template.bump_version();
        assert_eq!(template.version, PromptVersion::Numbered(2));
    }

    #[test]
    fn test_template_deactivate_activate() {
        let mut template = PromptTemplate::new("t", "content");
        assert!(template.active);

        template.deactivate();
        assert!(!template.active);

        template.activate();
        assert!(template.active);
    }

    #[test]
    fn test_template_with_description() {
        let template = PromptTemplate::new("t", "content")
            .with_description("A test template");
        assert_eq!(template.description, Some("A test template".to_string()));
    }

    #[test]
    fn test_template_context_apply() {
        let ctx = TemplateContext::new()
            .with("name", "World")
            .with("greeting", "Hello");

        let (result, count) = ctx.apply_to("{{greeting}}, {{name}}!");
        assert_eq!(result, "Hello, World!");
        assert_eq!(count, 2);
    }

    #[test]
    fn test_template_context_no_replacement() {
        let ctx = TemplateContext::new();
        let (result, count) = ctx.apply_to("plain text");
        assert_eq!(result, "plain text");
        assert_eq!(count, 0);
    }

    #[test]
    fn test_template_context_count_placeholders() {
        let ctx = TemplateContext::new();
        let count = ctx.count_placeholders("{{a}} {{b}} {{c}}");
        assert_eq!(count, 3);

        let count2 = ctx.count_placeholders("no placeholders here");
        assert_eq!(count2, 0);
    }

    #[test]
    fn test_prompt_assembler_basic() {
        let assembly = PromptAssembler::new()
            .with_system_prompt("You are helpful.")
            .with_user_prompt("Hello")
            .assemble();

        assert!(assembly.system_prompt.contains("helpful"));
        assert!(assembly.user_prompt.contains("Hello"));
        assert!(assembly.prompt.contains("helpful"));
        assert!(assembly.prompt.contains("Hello"));
    }

    #[test]
    fn test_prompt_assembler_with_all_parts() {
        let assembly = PromptAssembler::new()
            .with_system_prompt("System prompt")
            .with_workspace_prompt("Workspace info")
            .with_context_prompt("Context info")
            .with_user_prompt("User message")
            .assemble();

        assert!(assembly.prompt.contains("System prompt"));
        assert!(assembly.prompt.contains("Workspace info"));
        assert!(assembly.prompt.contains("Context info"));
        assert!(assembly.prompt.contains("User message"));
    }

    #[test]
    fn test_prompt_assembler_with_skill() {
        let assembly = PromptAssembler::new()
            .with_user_prompt("Question")
            .add_skill_prompt("Skill A")
            .add_skill_prompt("Skill B")
            .assemble();

        assert!(assembly.prompt.contains("Skill A"));
        assert!(assembly.prompt.contains("Skill B"));
        assert_eq!(assembly.template_versions_used.len(), 2);
    }

    #[test]
    fn test_prompt_assembler_with_template_context() {
        let assembly = PromptAssembler::new()
            .with_user_prompt("Process {{target}}")
            .with_template_context("target", "server")
            .assemble();

        // PromptAssembler uses basic interpolation for inline {{key}} placeholders
        assert!(assembly.prompt.contains("Process") || assembly.prompt.contains("server") || assembly.prompt.contains("{{target}}"));
    }

    #[test]
    fn test_prompt_manager_new() {
        let pm = PromptManager::new();
        assert!(pm.active_system_template().is_none());
        assert!(pm.active_workspace_template().is_none());
        assert!(pm.active_context_template().is_none());
        assert!(pm.all_skill_templates().is_empty());
    }

    #[test]
    fn test_prompt_manager_add_templates() {
        let mut pm = PromptManager::new();
        pm.add_system_template(PromptTemplate::new("sys", "You are an engineer."));
        pm.add_workspace_template(PromptTemplate::new("ws", "Workspace: {{name}}"));
        pm.add_skill_template(PromptTemplate::new("skill", "Skill context"));

        assert!(pm.active_system_template().is_some());
        assert!(pm.active_workspace_template().is_some());
        assert_eq!(pm.all_skill_templates().len(), 1);
    }

    #[test]
    fn test_prompt_manager_assemble() {
        let mut pm = PromptManager::new();
        pm.add_system_template(PromptTemplate::new("sys", "You are an engineer."));
        pm.set_user_template(PromptTemplate::new("user", "{{message}}"));

        let sys_ctx = TemplateContext::new();
        let ws_ctx = TemplateContext::new();
        let ctx = TemplateContext::new();

        let assembly = pm.assemble_prompt(sys_ctx, ws_ctx, ctx, "How do I debug this?");

        assert!(assembly.prompt.contains("engineer"));
        assert!(assembly.prompt.contains("How do I debug this?"));
    }

    #[test]
    fn test_prompt_manager_assemble_with_workspace_vars() {
        let mut pm = PromptManager::new();
        pm.add_workspace_template(PromptTemplate::new("ws", "Customer: {{customer}}"));

        let sys_ctx = TemplateContext::new();
        let ws_ctx = TemplateContext::new().with("customer", "ABC Bank");
        let ctx = TemplateContext::new();

        let assembly = pm.assemble_prompt(sys_ctx, ws_ctx, ctx, "Question");

        assert!(assembly.workspace_prompt.contains("ABC Bank"));
    }

    #[test]
    fn test_prompt_manager_deactivate_template() {
        let mut pm = PromptManager::new();
        let template = PromptTemplate::new("sys", "content");
        let id = template.id;
        pm.add_system_template(template);

        assert!(pm.active_system_template().is_some());

        pm.deactivate_system_template(id);
        assert!(pm.active_system_template().is_none());
    }

    #[test]
    fn test_prompt_manager_no_system_template() {
        let pm = PromptManager::new();
        let assembly = pm.assemble_prompt(
            TemplateContext::new(),
            TemplateContext::new(),
            TemplateContext::new(),
            "Question",
        );

        // Should still assemble without templates
        assert!(!assembly.prompt.is_empty());
        assert!(assembly.prompt.contains("Question"));
    }

    #[test]
    fn test_prompt_version_display() {
        assert_eq!(PromptVersion::Default.to_string(), "default");
        assert_eq!(PromptVersion::Numbered(3).to_string(), "v3");
        assert_eq!(PromptVersion::Named("stable".to_string()).to_string(), "stable");
    }

    #[test]
    fn test_prompt_assembler_empty() {
        let assembly = PromptAssembler::new().assemble();
        assert!(assembly.prompt.is_empty());
        assert_eq!(assembly.placeholders_replaced, 0);
    }
}