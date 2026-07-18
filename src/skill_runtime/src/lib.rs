//! Skill Runtime — discovers, loads, validates, and manages skills.
//!
//! The Skill Runtime is responsible for:
//! - Discovering skill directories on disk
//! - Loading and parsing all YAML manifests (manifest.yaml, technology.yaml, intents.yaml, etc.)
//! - Validating skills against schema requirements
//! - Enabling/disabling skills at runtime
//! - Resolving and checking skill dependencies
//! - Providing access to intents, workflows, detection rules, and commands from loaded skills

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde_yaml;
use tracing::{debug, info, warn};
use wikilabs_data_types::*;

/// Skill runtime: discover, load, validate, enable/disable skills.
pub struct SkillRuntime {
    /// All loaded skills keyed by ID.
    skills: HashMap<String, LoadedSkill>,
    /// Base directory where skill directories are found.
    skill_base_dir: PathBuf,
    /// Set of enabled skill IDs.
    enabled_skills: HashMap<String, bool>,
    /// Schema version supported by this runtime.
    schema_version: String,
}

impl SkillRuntime {
    /// Create a new SkillRuntime with the given base directory for skills.
    pub fn new(skill_base_dir: &str) -> Self {
        Self {
            skills: HashMap::new(),
            skill_base_dir: PathBuf::from(skill_base_dir),
            enabled_skills: HashMap::new(),
            schema_version: "1.0".to_string(),
        }
    }

    /// Discover all skill directories in the base directory.
    ///
    /// Scans the base directory for subdirectories that contain a manifest.yaml file.
    pub fn discover_skills(&mut self) -> Result<Vec<String>> {
        let base = &self.skill_base_dir;
        if !base.exists() {
            debug!("Skill base directory does not exist: {:?}", base);
            return Ok(Vec::new());
        }

        let mut skill_ids = Vec::new();

        for entry in fs::read_dir(base).context("Failed to read skill base directory")? {
            let entry = entry?;
            let path = entry.path();

            if !path.is_dir() {
                continue;
            }

            let manifest_path = path.join("manifest.yaml");
            if manifest_path.exists() {
                let skill_id = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                skill_ids.push(skill_id.clone());
                self.enabled_skills
                    .insert(skill_id, true); // discovered skills are enabled by default
                debug!("Discovered skill: {}", path.display());
            }
        }

        info!("Discovered {} skills in {:?}", skill_ids.len(), base);
        Ok(skill_ids)
    }

