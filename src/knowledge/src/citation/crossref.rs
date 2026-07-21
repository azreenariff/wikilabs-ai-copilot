//! Cross-reference tracking between knowledge documents.

use super::Citation;
use chrono::Utc;

/// A cross-reference linking two knowledge documents.
#[derive(Debug, Clone)]
pub struct CrossReference {
    pub source_id: String,
    pub target_id: String,
    pub reference_type: ReferenceType,
    pub context: Option<String>,
    pub strength: f32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Type of cross-reference between documents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReferenceType {
    /// Direct citation.
    Citation,
    /// Related to (topic similarity).
    Related,
    /// Prerequisite for.
    Prerequisite,
    /// Extension of.
    Extension,
    /// Supersedes.
    Supersedes,
    /// Alternative to.
    Alternative,
    /// Part of (document hierarchy).
    PartOf,
    /// Parent of (document hierarchy).
    ParentOf,
    /// Implementation of.
    Implementation,
    /// Spec for (spec-to-impl).
    SpecFor,
}

impl CrossReference {
    pub fn new(source_id: &str, target_id: &str, reference_type: ReferenceType) -> Self {
        Self {
            source_id: source_id.to_string(),
            target_id: target_id.to_string(),
            reference_type,
            context: None,
            strength: 1.0,
            created_at: Utc::now(),
        }
    }

    pub fn with_context(mut self, context: &str) -> Self {
        self.context = Some(context.to_string());
        self
    }

    pub fn with_strength(mut self, strength: f32) -> Self {
        self.strength = strength.clamp(0.0, 1.0);
        self
    }
}

/// Build a cross-reference graph from citations.
pub fn build_crossref_graph(citations: &[Citation]) -> Vec<CrossReference> {
    let mut refs = Vec::new();
    for citation in citations {
        for related in &citation.related_ids {
            refs.push(CrossReference::new(
                &citation.id,
                related,
                ReferenceType::Related,
            ));
        }
    }
    refs
}
