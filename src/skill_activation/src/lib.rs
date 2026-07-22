//! Dynamic Skill Activation Engine — activates detected skills in the workspace.
//!
//! The Skill Activation Engine is responsible for:
//! - Receiving discovery reports from the Skill Discovery Engine
//! - Matching detected technologies to skill definitions
//! - Activating skills dynamically at runtime
//! - Managing skill lifecycle (activate, deactivate, update)
//! - Providing skill availability to the AI Copilot
//!
//! ## Activation Flow
//!
//! 1. **Receive** discovery report with detected technologies
//! 2. **Match** against skill definitions in the Skill Runtime
//! 3. **Activate** matching skills dynamically
//! 4. **Notify** the AI Copilot of new capabilities
//! 5. **Monitor** skill health and deactivate on failure

use std::collections::HashMap;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

/// A detected skill ready for activation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivationCandidate {
    /// Skill ID.
    pub skill_id: String,
    /// Skill name.
    pub skill_name: String,
    /// Technology domain.
    pub technology: String,
    /// Confidence score from discovery.
    pub confidence: f64,
    /// Whether to auto-activate.
    pub auto_activate: bool,
    /// Activation reason.
    pub reason: String,
}

/// An activated skill with lifecycle state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivatedSkill {
    /// Skill ID.
    pub skill_id: String,
    /// Skill name.
    pub skill_name: String,
    /// Activation timestamp.
    pub activated_at: String,
    /// Activation confidence.
    pub confidence: f64,
    /// Current state.
    pub state: ActivationState,
    /// Last health check timestamp.
    pub last_health_check: Option<String>,
    /// Number of health check failures.
    pub failure_count: u32,
}

/// Current state of an activated skill.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ActivationState {
    /// Skill is active and healthy.
    Active,
    /// Skill has been deactivated.
    Inactive,
    /// Skill failed a health check.
    Degraded,
    /// Skill has been removed.
    Removed,
}

/// Skill activation configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivationConfig {
    /// Whether to auto-activate skills on detection.
    pub auto_activate: bool,
    /// Minimum confidence for auto-activation.
    pub auto_activate_threshold: f64,
    /// Whether to perform health checks.
    pub health_checks: bool,
    /// Health check interval in seconds.
    pub health_check_interval: u64,
    /// Maximum failure count before deactivation.
    pub max_failure_count: u32,
}

impl Default for ActivationConfig {
    fn default() -> Self {
        Self {
            auto_activate: true,
            auto_activate_threshold: 0.5,
            health_checks: true,
            health_check_interval: 300, // 5 minutes
            max_failure_count: 3,
        }
    }
}

/// Dynamic Skill Activation Engine.
pub struct SkillActivationEngine {
    /// Configuration.
    config: ActivationConfig,
    /// Currently activated skills.
    activated_skills: HashMap<String, ActivatedSkill>,
    /// Skill ID to definition mapping.
    skill_definitions: HashMap<String, SkillDefinition>,
}

/// A skill definition from the Skill Runtime.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDefinition {
    /// Skill ID.
    pub id: String,
    /// Skill name.
    pub name: String,
    /// Technology domain.
    pub technology: String,
    /// Category.
    pub category: String,
    /// Whether the skill is currently enabled.
    pub enabled: bool,
    /// Required dependencies.
    pub dependencies: Vec<String>,
}

impl SkillActivationEngine {
    /// Create a new activation engine with default config.
    pub fn new() -> Self {
        Self {
            config: ActivationConfig::default(),
            activated_skills: HashMap::new(),
            skill_definitions: HashMap::new(),
        }
    }

    /// Create with a custom configuration.
    pub fn with_config(config: ActivationConfig) -> Self {
        Self {
            config,
            activated_skills: HashMap::new(),
            skill_definitions: HashMap::new(),
        }
    }

    /// Register a skill definition.
    pub fn register_skill(&mut self, definition: SkillDefinition) {
        debug!("Registered skill definition: {}", definition.id);
        self.skill_definitions.insert(definition.id.clone(), definition);
    }

