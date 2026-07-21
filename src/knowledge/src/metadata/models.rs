//! Metadata store models.
//!
//! Rust types representing rows in the `knowledge_metadata` table.
//! Also provides helper methods to convert between metadata entries and
//! graph node/edge types from the core data-types crate.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use wikilabs_data_types::GraphNode;

/// Graph-ready node category for metadata entries.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NodeCategory {
    Technology,
    Workflow,
    Command,
    Documentation,
    Error,
    Sop,
    Skill,
}

impl NodeCategory {
    /// Convert to database string representation.
    pub fn as_str(&self) -> &str {
        match self {
            NodeCategory::Technology => "technology",
            NodeCategory::Workflow => "workflow",
            NodeCategory::Command => "command",
            NodeCategory::Documentation => "documentation",
            NodeCategory::Error => "error",
            NodeCategory::Sop => "sop",
            NodeCategory::Skill => "skill",
        }
    }
}

impl From<NodeCategory> for String {
    fn from(cat: NodeCategory) -> Self {
        cat.as_str().to_string()
    }
}

impl From<&str> for NodeCategory {
    fn from(s: &str) -> Self {
        match s {
            "technology" => Self::Technology,
            "workflow" => Self::Workflow,
            "command" => Self::Command,
            "documentation" => Self::Documentation,
            "error" => Self::Error,
            "sop" => Self::Sop,
            "skill" => Self::Skill,
            _ => Self::Documentation,
        }
    }
}

/// Graph edge / relationship types.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RelationshipType {
    TechnologyWorkflow,
    TechnologyCommands,
    TechnologyDocumentation,
    TechnologyVendorKb,
    WorkflowSop,
    WorkflowSkill,
    CommandTroubleshootingGuide,
    ErrorVendorKb,
    ErrorWorkflow,
    ErrorBestPractice,
}

impl RelationshipType {
    /// Convert to database string representation.
    pub fn as_str(&self) -> &str {
        match self {
            RelationshipType::TechnologyWorkflow => "Technology->Workflow",
            RelationshipType::TechnologyCommands => "Technology->Commands",
            RelationshipType::TechnologyDocumentation => "Technology->Documentation",
            RelationshipType::TechnologyVendorKb => "Technology->Vendor KB",
            RelationshipType::WorkflowSop => "Workflow->SOP",
            RelationshipType::WorkflowSkill => "Workflow->Skill",
            RelationshipType::CommandTroubleshootingGuide => "Command->Troubleshooting Guide",
            RelationshipType::ErrorVendorKb => "Error->Vendor KB",
            RelationshipType::ErrorWorkflow => "Error->Workflow",
            RelationshipType::ErrorBestPractice => "Error->Best Practice",
        }
    }
}

impl From<&str> for RelationshipType {
    fn from(s: &str) -> Self {
        match s {
            "Technology->Workflow" => Self::TechnologyWorkflow,
            "Technology->Commands" => Self::TechnologyCommands,
            "Technology->Documentation" => Self::TechnologyDocumentation,
            "Technology->Vendor KB" => Self::TechnologyVendorKb,
            "Workflow->SOP" => Self::WorkflowSop,
            "Workflow->Skill" => Self::WorkflowSkill,
            "Command->Troubleshooting Guide" => Self::CommandTroubleshootingGuide,
            "Error->Vendor KB" => Self::ErrorVendorKb,
            "Error->Workflow" => Self::ErrorWorkflow,
            "Error->Best Practice" => Self::ErrorBestPractice,
            _ => Self::TechnologyDocumentation,
        }
    }
}

/// Security classification level.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum SecurityClassification {
    #[default]
    Public,
    Internal,
    Confidential,
    Restricted,
}

impl SecurityClassification {
    pub fn as_str(&self) -> &str {
        match self {
            SecurityClassification::Public => "public",
            SecurityClassification::Internal => "internal",
            SecurityClassification::Confidential => "confidential",
            SecurityClassification::Restricted => "restricted",
        }
    }
}

impl From<&str> for SecurityClassification {
    fn from(s: &str) -> Self {
        match s {
            "public" => Self::Public,
            "internal" => Self::Internal,
            "confidential" => Self::Confidential,
            "restricted" => Self::Restricted,
            _ => Self::default(),
        }
    }
}

