//! Skill Discovery Engine — scans the workspace and detects technologies, skills, and patterns.
//!
//! The Skill Discovery Engine is responsible for:
//! - Scanning workspace directories for technology signatures
//! - Matching file patterns, command presence, and configuration files
//! - Computing confidence scores for each detected technology
//! - Ranking and prioritizing detected skills
//! - Producing a discovery report for the Skill Activation Engine
//!
//! ## Discovery Flow
//!
//! 1. **Scan** workspace directories using glob patterns
//! 2. **Match** file patterns and command outputs against technology signatures
//! 3. **Score** confidence based on signal strength
//! 4. **Rank** by priority and relevance
//! 5. **Report** results to the Skill Platform

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

/// A single detected technology signature in the workspace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechSignal {
    /// Unique signal ID.
    pub id: String,
    /// Technology domain detected (e.g., "Linux", "Kubernetes").
    pub technology: String,
    /// Source of the signal (file path, command output, etc.).
    pub source: String,
    /// Confidence score (0.0–1.0).
    pub confidence: f64,
    /// Priority level (higher = more important).
    pub priority: u32,
    /// Pattern that matched.
    pub pattern: String,
    /// Extracted metadata.
    pub metadata: HashMap<String, String>,
}

/// A discovered skill candidate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredSkill {
    /// Skill ID.
    pub id: String,
    /// Skill name.
    pub name: String,
    /// Technology domain.
    pub technology: String,
    /// Category.
    pub category: String,
    /// Confidence score.
    pub confidence: f64,
    /// Number of supporting signals.
    pub signal_count: usize,
    /// List of supporting signals.
    pub signals: Vec<TechSignal>,
    /// Whether this skill already has a definition.
    pub has_definition: bool,
    /// Recommended action.
    pub recommendation: String,
}

/// Discovery report with all findings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryReport {
    /// Detected technologies.
    pub technologies: Vec<TechSignal>,
    /// Discovered skill candidates.
    pub skills: Vec<DiscoveredSkill>,
    /// Scanned directories.
    pub scanned_dirs: Vec<String>,
    /// Total files scanned.
    pub files_scanned: usize,
    /// Scan timestamp.
    pub timestamp: String,
}

/// Configuration for the discovery engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryConfig {
    /// Directories to scan.
    pub scan_dirs: Vec<PathBuf>,
    /// Glob patterns for file matching.
    pub file_patterns: Vec<String>,
    /// Confidence threshold (0.0–1.0).
    pub confidence_threshold: f64,
    /// Whether to check for command presence.
    pub check_commands: bool,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            scan_dirs: vec![PathBuf::from(".")],
            file_patterns: vec![
                "**/manifest.yaml".to_string(),
                "**/technology.yaml".to_string(),
                "**/*.sh".to_string(),
                "**/*.py".to_string(),
                "**/Dockerfile".to_string(),
                "**/*.yml".to_string(),
                "**/*.yaml".to_string(),
            ],
            confidence_threshold: 0.3,
            check_commands: true,
        }
    }
}

/// Skill Discovery Engine.
pub struct SkillDiscoveryEngine {
    config: DiscoveryConfig,
    /// Registered technology signatures.
    signatures: Vec<TechSignature>,
    /// Known skill definitions.
    known_skills: HashMap<String, KnownSkill>,
}

/// A technology signature for detection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechSignature {
    /// Technology domain.
    pub domain: String,
    /// File patterns to match.
    pub file_patterns: Vec<String>,
    /// Command patterns to check.
    pub command_patterns: Vec<String>,
    /// Base confidence score.
    pub base_confidence: f64,
    /// Priority.
    pub priority: u32,
}

/// A known skill definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnownSkill {
    /// Skill ID.
    pub id: String,
    /// Skill name.
    pub name: String,
    /// Technology domain.
    pub technology: String,
    /// Category.
    pub category: String,
    /// Required detection signals.
    pub required_signals: Vec<String>,
    /// Optional signals.
    pub optional_signals: Vec<String>,
}

impl SkillDiscoveryEngine {
    /// Create a new discovery engine with default config.
    pub fn new() -> Self {
        Self {
            config: DiscoveryConfig::default(),
            signatures: Vec::new(),
            known_skills: HashMap::new(),
        }
    }

