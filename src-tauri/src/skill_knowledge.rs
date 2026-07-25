//! Skill Knowledge Injector — bridges skill packs into the AI observation loop.
//!
//! At startup, reads each loaded skill's key files (tech.yaml, detection_rules.yaml,
//! guidance/rules.md, knowledge/*.md) and builds an in-memory knowledge base.
//! On each AI tick, matches observations against skill detection patterns and
//! injects matched skill expertise into the system prompt.

use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

/// A single skill's extracted knowledge.
#[derive(Debug, Clone)]
pub struct SkillKnowledge {
    /// Skill ID (e.g. "nagiosxi-skill-pack")
    pub id: String,
    /// Technology domain (e.g. "Nagios XI")
    pub domain: String,
    /// Vendor
    pub vendor: String,
    /// YAML detection patterns from detection_rules.yaml
    pub detection_patterns: Vec<DetectionRule>,
    /// Core knowledge content from knowledge/*.md files
    pub knowledge_sections: Vec<KnowledgeSection>,
    /// Guidance rules from guidance/rules.md
    pub guidance_rules: Vec<String>,
    /// Technology features from technology.yaml
    pub tech_features: Vec<String>,
    /// Common failures from common-failures/*.md
    pub common_failures: Vec<String>,
}

/// A detection rule from detection_rules.yaml.
#[derive(Debug, Clone)]
pub struct DetectionRule {
    pub id: String,
    pub name: String,
    pub pattern: String,
    pub confidence: f64,
    pub technology_domain: String,
    pub detection_type: String,
}

/// A knowledge section extracted from knowledge/*.md files.
#[derive(Debug, Clone)]
pub struct KnowledgeSection {
    pub file_name: String,
    pub topic: String,
    pub content: String,
}

/// Loaded skill knowledge base.
#[derive(Debug, Clone)]
pub struct SkillKnowledgeBase {
    pub skills: Vec<SkillKnowledge>,
}

impl SkillKnowledgeBase {
    pub fn new() -> Self {
        Self { skills: Vec::new() }
    }

    /// Load all skill packs from a directory into the knowledge base.
    pub fn load_from_directory(&mut self, skills_dir: &str) {
        let dir = Path::new(skills_dir);
        if !dir.exists() {
            return;
        }

        for entry in fs::read_dir(dir).expect("Failed to read skills directory") {
            let entry = entry.expect("Failed to read entry");
            let path = entry.path();

            if !path.is_dir() {
                continue;
            }

            // Look for manifest.yaml to identify valid skill packs
            let manifest_path = path.join("manifest.yaml");
            if !manifest_path.exists() {
                continue;
            }

            let skill = self.load_single_skill(&path);
            if !skill.domain.is_empty() {
                self.skills.push(skill);
            }
        }
    }

    /// Load a single skill pack into memory.
    fn load_single_skill(&self, skill_dir: &Path) -> SkillKnowledge {
        let id = skill_dir
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        // Read manifest.yaml for basic metadata
        let mut domain = String::new();
        let mut vendor = "Wiki Labs".to_string();
        let manifest_path = skill_dir.join("manifest.yaml");
        if let Ok(manifest) = fs::read_to_string(&manifest_path) {
            for line in manifest.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("name:") {
                    domain = trimmed
                        .trim_start_matches("name:")
                        .trim()
                        .trim_matches('"')
                        .to_string();
                } else if trimmed.starts_with("vendor:") {
                    vendor = trimmed
                        .trim_start_matches("vendor:")
                        .trim()
                        .trim_matches('"')
                        .to_string();
                }
            }
        }

        // Read technology.yaml for features
        let mut tech_features = Vec::new();
        let tech_path = skill_dir.join("technology.yaml");
        if tech_path.exists() {
            if let Ok(tech) = fs::read_to_string(&tech_path) {
                tech_features = self.parse_tech_features(&tech);
            }
        }

