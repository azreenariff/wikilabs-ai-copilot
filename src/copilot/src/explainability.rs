//! Explainability — traceable reasoning for every recommendation.
//!
//! Every recommendation includes a clear, traceable explanation
//! that shows exactly why the Copilot made it, what evidence
//! supports it, and what alternatives were considered.
//!
//! This is critical for engineer trust — they must understand
//! the reasoning before deciding to act.

use crate::Evidence;
use serde::{Deserialize, Serialize};
use std::fmt;

/// A single explanation node.
///
/// Each node has:
/// - A label describing what this reasoning step is about
/// - A detailed explanation
/// - Links to supporting evidence (indices into the Evidence list)
/// - Whether this node is mandatory or optional
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplanationNode {
    pub label: String,
    pub explanation: String,
    pub evidence_indices: Vec<usize>,
    pub is_mandatory: bool,
}

impl ExplanationNode {
    pub fn new(
        label: impl Into<String>,
        explanation: impl Into<String>,
        evidence_indices: Vec<usize>,
    ) -> Self {
        ExplanationNode {
            label: label.into(),
            explanation: explanation.into(),
            evidence_indices,
            is_mandatory: false,
        }
    }

    pub fn mandatory(mut self) -> Self {
        self.is_mandatory = true;
        self
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }
}

/// Full explainability record for a recommendation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Explainability {
    pub reason: String,
    pub nodes: Vec<ExplanationNode>,
    pub alternatives_considered: Vec<String>,
    pub certainty: f64,
    pub limitations: Vec<String>,
}

impl Explainability {
    pub fn new(reason: impl Into<String>) -> Self {
        Explainability {
            reason: reason.into(),
            nodes: Vec::new(),
            alternatives_considered: Vec::new(),
            certainty: 0.0,
            limitations: Vec::new(),
        }
    }

    pub fn with_nodes(mut self, nodes: Vec<ExplanationNode>) -> Self {
        self.nodes = nodes;
        self
    }

    pub fn with_alternatives(mut self, alternatives: Vec<String>) -> Self {
        self.alternatives_considered = alternatives;
        self
    }

    pub fn with_certainty(mut self, certainty: f64) -> Self {
        self.certainty = certainty.clamp(0.0, 1.0);
        self
    }

    pub fn with_limitations(mut self, limitations: Vec<String>) -> Self {
        self.limitations = limitations;
        self
    }

    pub fn add_node(mut self, node: ExplanationNode) -> Self {
        self.nodes.push(node);
        self
    }

    pub fn add_alternative(mut self, alternative: impl Into<String>) -> Self {
        self.alternatives_considered.push(alternative.into());
        self
    }

    pub fn add_limitation(mut self, limitation: impl Into<String>) -> Self {
        self.limitations.push(limitation.into());
        self
    }

    /// Build a human-readable explanation string.
    pub fn to_explanation(&self) -> String {
        let mut parts = Vec::new();

        // Reason
        parts.push(format!("**Reason:** {}", self.reason));

        // Nodes
        for node in &self.nodes {
            let evidence_str = if node.evidence_indices.is_empty() {
                String::new()
            } else {
                format!(
                    " [Evidence: {}]",
                    node.evidence_indices
                        .iter()
                        .map(|i| format!("[{i}]"))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            };
            let mandatory = if node.is_mandatory { " (Required)" } else { "" };
            parts.push(format!(
                "- **{}**{}: {}",
                node.label, mandatory, node.explanation
            ));
            if !evidence_str.is_empty() {
                parts.push(format!("  {evidence_str}"));
            }
        }

        // Alternatives
        if !self.alternatives_considered.is_empty() {
            parts.push("".to_string());
            parts.push("**Alternatives considered:**".to_string());
            for alt in &self.alternatives_considered {
                parts.push(format!("- {alt}"));
            }
        }

        // Limitations
        if !self.limitations.is_empty() {
            parts.push("".to_string());
            parts.push("**Limitations:**".to_string());
            for lim in &self.limitations {
                parts.push(format!("- {lim}"));
            }
        }

        parts.join("\n")
    }

    /// Get the number of evidence references in the explanation.
    pub fn evidence_count(&self) -> usize {
        self.nodes.iter().map(|n| n.evidence_indices.len()).sum()
    }

    /// Check if this explanation has mandatory nodes.
    pub fn has_mandatory_nodes(&self) -> bool {
        self.nodes.iter().any(|n| n.is_mandatory)
    }
}

/// Factory for building explainability records from evidence.
pub struct ExplainabilityBuilder {
    reason: String,
    nodes: Vec<ExplanationNode>,
    alternatives: Vec<String>,
    certainty: f64,
    limitations: Vec<String>,
}

impl ExplainabilityBuilder {
    pub fn new(reason: impl Into<String>) -> Self {
        ExplainabilityBuilder {
            reason: reason.into(),
            nodes: Vec::new(),
            alternatives: Vec::new(),
            certainty: 0.0,
            limitations: Vec::new(),
        }
    }