/// A metadata entry for a knowledge document.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MetadataEntry {
    /// Row ID in the metadata table.
    pub id: String,
    /// Associated knowledge document ID.
    pub document_id: String,

    // ── Document Identity ──────────────────────────────────────
    pub title: String,
    pub knowledge_pack: String,
    pub vendor: String,
    pub product: String,
    pub version: String,
    pub technology: String,

    // ── Author & Publication ──────────────────────────────────
    pub author: String,
    pub publication_date: Option<String>,
    pub last_indexed: String,

    // ── Classification ────────────────────────────────────────
    pub security_classification: String,
    pub customer_scope: String,
    pub language: String,

    // ── Embedding ─────────────────────────────────────────────
    pub embedding_version: String,

    // ── Tags ──────────────────────────────────────────────────
    pub tags: String,

    // ── Graph-Ready ───────────────────────────────────────────
    pub node_type: String,
    pub relationship_type: String,
    pub source_node_id: String,
    pub target_node_id: String,
    pub edge_properties: String,

    // ── Timestamps ────────────────────────────────────────────
    pub created_at: String,
    pub updated_at: String,
}

impl MetadataEntry {
    /// Create a new metadata entry with default values.
    pub fn new(document_id: &str, title: &str, knowledge_pack: &str) -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            id: Uuid::new_v4().to_string(),
            document_id: document_id.to_string(),
            title: title.to_string(),
            knowledge_pack: knowledge_pack.to_string(),
            vendor: String::new(),
            product: String::new(),
            version: String::new(),
            technology: String::new(),
            author: String::new(),
            publication_date: None,
            last_indexed: now.clone(),
            security_classification: "public".to_string(),
            customer_scope: String::new(),
            language: "en".to_string(),
            embedding_version: String::new(),
            tags: String::new(),
            node_type: "documentation".to_string(),
            relationship_type: String::new(),
            source_node_id: String::new(),
            target_node_id: String::new(),
            edge_properties: "null".to_string(),
            created_at: now.clone(),
            updated_at: now,
        }
    }

    /// Parse a metadata entry from a database row.
    pub fn from_row(
        id: String,
        document_id: String,
        title: String,
        knowledge_pack: String,
        vendor: String,
        product: String,
        version: String,
        technology: String,
        author: String,
        publication_date: String,
        last_indexed: String,
        security_classification: String,
        customer_scope: String,
        language: String,
        embedding_version: String,
        tags: String,
        node_type: String,
        relationship_type: String,
        source_node_id: String,
        target_node_id: String,
        edge_properties: String,
        created_at: String,
        updated_at: String,
    ) -> Self {
        Self {
            id,
            document_id,
            title,
            knowledge_pack,
            vendor,
            product,
            version,
            technology,
            author,
            publication_date: if publication_date.is_empty() {
                None
            } else {
                Some(publication_date)
            },
            last_indexed,
            security_classification,
            customer_scope,
            language,
            embedding_version,
            tags,
            node_type,
            relationship_type,
            source_node_id,
            target_node_id,
            edge_properties,
            created_at,
            updated_at,
        }
    }

    /// Convert this entry to graph node fields.
    pub fn to_node(&self) -> wikilabs_data_types::GraphNode {
        use wikilabs_data_types::NodeCategory as Nc;
        let category = match self.node_type.as_str() {
            "technology" => Nc::Technology,
            "workflow" => Nc::Workflow,
            "command" => Nc::Command,
            "documentation" => Nc::Documentation,
            "error" => Nc::Error,
            "sop" => Nc::Sop,
            "skill" => Nc::Skill,
            _ => Nc::Documentation,
        };

        let technologies: Vec<String> = self
            .technology
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let properties: serde_json::Value =
            serde_json::from_str(&self.edge_properties).unwrap_or(serde_json::Value::Null);

        let created_at: DateTime<Utc> = self.created_at.parse().unwrap_or_else(|_| Utc::now());
        let updated_at: DateTime<Utc> = self.updated_at.parse().unwrap_or_else(|_| Utc::now());

        GraphNode {
            id: Uuid::parse_str(&self.id).unwrap_or_else(|_| Uuid::new_v4()),
            category,
            title: self.title.clone(),
            label: self.title.clone(),
            description: self.title.clone(),
            pack_name: self.knowledge_pack.clone(),
            technologies,
            vendor: self.vendor.clone(),
            created_at,
            updated_at,
            properties,
            vendor_metadata: None,
        }
    }
}
