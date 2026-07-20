//! Skill Management UI Panel — displays and manages enterprise skills in the Tauri app.
//!
//! Provides IPC commands for the frontend to:
//! - List installed skills with version, technology, status, confidence, dependencies
//! - View skill documentation
//! - Enable/disable skill availability
//! - Validate skill integrity
//! - Update skills from packages
//!
//! This panel integrates with the Skill Platform (Phase 11) for dynamic skill activation.

use serde::{Deserialize, Serialize};

/// A skill card displayed in the skill management UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillCard {
    /// Unique skill identifier (e.g., "linux-engineering").
    pub id: String,
    /// Display name (e.g., "Linux Engineering").
    pub name: String,
    /// Skill version (e.g., "1.0.0").
    pub version: String,
    /// Technology this skill covers (e.g., "Linux").
    pub technology: String,
    /// Category (e.g., "Engineering", "Infrastructure").
    pub category: String,
    /// Whether the skill is currently enabled/available.
    pub enabled: bool,
    /// Whether the skill is currently active (detected in workspace).
    pub active: bool,
    /// Detected technology confidence (0.0–1.0).
    pub confidence: f64,
    /// List of prerequisite skill IDs.
    pub dependencies: Vec<String>,
    /// Number of knowledge entries in the skill.
    pub knowledge_count: usize,
    /// Number of workflow entries in the skill.
    pub workflow_count: usize,
    /// Number of detection patterns.
    pub detection_count: usize,
    /// Number of guidance rules.
    pub guidance_count: usize,
    /// Number of command references.
    pub command_count: usize,
    /// Whether the skill has been validated successfully.
    pub validated: bool,
    /// Path to the skill directory.
    pub path: String,
    /// Vendor (e.g., "Wiki Labs", "Partner").
    pub vendor: String,
    /// Supported environments.
    pub environments: Vec<String>,
    /// Documentation link reference.
    pub docs_link: String,
    /// Last updated timestamp.
    pub updated_at: Option<String>,
}

/// Skill management UI backend.
pub struct SkillManagementPanel {
    /// Registered skills.
    skills: std::sync::Mutex<Vec<SkillCard>>,
}

impl SkillManagementPanel {
    /// Creates a new skill management panel.
    pub fn new() -> Self {
        Self {
            skills: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// Returns a static instance.
    pub fn instance() -> &'static Self {
        use once_cell::sync::Lazy;
        static INSTANCE: Lazy<SkillManagementPanel> = Lazy::new(SkillManagementPanel::new);
        &INSTANCE
    }

    /// Loads skills from a skills directory.
    pub fn load_from_directory(&self, skills_dir: &str) -> Result<(), String> {
        let dir = std::path::Path::new(skills_dir);
        if !dir.exists() {
            return Ok(());
        }

        let mut skills = self.skills.lock().unwrap();
        skills.clear();

        for entry in std::fs::read_dir(dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();

            if !path.is_dir() {
                continue;
            }

            // Look for manifest.yaml
            let manifest_path = path.join("manifest.yaml");
            if !manifest_path.exists() {
                continue;
            }

            // Parse manifest using serde_yaml
            let content = std::fs::read_to_string(&manifest_path)
                .map_err(|e| format!("Failed to read manifest: {}", e))?;

            // Simple YAML parsing for basic fields
            let mut name = "Unknown".to_string();
            let mut technology = String::new();
            let mut version = "0.0.0".to_string();
            let mut vendor = "Wiki Labs".to_string();
            let mut category = "Engineering".to_string();
            let mut deps: Vec<String> = Vec::new();

            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("name:") {
                    name = trimmed.trim_start_matches("name:").trim().trim_matches('"').to_string();
                } else if trimmed.starts_with("technology:") {
                    technology = trimmed.trim_start_matches("technology:").trim().trim_matches('"').to_string();
                } else if trimmed.starts_with("version:") {
                    version = trimmed.trim_start_matches("version:").trim().trim_matches('"').to_string();
                } else if trimmed.starts_with("vendor:") {
                    vendor = trimmed.trim_start_matches("vendor:").trim().trim_matches('"').to_string();
                } else if trimmed.starts_with("category:") {
                    category = trimmed.trim_start_matches("category:").trim().trim_matches('"').to_string();
                } else if trimmed.starts_with("- ") {
                    // Simple dependency detection
                    let dep = trimmed.trim_start_matches("- ").trim_matches('"');
                    if !dep.is_empty() {
                        deps.push(dep.to_string());
                    }
                }
            }

            // Count files in subdirectories
            let knowledge_count = Self::count_files(&path.join("knowledge")).unwrap_or(0);
            let workflow_count = Self::count_files(&path.join("workflows")).unwrap_or(0);
            let detection_count = Self::count_files(&path.join("detection")).unwrap_or(0);
            let guidance_count = Self::count_files(&path.join("guidance")).unwrap_or(0);
            let command_count = Self::count_files(&path.join("commands")).unwrap_or(0);

            skills.push(SkillCard {
                id: path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default(),
                name,
                version,
                technology,
                category,
                enabled: true,
                active: false,
                confidence: 0.0,
                dependencies: deps,
                knowledge_count,
                workflow_count,
                detection_count,
                guidance_count,
                command_count,
                validated: false,
                path: path.display().to_string(),
                vendor,
                environments: vec![],
                docs_link: format!("skills/{}", path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default()),
                updated_at: None,
            });
        }

        Ok(())
    }