    pub fn add_node(mut self, label: impl Into<String>, explanation: impl Into<String>) -> Self {
        self.nodes
            .push(ExplanationNode::new(label, explanation, Vec::new()));
        self
    }

    pub fn add_node_with_evidence(
        mut self,
        label: impl Into<String>,
        explanation: impl Into<String>,
        evidence_indices: Vec<usize>,
    ) -> Self {
        self.nodes
            .push(ExplanationNode::new(label, explanation, evidence_indices));
        self
    }

    pub fn with_certainty(mut self, certainty: f64) -> Self {
        self.certainty = certainty.clamp(0.0, 1.0);
        self
    }

    pub fn with_alternative(mut self, alternative: impl Into<String>) -> Self {
        self.alternatives.push(alternative.into());
        self
    }

    pub fn with_limitation(mut self, limitation: impl Into<String>) -> Self {
        self.limitations.push(limitation.into());
        self
    }

    pub fn build(self) -> Explainability {
        Explainability {
            reason: self.reason,
            nodes: self.nodes,
            alternatives_considered: self.alternatives,
            certainty: self.certainty,
            limitations: self.limitations,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_explainability_basic() {
        let expl = Explainability::new("Memory is high").with_certainty(0.8);
        assert_eq!(expl.reason, "Memory is high");
        assert_eq!(expl.certainty, 0.8);
        assert!(expl.nodes.is_empty());
    }

    #[test]
    fn test_explainability_with_node() {
        let node = ExplanationNode::new("Evidence", "Memory at 85%", vec![0]);
        let expl = Explainability::new("Memory is high")
            .add_node(node)
            .with_certainty(0.85);
        assert_eq!(expl.nodes.len(), 1);
        assert_eq!(expl.nodes[0].evidence_indices, vec![0]);
    }

    #[test]
    fn test_explainability_to_string() {
        let node = ExplanationNode::new("Evidence", "Memory at 85%", vec![0]);
        let expl = Explainability::new("Memory is high")
            .add_node(node)
            .add_alternative("Do nothing")
            .add_limitation("Limited to current observation");
        let output = expl.to_explanation();
        assert!(output.contains("Memory is high"));
        assert!(output.contains("Evidence"));
        assert!(output.contains("[0]"));
        assert!(output.contains("Do nothing"));
        assert!(output.contains("Limited"));
    }

    #[test]
    fn test_explainability_builder() {
        let expl = ExplainabilityBuilder::new("High memory")
            .add_node("Observation", "Pod memory at 85%")
            .with_certainty(0.9)
            .with_alternative("Ignore")
            .with_limitation("Only one pod monitored")
            .build();
        assert_eq!(expl.nodes.len(), 1);
        assert_eq!(expl.certainty, 0.9);
        assert_eq!(expl.alternatives_considered.len(), 1);
        assert_eq!(expl.limitations.len(), 1);
    }

    #[test]
    fn test_explainability_evidence_count() {
        let node1 = ExplanationNode::new("A", "Text", vec![0, 1]);
        let node2 = ExplanationNode::new("B", "Text", vec![2]);
        let node3 = ExplanationNode::new("C", "Text", vec![]);
        let expl = Explainability::new("Test")
            .add_node(node1)
            .add_node(node2)
            .add_node(node3);
        assert_eq!(expl.evidence_count(), 3);
    }

    #[test]
    fn test_mandatory_node() {
        let node = ExplanationNode::new("Required", "Must check", vec![]).mandatory();
        assert!(node.is_mandatory);
    }

    #[test]
    fn test_has_mandatory_nodes() {
        let expl = Explainability::new("Test")
            .add_node(ExplanationNode::new("A", "B", vec![]).mandatory());
        assert!(expl.has_mandatory_nodes());

        let expl2 = Explainability::new("Test").add_node(ExplanationNode::new("A", "B", vec![]));
        assert!(!expl2.has_mandatory_nodes());
    }

    #[test]
    fn test_explanation_node_defaults() {
        let node = ExplanationNode::new("Label", "Explanation", vec![0]);
        assert_eq!(node.label, "Label");
        assert_eq!(node.explanation, "Explanation");
        assert_eq!(node.evidence_indices, vec![0]);
        assert!(!node.is_mandatory);
    }
}
