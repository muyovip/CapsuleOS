use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet, BTreeMap};
use thiserror::Error;

// ============================================================================
// Type Aliases
// ============================================================================

pub type Hash = String;
pub type NodeId = String;

// ============================================================================
// Expression (simplified for this module - in practice, import from glyph_parser)
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Expression {
    Literal(i64),
    Var(String),
    Lambda { param: String, body: Box<Expression> },
    Apply { func: Box<Expression>, arg: Box<Expression> },
    Let { name: String, value: Box<Expression>, body: Box<Expression> },
}

// ============================================================================
// Core Data Structures
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: NodeId,
    pub root_ref: Hash,
    pub data: Expression,
    pub metadata: NodeMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeMetadata {
    pub timestamp: u64,
    pub lineage_depth: u32,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct GraphEdge {
    pub from: Hash,
    pub to: Hash,
    pub edge_type: EdgeType,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum EdgeType {
    Dependency,
    Derivation,
    Reference,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisGraph {
    nodes: HashMap<Hash, GraphNode>,
    edges: Vec<GraphEdge>,
    root_hash: Hash,
}

// ============================================================================
// Error Types
// ============================================================================

#[derive(Error, Debug)]
pub enum GraphError {
    #[error("Node not found: {0}")]
    NodeNotFound(Hash),
    
    #[error("Node already exists: {0}")]
    NodeAlreadyExists(Hash),
    
    #[error("Cycle detected: adding edge would create a cycle")]
    CycleDetected,
    
    #[error("Self-loop forbidden: {0} -> {0}")]
    SelfLoopForbidden(Hash),
    
    #[error("Root reference mismatch: expected {expected}, got {actual}")]
    RootRefMismatch { expected: Hash, actual: Hash },
    
    #[error("Invalid root hash")]
    InvalidRootHash,
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

// ============================================================================
// Hash Computation
// ============================================================================

pub fn compute_node_hash(node: &GraphNode) -> Hash {
    let mut hasher = Sha256::new();
    
    // Canonical serialization for hashing
    let serialized = canonical_serialize_node(node);
    hasher.update(b"GlyphV1:Node:");
    hasher.update(&serialized);
    
    hex::encode(hasher.finalize())
}

pub fn compute_root_hash(root_node: &GraphNode) -> Hash {
    let mut hasher = Sha256::new();
    
    let serialized = canonical_serialize_node(root_node);
    hasher.update(b"GlyphV1:Root:");
    hasher.update(&serialized);
    
    hex::encode(hasher.finalize())
}

fn canonical_serialize_node(node: &GraphNode) -> Vec<u8> {
    let mut buffer = Vec::new();
    ciborium::into_writer(node, &mut buffer).expect("Node serialization failed");
    buffer
}

// ============================================================================
// GenesisGraph Implementation
// ============================================================================

impl GenesisGraph {
    /// Create a new GenesisGraph with a root node
    pub fn new(root_node: GraphNode) -> Result<Self, GraphError> {
        let root_hash = compute_root_hash(&root_node);
        
        // Verify root_ref points to itself (root is self-referential)
        if root_node.root_ref != root_hash {
            return Err(GraphError::RootRefMismatch {
                expected: root_hash.clone(),
                actual: root_node.root_ref.clone(),
            });
        }
        
        let mut nodes = HashMap::new();
        nodes.insert(root_hash.clone(), root_node);
        
        Ok(Self {
            nodes,
            edges: Vec::new(),
            root_hash,
        })
    }
    
    /// Get the root hash
    pub fn root_hash(&self) -> &Hash {
        &self.root_hash
    }
    
    /// Get a node by hash
    pub fn get_node(&self, hash: &Hash) -> Option<&GraphNode> {
        self.nodes.get(hash)
    }
    
    /// Get all nodes
    pub fn nodes(&self) -> &HashMap<Hash, GraphNode> {
        &self.nodes
    }
    
    /// Get all edges
    pub fn edges(&self) -> &[GraphEdge] {
        &self.edges
    }
    
    /// Insert a new node into the graph
    pub fn insert_node(&mut self, node: GraphNode) -> Result<Hash, GraphError> {
        // Verify root_ref matches the graph's root
        if node.root_ref != self.root_hash {
            return Err(GraphError::RootRefMismatch {
                expected: self.root_hash.clone(),
                actual: node.root_ref.clone(),
            });
        }
        
        let node_hash = compute_node_hash(&node);
        
        // Check if node already exists
        if self.nodes.contains_key(&node_hash) {
            return Err(GraphError::NodeAlreadyExists(node_hash));
        }
        
        self.nodes.insert(node_hash.clone(), node);
        Ok(node_hash)
    }
    
    /// Link two nodes with an edge
    pub fn link_nodes(
        &mut self,
        from: Hash,
        to: Hash,
        edge_type: EdgeType,
    ) -> Result<(), GraphError> {
        // Verify both nodes exist
        if !self.nodes.contains_key(&from) {
            return Err(GraphError::NodeNotFound(from));
        }
        if !self.nodes.contains_key(&to) {
            return Err(GraphError::NodeNotFound(to));
        }
        
        // Forbid self-loops
        if from == to {
            return Err(GraphError::SelfLoopForbidden(from));
        }
        
        // Check for cycles before adding edge
        let new_edge = GraphEdge {
            from: from.clone(),
            to: to.clone(),
            edge_type,
        };
        
        // Temporarily add edge to check for cycles
        self.edges.push(new_edge.clone());
        
        if self.has_cycle() {
            // Remove the edge we just added
            self.edges.pop();
            return Err(GraphError::CycleDetected);
        }
        
        // Edge is valid and already added
        Ok(())
    }
    
    /// Delete a node and all its associated edges
    pub fn delete_node(&mut self, hash: &Hash) -> Result<GraphNode, GraphError> {
        // Cannot delete root node
        if hash == &self.root_hash {
            return Err(GraphError::InvalidRootHash);
        }
        
        // Remove node
        let node = self
            .nodes
            .remove(hash)
            .ok_or_else(|| GraphError::NodeNotFound(hash.clone()))?;
        
        // Remove all edges connected to this node
        self.edges.retain(|edge| edge.from != *hash && edge.to != *hash);
        
        Ok(node)
    }
    
    /// Check if the graph contains a cycle using DFS
    fn has_cycle(&self) -> bool {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        
        // Build adjacency list
        let mut adj_list: HashMap<&Hash, Vec<&Hash>> = HashMap::new();
        for edge in &self.edges {
            adj_list.entry(&edge.from).or_insert_with(Vec::new).push(&edge.to);
        }
        
        // Check all nodes
        for node_hash in self.nodes.keys() {
            if !visited.contains(node_hash) {
                if self.has_cycle_util(node_hash, &adj_list, &mut visited, &mut rec_stack) {
                    return true;
                }
            }
        }
        
        false
    }
    
    /// DFS helper for cycle detection
    fn has_cycle_util<'a>(
        &self,
        node: &'a Hash,
        adj_list: &HashMap<&'a Hash, Vec<&'a Hash>>,
        visited: &mut HashSet<&'a Hash>,
        rec_stack: &mut HashSet<&'a Hash>,
    ) -> bool {
        visited.insert(node);
        rec_stack.insert(node);
        
        if let Some(neighbors) = adj_list.get(node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    if self.has_cycle_util(neighbor, adj_list, visited, rec_stack) {
                        return true;
                    }
                } else if rec_stack.contains(neighbor) {
                    // Back edge found - cycle detected
                    return true;
                }
            }
        }
        
        rec_stack.remove(node);
        false
    }
    
    /// Topological sort of nodes (returns None if cycle exists)
    pub fn topological_sort(&self) -> Option<Vec<Hash>> {
        if self.has_cycle() {
            return None;
        }
        
        let mut in_degree: HashMap<Hash, usize> = HashMap::new();
        let mut adj_list: HashMap<Hash, Vec<Hash>> = HashMap::new();
        
        // Initialize in-degree and adjacency list
        for node_hash in self.nodes.keys() {
            in_degree.insert(node_hash.clone(), 0);
            adj_list.insert(node_hash.clone(), Vec::new());
        }
        
        // Build graph
        for edge in &self.edges {
            adj_list.get_mut(&edge.from).unwrap().push(edge.to.clone());
            *in_degree.get_mut(&edge.to).unwrap() += 1;
        }
        
        // Find all nodes with in-degree 0
        let mut queue: Vec<Hash> = in_degree
            .iter()
            .filter(|(_, &degree)| degree == 0)
            .map(|(hash, _)| hash.clone())
            .collect();
        
        // Sort queue for deterministic ordering
        queue.sort();
        
        let mut result = Vec::new();
        
        while let Some(node) = queue.pop() {
            result.push(node.clone());
            
            // Reduce in-degree for neighbors
            if let Some(neighbors) = adj_list.get(&node) {
                for neighbor in neighbors {
                    let degree = in_degree.get_mut(neighbor).unwrap();
                    *degree -= 1;
                    
                    if *degree == 0 {
                        queue.push(neighbor.clone());
                        queue.sort();
                    }
                }
            }
        }
        
        // If we processed all nodes, return the result
        if result.len() == self.nodes.len() {
            Some(result)
        } else {
            None
        }
    }
    
    /// Serialize the graph to canonical CBOR
    pub fn canonical_serialize(&self) -> Result<Vec<u8>, GraphError> {
        // Create a deterministic representation
        let mut sorted_nodes: BTreeMap<Hash, &GraphNode> = BTreeMap::new();
        for (hash, node) in &self.nodes {
            sorted_nodes.insert(hash.clone(), node);
        }
        
        // Sort edges deterministically
        let mut sorted_edges = self.edges.clone();
        sorted_edges.sort_by(|a, b| {
            match a.from.cmp(&b.from) {
                std::cmp::Ordering::Equal => a.to.cmp(&b.to),
                other => other,
            }
        });
        
        #[derive(Serialize)]
        struct CanonicalGraph<'a> {
            root_hash: &'a Hash,
            nodes: BTreeMap<Hash, &'a GraphNode>,
            edges: Vec<GraphEdge>,
        }
        
        let canonical = CanonicalGraph {
            root_hash: &self.root_hash,
            nodes: sorted_nodes,
            edges: sorted_edges,
        };
        
        let mut buffer = Vec::new();
        ciborium::into_writer(&canonical, &mut buffer)
            .map_err(|e| GraphError::SerializationError(e.to_string()))?;
        
        Ok(buffer)
    }
    
    /// Get nodes in deterministic order (lexicographic by node_id)
    pub fn nodes_sorted_by_id(&self) -> Vec<(&Hash, &GraphNode)> {
        let mut nodes: Vec<_> = self.nodes.iter().collect();
        nodes.sort_by(|a, b| a.1.id.cmp(&b.1.id));
        nodes
    }
    
    /// Get lineage path from root to a node
    pub fn get_lineage(&self, target: &Hash) -> Option<Vec<Hash>> {
        if !self.nodes.contains_key(target) {
            return None;
        }
        
        // BFS from root to target
        let mut queue = vec![self.root_hash.clone()];
        let mut parent: HashMap<Hash, Hash> = HashMap::new();
        let mut visited = HashSet::new();
        
        // Build adjacency list
        let mut adj_list: HashMap<Hash, Vec<Hash>> = HashMap::new();
        for edge in &self.edges {
            adj_list
                .entry(edge.from.clone())
                .or_insert_with(Vec::new)
                .push(edge.to.clone());
        }
        
        visited.insert(self.root_hash.clone());
        
        while let Some(current) = queue.pop() {
            if current == *target {
                // Reconstruct path
                let mut path = vec![current.clone()];
                let mut node = &current;
                
                while let Some(p) = parent.get(node) {
                    path.push(p.clone());
                    node = p;
                }
                
                path.reverse();
                return Some(path);
            }
            
            if let Some(neighbors) = adj_list.get(&current) {
                for neighbor in neighbors {
                    if !visited.contains(neighbor) {
                        visited.insert(neighbor.clone());
                        parent.insert(neighbor.clone(), current.clone());
                        queue.insert(0, neighbor.clone()); // BFS
                    }
                }
            }
        }
        
        None
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

pub fn create_root_node() -> GraphNode {
    let metadata = NodeMetadata {
        timestamp: 0,
        lineage_depth: 0,
        tags: vec!["genesis".to_string(), "root".to_string()],
    };
    
    let data = Expression::Literal(0); // Genesis value
    
    let node = GraphNode {
        id: "⊙₀".to_string(),
        root_ref: String::new(), // Will be set after hash computation
        data,
        metadata,
    };
    
    // Compute hash and create final node
    let temp_hash = compute_root_hash(&node);
    GraphNode {
        root_ref: temp_hash,
        ..node
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    #[test]
    fn test_create_root_node() {
        let root = create_root_node();
        assert_eq!(root.id, "⊙₀");
        assert_eq!(root.metadata.lineage_depth, 0);
        
        let root_hash = compute_root_hash(&root);
        assert_eq!(root.root_ref, root_hash);
    }

    #[test]
    fn test_genesis_graph_creation() {
        let root = create_root_node();
        let graph = GenesisGraph::new(root.clone()).unwrap();
        
        assert_eq!(graph.root_hash(), &root.root_ref);
        assert_eq!(graph.nodes().len(), 1);
        assert!(graph.get_node(graph.root_hash()).is_some());
    }

    #[test]
    fn test_root_ref_enforcement() {
        let root = create_root_node();
        let graph = GenesisGraph::new(root.clone()).unwrap();
        
        // Try to insert node with wrong root_ref
        let mut bad_node = GraphNode {
            id: "node1".to_string(),
            root_ref: "wrong_hash".to_string(),
            data: Expression::Literal(1),
            metadata: NodeMetadata {
                timestamp: current_timestamp(),
                lineage_depth: 1,
                tags: vec![],
            },
        };
        
        let result = graph.clone().insert_node(bad_node.clone());
        assert!(matches!(result, Err(GraphError::RootRefMismatch { .. })));
        
        // Fix root_ref and try again
        bad_node.root_ref = graph.root_hash().clone();
        let mut graph = graph;
        let result = graph.insert_node(bad_node);
        assert!(result.is_ok());
    }

    #[test]
    fn test_insert_node() {
        let root = create_root_node();
        let mut graph = GenesisGraph::new(root.clone()).unwrap();
        
        let node = GraphNode {
            id: "node1".to_string(),
            root_ref: graph.root_hash().clone(),
            data: Expression::Var("x".to_string()),
            metadata: NodeMetadata {
                timestamp: current_timestamp(),
                lineage_depth: 1,
                tags: vec!["test".to_string()],
            },
        };
        
        let hash = graph.insert_node(node.clone()).unwrap();
        assert_eq!(graph.nodes().len(), 2);
        assert!(graph.get_node(&hash).is_some());
    }

    #[test]
    fn test_duplicate_node_rejection() {
        let root = create_root_node();
        let mut graph = GenesisGraph::new(root.clone()).unwrap();
        
        let node = GraphNode {
            id: "node1".to_string(),
            root_ref: graph.root_hash().clone(),
            data: Expression::Literal(42),
            metadata: NodeMetadata {
                timestamp: current_timestamp(),
                lineage_depth: 1,
                tags: vec![],
            },
        };
        
        graph.insert_node(node.clone()).unwrap();
        let result = graph.insert_node(node);
        assert!(matches!(result, Err(GraphError::NodeAlreadyExists(_))));
    }

    #[test]
    fn test_link_nodes() {
        let root = create_root_node();
        let mut graph = GenesisGraph::new(root.clone()).unwrap();
        
        let node1 = GraphNode {
            id: "node1".to_string(),
            root_ref: graph.root_hash().clone(),
            data: Expression::Literal(1),
            metadata: NodeMetadata {
                timestamp: current_timestamp(),
                lineage_depth: 1,
                tags: vec![],
            },
        };
        
        let node2 = GraphNode {
            id: "node2".to_string(),
            root_ref: graph.root_hash().clone(),
            data: Expression::Literal(2),
            metadata: NodeMetadata {
                timestamp: current_timestamp(),
                lineage_depth: 1,
                tags: vec![],
            },
        };
        
        let hash1 = graph.insert_node(node1).unwrap();
        let hash2 = graph.insert_node(node2).unwrap();
        
        graph
            .link_nodes(hash1.clone(), hash2.clone(), EdgeType::Dependency)
            .unwrap();
        
        assert_eq!(graph.edges().len(), 1);
        assert_eq!(graph.edges()[0].from, hash1);
        assert_eq!(graph.edges()[0].to, hash2);
    }

    #[test]
    fn test_self_loop_forbidden() {
        let root = create_root_node();
        let mut graph = GenesisGraph::new(root.clone()).unwrap();
        
        let node = GraphNode {
            id: "node1".to_string(),
            root_ref: graph.root_hash().clone(),
            data: Expression::Literal(1),
            metadata: NodeMetadata {
                timestamp: current_timestamp(),
                lineage_depth: 1,
                tags: vec![],
            },
        };
        
        let hash = graph.insert_node(node).unwrap();
        let result = graph.link_nodes(hash.clone(), hash.clone(), EdgeType::Dependency);
        
        assert!(matches!(result, Err(GraphError::SelfLoopForbidden(_))));
    }

    #[test]
    fn test_cycle_detection() {
        let root = create_root_node();
        let mut graph = GenesisGraph::new(root.clone()).unwrap();
        
        let node1 = GraphNode {
            id: "node1".to_string(),
            root_ref: graph.root_hash().clone(),
            data: Expression::Literal(1),
            metadata: NodeMetadata {
                timestamp: current_timestamp(),
                lineage_depth: 1,
                tags: vec![],
            },
        };
        
        let node2 = GraphNode {
            id: "node2".to_string(),
            root_ref: graph.root_hash().clone(),
            data: Expression::Literal(2),
            metadata: NodeMetadata {
                timestamp: current_timestamp(),
                lineage_depth: 1,
                tags: vec![],
            },
        };
        
        let node3 = GraphNode {
            id: "node3".to_string(),
            root_ref: graph.root_hash().clone(),
            data: Expression::Literal(3),
            metadata: NodeMetadata {
                timestamp: current_timestamp(),
                lineage_depth: 1,
                tags: vec![],
            },
        };
        
        let hash1 = graph.insert_node(node1).unwrap();
        let hash2 = graph.insert_node(node2).unwrap();
        let hash3 = graph.insert_node(node3).unwrap();
        
        // Create edges: 1 -> 2 -> 3
        graph.link_nodes(hash1.clone(), hash2.clone(), EdgeType::Dependency).unwrap();
        graph.link_nodes(hash2.clone(), hash3.clone(), EdgeType::Dependency).unwrap();
        
        // Try to create cycle: 3 -> 1
        let result = graph.link_nodes(hash3, hash1, EdgeType::Dependency);
        assert!(matches!(result, Err(GraphError::CycleDetected)));
    }

    #[test]
    fn test_delete_node() {
        let root = create_root_node();
        let mut graph = GenesisGraph::new(root.clone()).unwrap();
        
        let node = GraphNode {
            id: "node1".to_string(),
            root_ref: graph.root_hash().clone(),
            data: Expression::Literal(1),
            metadata: NodeMetadata {
                timestamp: current_timestamp(),
                lineage_depth: 1,
                tags: vec![],
            },
        };
        
        let hash = graph.insert_node(node.clone()).unwrap();
        assert_eq!(graph.nodes().len(), 2);
        
        let deleted = graph.delete_node(&hash).unwrap();
        assert_eq!(deleted.id, node.id);
        assert_eq!(graph.nodes().len(), 1);
    }

    #[test]
    fn test_cannot_delete_root() {
        let root = create_root_node();
        let mut graph = GenesisGraph::new(root.clone()).unwrap();
        
        let result = graph.delete_node(graph.root_hash());
        assert!(matches!(result, Err(GraphError::InvalidRootHash)));
    }

    #[test]
    fn test_delete_node_removes_edges() {
        let root = create_root_node();
        let mut graph = GenesisGraph::new(root.clone()).unwrap();
        
        let node1 = GraphNode {
            id: "node1".to_string(),
            root_ref: graph.root_hash().clone(),
            data: Expression::Literal(1),
            metadata: NodeMetadata {
                timestamp: current_timestamp(),
                lineage_depth: 1,
                tags: vec![],
            },
        };
        
        let node2 = GraphNode {
            id: "node2".to_string(),
            root_ref: graph.root_hash().clone(),
            data: Expression::Literal(2),
            metadata: NodeMetadata {
                timestamp: current_timestamp(),
                lineage_depth: 1,
                tags: vec![],
            },
        };
        
        let hash1 = graph.insert_node(node1).unwrap();
        let hash2 = graph.insert_node(node2).unwrap();
        
        graph.link_nodes(hash1.clone(), hash2.clone(), EdgeType::Dependency).unwrap();
        assert_eq!(graph.edges().len(), 1);
        
        graph.delete_node(&hash1).unwrap();
        assert_eq!(graph.edges().len(), 0);
    }

    #[test]
    fn test_canonical_serialization() {
        let root = create_root_node();
        let mut graph = GenesisGraph::new(root.clone()).unwrap();
        
        let node = GraphNode {
            id: "node1".to_string(),
            root_ref: graph.root_hash().clone(),
            data: Expression::Literal(42),
            metadata: NodeMetadata {
                timestamp: 1234567890,
                lineage_depth: 1,
                tags: vec!["test".to_string()],
            },
        };
        
        graph.insert_node(node).unwrap();
        
        let serialized1 = graph.canonical_serialize().unwrap();
        let serialized2 = graph.canonical_serialize().unwrap();
        
        assert_eq!(serialized1, serialized2, "Serialization must be deterministic");
    }

    #[test]
    fn test_serialization_stability_across_insertions() {
        let root = create_root_node();
        
        // Create two graphs and insert nodes in different order
        let mut graph1 = GenesisGraph::new(root.clone()).unwrap();
        let mut graph2 = GenesisGraph::new(root.clone()).unwrap();
        
        let node_a = GraphNode {
            id: "node_a".to_string(),
            root_ref: graph1.root_hash().clone(),
            data: Expression::Literal(1),
            metadata: NodeMetadata {
                timestamp: 1000,
                lineage_depth: 1,
                tags: vec![],
            },
        };
        
        let node_b = GraphNode {
            id: "node_b".to_string(),
            root_ref: graph1.root_hash().clone(),
            data: Expression::Literal(2),
            metadata: NodeMetadata {
                timestamp: 2000,
                lineage_depth: 1,
                tags: vec![],
            },
        };
        
        // Insert in order A, B
        graph1.insert_node(node_a.clone()).unwrap();
        graph1.insert_node(node_b.clone()).unwrap();
        
        // Insert in order B, A
        graph2.insert_node(node_b).unwrap();
        graph2.insert_node(node_a).unwrap();
        
        let s1 = graph1.canonical_serialize().unwrap();
        let s2 = graph2.canonical_serialize().unwrap();
        
        assert_eq!(s1, s2, "Insertion order should not affect serialization");
    }

    #[test]
    fn test_topological_sort() {
        let root = create_root_node();
        let mut graph = GenesisGraph::new(root.clone()).unwrap();
        
        let node1 = GraphNode {
            id: "node1".to_string(),
            root_ref: graph.root_hash().clone(),
            data: Expression::Literal(1),
            metadata: NodeMetadata {
                timestamp: current_timestamp(),
                lineage_depth: 1,
                tags: vec![],
            },
        };
        
        let node2 = GraphNode {
            id: "node2".to_string(),
            root_ref: graph.root_hash().clone(),
            data: Expression::Literal(2),
            metadata: NodeMetadata {
                timestamp: current_timestamp(),
                lineage_depth: 1,
                tags: vec![],
            },
        };
        
        let hash1 = graph.insert_node(node1).unwrap();
        let hash2 = graph.insert_node(node2).unwrap();
        
        graph.link_nodes(graph.root_hash().clone(), hash1.clone(), EdgeType::Dependency).unwrap();
        graph.link_nodes(hash1, hash2, EdgeType::Dependency).unwrap();
        
        let sorted = graph.topological_sort().unwrap();
        assert_eq!(sorted.len(), 3);
        
        // Root should come first
        assert_eq!(sorted[0], *graph.root_hash());
    }

    #[test]
    fn test_topological_sort_with_cycle() {
        let root = create_root_node();
        let mut graph = GenesisGraph::new(root.clone()).unwrap();
        
        let node1 = GraphNode {
            id: "node1".to_string(),
            root_ref: graph.root_hash().clone(),
            data: Expression::Literal(1),
            metadata: NodeMetadata {
                timestamp: current_timestamp(),
                lineage_depth: 1,
                tags: vec![],
            },
        };
        
        let node2 = GraphNode {
            id: "node2".to_string(),
            root_ref: graph.root_hash().clone(),
            data: Expression::Literal(2),
            metadata: NodeMetadata {
                timestamp: current_timestamp(),
                lineage_depth: 1,
                tags: vec![],
            },
        };
        
        let hash1 = graph.insert_node(node1).unwrap();
        let hash2 = graph.insert_node(node2).unwrap();
        
        // Create cycle: 1 -> 2, 2 -> 1 (will fail on second edge)
        graph.link_nodes(hash1.clone(), hash2.clone(), EdgeType::Dependency).unwrap();
        let _ = graph.link_nodes(hash2, hash1, EdgeType::Dependency);
        
        // Graph should still be acyclic
        let sorted = graph.topological_sort();
        assert!(sorted.is_some(), "Graph should remain acyclic after rejecting cycle");
    }

    #[test]
    fn test_lineage_path() {
        let root = create_root_node();
        let mut graph = GenesisGraph::new(root.clone()).unwrap();
        
        let node1 = GraphNode {
            id: "node1".to_string(),
            root_ref: graph.root_hash().clone(),
            data: Expression::Literal(1),
            metadata: NodeMetadata {
                timestamp: current_timestamp(),
                lineage_depth: 1,
                tags: vec![],
            },
        };
        
        let node2 = GraphNode {
            id: "node2".to_string(),
            root_ref: graph.root_hash().clone(),
            data: Expression::Literal(2),
            metadata: NodeMetadata {
                timestamp: current_timestamp(),
                lineage_depth: 2,
                tags: vec![],
            },
        };
        
        let hash1 = graph.insert_node(node1).unwrap();
        let hash2 = graph.insert_node(node2).unwrap();
        
        graph.link_nodes(graph.root_hash().clone(), hash1.clone(), EdgeType::Derivation).unwrap();
        graph.link_nodes(hash1.clone(), hash2.clone(), EdgeType::Derivation).unwrap();
        
        let lineage = graph.get_lineage(&hash2).unwrap();
        assert_eq!(lineage.len(), 3);
        assert_eq!(lineage[0], *graph.root_hash());
        assert_eq!(lineage[1], hash1);
        assert_eq!(lineage[2], hash2);
    }

    #[test]
    fn test_nodes_sorted_by_id() {
        let root = create_root_node();
        let mut graph = GenesisGraph::new(root.clone()).unwrap();
        
        let node_c = GraphNode {
            id: "c_node".to_string(),
            root_ref: graph.root_hash().clone(),
            data: Expression::Literal(3),
            metadata: NodeMetadata {
                timestamp: current_timestamp(),
                lineage_depth: 1,
                tags: vec![],
            },
        };
        
        let node_a = GraphNode {
            id: "a_node".to_string(),
            root_ref: graph.root_hash().clone(),
            data: Expression::Literal(1),
            metadata: NodeMetadata {
                timestamp: current_timestamp(),
                lineage_depth: 1,
                tags: vec![],
            },
        };
        
        let node_b = GraphNode {
            id: "b_node".to_string(),
            root_ref: graph.root_hash().clone(),
            data: Expression::Literal(2),
            metadata: NodeMetadata {
                timestamp: current_timestamp(),
                lineage_depth: 1,
                tags: vec![],
            },
        };
        
        graph.insert_node(node_c).unwrap();
        graph.insert_node(node_a).unwrap();
        graph.insert_node(node_b).unwrap();
        
        let sorted = graph.nodes_sorted_by_id();
        assert_eq!(sorted[0].1.id, "a_node");
        assert_eq!(sorted[1].1.id, "b_node");
        assert_eq!(sorted[2].1.id, "c_node");
        assert_eq!(sorted[3].1.id, "⊙₀");
    }

    #[test]
    fn test_comprehensive() {
        println!("\n=== Comprehensive GenesisGraph Test ===");
        
        let root = create_root_node();
        let mut graph = GenesisGraph::new(root.clone()).unwrap();
        
        println!("✓ Created genesis graph with root node");
        println!("  Root ID: {}", root.id);
        println!("  Root hash: {}", graph.root_hash());
        
        // Create a chain of nodes
        let mut hashes = Vec::new();
        for i in 1..=10 {
            let node = GraphNode {
                id: format!("node_{}", i),
                root_ref: graph.root_hash().clone(),
                data: Expression::Let {
                    name: format!("x{}", i),
                    value: Box::new(Expression::Literal(i)),
                    body: Box::new(Expression::Var(format!("x{}", i))),
                },
                metadata: NodeMetadata {
                    timestamp: current_timestamp() + i as u64,
                    lineage_depth: (i / 3) as u32,
                    tags: vec![format!("level_{}", i / 3)],
                },
            };
            
            let hash = graph.insert_node(node).unwrap();
            
            // Link to previous node
            if i > 1 {
                graph.link_nodes(
                    hashes[i - 2].clone(),
                    hash.clone(),
                    EdgeType::Derivation,
                ).unwrap();
            } else {
                graph.link_nodes(
                    graph.root_hash().clone(),
                    hash.clone(),
                    EdgeType::Derivation,
                ).unwrap();
            }
            
            hashes.push(hash);
        }
        
        println!("✓ Added 10 nodes in chain structure");
        println!("  Total nodes: {}", graph.nodes().len());
        println!("  Total edges: {}", graph.edges().len());
        
        // Test topological sort
        let sorted = graph.topological_sort().unwrap();
        assert_eq!(sorted.len(), graph.nodes().len());
        println!("✓ Topological sort successful");
        
        // Test lineage
        let last_hash = &hashes[hashes.len() - 1];
        let lineage = graph.get_lineage(last_hash).unwrap();
        println!("✓ Lineage path length: {}", lineage.len());
        
        // Test serialization
        let serialized1 = graph.canonical_serialize().unwrap();
        let serialized2 = graph.canonical_serialize().unwrap();
        assert_eq!(serialized1, serialized2);
        println!("✓ Canonical serialization is stable");
        
        // Test deletion
        let node_to_delete = &hashes[5];
        graph.delete_node(node_to_delete).unwrap();
        println!("✓ Node deletion successful");
        println!("  Remaining nodes: {}", graph.nodes().len());
        
        // Verify graph is still valid
        assert!(!graph.has_cycle());
        println!("✓ Graph remains acyclic after deletion");
        
        println!("\n=== All comprehensive tests passed ===");
    }
}
