//! Skill SDK — Skill Development Kit for creating, validating, and generating skills.
//!
//! The Skill SDK provides:
//! - Template generation for new skills
//! - Schema validation for skill components
//! - Pre-built schema registry for all YAML files
//! - Developer documentation

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde_json::Value;
use tracing::{debug, info};
use wikilabs_data_types::*;

/// Skill Development Kit for creating and validating skills.
pub struct SkillSDK {
    /// Directory containing template files.
    templates_dir: PathBuf,
    /// Directory containing JSON schema files.
    schemas_dir: PathBuf,
    /// Registry of schemas indexed by name.
    schema_registry: SchemaRegistry,
}

/// Registry of schemas for different skill components.
pub struct SchemaRegistry {
    schemas: HashMap<String, Value>,
}

/// Output of skill template generation.
pub struct SkillTemplate {
    /// Name of the generated skill.
    pub skill_name: String,
    /// Directory structure of the template.
    pub directory_structure: Vec<String>,
    /// Generated files with paths, content, and schema.
    pub generated_files: Vec<GeneratedFile>,
}

/// A generated file in a skill template.
pub struct GeneratedFile {
    /// Relative file path within the skill directory.
    pub path: String,
    /// File content.
    pub content: String,
    /// Schema used for this file (e.g., "manifest").
    pub schema: String,
}

/// Validation report for a skill.
pub struct ValidationReport {
    /// Whether the skill passed all validations.
    pub is_valid: bool,
    /// List of validation errors.
    pub errors: Vec<String>,
    /// List of non-critical warnings.
    pub warnings: Vec<String>,
    /// List of schema names that were checked.
    pub checked_schemas: Vec<String>,
}

impl SchemaRegistry {
    /// Create a new schema registry from a directory of JSON schema files.
    fn from_dir(dir: &Path) -> Result<Self> {
        let mut schemas = HashMap::new();

        if !dir.exists() {
            return Ok(Self { schemas });
        }

        for entry in fs::read_dir(dir).context("Failed to read schemas directory")? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().map(|e| e == "json").unwrap_or(false) {
                let schema_name = path
                    .file_stem()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                let content = fs::read_to_string(&path)
                    .with_context(|| format!("Failed to read schema: {:?}", path))?;
                let value: Value = serde_json::from_str(&content)
                    .with_context(|| format!("Failed to parse schema: {:?}", path))?;
                schemas.insert(schema_name, value);
            }
        }

        Ok(Self { schemas })
    }

    /// Get a schema by name.
    pub fn get_schema(&self, name: &str) -> Option<&Value> {
        self.schemas.get(name)
    }

    /// Get all registered schema names.
    pub fn schema_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.schemas.keys().cloned().collect();
        names.sort();
        names
    }

    /// Check if a schema exists.
    pub fn has_schema(&self, name: &str) -> bool {
        self.schemas.contains_key(name)
    }
}

impl SkillSDK {
    /// Create a new SkillSDK pointing to the given directory.
    ///
    /// The directory should contain:
    /// - templates/ — YAML template files
    /// - schemas/ — JSON schema files
    pub fn new(sdk_dir: &str) -> Result<Self> {
        let base = PathBuf::from(sdk_dir);
        let templates_dir = base.join("templates");
        let schemas_dir = base.join("schemas");

        let schema_registry = SchemaRegistry::from_dir(&schemas_dir)?;

        Ok(Self {
            templates_dir,
            schemas_dir,
            schema_registry,
        })
    }

