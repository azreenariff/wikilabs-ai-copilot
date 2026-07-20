//! Knowledge association — semantic linking, graph edges, and relationship mapping.
//!
//! Discovers and manages relationships between knowledge documents,
//! including topic associations, semantic proximity, and structural links.

pub mod filter;
pub mod graph;
pub mod proximity;
pub mod topic;
pub mod workspace_store;

pub use filter::WorkspaceKnowledgeFilter;
pub use graph::KnowledgeGraph;
pub use proximity::SemanticProximity;
pub use topic::TopicAssociation;
pub use workspace_store::{AssociationStatus, WorkspaceKnowledgeStore};

/// Edge type in the knowledge graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EdgeType {
    /// Related topics.
    Related,
    /// Prerequisite.
    Prerequisite,
    /// Extension.
    Extension,
    /// Supersedes.
    Supersedes,
    /// Implementation.
    Implementation,
    /// Specification.
    Specification,
    /// Part of.
    PartOf,
    /// Alternative.
    Alternative,
    /// Depends on.
    DependsOn,
    /// Contradicts.
    Contradicts,
}

/// Weight of a relationship.
#[derive(Debug, Clone, PartialEq)]
pub enum Weight {
    /// Strong relationship.
    Strong,
    /// Moderate relationship.
    Moderate,
    /// Weak relationship.
    Weak,
    /// Numeric weight (0.0 to 1.0).
    Numeric(f32),
}

impl Weight {
    pub fn to_f32(&self) -> f32 {
        match self {
            Weight::Strong => 0.9,
            Weight::Moderate => 0.6,
            Weight::Weak => 0.3,
            Weight::Numeric(w) => w.min(1.0).max(0.0),
        }
    }
}

/// Relationship metadata.
#[derive(Debug, Clone)]
pub struct Relationship {
    pub source_id: String,
    pub target_id: String,
    pub edge_type: EdgeType,
    pub weight: Weight,
    pub confidence: f32,
    pub discovery_method: DiscoveryMethod,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// How the relationship was discovered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiscoveryMethod {
    /// Automatic semantic analysis.
    Automatic,
    /// Manual curation.
    Manual,
    /// Citation analysis.
    CitationAnalysis,
    /// Co-occurrence in text.
    CoOccurrence,
    /// Topic modeling.
    TopicModeling,
    /// User feedback.
    UserFeedback,
}

/// A single edge in the knowledge graph.
#[derive(Debug, Clone)]
pub struct KnowledgeEdge {
    pub source: String,
    pub target: String,
    pub edge_type: EdgeType,
    pub weight: f32,
    pub confidence: f32,
    pub metadata: serde_json::Value,
}

/// Result of graph analysis.
#[derive(Debug, Clone)]
pub struct GraphAnalysis {
    pub node_count: usize,
    pub edge_count: usize,
    pub connected_components: usize,
    pub average_degree: f32,
    pub diameter: Option<usize>,
    pub central_nodes: Vec<String>,
}