    /// Process a discovery report and activate matching skills.
    pub fn process_discovery(
        &mut self,
        candidates: Vec<ActivationCandidate>,
    ) -> Result<Vec<ActivatedSkill>> {
        let mut activated = Vec::new();

        for candidate in candidates {
            if !self.should_activate(&candidate) {
                debug!("Skipping activation for {}: confidence too low or manual mode", candidate.skill_id);
                continue;
            }

            // Check if already activated
            if self.activated_skills.contains_key(&candidate.skill_id) {
                debug!("Skill already activated: {}", candidate.skill_id);
                continue;
            }

            // Check dependencies
            if let Some(def) = self.skill_definitions.get(&candidate.skill_id) {
                if !self.check_dependencies(&def.dependencies) {
                    warn!(
                        "Skipping activation for {}: dependencies not met",
                        candidate.skill_id
                    );
                    continue;
                }
            }

            // Activate the skill
            let activated_skill = ActivatedSkill {
                skill_id: candidate.skill_id.clone(),
                skill_name: candidate.skill_name.clone(),
                activated_at: chrono::Utc::now().to_rfc3339(),
                confidence: candidate.confidence,
                state: ActivationState::Active,
                last_health_check: None,
                failure_count: 0,
            };

            self.activated_skills.insert(candidate.skill_id.clone(), activated_skill.clone());
            activated.push(activated_skill);

            info!(
                "Activated skill: {} (confidence: {:.2}, auto: {})",
                candidate.skill_id, candidate.confidence, candidate.auto_activate
            );
        }

        Ok(activated)
    }

    /// Check if a candidate should be activated.
    fn should_activate(&self, candidate: &ActivationCandidate) -> bool {
        if self.config.auto_activate {
            candidate.auto_activate || candidate.confidence >= self.config.auto_activate_threshold
        } else {
            // Manual mode: only activate if explicitly requested
            candidate.auto_activate
        }
    }

    /// Check if all dependencies are activated.
    fn check_dependencies(&self, dependencies: &[String]) -> bool {
        for dep in dependencies {
            if !self.activated_skills.contains_key(dep) {
                debug!("Dependency not activated: {} for {}", dep, dependencies.first().map(|s| s.as_str()).unwrap_or("?"));
                return false;
            }
        }
        true
    }

    /// Deactivate a skill by ID.
    pub fn deactivate(&mut self, skill_id: &str) -> Result<()> {
        if let Some(skill) = self.activated_skills.get_mut(skill_id) {
            skill.state = ActivationState::Inactive;
            info!("Deactivated skill: {}", skill_id);
            Ok(())
        } else {
            Err(anyhow!("Skill not activated: {}", skill_id))
        }
    }

    /// Remove a skill entirely.
    pub fn remove(&mut self, skill_id: &str) -> Result<()> {
        if let Some(skill) = self.activated_skills.get_mut(skill_id) {
            skill.state = ActivationState::Removed;
            info!("Removed skill: {}", skill_id);
            Ok(())
        } else {
            Err(anyhow!("Skill not activated: {}", skill_id))
        }
    }

    /// Perform a health check on a skill.
    pub fn health_check(&mut self, skill_id: &str) -> Result<ActivationState> {
        if let Some(skill) = self.activated_skills.get_mut(skill_id) {
            skill.last_health_check = Some(chrono::Utc::now().to_rfc3339());

            // In a real implementation, this would:
            // 1. Check if the skill's files are still accessible
            // 2. Verify dependency integrity
            // 3. Check for configuration changes
            // For now, always report healthy
            let is_healthy = true;

            if !is_healthy {
                skill.failure_count += 1;
                if skill.failure_count >= self.config.max_failure_count {
                    skill.state = ActivationState::Removed;
                    warn!(
                        "Skill {} removed after {} failures",
                        skill_id, skill.failure_count
                    );
                } else {
                    skill.state = ActivationState::Degraded;
                    warn!(
                        "Skill {} degraded (failure {}/{})",
                        skill_id, skill.failure_count, self.config.max_failure_count
                    );
                }
            } else {
                if skill.failure_count > 0 {
                    skill.failure_count = 0;
                    skill.state = ActivationState::Active;
                    info!("Skill {} recovered", skill_id);
                }
            }

            Ok(skill.state.clone())
        } else {
            Err(anyhow!("Skill not activated: {}", skill_id))
        }
    }