    /// Create a new skill template with the given name.
    ///
    /// Returns a SkillTemplate with all generated files and directory structure.
    /// Does NOT write to disk — use the generated_files to write manually.
    pub fn create_skill_template(&self, skill_name: &str) -> Result<SkillTemplate> {
        let name = skill_name.replace(' ', "-");

        let mut generated_files = Vec::new();
        let mut directory_structure = Vec::new();

        // Generate manifest.yaml
        let manifest_content = self.generate_manifest(&name, &name, "0.1.0").to_string();
        generated_files.push(GeneratedFile {
            path: "manifest.yaml".to_string(),
            content: manifest_content,
            schema: "manifest".to_string(),
        });
        directory_structure.push(name.clone());

        // Generate technology.yaml
        let tech_content = self.generate_tech_definition(&name, &name).to_string();
        generated_files.push(GeneratedFile {
            path: "technology.yaml".to_string(),
            content: tech_content,
            schema: "technology".to_string(),
        });

        // Generate intents.yaml
        let intents_content = self.generate_workflows(&name).to_string();
        generated_files.push(GeneratedFile {
            path: "intents.yaml".to_string(),
            content: intents_content,
            schema: "intents".to_string(),
        });

        // Generate workflows.yaml
        let workflows_content = self.generate_workflows(&name).to_string();
        generated_files.push(GeneratedFile {
            path: "workflows.yaml".to_string(),
            content: workflows_content,
            schema: "workflows".to_string(),
        });

        // Generate detection_rules.yaml
        let rules_content = self.generate_detection_rules(&name).to_string();
        generated_files.push(GeneratedFile {
            path: "detection_rules.yaml".to_string(),
            content: rules_content,
            schema: "detection_rules".to_string(),
        });

        // Generate commands.yaml
        generated_files.push(GeneratedFile {
            path: "commands.yaml".to_string(),
            content: "---\n# Command definitions for this skill\n# See: https://docs.wikilabs.ai/skill-schema-reference\ncommands: []\n".to_string(),
            schema: "commands".to_string(),
        });

        // Generate best_practices.yaml
        generated_files.push(GeneratedFile {
            path: "best_practices.yaml".to_string(),
            content: "---\n# Best practices for this skill\n# See: https://docs.wikilabs.ai/skill-schema-reference\n- Verify all detections before acting\n- Ask for confirmation before destructive actions\n- Document any manual steps taken\n".to_string(),
            schema: "best_practices".to_string(),
        });

        // Generate known_issues.yaml
        generated_files.push(GeneratedFile {
            path: "known_issues.yaml".to_string(),
            content: "---\n# Known issues and limitations for this skill\n# See: https://docs.wikilabs.ai/skill-schema-reference\n- No known issues for new skill\n".to_string(),
            schema: "known_issues".to_string(),
        });

        // Add subdirectory
        directory_structure.push("templates/".to_string());
        directory_structure.push("schemas/".to_string());

        Ok(SkillTemplate {
            skill_name: name,
            directory_structure,
            generated_files,
        })
    }