    /// Create with a custom configuration.
    pub fn with_config(config: DiscoveryConfig) -> Self {
        Self {
            config,
            signatures: Vec::new(),
            known_skills: HashMap::new(),
        }
    }

    /// Register a technology signature for detection.
    pub fn register_signature(&mut self, sig: TechSignature) {
        debug!("Registered signature for: {}", sig.domain);
        self.signatures.push(sig);
    }

    /// Register a known skill definition.
    pub fn register_known_skill(&mut self, skill: KnownSkill) {
        debug!("Registered known skill: {}", skill.id);
        self.known_skills.insert(skill.id.clone(), skill);
    }

    /// Scan the workspace for technology signals.
    pub fn scan(&self) -> Result<DiscoveryReport> {
        let mut report = DiscoveryReport {
            technologies: Vec::new(),
            skills: Vec::new(),
            scanned_dirs: Vec::new(),
            files_scanned: 0,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        // Scan each configured directory
        for scan_dir in &self.config.scan_dirs {
            if !scan_dir.exists() {
                debug!("Scan directory does not exist: {:?}", scan_dir);
                continue;
            }

            self.scan_directory(scan_dir, &mut report)?;
            report
                .scanned_dirs
                .push(scan_dir.display().to_string());
        }

        // Detect skills from signals
        report.skills = self.detect_skills(&report.technologies);

        // Filter by confidence threshold
        report.technologies.retain(|s| s.confidence >= self.config.confidence_threshold);
        report.skills.retain(|s| s.confidence >= self.config.confidence_threshold);

        info!(
            "Discovery complete: {} signals, {} skills",
            report.technologies.len(),
            report.skills.len()
        );

        Ok(report)
    }

    /// Scan a single directory recursively.
    fn scan_directory(
        &self,
        dir: &Path,
        report: &mut DiscoveryReport,
    ) -> Result<()> {
        // Collect all files matching patterns
        for pattern in &self.config.file_patterns {
            let matcher = globset::Glob::new(pattern)
                .context("Invalid glob pattern")?
                .compile_matcher();

            let walk = walkdir::WalkDir::new(dir);
            for entry in walk.into_iter().filter_map(|e| e.ok()) {
                if entry.file_type().is_file() {
                    report.files_scanned += 1;
                    let path = entry.path();

                    if matcher.is_match(path) {
                        // Check each technology signature
                        for sig in &self.signatures {
                            for file_pattern in &sig.file_patterns {
                                let file_matcher = globset::Glob::new(file_pattern)
                                    .context("Invalid file pattern")?
                                    .compile_matcher();

                                if file_matcher.is_match(path) {
                                    let signal_id = format!(
                                        "{}-{}",
                                        sig.domain,
                                        path.file_name()
                                            .map(|n| n.to_string_lossy())
                                            .unwrap_or_default()
                                    );

                                    let mut metadata = HashMap::new();
                                    metadata.insert("path".to_string(), path.display().to_string());

                                    // Calculate confidence based on match quality
                                    let confidence = self.calculate_confidence(sig, path);

                                    report.technologies.push(TechSignal {
                                        id: signal_id,
                                        technology: sig.domain.clone(),
                                        source: path.display().to_string(),
                                        confidence,
                                        priority: sig.priority,
                                        pattern: file_pattern.clone(),
                                        metadata,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        // Check for command presence if configured
        if self.config.check_commands {
            self.check_command_presence(dir, report)?;
        }

        Ok(())
    }

    /// Check for command presence in the workspace.
    fn check_command_presence(
        &self,
        dir: &Path,
        report: &mut DiscoveryReport,
    ) -> Result<()> {
        // Read script files and check for command patterns
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            if let Ok(content) = fs::read_to_string(&path) {
                for sig in &self.signatures {
                    for cmd_pattern in &sig.command_patterns {
                        if content.contains(cmd_pattern) {
                            let signal_id = format!("{}-cmd-{}", sig.domain, path.file_name().map(|n| n.to_string_lossy()).unwrap_or_default());
                            report.technologies.push(TechSignal {
                                id: signal_id,
                                technology: sig.domain.clone(),
                                source: path.display().to_string(),
                                confidence: sig.base_confidence * 0.5, // Lower confidence for command-only
                                priority: sig.priority,
                                pattern: cmd_pattern.clone(),
                                metadata: HashMap::new(),
                            });
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Calculate confidence score for a signal.
    fn calculate_confidence(&self, sig: &TechSignature, path: &Path) -> f64 {
        let mut confidence = sig.base_confidence;

        // Boost confidence for well-known config files
        if let Some(file_name) = path.file_name() {
            let name = file_name.to_string_lossy().to_lowercase();

            if name.contains("manifest") || name.contains("config") {
                confidence += 0.1;
            }
            if name.contains("docker") || name.contains("kube") {
                confidence += 0.15;
            }
        }

        // Cap at 1.0
        confidence.min(1.0)
    }

    /// Detect skills from technology signals.
    fn detect_skills(&self, signals: &[TechSignal]) -> Vec<DiscoveredSkill> {
        let mut skills = Vec::new();

        for (skill_id, known_skill) in &self.known_skills {
            let mut skill_signals = Vec::new();
            let mut has_required = false;

            for required_signal in &known_skill.required_signals {
                if signals.iter().any(|s| s.id == *required_signal || s.technology == *required_signal) {
                    has_required = true;
                    if let Some(sig) = signals.iter().find(|s| s.id == *required_signal || s.technology == *required_signal) {
                        skill_signals.push(sig.clone());
                    }
                }
            }

            if has_required {
                for optional_signal in &known_skill.optional_signals {
                    if let Some(sig) = signals.iter().find(|s| s.id == *optional_signal || s.technology == *optional_signal) {
                        skill_signals.push(sig.clone());
                    }
                }
            }

            if !skill_signals.is_empty() {
                let confidence = if has_required {
                    skill_signals.iter().map(|s| s.confidence).sum::<f64>() / skill_signals.len() as f64
                } else {
                    0.0
                };

                let has_definition = self.known_skills.contains_key(skill_id);

                skills.push(DiscoveredSkill {
                    id: skill_id.clone(),
                    name: known_skill.name.clone(),
                    technology: known_skill.technology.clone(),
                    category: known_skill.category.clone(),
                    confidence,
                    signal_count: skill_signals.len(),
                    signals: skill_signals,
                    has_definition,
                    recommendation: self.recommend_action(has_required, has_definition),
                });
            }
        }

        // Sort by confidence descending
        skills.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));

        skills
    }

    /// Recommend an action based on detection.
    fn recommend_action(&self, has_required: bool, has_definition: bool) -> String {
        if has_definition {
            "Skill is already defined and available".to_string()
        } else if has_required {
            "Consider creating a skill definition for this technology".to_string()
        } else {
            "Insufficient signals for skill recommendation".to_string()
        }
    }

    /// Get the discovery configuration.
    pub fn config(&self) -> &DiscoveryConfig {
        &self.config
    }

    /// Set the discovery configuration.
    pub fn set_config(&mut self, config: DiscoveryConfig) {
        self.config = config;
    }

    /// Get registered signatures.
    pub fn signatures(&self) -> &[TechSignature] {
        &self.signatures
    }

    /// Get known skills.
    pub fn known_skills(&self) -> &HashMap<String, KnownSkill> {
        &self.known_skills
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_engine() {
        let engine = SkillDiscoveryEngine::new();
        assert!(engine.signatures().is_empty());
        assert!(engine.known_skills().is_empty());
    }

    #[test]
    fn test_register_signature() {
        let mut engine = SkillDiscoveryEngine::new();
        engine.register_signature(TechSignature {
            domain: "Linux".to_string(),
            file_patterns: vec!["/etc/hostname".to_string()],
            command_patterns: vec!["uname".to_string()],
            base_confidence: 0.8,
            priority: 10,
        });
        assert_eq!(engine.signatures().len(), 1);
    }

    #[test]
    fn test_register_known_skill() {
        let mut engine = SkillDiscoveryEngine::new();
        engine.register_known_skill(KnownSkill {
            id: "linux-engineering".to_string(),
            name: "Linux Engineering".to_string(),
            technology: "Linux".to_string(),
            category: "Engineering".to_string(),
            required_signals: vec!["Linux".to_string()],
            optional_signals: vec!["Docker".to_string()],
        });
        assert_eq!(engine.known_skills().len(), 1);
    }

    #[test]
    fn test_scan_empty_dir() {
        let engine = SkillDiscoveryEngine::new();
        let report = engine.scan().unwrap();
        assert!(report.technologies.is_empty());
        assert!(report.skills.is_empty());
    }
}