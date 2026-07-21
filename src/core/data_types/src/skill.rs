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

// ────────────────────────────────────────────────────────
// Phase 11 — Enterprise Skill Platform Types
// ────────────────────────────────────────────────────────

/// How a technology was detected, driving skill discovery.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DetectionSource {
    /// Detected via browser URL / web content
    Browser,
    /// Detected via terminal command or output
    Terminal,
    /// Detected via file system patterns
    Filesystem,
    /// Detected via active window title
    ActiveWindow,
    /// Detected via environment variables
    Environment,
    /// Detected via running processes
    Process,
    /// User explicitly specified
    User,
}

impl std::fmt::Display for DetectionSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DetectionSource::Browser => write!(f, "browser"),
            DetectionSource::Terminal => write!(f, "terminal"),
            DetectionSource::Filesystem => write!(f, "filesystem"),
            DetectionSource::ActiveWindow => write!(f, "active_window"),
            DetectionSource::Environment => write!(f, "environment"),
            DetectionSource::Process => write!(f, "process"),
            DetectionSource::User => write!(f, "user"),
        }
    }
}

/// Evidence that supports a skill activation decision.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActivationEvidence {
    /// Source of the evidence.
    pub source: DetectionSource,
    /// The specific signal (e.g., "oc version", "vcenter.example.com").
    pub signal: String,
    /// Confidence contributed by this evidence point (0.0–1.0).
    pub confidence: f32,
    /// Human-readable explanation.
    pub explanation: String,
}

impl ActivationEvidence {
    pub fn new(
        source: DetectionSource,
        signal: impl Into<String>,
        confidence: f32,
        explanation: impl Into<String>,
    ) -> Self {
        Self {
            source,
            signal: signal.into(),
            confidence,
            explanation: explanation.into(),
        }
    }
}

/// Lifecycle state of a skill during runtime.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillLifecycle {
    /// Skill is installed but not available (disabled).
    Disabled,
    /// Skill is available and loaded.
    Available,
    /// Skill is actively matched to current context.
    Active,
    /// Skill is temporarily inactive (e.g., low confidence).
    Suspended,
}

impl std::fmt::Display for SkillLifecycle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SkillLifecycle::Disabled => write!(f, "disabled"),
            SkillLifecycle::Available => write!(f, "available"),
            SkillLifecycle::Active => write!(f, "active"),
            SkillLifecycle::Suspended => write!(f, "suspended"),
        }
    }
}

/// Confidence level for skill activation decisions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfidenceLevel {
    /// Very low — should not activate.
    Low,
    /// Moderate — conditional activation.
    Moderate,
    /// High — confident activation.
    High,
}

impl ConfidenceLevel {
    pub fn from_score(score: f32) -> Self {
        let s = score.clamp(0.0, 1.0);
        if s >= 0.8 {
            ConfidenceLevel::High
        } else if s >= 0.5 {
            ConfidenceLevel::Moderate
        } else {
            ConfidenceLevel::Low
        }
    }
}

/// Result of skill discovery for a given technology.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkillDiscoveryResult {
    /// Skill ID that was discovered.
    pub skill_id: String,
    /// Display name.
    pub skill_name: String,
    /// Combined confidence score (0.0–1.0).
    pub confidence: f32,
    /// Derived confidence level.
    pub confidence_level: ConfidenceLevel,
    /// All evidence pieces supporting this discovery.
    pub evidence: Vec<ActivationEvidence>,
    /// Whether this skill should be activated.
    pub should_activate: bool,
    /// Technology domain matched.
    pub technology_domain: String,
}

impl SkillDiscoveryResult {
    pub fn new(
        skill_id: impl Into<String>,
        skill_name: impl Into<String>,
        confidence: f32,
        technology_domain: impl Into<String>,
    ) -> Self {
        let confidence = confidence.clamp(0.0, 1.0);
        Self {
            skill_id: skill_id.into(),
            skill_name: skill_name.into(),
            confidence,
            confidence_level: ConfidenceLevel::from_score(confidence),
            evidence: Vec::new(),
            should_activate: confidence >= 0.5,
            technology_domain: technology_domain.into(),
        }
    }

    /// Add evidence and recalculate combined confidence.
    pub fn add_evidence(mut self, evidence: ActivationEvidence) -> Self {
        self.evidence.push(evidence);
        if !self.evidence.is_empty() {
            let avg: f32 = self
                .evidence
                .iter()
                .map(|e| e.confidence)
                .sum::<f32>()
                / self.evidence.len() as f32;
            self.confidence = avg;
            self.confidence_level = ConfidenceLevel::from_score(avg);
            self.should_activate = avg >= 0.5;
        }
        self
    }
}