    /// Validate a skill at the given path.
    ///
    /// Checks that all required files exist and parses them to verify structure.
    pub fn validate_skill(&self, skill_path: &str) -> Result<ValidationReport> {
        let path = PathBuf::from(skill_path);
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut checked_schemas = Vec::new();

        if !path.exists() {
            return Ok(ValidationReport {
                is_valid: false,
                errors: vec![format!("Skill directory does not exist: {:?}", path)],
                warnings,
                checked_schemas,
            });
        }

        // Check for manifest.yaml (required)
        let manifest_path = path.join("manifest.yaml");
        if !manifest_path.exists() {
            errors.push("Missing required file: manifest.yaml".to_string());
        } else {
            checked_schemas.push("manifest".to_string());
            if let Ok(content) = fs::read_to_string(&manifest_path) {
                match serde_yaml::from_str::<Value>(&content) {
                    Ok(val) => {
                        if let Some(id) = val.get("id") {
                            if id.as_str().map(|s| s.is_empty()).unwrap_or(true) {
                                errors.push("manifest.yaml: 'id' field is empty".to_string());
                            }
                        } else {
                            errors.push("manifest.yaml: missing required 'id' field".to_string());
                        }
                        if let Some(name) = val.get("name") {
                            if name.as_str().map(|s| s.is_empty()).unwrap_or(true) {
                                errors.push("manifest.yaml: 'name' field is empty".to_string());
                            }
                        } else {
                            errors.push("manifest.yaml: missing required 'name' field".to_string());
                        }
                        if let Some(version) = val.get("version") {
                            if version.as_str().map(|s| s.is_empty()).unwrap_or(true) {
                                errors.push("manifest.yaml: 'version' field is empty".to_string());
                            }
                        } else {
                            errors.push("manifest.yaml: missing required 'version' field".to_string());
                        }
                    }
                    Err(e) => {
                        errors.push(format!("manifest.yaml: YAML parse error: {}", e));
                    }
                }
            }
        }

        // Check for technology.yaml (optional but recommended)
        let tech_path = path.join("technology.yaml");
        if tech_path.exists() {
            checked_schemas.push("technology".to_string());
            if let Ok(content) = fs::read_to_string(&tech_path) {
                match serde_yaml::from_str::<Value>(&content) {
                    Ok(_) => {}
                    Err(e) => {
                        errors.push(format!("technology.yaml: YAML parse error: {}", e));
                    }
                }
            }
        } else {
            warnings.push("No technology.yaml — skill has no technology definition".to_string());
        }

        // Check for intents.yaml (optional)
        let intents_path = path.join("intents.yaml");
        if intents_path.exists() {
            checked_schemas.push("intents".to_string());
            if let Ok(content) = fs::read_to_string(&intents_path) {
                match serde_yaml::from_str::<Vec<Value>>(&content) {
                    Ok(items) => {
                        for (i, item) in items.iter().enumerate() {
                            if item.get("id").and_then(|v| v.as_str()).map(|s| s.is_empty()).unwrap_or(true) {
                                errors.push(format!("intents.yaml[{}]: 'id' is empty", i));
                            }
                            if item.get("patterns").is_none() {
                                errors.push(format!("intents.yaml[{}]: missing 'patterns' field", i));
                            }
                        }
                    }
                    Err(e) => {
                        errors.push(format!("intents.yaml: YAML parse error: {}", e));
                    }
                }
            }
        } else {
            warnings.push("No intents.yaml — skill has no intent definitions".to_string());
        }

        // Check for detection_rules.yaml (optional)
        let rules_path = path.join("detection_rules.yaml");
        if rules_path.exists() {
            checked_schemas.push("detection_rules".to_string());
            if let Ok(content) = fs::read_to_string(&rules_path) {
                match serde_yaml::from_str::<Vec<Value>>(&content) {
                    Ok(_) => {}
                    Err(e) => {
                        errors.push(format!("detection_rules.yaml: YAML parse error: {}", e));
                    }
                }
            }
        }

        // Check for workflows.yaml (optional)
        let workflows_path = path.join("workflows.yaml");
        if workflows_path.exists() {
            checked_schemas.push("workflows".to_string());
            if let Ok(content) = fs::read_to_string(&workflows_path) {
                match serde_yaml::from_str::<Vec<Value>>(&content) {
                    Ok(_) => {}
                    Err(e) => {
                        errors.push(format!("workflows.yaml: YAML parse error: {}", e));
                    }
                }
            }
        }

        // Check for commands.yaml (optional)
        let commands_path = path.join("commands.yaml");
        if commands_path.exists() {
            checked_schemas.push("commands".to_string());
            if let Ok(content) = fs::read_to_string(&commands_path) {
                match serde_yaml::from_str::<Value>(&content) {
                    Ok(_) => {}
                    Err(e) => {
                        errors.push(format!("commands.yaml: YAML parse error: {}", e));
                    }
                }
            }
        }

        let is_valid = errors.is_empty();
        if is_valid {
            info!("Skill '{}' validation passed", skill_path);
        } else {
            info!(
                "Skill '{}' validation failed: {} errors, {} warnings",
                skill_path,
                errors.len(),
                warnings.len()
            );
        }

        Ok(ValidationReport {
            is_valid,
            errors,
            warnings,
            checked_schemas,
        })
    }

    /// Get a schema by name from the registry.
    pub fn get_schema(&self, schema_name: &str) -> Option<&Value> {
        self.schema_registry.get_schema(schema_name)
    }

    /// Get all registered schema names.
    pub fn get_all_schema_names(&self) -> Vec<String> {
        self.schema_registry.schema_names()
    }

    /// Generate a manifest YAML value from parameters.
    pub fn generate_manifest(&self, name: &str, domain: &str, version: &str) -> Value {
        serde_json::json!({
            "id": name,
            "name": name.replace('-', " ").replace('_', " "),
            "version": version,
            "description": "A skill for {} technology".replace("{}", domain),
            "author": "Wiki Labs Team",
            "technology_domain": domain,
            "dependencies": [],
            "enabled": true,
            "schema_version": "1.0",
            "keywords": [],
            "icon": "",
            "tags": []
        })
    }

