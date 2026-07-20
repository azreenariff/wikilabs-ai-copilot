//! Engineering context — combined view of the user's development environment.

use crate::TechnologyInference;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Combined engineering context from multiple observations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EngineeringContext {
    /// Unique context identifier.
    pub id: Uuid,
    /// Workspace path.
    pub workspace_path: String,
    /// Known technologies and their confidence scores.
    pub technologies: Vec<TechnologyInference>,
    /// Confidence scores keyed by technology name.
    pub confidence_scores: HashMap<String, f32>,
    /// Primary intent detected from conversation.
    pub primary_intent: Option<String>,
    /// Secondary intents.
    pub secondary_intents: Vec<String>,
    /// Timestamp when this context was last updated.
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

impl EngineeringContext {
    /// Create a new engineering context.
    pub fn new(workspace_path: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            workspace_path: workspace_path.into(),
            technologies: Vec::new(),
            confidence_scores: HashMap::new(),
            primary_intent: None,
            secondary_intents: Vec::new(),
            last_updated: chrono::Utc::now(),
        }
    }

    /// Add or update a technology inference.
    pub fn add_technology(&mut self, inference: TechnologyInference) {
        self.technologies.push(inference.clone());
        self.confidence_scores
            .insert(inference.name.clone(), inference.confidence);
        self.last_updated = chrono::Utc::now();
    }
}