/// Result of the dynamic activation engine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActivationResult {
    /// The skill that was activated.
    pub skill_id: String,
    /// Current lifecycle state.
    pub lifecycle: SkillLifecycle,
    /// Confidence score.
    pub confidence: f32,
    /// Confidence level.
    pub confidence_level: ConfidenceLevel,
    /// All evidence supporting activation.
    pub evidence: Vec<ActivationEvidence>,
    /// Workspace restrictions that may limit this skill.
    pub restrictions: Vec<String>,
    /// Whether activation was approved or blocked.
    pub approved: bool,
    /// Timestamp of activation (ISO 8601).
    pub activated_at: String,
}

impl ActivationResult {
    pub fn new(
        skill_id: impl Into<String>,
        confidence: f32,
        lifecycle: SkillLifecycle,
        evidence: Vec<ActivationEvidence>,
        restrictions: Vec<String>,
        approved: bool,
        activated_at: String,
    ) -> Self {
        Self {
            skill_id: skill_id.into(),
            lifecycle,
            confidence,
            confidence_level: ConfidenceLevel::from_score(confidence),
            evidence,
            restrictions,
            approved,
            activated_at,
        }
    }
}

/// Skill catalog entry — what the registry knows about each skill.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkillCatalogEntry {
    /// Skill ID.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Version.
    pub version: String,
    /// Technology domain.
    pub technology_domain: String,
    /// Current lifecycle state.
    pub lifecycle: SkillLifecycle,
    /// Current confidence score (0.0–1.0).
    pub confidence: f32,
    /// Dependencies.
    pub dependencies: Vec<String>,
    /// Whether the skill is available for activation.
    pub available: bool,
    /// Last health check timestamp.
    pub last_healthy: String,
    /// Validation errors (empty if valid).
    pub validation_errors: Vec<String>,
}

/// Workspace policy — restrictions set by the customer/workspace.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkspacePolicy {
    /// Customer or organization name.
    pub customer: String,
    /// Technologies explicitly allowed.
    pub allowed_technologies: Vec<String>,
    /// Technologies explicitly disallowed.
    pub blocked_technologies: Vec<String>,
    /// Whether production guidance is permitted.
    pub allow_production_guidance: bool,
    /// Whether self-remediation guidance is permitted.
    pub allow_self_remediation: bool,
    /// List of SOP document references.
    pub sop_references: Vec<String>,
}

impl WorkspacePolicy {
    /// Check if a technology is allowed by this policy.
    pub fn is_technology_allowed(&self, technology: &str) -> bool {
        let tech = technology.to_lowercase();
        if self
            .blocked_technologies
            .iter()
            .any(|b| b.to_lowercase() == tech)
        {
            return false;
        }
        if !self.allowed_technologies.is_empty() {
            return self
                .allowed_technologies
                .iter()
                .any(|a| a.to_lowercase() == tech);
        }
        true
    }

    /// Check if production guidance is allowed for this technology.
    pub fn allow_production_guidance(&self, technology: &str) -> bool {
        self.allow_production_guidance && self.is_technology_allowed(technology)
    }

    /// Get all active restrictions as human-readable strings.
    pub fn active_restrictions(&self) -> Vec<String> {
        let mut restrictions = Vec::new();
        if !self.blocked_technologies.is_empty() {
            restrictions.push(format!(
                "Blocked technologies: {}",
                self.blocked_technologies.join(", ")
            ));
        }
        if !self.allow_production_guidance {
            restrictions.push("No production modification guidance".to_string());
        }
        if !self.allow_self_remediation {
            restrictions.push("Self-remediation guidance disabled".to_string());
        }
        restrictions
    }
}

/// Skill package metadata — format for .wls-skill packages.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkillPackageMetadata {
    /// Package version.
    pub package_version: String,
    /// Format spec version.
    pub format_version: String,
    /// Skill ID.
    pub skill_id: String,
    /// Skill name.
    pub skill_name: String,
    /// Skill version.
    pub skill_version: String,
    /// Vendor/author.
    pub vendor: String,
    /// Technology domain.
    pub technology_domain: String,
    /// Files included in package.
    pub files: Vec<String>,
    /// MD5 checksum of package (if verified).
    pub checksum: Option<String>,
}