    /// Load a single skill by ID from the base directory.
    ///
    /// Reads all YAML files in the skill directory and parses them into the LoadedSkill struct.
    pub fn load_skill(&mut self, skill_id: &str) -> Result<LoadedSkill> {
        let skill_path = self.skill_base_dir.join(skill_id);

        if !skill_path.exists() {
            return Err(anyhow!("Skill directory not found: {:?}", skill_path));
        }

        let mut loaded = LoadedSkill {
            manifest: SkillManifest::new(skill_id, skill_id),
            technology: None,
            intents: Vec::new(),
            workflows: Vec::new(),
            detection_rules: Vec::new(),
            commands: Vec::new(),
            best_practices: Vec::new(),
            known_issues: Vec::new(),
            validation_errors: Vec::new(),
        };

        // Load manifest.yaml
        let manifest_path = skill_path.join("manifest.yaml");
        if manifest_path.exists() {
            let content = fs::read_to_string(&manifest_path)
                .with_context(|| format!("Failed to read manifest at {:?}", manifest_path))?;
            let manifest: SkillManifest = serde_yaml::from_str(&content)
                .with_context(|| "Failed to parse manifest.yaml")?;
            loaded.manifest = manifest;
        } else {
            loaded.validation_errors.push("No manifest.yaml found".to_string());
        }

        // Load technology.yaml
        let tech_path = skill_path.join("technology.yaml");
        if tech_path.exists() {
            let content = fs::read_to_string(&tech_path)
                .with_context(|| format!("Failed to read technology at {:?}", tech_path))?;
            let tech: TechnologyDefinition = serde_yaml::from_str(&content)
                .with_context(|| "Failed to parse technology.yaml")?;
            loaded.technology = Some(tech);
        }

        // Load intents.yaml
        let intents_path = skill_path.join("intents.yaml");
        if intents_path.exists() {
            let content = fs::read_to_string(&intents_path)
                .with_context(|| format!("Failed to read intents at {:?}", intents_path))?;
            let intents_list: Vec<IntentDefinition> = serde_yaml::from_str(&content)
                .with_context(|| "Failed to parse intents.yaml")?;
            loaded.intents = intents_list;
        }

        // Load workflows.yaml
        let workflows_path = skill_path.join("workflows.yaml");
        if workflows_path.exists() {
            let content = fs::read_to_string(&workflows_path)
                .with_context(|| format!("Failed to read workflows at {:?}", workflows_path))?;
            let workflows_list: Vec<WorkflowDefinition> = serde_yaml::from_str(&content)
                .with_context(|| "Failed to parse workflows.yaml")?;
            loaded.workflows = workflows_list;
        }

        // Load detection_rules.yaml
        let rules_path = skill_path.join("detection_rules.yaml");
        if rules_path.exists() {
            let content = fs::read_to_string(&rules_path)
                .with_context(|| format!("Failed to read detection rules at {:?}", rules_path))?;
            let rules_list: Vec<DetectionRule> = serde_yaml::from_str(&content)
                .with_context(|| "Failed to parse detection_rules.yaml")?;
            loaded.detection_rules = rules_list;
        }

        // Load commands.yaml
        let commands_path = skill_path.join("commands.yaml");
        if commands_path.exists() {
            let content = fs::read_to_string(&commands_path)
                .with_context(|| format!("Failed to read commands at {:?}", commands_path))?;
            let commands_list: Vec<CommandDefinition> = serde_yaml::from_str(&content)
                .with_context(|| "Failed to parse commands.yaml")?;
            loaded.commands = commands_list;
        }

        // Load best_practices.yaml
        let bp_path = skill_path.join("best_practices.yaml");
        if bp_path.exists() {
            let content = fs::read_to_string(&bp_path)
                .with_context(|| format!("Failed to read best practices at {:?}", bp_path))?;
            // Could be a list of strings or a map
            match serde_yaml::from_str::<Vec<String>>(&content) {
                Ok(list) => loaded.best_practices = list,
                Err(_) => {
                    if let Ok(map) = serde_yaml::from_str::<HashMap<String, String>>(&content) {
                        loaded.best_practices = map.into_iter().map(|(_, v)| v).collect();
                    }
                }
            }
        }

        // Load known_issues.yaml
        let ki_path = skill_path.join("known_issues.yaml");
        if ki_path.exists() {
            let content = fs::read_to_string(&ki_path)
                .with_context(|| format!("Failed to read known issues at {:?}", ki_path))?;
            match serde_yaml::from_str::<Vec<String>>(&content) {
                Ok(list) => loaded.known_issues = list,
                Err(_) => {
                    if let Ok(map) = serde_yaml::from_str::<HashMap<String, String>>(&content) {
                        loaded.known_issues = map.into_iter().map(|(_, v)| v).collect();
                    }
                }
            }
        }

        // Validate the loaded skill
        self.validate_skill(skill_id)?;

        self.skills.insert(skill_id.to_string(), loaded.clone());
        debug!("Loaded skill: {}", skill_id);
        Ok(loaded)
    }

    /// Validate a skill by checking manifest required fields and dependency existence.
    ///
    /// Returns a list of validation errors (empty if valid).
    pub fn validate_skill(&mut self, skill_id: &str) -> Result<Vec<String>> {
        let mut errors = Vec::new();

        if let Some(skill) = self.skills.get(skill_id) {
            // Check manifest required fields
            if skill.manifest.id.is_empty() {
                errors.push("Manifest 'id' is empty".to_string());
            }
            if skill.manifest.name.is_empty() {
                errors.push("Manifest 'name' is empty".to_string());
            }
            if skill.manifest.version.is_empty() {
                errors.push("Manifest 'version' is empty".to_string());
            }
            if skill.manifest.schema_version != self.schema_version {
                warn!(
                    "Skill {} has schema version {} but runtime expects {}",
                    skill_id, skill.manifest.schema_version, self.schema_version
                );
            }

            // Check dependency existence
            for dep in &skill.manifest.dependencies {
                if !self.skills.contains_key(dep) {
                    errors.push(format!("Missing dependency: {}", dep));
                }
            }

            // Check intent patterns compile as regex
            for intent in &skill.intents {
                for pattern in &intent.patterns {
                    if let Err(e) = regex::Regex::new(pattern) {
                        errors.push(format!(
                            "Invalid regex in intent '{}': {}",
                            intent.id, e
                        ));
                    }
                }
            }

            // Check detection rule patterns compile as regex
            for rule in &skill.detection_rules {
                if let Err(e) = regex::Regex::new(&rule.pattern) {
                    errors.push(format!(
                        "Invalid regex in detection rule '{}': {}",
                        rule.id, e
                    ));
                }
            }
        }

        Ok(errors)
    }

