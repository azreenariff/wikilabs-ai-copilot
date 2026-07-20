//! Topic association — extract and map topics from knowledge documents.

use super::{DiscoveryMethod, EdgeType, Relationship, Weight};
use std::collections::HashMap;
use tracing::debug;

/// A topic extracted from knowledge content.
#[derive(Debug, Clone)]
pub struct Topic {
    pub name: String,
    pub confidence: f32,
    pub category: String,
    pub document_ids: Vec<String>,
    pub weight: f32,
}

/// Association between topics.
#[derive(Debug, Clone)]
pub struct TopicAssociation {
    pub source_topic: String,
    pub target_topic: String,
    pub association_type: TopicAssociationType,
    pub strength: f32,
    pub evidence: Vec<String>,
    pub discovery_method: DiscoveryMethod,
}

/// How topics are associated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TopicAssociationType {
    /// Topics are synonyms.
    Synonym,
    /// Topics are related concepts.
    Related,
    /// One is a subset of the other.
    Subset,
    /// One is a superset of the other.
    Superset,
    /// Topics co-occur frequently.
    CoOccurrence,
    /// One leads to the other.
    Sequential,
}

/// Topic model for extracting and relating topics.
pub struct TopicModel {
    topics: HashMap<String, Topic>,
    associations: Vec<TopicAssociation>,
}

impl TopicModel {
    pub fn new() -> Self {
        Self {
            topics: HashMap::new(),
            associations: Vec::new(),
        }
    }

    /// Add a topic.
    pub fn add_topic(&mut self, topic: Topic) {
        let name = topic.name.clone();
        self.topics.insert(name.clone(), topic);
        debug!(topic = %name, "Added topic");
    }

    /// Add an association between topics.
    pub fn add_association(&mut self, association: TopicAssociation) {
        let source = association.source_topic.clone();
        let target = association.target_topic.clone();
        self.associations.push(association);
        debug!(
            source = %source,
            target = %target,
            "Added topic association"
        );
    }

    /// Get a topic by name.
    pub fn get_topic(&self, name: &str) -> Option<&Topic> {
        self.topics.get(name)
    }

    /// Get all topics.
    pub fn topics(&self) -> &HashMap<String, Topic> {
        &self.topics
    }

    /// Get associations for a topic.
    pub fn get_associations(&self, topic: &str) -> Vec<&TopicAssociation> {
        self.associations
            .iter()
            .filter(|a| a.source_topic == topic || a.target_topic == topic)
            .collect()
    }

    /// Find related topics to a given topic.
    pub fn find_related(&self, topic: &str, min_strength: f32) -> Vec<&Topic> {
        self.get_associations(topic)
            .into_iter()
            .filter(|a| a.strength >= min_strength)
            .flat_map(|a| {
                if a.source_topic == topic {
                    self.topics.get(&a.target_topic)
                } else {
                    self.topics.get(&a.source_topic)
                }
            })
            .collect()
    }

    /// Convert topic associations to knowledge relationships.
    pub fn to_relationships(&self) -> Vec<Relationship> {
        self.associations
            .iter()
            .map(|a| {
                let edge_type = match &a.association_type {
                    TopicAssociationType::Synonym => EdgeType::Alternative,
                    TopicAssociationType::Related => EdgeType::Related,
                    TopicAssociationType::Subset => EdgeType::PartOf,
                    TopicAssociationType::Superset => EdgeType::Extension,
                    TopicAssociationType::CoOccurrence => EdgeType::Related,
                    TopicAssociationType::Sequential => EdgeType::Prerequisite,
                };
                Relationship {
                    source_id: a.source_topic.clone(),
                    target_id: a.target_topic.clone(),
                    edge_type,
                    weight: Weight::Numeric(a.strength),
                    confidence: a.strength,
                    discovery_method: a.discovery_method.clone(),
                    metadata: serde_json::json!({
                        "association_type": format!("{:?}", a.association_type),
                        "evidence": &a.evidence,
                    }),
                    created_at: chrono::Utc::now(),
                }
            })
            .collect()
    }
}

impl Default for TopicModel {
    fn default() -> Self {
        Self::new()
    }
}