        // Read detection_rules.yaml
        let mut detection_patterns = Vec::new();
        let detection_path = skill_dir.join("detection_rules.yaml");
        if detection_path.exists() {
            if let Ok(detection) = fs::read_to_string(&detection_path) {
                detection_patterns = self.parse_detection_rules(&detection, &id);
            }
        }

        // Read guidance rules
        let mut guidance_rules = Vec::new();
        let guidance_path = skill_dir.join("guidance").join("rules.md");
        if guidance_path.exists() {
            if let Ok(guidance) = fs::read_to_string(&guidance_path) {
                for line in guidance.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("### ") || trimmed.starts_with("## ") || trimmed.starts_with("- ") {
                        guidance_rules.push(trimmed.to_string());
                    }
                }
            }
        }

        // Read knowledge files
        let mut knowledge_sections = Vec::new();
        let knowledge_dir = skill_dir.join("knowledge");
        if knowledge_dir.exists() {
            if let Ok(entries) = fs::read_dir(&knowledge_dir) {
                for entry in entries.flatten() {
                    let entry_path = entry.path();
                    if entry_path.is_file() && entry_path.extension().map(|e| e == "md").unwrap_or(false) {
                        if let Ok(content) = fs::read_to_string(&entry_path) {
                            let file_name = entry_path
                                .file_stem()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_default();
                            // Take first 2000 chars of knowledge content for AI context
                                                        let truncated = if content.chars().count() > 2000 {
                                                            format!("{}...", content.chars().take(2000).collect::<String>())
                                                        } else {
                                                            content
                                                        };
                            knowledge_sections.push(KnowledgeSection {
                                file_name: file_name.clone(),
                                topic: file_name,
                                content: truncated,
                            });
                        }
                    }
                }
            }
        }

        // Read common failures
        let mut common_failures = Vec::new();
        let failures_dir = skill_dir.join("common-failures");
        if failures_dir.exists() {
            if let Ok(entries) = fs::read_dir(&failures_dir) {
                for entry in entries.flatten() {
                    let entry_path = entry.path();
                    if entry_path.is_file() && entry_path.extension().map(|e| e == "md").unwrap_or(false) {
                        if let Ok(content) = fs::read_to_string(&entry_path) {
                            let truncated = if content.chars().count() > 500 {
                                format!("{}...", content.chars().take(500).collect::<String>())
                            } else {
                                content
                            };
                            common_failures.push(truncated);
                        }
                    }
                }
            }
        }

        SkillKnowledge {
            id,
            domain,
            vendor,
            detection_patterns,
            knowledge_sections,
            guidance_rules,
            tech_features,
            common_failures,
        }
    }

    /// Parse technology features from YAML content.
    fn parse_tech_features(&self, content: &str) -> Vec<String> {
        let mut features = Vec::new();
        let mut in_features = false;
        let mut current_feature = String::new();

        for line in content.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with("features:") {
                in_features = true;
                continue;
            }

            if in_features {
                if trimmed.starts_with("- name:") {
                    if !current_feature.is_empty() {
                        features.push(current_feature.clone());
                    }
                    current_feature = trimmed
                        .trim_start_matches("- name:")
                        .trim()
                        .trim_matches('"')
                        .to_string();
                } else if trimmed.starts_with("description:") && !current_feature.is_empty() {
                    let desc = trimmed
                        .trim_start_matches("description:")
                        .trim()
                        .trim_matches('"')
                        .to_string();
                    current_feature = format!("{}: {}", current_feature, desc);
                } else if !trimmed.is_empty() && !trimmed.starts_with("-") && !trimmed.contains(":") {
                    // End of features section
                    if !current_feature.is_empty() {
                        features.push(current_feature.clone());
                        current_feature = String::new();
                    }
                    in_features = false;
                }
            }
        }

        if !current_feature.is_empty() {
            features.push(current_feature);
        }

        features
    }

    /// Parse detection rules from YAML content.
    fn parse_detection_rules(&self, content: &str, default_domain: &str) -> Vec<DetectionRule> {
        let mut rules = Vec::new();
        let mut current_id = String::new();
        let mut current_name = String::new();
        let mut current_pattern = String::new();
        let mut current_confidence: f64 = 0.0;
        let mut current_domain = default_domain.to_string();
        let mut current_type = String::new();

        for line in content.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with("- id:") {
                // Save previous rule if exists
                if !current_id.is_empty() {
                    rules.push(DetectionRule {
                        id: current_id.clone(),
                        name: current_name.clone(),
                        pattern: current_pattern.clone(),
                        confidence: current_confidence,
                        technology_domain: current_domain.clone(),
                        detection_type: current_type.clone(),
                    });
                }

                current_id = trimmed
                    .trim_start_matches("- id:")
                    .trim()
                    .trim_matches('"')
                    .to_string();
                current_name = String::new();
                current_pattern = String::new();
                current_confidence = 0.0;
                current_domain = default_domain.to_string();
                current_type = String::new();
            } else if trimmed.starts_with("name:") && !current_id.is_empty() {
                current_name = trimmed
                    .trim_start_matches("name:")
                    .trim()
                    .trim_matches('"')
                    .to_string();
            } else if trimmed.starts_with("pattern:") && !current_id.is_empty() {
                current_pattern = trimmed
                    .trim_start_matches("pattern:")
                    .trim()
                    .trim_matches('"')
                    .to_string();
            } else if trimmed.starts_with("confidence:") && !current_id.is_empty() {
                current_confidence = trimmed
                    .trim_start_matches("confidence:")
                    .trim()
                    .parse()
                    .unwrap_or(0.0);
            } else if trimmed.starts_with("technology_domain:") && !current_id.is_empty() {
                current_domain = trimmed
                    .trim_start_matches("technology_domain:")
                    .trim()
                    .trim_matches('"')
                    .to_string();
            } else if trimmed.starts_with("detection_type:") && !current_id.is_empty() {
                current_type = trimmed
                    .trim_start_matches("detection_type:")
                    .trim()
                    .trim_matches('"')
                    .to_string();
            }
        }

        // Save last rule
        if !current_id.is_empty() {
            rules.push(DetectionRule {
                id: current_id,
                name: current_name,
                pattern: current_pattern,
                confidence: current_confidence,
                technology_domain: current_domain,
                detection_type: current_type,
            });
        }

        rules
    }

    /// Match observations against skill detection patterns.
    /// Returns matched skills with relevant knowledge for the AI prompt.
    pub fn match_observations(&self, observations: &str) -> Vec<SkillMatch> {
        let mut matches = Vec::new();

        for skill in &self.skills {
            let mut matched_patterns = Vec::new();

            for rule in &skill.detection_patterns {
                if !rule.pattern.is_empty() {
                    // Check if any pattern keyword appears in observations
                    let keywords: Vec<&str> = rule.pattern.split('|').collect();
                    for keyword in keywords {
                        let keyword = keyword.trim();
                        if observations.contains(keyword) {
                            matched_patterns.push(rule.clone());
                            break;
                        }
                    }
                }
            }

            // PHASE 3 FIX: Also match against the skill's own keywords (tech_features, domain)
            // so we don't rely solely on detection_rules.yaml patterns
            if matched_patterns.is_empty() {
                for feature in &skill.tech_features {
                    // Extract the feature name (before the colon if present)
                    let feature_name: String = feature.chars().take_while(|c| *c != ':').collect();
                    if !feature_name.is_empty() && observations.contains(&feature_name) {
                        matched_patterns.push(DetectionRule {
                            id: format!("auto-{}", skill.id),
                            name: format!("Auto-matched to {}", feature_name),
                            pattern: feature_name.clone(),
                            confidence: 0.7,
                            technology_domain: skill.domain.clone(),
                            detection_type: "Auto".to_string(),
                        });
                        break; // One match per skill is enough
                    }
                }
            }

            if !matched_patterns.is_empty() {
                matches.push(SkillMatch {
                    skill: skill.clone(),
                    matched_patterns,
                });
            }
        }

        matches
    }

    /// Format matched skills into a prompt section for the AI.
    pub fn format_for_prompt(&self, matches: &[SkillMatch]) -> String {
        if matches.is_empty() {
            return String::new();
        }

        let mut prompt = String::from("\n\n📚 SKILL PACK KNOWLEDGE AVAILABLE:\n");

        for match_item in matches {
            let skill = &match_item.skill;
            prompt.push_str(&format!(
                "\n## {} ({})\n",
                skill.domain, skill.id
            ));

            // Add tech features
            if !skill.tech_features.is_empty() {
                prompt.push_str("### Core Components:\n");
                for feature in &skill.tech_features[..skill.tech_features.len().min(5)] {
                    prompt.push_str(&format!("- {}\n", feature));
                }
            }

            // Add common failure modes
            if !skill.common_failures.is_empty() {
                prompt.push_str("\n### Common Failure Modes:\n");
                for failure in &skill.common_failures[..skill.common_failures.len().min(2)] {
                    // Take first line as summary
                    let summary = failure.lines().next().unwrap_or(failure);
                    prompt.push_str(&format!("- {}\n", summary));
                }
            }

            // Add guidance rules
            if !skill.guidance_rules.is_empty() {
                prompt.push_str("\n### Safety Rules:\n");
                for rule in &skill.guidance_rules[..skill.guidance_rules.len().min(3)] {
                    prompt.push_str(&format!("- {}\n", rule));
                }
            }

            // Add top knowledge sections
            if !skill.knowledge_sections.is_empty() {
                prompt.push_str("\n### Knowledge Base:\n");
                for section in &skill.knowledge_sections[..skill.knowledge_sections.len().min(2)] {
                    // First line as topic
                    let first_line = section.content.lines().next().unwrap_or("");
                    prompt.push_str(&format!(
                        "- **{}**: {}\n",
                        section.topic,
                        if first_line.len() > 80 {
                            &first_line[..80]
                        } else {
                            first_line
                        }
                    ));
                }
            }
        }

        prompt.push_str("\n---\nUse this knowledge to provide SPECIFIC, actionable guidance based on what you observe.");

        prompt
    }
}