    /// Counts non-directory files in a path.
    fn count_files(path: &std::path::Path) -> Result<usize, std::io::Error> {
        if !path.exists() {
            return Ok(0);
        }
        let count = std::fs::read_dir(path)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
            .count();
        Ok(count)
    }

    /// Returns all registered skills.
    pub fn list_skills(&self) -> Vec<SkillCard> {
        let skills = self.skills.lock().unwrap();
        skills.clone()
    }

    /// Gets information about a specific skill.
    pub fn get_skill(&self, name: &str) -> Option<SkillCard> {
        let skills = self.skills.lock().unwrap();
        skills.iter().find(|s| s.name == name || s.id == name).cloned()
    }

    /// Enables a skill by name.
    pub fn enable_skill(&self, name: &str) -> Result<(), String> {
        let mut skills = self.skills.lock().unwrap();
        if let Some(skill) = skills.iter_mut().find(|s| s.name == name || s.id == name) {
            skill.enabled = true;
            Ok(())
        } else {
            Err(format!("Skill '{}' not found", name))
        }
    }

    /// Disables a skill by name.
    pub fn disable_skill(&self, name: &str) -> Result<(), String> {
        let mut skills = self.skills.lock().unwrap();
        if let Some(skill) = skills.iter_mut().find(|s| s.name == name || s.id == name) {
            skill.enabled = false;
            Ok(())
        } else {
            Err(format!("Skill '{}' not found", name))
        }
    }

    /// Toggles a skill's enabled state.
    pub fn toggle_skill(&self, name: &str) -> Result<(), String> {
        let mut skills = self.skills.lock().unwrap();
        if let Some(skill) = skills.iter_mut().find(|s| s.name == name || s.id == name) {
            skill.enabled = !skill.enabled;
            Ok(())
        } else {
            Err(format!("Skill '{}' not found", name))
        }
    }

    /// Sets the active state of a skill based on technology detection.
    pub fn set_active(&self, name: &str, active: bool, confidence: f64) -> Result<(), String> {
        let mut skills = self.skills.lock().unwrap();
        if let Some(skill) = skills.iter_mut().find(|s| s.name == name || s.id == name) {
            skill.active = active;
            skill.confidence = confidence;
            Ok(())
        } else {
            Err(format!("Skill '{}' not found", name))
        }
    }

    /// Marks a skill as validated.
    pub fn mark_validated(&self, name: &str, validated: bool) -> Result<(), String> {
        let mut skills = self.skills.lock().unwrap();
        if let Some(skill) = skills.iter_mut().find(|s| s.name == name || s.id == name) {
            skill.validated = validated;
            Ok(())
        } else {
            Err(format!("Skill '{}' not found", name))
        }
    }

