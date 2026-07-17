//! Skill system data types.
//!
//! Defines the core structures for the Skill Runtime and Skill SDK:
//! - SkillManifest — YAML-parsed skill metadata
//! - TechnologyDefinition — technology domain description
//! - IntentDefinition — technology-aware intent patterns
//! - WorkflowDefinition — workflow states and transitions
//! - DetectionRule — pattern-matching rules for technology detection
//! - CommandDefinition — skill-provided CLI commands

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Manifest parsed from manifest.yaml in a skill directory.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SkillManifest {
    /// Unique identifier for this skill (e.g., "openshift-advanced").
    pub id: String,
    /// Display name.
    pub name: String,
    /// Semantic version string.
    pub version: String,
    /// Short human-readable description.
    pub description: String,
    /// Skill author or team.
    pub author: String,
    /// Technology domain this skill covers (e.g., "openshift", "linux").
    pub technology_domain: String,
    /// Dependency skill IDs that must be loaded first.
    pub dependencies: Vec<String>,
    /// Whether the skill is enabled by default.
    pub enabled: bool,
    /// Schema version for forward compatibility.
    pub schema_version: String,
    /// Optional keywords for search.
    #[serde(default)]
    pub keywords: Vec<String>,
    /// Optional icon name or URL.
    #[serde(default)]
    pub icon: String,
    /// Optional tags for categorization.
    #[serde(default)]
    pub tags: Vec<String>,
}

impl SkillManifest {
    /// Create a new manifest with default values.
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            version: "0.1.0".to_string(),
            description: String::new(),
            author: String::new(),
            technology_domain: String::new(),
            dependencies: Vec::new(),
            enabled: true,
            schema_version: "1.0".to_string(),
            keywords: Vec::new(),
            icon: String::new(),
            tags: Vec::new(),
        }
    }
}

/// Technology definition parsed from technology.yaml.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TechnologyDefinition {
    /// Primary domain identifier (e.g., "openshift").
    pub domain: String,
    /// Technology version this skill targets.
    pub version: String,
    /// Full description of the technology.
    pub description: String,
    /// List of key features or capabilities.
    pub features: Vec<String>,
    /// Related technology domains.
    #[serde(default)]
    pub related_domains: Vec<String>,
    /// Optional documentation URL.
    #[serde(default)]
    pub documentation_url: String,
}

impl TechnologyDefinition {
    /// Create a new technology definition with defaults.
    pub fn new(domain: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            domain: domain.into(),
            version: version.into(),
            description: String::new(),
            features: Vec::new(),
            related_domains: Vec::new(),
            documentation_url: String::new(),
        }
    }
}

/// Intent definition from intents.yaml — technology-specific recognition patterns.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IntentDefinition {
    /// Unique intent identifier.
    pub id: String,
    /// Intent name.
    pub name: String,
    /// Description of when this intent triggers.
    pub description: String,
    /// Regex patterns that match this intent.
    pub patterns: Vec<String>,
    /// Confidence boost when matched.
    #[serde(default = "default_intent_confidence")]
    pub confidence_boost: f32,
    /// Required technology domain.
    #[serde(default)]
    pub required_domain: String,
    /// Optional priority (higher = matched first).
    #[serde(default)]
    pub priority: u32,
}

fn default_intent_confidence() -> f32 {
    0.75
}

/// Workflow definition from workflows.yaml.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkflowDefinition {
    /// Unique workflow identifier.
    pub id: String,
    /// Workflow name.
    pub name: String,
    /// Description of the workflow's purpose.
    pub description: String,
    /// Sequential states the workflow passes through.
    pub states: Vec<WorkflowState>,
    /// Allowed state transitions.
    #[serde(default)]
    pub transitions: Vec<WorkflowTransition>,
    /// Evidence required to complete the workflow.
    #[serde(default)]
    pub evidence_requirements: Vec<String>,
    /// Whether this workflow is optional.
    #[serde(default = "default_true")]
    pub required: bool,
}

fn default_true() -> bool {
    true
}

/// A single state in a workflow.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkflowState {
    /// State identifier.
    pub id: String,
    /// State name.
    pub name: String,
    /// State description.
    pub description: String,
    /// Whether this state is the initial state.
    #[serde(default)]
    pub initial: bool,
    /// Whether this state is the terminal state.
    #[serde(default)]
    pub terminal: bool,
    /// Optional commands to run when entering this state.
    #[serde(default)]
    pub commands: Vec<String>,
}

/// Allowed transition between workflow states.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkflowTransition {
    /// Source state ID.
    pub from: String,
    /// Target state ID.
    pub to: String,
    /// Condition for the transition.
    #[serde(default)]
    pub condition: String,
    /// Description of this transition.
    #[serde(default)]
    pub description: String,
}