    /// Get an activated skill by ID.
    pub fn get_skill(&self, skill_id: &str) -> Option<&ActivatedSkill> {
        self.activated_skills.get(skill_id)
    }

    /// Get all activated skills.
    pub fn get_all_skills(&self) -> Vec<&ActivatedSkill> {
        self.activated_skills.values().collect()
    }

    /// Get all active skills (exclude inactive, degraded, removed).
    pub fn get_active_skills(&self) -> Vec<&ActivatedSkill> {
        self.activated_skills
            .values()
            .filter(|s| s.state == ActivationState::Active)
            .collect()
    }

    /// Get all skill IDs.
    pub fn skill_ids(&self) -> Vec<String> {
        self.activated_skills.keys().cloned().collect()
    }

    /// Get skill count.
    pub fn skill_count(&self) -> usize {
        self.activated_skills.len()
    }

    /// Get registered skill definitions.
    pub fn skill_definitions(&self) -> &HashMap<String, SkillDefinition> {
        &self.skill_definitions
    }

    /// Get configuration.
    pub fn config(&self) -> &ActivationConfig {
        &self.config
    }

    /// Set configuration.
    pub fn set_config(&mut self, config: ActivationConfig) {
        self.config = config;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_engine() {
        let engine = SkillActivationEngine::new();
        assert_eq!(engine.skill_count(), 0);
    }

    #[test]
    fn test_register_skill() {
        let mut engine = SkillActivationEngine::new();
        engine.register_skill(SkillDefinition {
            id: "linux".to_string(),
            name: "Linux Engineering".to_string(),
            technology: "Linux".to_string(),
            category: "Engineering".to_string(),
            enabled: true,
            dependencies: vec![],
        });
        assert_eq!(engine.skill_definitions().len(), 1);
    }

    #[test]
    fn test_activate_skill() {
        let mut engine = SkillActivationEngine::new();
        engine.register_skill(SkillDefinition {
            id: "linux".to_string(),
            name: "Linux Engineering".to_string(),
            technology: "Linux".to_string(),
            category: "Engineering".to_string(),
            enabled: true,
            dependencies: vec![],
        });

        let candidates = vec![ActivationCandidate {
            skill_id: "linux".to_string(),
            skill_name: "Linux Engineering".to_string(),
            technology: "Linux".to_string(),
            confidence: 0.8,
            auto_activate: true,
            reason: "Linux files detected".to_string(),
        }];

        let activated = engine.process_discovery(candidates).unwrap();
        assert_eq!(activated.len(), 1);
        assert_eq!(engine.skill_count(), 1);
    }

    #[test]
    fn test_deactivate_skill() {
        let mut engine = SkillActivationEngine::new();
        engine.register_skill(SkillDefinition {
            id: "linux".to_string(),
            name: "Linux Engineering".to_string(),
            technology: "Linux".to_string(),
            category: "Engineering".to_string(),
            enabled: true,
            dependencies: vec![],
        });

        let candidates = vec![ActivationCandidate {
            skill_id: "linux".to_string(),
            skill_name: "Linux Engineering".to_string(),
            technology: "Linux".to_string(),
            confidence: 0.8,
            auto_activate: true,
            reason: "Linux files detected".to_string(),
        }];

        engine.process_discovery(candidates).unwrap();

        // Deactivate
        engine.deactivate("linux").unwrap();
        assert_eq!(engine.get_active_skills().len(), 0);
    }

    #[test]
    fn test_health_check() {
        let mut engine = SkillActivationEngine::new();
        engine.register_skill(SkillDefinition {
            id: "linux".to_string(),
            name: "Linux Engineering".to_string(),
            technology: "Linux".to_string(),
            category: "Engineering".to_string(),
            enabled: true,
            dependencies: vec![],
        });

        let candidates = vec![ActivationCandidate {
            skill_id: "linux".to_string(),
            skill_name: "Linux Engineering".to_string(),
            technology: "Linux".to_string(),
            confidence: 0.8,
            auto_activate: true,
            reason: "Linux files detected".to_string(),
        }];

        engine.process_discovery(candidates).unwrap();

        // Health check
        let state = engine.health_check("linux").unwrap();
        assert_eq!(state, ActivationState::Active);
    }
}