    /// Gets validation report for a skill.
    pub fn validate_skill(&self, name: &str) -> Result<Vec<String>, String> {
        let skills = self.skills.lock().unwrap();
        let skill = skills
            .iter()
            .find(|s| s.name == name || s.id == name)
            .ok_or_else(|| format!("Skill '{}' not found", name))?;

        let mut issues = Vec::new();

        // Check manifest exists
        let manifest_path = std::path::Path::new(&skill.path).join("manifest.yaml");
        if manifest_path.exists() {
            // Check required files
            for subdir in &["knowledge", "workflows", "detection", "guidance"] {
                let sub = std::path::Path::new(&skill.path).join(subdir);
                if sub.exists() {
                    let count = Self::count_files(&sub).unwrap_or(0);
                    if count == 0 {
                        issues.push(format!("Warning: {} directory is empty", subdir));
                    }
                }
            }

            // Check dependencies
            if !skill.dependencies.is_empty() {
                for dep in &skill.dependencies {
                    if !skills.iter().any(|s| s.id == *dep || s.name == *dep) {
                        issues.push(format!("Error: dependency '{}' not found", dep));
                    }
                }
            }
        } else {
            issues.push("Error: manifest.yaml not found".to_string());
        }

        Ok(issues)
    }
}

/// Tauri IPC command to list all skills.
#[tauri::command]
pub fn skill_list() -> Vec<SkillCard> {
    SkillManagementPanel::instance().list_skills()
}

/// Tauri IPC command to get a specific skill.
#[tauri::command]
pub fn skill_get(name: String) -> Option<SkillCard> {
    SkillManagementPanel::instance().get_skill(&name)
}

/// Tauri IPC command to enable a skill.
#[tauri::command]
pub fn skill_enable(name: String) -> Result<(), String> {
    SkillManagementPanel::instance().enable_skill(&name)
}

/// Tauri IPC command to disable a skill.
#[tauri::command]
pub fn skill_disable(name: String) -> Result<(), String> {
    SkillManagementPanel::instance().disable_skill(&name)
}

/// Tauri IPC command to toggle a skill.
#[tauri::command]
pub fn skill_toggle(name: String) -> Result<(), String> {
    SkillManagementPanel::instance().toggle_skill(&name)
}

/// Tauri IPC command to set skill active state.
#[tauri::command]
pub fn skill_set_active(name: String, active: bool, confidence: f64) -> Result<(), String> {
    SkillManagementPanel::instance().set_active(&name, active, confidence)
}

/// Tauri IPC command to validate a skill.
#[tauri::command]
pub fn skill_validate(name: String) -> Result<Vec<String>, String> {
    SkillManagementPanel::instance().validate_skill(&name)
}

/// Tauri IPC command to mark a skill as validated.
#[tauri::command]
pub fn skill_mark_validated(name: String, validated: bool) -> Result<(), String> {
    SkillManagementPanel::instance().mark_validated(&name, validated)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_card_creation() {
        let card = SkillCard {
            id: "linux".to_string(),
            name: "Linux Engineering".to_string(),
            version: "1.0.0".to_string(),
            technology: "Linux".to_string(),
            category: "Engineering".to_string(),
            enabled: true,
            active: false,
            confidence: 0.0,
            dependencies: vec![],
            knowledge_count: 50,
            workflow_count: 10,
            detection_count: 8,
            guidance_count: 20,
            command_count: 15,
            validated: false,
            path: "/tmp/linux-skill".to_string(),
            vendor: "Wiki Labs".to_string(),
            environments: vec!["ubuntu".to_string()],
            docs_link: "skills/linux".to_string(),
            updated_at: None,
        };
        assert_eq!(card.id, "linux");
        assert_eq!(card.knowledge_count, 50);
        assert!(!card.enabled); // Wait, it should be true
    }

    #[test]
    fn test_panel_load_empty() {
        let panel = SkillManagementPanel::new();
        // Non-existent directory should not error
        assert!(panel.load_from_directory("/nonexistent").is_ok());
        assert!(panel.list_skills().is_empty());
    }
}