/// Detection rule from detection_rules.yaml — matches technology artifacts.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DetectionRule {
    /// Rule identifier.
    pub id: String,
    /// Rule name.
    pub name: String,
    /// What type of artifact this detects (file, command, pattern, etc.).
    pub detection_type: DetectionType,
    /// The pattern to match.
    pub pattern: String,
    /// Confidence value when matched (0.0–1.0).
    #[serde(default = "default_detection_confidence")]
    pub confidence: f32,
    /// Which technology domain this rule belongs to.
    pub technology_domain: String,
    /// Priority (higher = evaluated first).
    #[serde(default)]
    pub priority: u32,
    /// Optional: regex flags.
    #[serde(default)]
    pub flags: String,
    /// Optional: what to extract as a capture group.
    #[serde(default)]
    pub extract: Option<String>,
}

fn default_detection_confidence() -> f32 {
    0.8
}

/// Type of artifact detected.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DetectionType {
    /// File or directory path pattern.
    File,
    /// Shell command output.
    Command,
    /// Text pattern in file content.
    Pattern,
    /// Command-line argument or flag.
    Argument,
    /// Environment variable.
    Environment,
}

impl std::fmt::Display for DetectionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DetectionType::File => write!(f, "file"),
            DetectionType::Command => write!(f, "command"),
            DetectionType::Pattern => write!(f, "pattern"),
            DetectionType::Argument => write!(f, "argument"),
            DetectionType::Environment => write!(f, "environment"),
        }
    }
}

/// Command definition from commands.yaml.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CommandDefinition {
    /// Command identifier.
    pub id: String,
    /// Command name.
    pub name: String,
    /// The command to execute.
    pub command: String,
    /// Description of what this command does.
    pub description: String,
    /// Whether the command requires sudo/elevation.
    #[serde(default)]
    pub requires_elevation: bool,
    /// Safety classification.
    #[serde(default = "default_safety")]
    pub safety_level: SafetyLevel,
    /// Technology domain this command belongs to.
    #[serde(default)]
    pub technology_domain: String,
    /// Whether this command is read-only.
    #[serde(default)]
    pub read_only: bool,
    /// Optional: allowed arguments.
    #[serde(default)]
    pub allowed_args: Vec<String>,
}

fn default_safety() -> SafetyLevel {
    SafetyLevel::ReadOnly
}

/// Safety level for commands.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SafetyLevel {
    /// Read-only — never modifies state.
    ReadOnly,
    /// Safe — may modify state but is generally safe.
    Safe,
    /// Destructive — may cause data loss.
    Destructive,
}

impl std::fmt::Display for SafetyLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SafetyLevel::ReadOnly => write!(f, "readonly"),
            SafetyLevel::Safe => write!(f, "safe"),
            SafetyLevel::Destructive => write!(f, "destructive"),
        }
    }
}

/// Skill discovery metadata — what was found on disk.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SkillDiscovery {
    /// The directory path where the skill was found.
    pub path: String,
    /// Whether the skill has a valid manifest.
    pub has_manifest: bool,
    /// Whether the skill has a valid technology definition.
    pub has_technology: bool,
    /// Whether the skill is enabled.
    pub enabled: bool,
    /// Any validation errors found.
    pub validation_errors: Vec<String>,
}

/// Skill loading result.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LoadedSkill {
    /// The skill manifest.
    pub manifest: SkillManifest,
    /// Technology definition (if available).
    pub technology: Option<TechnologyDefinition>,
    /// Intent definitions.
    pub intents: Vec<IntentDefinition>,
    /// Workflow definitions.
    pub workflows: Vec<WorkflowDefinition>,
    /// Detection rules.
    pub detection_rules: Vec<DetectionRule>,
    /// Command definitions.
    pub commands: Vec<CommandDefinition>,
    /// Best practices from best_practices.yaml.
    pub best_practices: Vec<String>,
    /// Known issues from known_issues.yaml.
    pub known_issues: Vec<String>,
    /// Validation errors encountered during loading.
    pub validation_errors: Vec<String>,
}

impl LoadedSkill {
    /// Check if this skill has any validation errors.
    pub fn has_errors(&self) -> bool {
        !self.validation_errors.is_empty()
    }

    /// Get all intent IDs from this skill.
    pub fn intent_ids(&self) -> Vec<&str> {
        self.intents.iter().map(|i| i.id.as_str()).collect()
    }

    /// Get all workflow IDs from this skill.
    pub fn workflow_ids(&self) -> Vec<&str> {
        self.workflows.iter().map(|w| w.id.as_str()).collect()
    }

    /// Get all command IDs from this skill.
    pub fn command_ids(&self) -> Vec<&str> {
        self.commands.iter().map(|c| c.id.as_str()).collect()
    }
}