    /// Enable a skill by ID.
    ///
    /// The skill must be loaded before enabling.
    pub fn enable_skill(&mut self, skill_id: &str) -> Result<()> {
        if !self.skills.contains_key(skill_id) {
            return Err(anyhow!("Skill not loaded: {}", skill_id));
        }
        if !self.check_dependencies(skill_id) {
            return Err(anyhow!("Skill dependencies not met for: {}", skill_id));
        }
        self.enabled_skills.insert(skill_id.to_string(), true);
        info!("Enabled skill: {}", skill_id);
        Ok(())
    }

    /// Disable a skill by ID.
    pub fn disable_skill(&mut self, skill_id: &str) -> Result<()> {
        if !self.skills.contains_key(skill_id) {
            return Err(anyhow!("Skill not loaded: {}", skill_id));
        }
        self.enabled_skills.insert(skill_id.to_string(), false);
        info!("Disabled skill: {}", skill_id);
        Ok(())
    }

    /// Check if a skill is enabled.
    pub fn is_enabled(&self, skill_id: &str) -> bool {
        self.enabled_skills.get(skill_id).copied().unwrap_or(false)
    }

    /// Get a specific loaded skill by ID.
    pub fn get_skill(&self, skill_id: &str) -> Option<&LoadedSkill> {
        self.skills.get(skill_id)
    }

    /// Get all loaded skills.
    pub fn get_all_skills(&self) -> Vec<&LoadedSkill> {
        self.skills.values().collect()
    }

    /// Get all currently enabled skills.
    pub fn get_enabled_skills(&self) -> Vec<&LoadedSkill> {
        self.skills
            .values()
            .filter(|s| self.enabled_skills.get(&s.manifest.id).copied().unwrap_or(false))
            .collect()
    }

    /// Check if all dependencies for a skill are loaded and enabled.
    pub fn check_dependencies(&self, skill_id: &str) -> bool {
        if let Some(skill) = self.skills.get(skill_id) {
            for dep in &skill.manifest.dependencies {
                // Dependency must be loaded
                if !self.skills.contains_key(dep) {
                    debug!("Dependency not loaded: {} for {}", dep, skill_id);
                    return false;
                }
                // Dependency must be enabled
                if !self.is_enabled(dep) {
                    debug!("Dependency not enabled: {} for {}", dep, skill_id);
                    return false;
                }
            }
        }
        true
    }

    /// Get the schema version supported by this runtime.
    pub fn get_schema_version(&self) -> &str {
        &self.schema_version
    }

    /// Collect all detection rules from all loaded skills.
    pub fn get_detection_rules(&self) -> Vec<&DetectionRule> {
        let mut rules: Vec<&DetectionRule> = Vec::new();
        for skill in self.skills.values() {
            rules.extend(skill.detection_rules.iter());
        }
        // Sort by priority descending
        rules.sort_by(|a, b| b.priority.cmp(&a.priority));
        rules
    }

    /// Collect all intent definitions from all loaded skills.
    pub fn get_all_intents(&self) -> Vec<&IntentDefinition> {
        let mut intents: Vec<&IntentDefinition> = Vec::new();
        for skill in self.skills.values() {
            intents.extend(skill.intents.iter());
        }
        // Sort by priority descending
        intents.sort_by(|a, b| b.priority.cmp(&a.priority));
        intents
    }

    /// Collect all workflow definitions from all loaded skills.
    pub fn get_all_workflows(&self) -> Vec<&WorkflowDefinition> {
        let mut workflows: Vec<&WorkflowDefinition> = Vec::new();
        for skill in self.skills.values() {
            workflows.extend(skill.workflows.iter());
        }
        workflows
    }

    /// Collect all command definitions from all loaded skills.
    pub fn get_all_commands(&self) -> Vec<&CommandDefinition> {
        let mut commands: Vec<&CommandDefinition> = Vec::new();
        for skill in self.skills.values() {
            commands.extend(skill.commands.iter());
        }
        commands
    }

