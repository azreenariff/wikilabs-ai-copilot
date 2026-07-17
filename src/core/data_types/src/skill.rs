//! Skill module metadata.

use serde::{Deserialize, Serialize};

/// Metadata about a skill module.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SkillMetadata {
    /// Unique identifier for this skill.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Semantic version string.
    pub version: String,
    /// Short description.
    pub description: String,
    /// Skill author or team.
    pub author: String,
    /// Technologies this skill covers.
    pub technologies: Vec<String>,
}

impl SkillMetadata {
    /// Create a new skill metadata entry.
    pub fn new(id: impl Into<String>, name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            version: version.into(),
            description: String::new(),
            author: String::new(),
            technologies: Vec::new(),
        }
    }
}