    /// Generate a workflows YAML value from parameters.
    pub fn generate_workflows(&self, name: &str) -> Value {
        serde_json::json!({
            "id": format!("{}-workflow", name),
            "name": format!("{} Workflow", name.replace('-', " ").replace('_', " ")),
            "description": format!("Workflow for {} skill", name),
            "states": [
                {
                    "id": "discovery",
                    "name": "Discovery",
                    "description": "Identify relevant technologies and configurations",
                    "initial": true,
                    "terminal": false,
                    "commands": []
                },
                {
                    "id": "analysis",
                    "name": "Analysis",
                    "description": "Analyze the detected configuration",
                    "initial": false,
                    "terminal": false,
                    "commands": []
                },
                {
                    "id": "resolution",
                    "name": "Resolution",
                    "description": "Propose and apply resolution",
                    "initial": false,
                    "terminal": true,
                    "commands": []
                }
            ],
            "transitions": [
                {"from": "discovery", "to": "analysis", "condition": "sufficient_data", "description": "Move to analysis when enough data is collected"},
                {"from": "analysis", "to": "resolution", "condition": "clear_recommendation", "description": "Move to resolution when recommendation is ready"},
                {"from": "analysis", "to": "discovery", "condition": "insufficient_data", "description": "Return to discovery when more information is needed"}
            ],
            "evidence_requirements": [
                "Detected technology version",
                "Relevant configuration files",
                "Error logs or symptoms"
            ],
            "required": true
        })
    }

    /// Generate detection rules YAML value from parameters.
    pub fn generate_detection_rules(&self, domain: &str) -> Value {
        serde_json::json!([
            {
                "id": format!("{}-file-1", domain),
                "name": format!("{} config file", domain.replace('-', " ")),
                "detection_type": "File",
                "pattern": format!("/etc/{}/", domain),
                "confidence": 0.85,
                "technology_domain": domain,
                "priority": 10,
                "flags": "",
                "extract": null
            },
            {
                "id": format!("{}-cmd-1", domain),
                "name": format!("{} command presence", domain.replace('-', " ")),
                "detection_type": "Command",
                "pattern": format!("^{} ", domain),
                "confidence": 0.8,
                "technology_domain": domain,
                "priority": 8,
                "flags": "",
                "extract": null
            }
        ])
    }