    /// Discover, load, and validate all skills in one call.
    pub fn discover_and_load_all(&mut self) -> Result<usize> {
        let discovered = self.discover_skills()?;
        let mut loaded = 0;
        for skill_id in &discovered {
            if let Ok(skill) = self.load_skill(skill_id) {
                if skill.has_errors() {
                    warn!(
                        "Skill {} loaded with errors: {:?}",
                        skill_id, skill.validation_errors
                    );
                }
                loaded += 1;
            }
        }
        info!("Loaded {} of {} discovered skills", loaded, discovered.len());
        Ok(loaded)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn create_test_skill_dir(base: &Path, skill_id: &str, manifest: &str) {
        let skill_dir = base.join(skill_id);
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(skill_dir.join("manifest.yaml"), manifest).unwrap();
    }

    fn setup_test_runtime() -> (SkillRuntime, PathBuf) {
        // Use unique temp dir per test to avoid parallel test interference
        // Use a counter-based approach with thread-local state
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let unique_id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let temp_dir = std::env::temp_dir().join(format!(
            "wikilabs_skill_runtime_test_{}",
            unique_id
        ));
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();
        let runtime = SkillRuntime::new(temp_dir.to_str().unwrap());
        (runtime, temp_dir)
    }

    fn cleanup_test_dir(dir: &Path) {
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_new_runtime() {
        let runtime = SkillRuntime::new("/tmp/test_skills");
        assert_eq!(runtime.get_schema_version(), "1.0");
        assert!(runtime.get_all_skills().is_empty());
        assert!(runtime.get_enabled_skills().is_empty());
    }

    #[test]
    fn test_discover_no_skills() {
        let (mut runtime, temp_dir) = setup_test_runtime();
        let discovered = runtime.discover_skills().unwrap();
        assert!(discovered.is_empty());
        cleanup_test_dir(&temp_dir);
    }

    #[test]
    fn test_discover_skills() {
        let (mut runtime, temp_dir) = setup_test_runtime();
        create_test_skill_dir(
            temp_dir.as_path(),
            "test-skill",
            r#"
id: test-skill
name: Test Skill
version: "0.1.0"
description: A test skill
author: Test Author
technology_domain: test
schema_version: "1.0"
enabled: true
"#,
        );

        let discovered = runtime.discover_skills().unwrap();
        assert_eq!(discovered.len(), 1);
        assert_eq!(discovered[0], "test-skill");

        cleanup_test_dir(&temp_dir);
    }

    #[test]
    fn test_load_skill() {
        let (mut runtime, temp_dir) = setup_test_runtime();
        create_test_skill_dir(
            temp_dir.as_path(),
            "test-skill",
            r#"
id: test-skill
name: Test Skill
version: "0.1.0"
description: A test skill
author: Test Author
technology_domain: test
schema_version: "1.0"
enabled: true
dependencies: []
"#,
        );

        runtime.discover_skills().unwrap();
        let loaded = runtime.load_skill("test-skill").unwrap();

        assert_eq!(loaded.manifest.id, "test-skill");
        assert_eq!(loaded.manifest.name, "Test Skill");
        assert_eq!(loaded.manifest.version, "0.1.0");
        assert!(loaded.manifest.enabled);
        assert!(loaded.validation_errors.is_empty()); // manifest is valid with dependencies: []

        cleanup_test_dir(&temp_dir);
    }

    #[test]
    fn test_enable_disable_skill() {
        let (mut runtime, temp_dir) = setup_test_runtime();
        create_test_skill_dir(
            temp_dir.as_path(),
            "test-skill",
            r#"
id: test-skill
name: Test Skill
version: "0.1.0"
description: A test skill
author: Test Author
technology_domain: test
schema_version: "1.0"
enabled: true
dependencies: []
"#,
        );

        runtime.discover_skills().unwrap();
        let _ = runtime.load_skill("test-skill").unwrap();

        // Enable
        runtime.enable_skill("test-skill").unwrap();
        assert!(runtime.is_enabled("test-skill"));

        // Disable
        runtime.disable_skill("test-skill").unwrap();
        assert!(!runtime.is_enabled("test-skill"));

        // Enable again
        runtime.enable_skill("test-skill").unwrap();
        assert!(runtime.is_enabled("test-skill"));

        cleanup_test_dir(&temp_dir);
    }

    #[test]
    fn test_check_dependencies() {
        let (mut runtime, temp_dir) = setup_test_runtime();
        let temp_path = temp_dir.clone();

        // Create base skill
        create_test_skill_dir(
            temp_path.as_path(),
            "base-skill",
            r#"
id: base-skill
name: Base Skill
version: "0.1.0"
description: Base
author: Test
technology_domain: base
schema_version: "1.0"
enabled: true
dependencies: []
"#,
        );

        // Create dependent skill
        create_test_skill_dir(
            temp_path.as_path(),
            "dependent-skill",
            r#"
id: dependent-skill
name: Dependent Skill
version: "0.1.0"
description: Depends on base
author: Test
technology_domain: test
schema_version: "1.0"
enabled: true
dependencies: [base-skill]
"#,
        );

        runtime.discover_skills().unwrap();
        let _ = runtime.load_skill("base-skill").unwrap();
        runtime.enable_skill("base-skill").unwrap();

        let _ = runtime.load_skill("dependent-skill").unwrap();
        assert!(runtime.check_dependencies("dependent-skill"));

        // Disable base and check dependency fails
        runtime.disable_skill("base-skill").unwrap();
        assert!(!runtime.check_dependencies("dependent-skill"));

        cleanup_test_dir(&temp_dir);
    }

    #[test]
    fn test_get_all_intents() {
        let (mut runtime, temp_dir) = setup_test_runtime();
        create_test_skill_dir(
            temp_dir.as_path(),
            "test-skill",
            r#"
id: test-skill
name: Test Skill
version: "0.1.0"
description: A test skill
author: Test Author
technology_domain: test
schema_version: "1.0"
enabled: true
dependencies: []
"#,
        );

        runtime.discover_skills().unwrap();
        let _ = runtime.load_skill("test-skill").unwrap();

        let intents = runtime.get_all_intents();
        assert!(intents.is_empty()); // No intents.yaml created

        cleanup_test_dir(&temp_dir);
    }

    #[test]
    fn test_get_all_skills() {
        let (mut runtime, temp_dir) = setup_test_runtime();
        create_test_skill_dir(
            temp_dir.as_path(),
            "skill-a",
            r#"
id: skill-a
name: Skill A
version: "0.1.0"
description: First skill
author: Test
technology_domain: test
schema_version: "1.0"
enabled: true
dependencies: []
"#,
        );
        create_test_skill_dir(
            temp_dir.as_path(),
            "skill-b",
            r#"
id: skill-b
name: Skill B
version: "0.1.0"
description: Second skill
author: Test
technology_domain: test
schema_version: "1.0"
enabled: true
dependencies: []
"#,
        );

        runtime.discover_skills().unwrap();
        let _ = runtime.load_skill("skill-a").unwrap();
        let _ = runtime.load_skill("skill-b").unwrap();

        let all = runtime.get_all_skills();
        assert_eq!(all.len(), 2);

        cleanup_test_dir(&temp_dir);
    }

    #[test]
    fn test_get_enabled_skills() {
        let (mut runtime, temp_dir) = setup_test_runtime();
        create_test_skill_dir(
            temp_dir.as_path(),
            "enabled-skill",
            r#"
id: enabled-skill
name: Enabled
version: "0.1.0"
description: This skill is enabled
author: Test
technology_domain: test
schema_version: "1.0"
enabled: true
dependencies: []
"#,
        );
        create_test_skill_dir(
            temp_dir.as_path(),
            "disabled-skill",
            r#"
id: disabled-skill
name: Disabled
version: "0.1.0"
description: This skill is disabled
author: Test
technology_domain: test
schema_version: "1.0"
enabled: false
dependencies: []
"#,
        );

        runtime.discover_skills().unwrap();
        let _ = runtime.load_skill("enabled-skill").unwrap();
        let _ = runtime.load_skill("disabled-skill").unwrap();

        runtime.disable_skill("enabled-skill").unwrap();
        runtime.enable_skill("disabled-skill").unwrap();

        let enabled = runtime.get_enabled_skills();
        assert_eq!(enabled.len(), 1);
        assert_eq!(enabled[0].manifest.id, "disabled-skill");

        cleanup_test_dir(&temp_dir);
    }

    #[test]
    fn test_validate_skill_invalid_manifest() {
        let (mut runtime, temp_dir) = setup_test_runtime();
        create_test_skill_dir(
            temp_dir.as_path(),
            "bad-skill",
            r#"
id: ""
name: ""
version: ""
description: Bad skill
author: Test
technology_domain: test
schema_version: "99.0"
enabled: true
dependencies: []
"#,
        );

        runtime.discover_skills().unwrap();
        let _ = runtime.load_skill("bad-skill").unwrap();

        let errors = runtime.validate_skill("bad-skill").unwrap();
        assert!(!errors.is_empty());

        cleanup_test_dir(&temp_dir);
    }

    #[test]
    fn test_enable_without_load() {
        let (mut runtime, _temp_dir) = setup_test_runtime();
        let result = runtime.enable_skill("nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not loaded"));
    }

    #[test]
    fn test_disable_without_load() {
        let (mut runtime, _temp_dir) = setup_test_runtime();
        let result = runtime.disable_skill("nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not loaded"));
    }

    #[test]
    fn test_skill_not_loaded() {
        let (mut runtime, _temp_dir) = setup_test_runtime();
        let skill = runtime.get_skill("nonexistent");
        assert!(skill.is_none());
    }

    #[test]
    fn test_get_schema_version() {
        let runtime = SkillRuntime::new("/tmp/test");
        assert_eq!(runtime.get_schema_version(), "1.0");
    }
}