/// A matched skill with its matched detection patterns.
#[derive(Debug, Clone)]
pub struct SkillMatch {
    pub skill: SkillKnowledge,
    pub matched_patterns: Vec<DetectionRule>,
}

/// Thread-safe skill knowledge base for the AI loop.
pub type SkillKnowledgeBaseArc = Arc<Mutex<SkillKnowledgeBase>>;

/// Get a shared instance of the skill knowledge base.
pub fn create_skill_knowledge_base(skills_dir: &str) -> SkillKnowledgeBaseArc {
    let mut kb = SkillKnowledgeBase::new();
    kb.load_from_directory(skills_dir);
    Arc::new(Mutex::new(kb))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_knowledge_base_creation() {
        let kb = SkillKnowledgeBase::new();
        assert!(kb.skills.is_empty());
    }

    #[test]
    fn test_load_nonexistent_directory() {
        let mut kb = SkillKnowledgeBase::new();
        kb.load_from_directory("/nonexistent/path");
        assert!(kb.skills.is_empty());
    }

    #[test]
    fn test_parse_detection_rules() {
        let kb = SkillKnowledgeBase::new();
        let yaml = r#"- id: test-detect
  name: Test Detection
  pattern: 'test|testing'
  confidence: 0.9
  technology_domain: Test Domain
  detection_type: Browser
"#;
        let rules = kb.parse_detection_rules(yaml, "Default Domain");
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].id, "test-detect");
        assert_eq!(rules[0].confidence, 0.9);
        assert_eq!(rules[0].technology_domain, "Test Domain");
    }
}