    /// Generate a technology definition YAML value from parameters.
    pub fn generate_tech_definition(&self, name: &str, domain: &str) -> Value {
        serde_json::json!({
            "domain": domain,
            "version": "0.1.0",
            "description": format!("Technology definition for {} domain", domain),
            "features": [],
            "related_domains": [],
            "documentation_url": ""
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn setup_test_sdk() -> (SkillSDK, PathBuf) {
        let temp_dir = std::env::temp_dir().join("wikilabs_skill_sdk_test");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(temp_dir.join("schemas")).unwrap();
        fs::create_dir_all(temp_dir.join("templates")).unwrap();

        // Write a simple manifest schema
        let schema = r#"{
            "type": "object",
            "required": ["id", "name", "version"],
            "properties": {
                "id": {"type": "string"},
                "name": {"type": "string"},
                "version": {"type": "string"}
            }
        }"#;
        fs::write(temp_dir.join("schemas").join("manifest.schema.json"), schema).unwrap();

        (SkillSDK::new(temp_dir.to_str().unwrap()).unwrap(), temp_dir)
    }

    fn cleanup(dir: &Path) {
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_new_sdk() {
        let (sdk, temp_dir) = setup_test_sdk();
        assert_eq!(sdk.get_schema("manifest").is_some(), true);
        assert_eq!(sdk.get_all_schema_names().len(), 1);
        cleanup(temp_dir.as_path());
    }

    #[test]
    fn test_create_skill_template() {
        let (sdk, temp_dir) = setup_test_sdk();
        let template = sdk.create_skill_template("test-skill").unwrap();

        assert_eq!(template.skill_name, "test-skill");
        assert!(!template.generated_files.is_empty());

        // Check manifest is present
        let manifest = template.generated_files.iter().find(|f| f.path == "manifest.yaml");
        assert!(manifest.is_some());

        let manifest = manifest.unwrap();
        assert_eq!(manifest.schema, "manifest");
        assert!(manifest.content.contains("\"id\": \"test-skill\""));

        cleanup(temp_dir.as_path());
    }

    #[test]
    fn test_generate_manifest() {
        let (sdk, temp_dir) = setup_test_sdk();
        let manifest = sdk.generate_manifest("openshift", "openshift", "0.1.0");

        assert_eq!(manifest["id"], "openshift");
        assert_eq!(manifest["name"], "openshift");
        assert_eq!(manifest["version"], "0.1.0");
        assert_eq!(manifest["technology_domain"], "openshift");
        assert_eq!(manifest["enabled"], true);
        assert_eq!(manifest["schema_version"], "1.0");

        cleanup(temp_dir.as_path());
    }

    #[test]
    fn test_generate_workflows() {
        let (sdk, temp_dir) = setup_test_sdk();
        let workflows = sdk.generate_workflows("test-skill");

        assert_eq!(workflows["id"], "test-skill-workflow");
        assert!(workflows["states"].is_array());
        assert!(workflows["transitions"].is_array());
        assert!(workflows["evidence_requirements"].is_array());
        assert_eq!(workflows["required"], true);

        let states: &Vec<Value> = workflows["states"].as_array().unwrap();
        assert_eq!(states.len(), 3);
        assert_eq!(states[0]["id"], "discovery");
        assert_eq!(states[0]["initial"], true);
        assert_eq!(states[2]["terminal"], true);

        cleanup(temp_dir.as_path());
    }

    #[test]
    fn test_generate_detection_rules() {
        let (sdk, temp_dir) = setup_test_sdk();
        let rules = sdk.generate_detection_rules("openshift");

        assert!(rules.is_array());
        assert_eq!(rules.as_array().unwrap().len(), 2);
        assert_eq!(rules[0]["detection_type"], "File");
        assert_eq!(rules[0]["technology_domain"], "openshift");
        assert_eq!(rules[1]["detection_type"], "Command");

        cleanup(temp_dir.as_path());
    }

    #[test]
    fn test_generate_tech_definition() {
        let (sdk, temp_dir) = setup_test_sdk();
        let tech = sdk.generate_tech_definition("Rust", "rust");

        assert_eq!(tech["domain"], "rust");
        assert_eq!(tech["version"], "0.1.0");
        assert!(tech["features"].is_array());

        cleanup(temp_dir.as_path());
    }

    #[test]
    fn test_validate_skill_missing_manifest() {
        let (sdk, temp_dir) = setup_test_sdk();
        // Create empty directory
        let skill_dir = temp_dir.join("no-manifest");
        fs::create_dir_all(&skill_dir).unwrap();

        let report = sdk.validate_skill(skill_dir.to_str().unwrap()).unwrap();
        assert!(!report.is_valid);
        assert!(report.errors.iter().any(|e| e.contains("manifest.yaml")));
        assert!(report.checked_schemas.is_empty());

        cleanup(temp_dir.as_path());
    }

    #[test]
    fn test_validate_skill_valid() {
        let (sdk, temp_dir) = setup_test_sdk();
        // Create valid skill
        let skill_dir = temp_dir.join("valid-skill");
        fs::create_dir_all(&skill_dir).unwrap();

        let manifest = r#"
id: valid-skill
name: Valid Skill
version: "0.1.0"
description: A valid skill
author: Test
technology_domain: test
enabled: true
schema_version: "1.0"
dependencies: []
"#;
        fs::write(skill_dir.join("manifest.yaml"), manifest).unwrap();

        let report = sdk.validate_skill(skill_dir.to_str().unwrap()).unwrap();
        assert!(report.is_valid);
        assert!(report.checked_schemas.contains(&"manifest".to_string()));

        cleanup(temp_dir.as_path());
    }

    #[test]
    fn test_validate_skill_empty_id() {
        let (sdk, temp_dir) = setup_test_sdk();
        // Create skill with empty id
        let skill_dir = temp_dir.join("bad-id-skill");
        fs::create_dir_all(&skill_dir).unwrap();

        let manifest = r#"
id: ""
name: Bad ID
version: "0.1.0"
description: Bad
author: Test
technology_domain: test
enabled: true
schema_version: "1.0"
dependencies: []
"#;
        fs::write(skill_dir.join("manifest.yaml"), manifest).unwrap();

        let report = sdk.validate_skill(skill_dir.to_str().unwrap()).unwrap();
        assert!(!report.is_valid);

        cleanup(temp_dir.as_path());
    }

    #[test]
    fn test_validate_skill_nonexistent() {
        let (sdk, temp_dir) = setup_test_sdk();
        let report = sdk.validate_skill("/nonexistent/path").unwrap();
        assert!(!report.is_valid);
        assert!(!report.errors.is_empty());

        cleanup(temp_dir.as_path());
    }

    #[test]
    fn test_validate_with_bad_yaml() {
        let (sdk, temp_dir) = setup_test_sdk();
        let skill_dir = temp_dir.join("bad-yaml-skill");
        fs::create_dir_all(&skill_dir).unwrap();

        // Write invalid YAML
        fs::write(skill_dir.join("manifest.yaml"), "key: [invalid: yaml: content").unwrap();

        let report = sdk.validate_skill(skill_dir.to_str().unwrap()).unwrap();
        assert!(!report.is_valid);
        assert!(report.errors.iter().any(|e| e.contains("YAML parse error")));

        cleanup(temp_dir.as_path());
    }

    #[test]
    fn test_validate_with_bad_intents() {
        let (sdk, temp_dir) = setup_test_sdk();
        let skill_dir = temp_dir.join("bad-intents-skill");
        fs::create_dir_all(&skill_dir).unwrap();

        // Write valid manifest
        let manifest = r#"
id: bad-intents-skill
name: Bad Intents
version: "0.1.0"
description: Bad intents
author: Test
technology_domain: test
enabled: true
schema_version: "1.0"
dependencies: []
"#;
        fs::write(skill_dir.join("manifest.yaml"), manifest).unwrap();

        // Write intents with empty id
        let intents = r#"- id: ""
  name: Bad Intent
  description: Bad
  patterns: []
"#;
        fs::write(skill_dir.join("intents.yaml"), intents).unwrap();

        let report = sdk.validate_skill(skill_dir.to_str().unwrap()).unwrap();
        assert!(!report.is_valid);

        cleanup(temp_dir.as_path());
    }

    #[test]
    fn test_schema_registry() {
        let (sdk, temp_dir) = setup_test_sdk();
        assert!(sdk.get_schema("manifest").is_some());
        assert!(sdk.get_schema("nonexistent").is_none());
        assert!(sdk.get_all_schema_names().contains(&"manifest".to_string()));

        cleanup(temp_dir.as_path());
    }

    #[test]
    fn test_skill_template_directory_structure() {
        let (sdk, temp_dir) = setup_test_sdk();
        let template = sdk.create_skill_template("my-skill").unwrap();

        assert_eq!(template.skill_name, "my-skill");
        assert!(template.directory_structure.contains(&"my-skill".to_string()));
        assert!(template.directory_structure.contains(&"templates/".to_string()));
        assert!(template.directory_structure.contains(&"schemas/".to_string()));

        // Count files
        let paths: Vec<&str> = template.generated_files.iter().map(|f| f.path.as_str()).collect();
        assert!(paths.contains(&"manifest.yaml"));
        assert!(paths.contains(&"technology.yaml"));
        assert!(paths.contains(&"intents.yaml"));
        assert!(paths.contains(&"workflows.yaml"));
        assert!(paths.contains(&"detection_rules.yaml"));
        assert!(paths.contains(&"commands.yaml"));
        assert!(paths.contains(&"best_practices.yaml"));
        assert!(paths.contains(&"known_issues.yaml"));

        cleanup(temp_dir.as_path());
    }

    #[test]
    fn test_validate_no_templates_dir() {
        let temp_dir = std::env::temp_dir().join("wikilabs_sdk_no_templates");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(temp_dir.join("schemas")).unwrap();
        fs::create_dir_all(temp_dir.join("templates")).unwrap();

        let schema = r#"{"type": "object", "required": ["id"], "properties": {"id": {"type": "string"}}}"#;
        fs::write(temp_dir.join("schemas").join("manifest.schema.json"), schema).unwrap();

        let sdk = SkillSDK::new(temp_dir.to_str().unwrap()).unwrap();
        assert!(sdk.get_schema("manifest").is_some());

        let _ = fs::remove_dir_all(&temp_dir);
    }
}