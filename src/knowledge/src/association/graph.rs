//! Knowledge graph — nodes, edges, traversal, and analysis.

use super::{EdgeType, KnowledgeEdge, Weight, GraphAnalysis};
use chrono::Utc;
use std::collections::{HashMap, HashSet};
use tracing::debug;

/// A node in the knowledge graph.
#[derive(Debug, Clone)]
pub struct GraphNode {
    pub id: String,
    pub title: String,
    pub knowledge_pack: String,
    pub node_type: String,
    pub embedding: Option<Vec<f32>>,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// A directed edge with type and weight.
#[derive(Debug, Clone)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
    pub edge_type: EdgeType,
    pub weight: f32,
    pub metadata: serde_json::Value,
}

/// The knowledge graph storing all nodes and edges.
pub struct KnowledgeGraph {
    nodes: HashMap<String, GraphNode>,
    edges: Vec<GraphEdge>,
    adjacency: HashMap<String, Vec<(String, EdgeType, f32)>>,
}

impl KnowledgeGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
            adjacency: HashMap::new(),
        }
    }

    /// Add a node to the graph.
    pub fn add_node(&mut self, node: GraphNode) {
        let id = node.id.clone();
        self.nodes.insert(id.clone(), node);
        self.adjacency.entry(id).or_insert_with(Vec::new);
        debug!(node_id = &id, "Added node to knowledge graph");
    }

    /// Add an edge to the graph.
    pub fn add_edge(&mut self, edge: GraphEdge) {
        let source = edge.source.clone();
        self.edges.push(edge.clone());
        let out_edges = self.adjacency.entry(source).or_insert_with(Vec::new);
        out_edges.push((edge.target.clone(), edge.edge_type.clone(), edge.weight));
        debug!(source = &edge.source, target = &edge.target, "Added edge to knowledge graph");
    }

    /// Remove a node and all its edges.
    pub fn remove_node(&mut self, node_id: &str) {
        self.nodes.remove(node_id);
        self.edges.retain(|e| e.source != node_id && e.target != node_id);
        self.adjacency.remove(node_id);
        // Remove from other nodes' adjacency lists
        for neighbors in self.adjacency.values_mut() {
            neighbors.retain(|(target, _, _)| target != node_id);
        }
        debug!(node_id = node_id, "Removed node from knowledge graph");
    }

    /// Get neighbors of a node.
    pub fn get_neighbors(&self, node_id: &str) -> &Vec<(String, EdgeType, f32)> {
        self.adjacency.get(node_id).unwrap_or(&Vec::new())
    }

    /// Get all edges connected to a node.
    pub fn get_edges(&self, node_id: &str) -> Vec<&GraphEdge> {
        self.edges.iter()
            .filter(|e| e.source == node_id || e.target == node_id)
            .collect()
    }

    /// Get a node by ID.
    pub fn get_node(&self, node_id: &str) -> Option<&GraphNode> {
        self.nodes.get(node_id)
    }

    /// Get all nodes.
    pub fn nodes(&self) -> &HashMap<String, GraphNode> {
        &self.nodes
    }

    /// Get all edges.
    pub fn edges(&self) -> &[GraphEdge] {
        &self.edges
    }

    /// Compute basic graph statistics.
    pub fn analyze(&self) -> GraphAnalysis {
        let node_count = self.nodes.len();
        let edge_count = self.edges.len();
        let total_degree: f32 = self.adjacency.values().map(|v| v.len() as f32).sum();
        let average_degree = if node_count > 0 {
            total_degree / node_count as f32
        } else {
            0.0
        };

        // Count connected components using BFS
        let mut visited = HashSet::new();
        let mut components = 0;

        for node_id in self.nodes.keys() {
            if visited.contains(node_id) {
                continue;
            }
            components += 1;
            // BFS
            let mut queue = vec![node_id.clone()];
            while let Some(current) = queue.pop() {
                if visited.contains(&current) {
                    continue;
                }
                visited.insert(current.clone());
                if let Some(neighbors) = self.adjacency.get(&current) {
                    for (target, _, _) in neighbors {
                        if !visited.contains(target) {
                            queue.push(target.clone());
                        }
                    }
                }
            }
        }

        // Find central nodes (highest degree)
        let mut degree_list: Vec<(String, usize)> = self
            .adjacency
            .iter()
            .map(|(id, neighbors)| (id.clone(), neighbors.len()))
            .collect();
        degree_list.sort_by(|a, b| b.1.cmp(&a.1));
        let central_nodes: Vec<String> = degree_list
            .iter()
            .take(5)
            .map(|(id, _)| id.clone())
            .collect();

        GraphAnalysis {
            node_count,
            edge_count,
            connected_components: components,
            average_degree,
            diameter: None, // Complex to compute, leave as None for now
            central_nodes,
        }
    }

    /// Find paths between two nodes (simple BFS, max depth 10).
    pub fn find_path(
        &self,
        source: &str,
        target: &str,
        max_depth: usize,
    ) -> Option<Vec<String>> {
        if source == target {
            return Some(vec![source.to_string()]);
        }

        let mut queue = vec![(source.to_string(), vec![source.to_string()])];
        let mut visited = HashSet::new();

        while let Some((current, path)) = queue.pop() {
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current.clone());

            if current == target {
                return Some(path);
            }

            if path.len() > max_depth {
                continue;
            }

            if let Some(neighbors) = self.adjacency.get(&current) {
                for (target_id, _, _) in neighbors {
                    if !visited.contains(target_id) {
                        let mut new_path = path.clone();
                        new_path.push(target_id.clone());
                        queue.push((target_id.clone(), new_path));
                    }
                }
            }
        }

        None
    }

    /// Export nodes as JSON for serialization.
    pub fn export_nodes(&self) -> Vec<serde_json::Value> {
        self.nodes
            .values()
            .map(|n| {
                serde_json::json!({
                    "id": n.id,
                    "title": n.title,
                    "knowledge_pack": n.knowledge_pack,
                    "node_type": n.node_type,
                    "metadata": n.metadata,
                    "created_at": n.created_at.to_rfc3339(),
                    "updated_at": n.updated_at.to_rfc3339(),
                })
            })
            .collect()
    }

    /// Export edges as JSON for serialization.
    pub fn export_edges(&self) -> Vec<serde_json::Value> {
        self.edges
            .iter()
            .map(|e| {
                serde_json::json!({
                    "source": e.source,
                    "target": e.target,
                    "edge_type": format!("{:?}", e.edge_type),
                    "weight": e.weight,
                    "metadata": e.metadata,
                })
            })
            .collect()
    }
}

impl Default for KnowledgeGraph {
    fn default() -> Self {
        Self::new()
    }
}