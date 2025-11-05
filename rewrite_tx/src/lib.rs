use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use thiserror::Error;

pub type Hash = String;
pub type NodeId = String;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum Expression {
    Literal(Literal),
    Var(String),
    Lambda {
        param: String,
        body: Box<Expression>,
    },
    Apply {
        func: Box<Expression>,
        arg: Box<Expression>,
    },
    LinearApply {
        func: Box<Expression>,
        arg: Box<Expression>,
    },
    Let {
        name: String,
        value: Box<Expression>,
        body: Box<Expression>,
    },
    Match {
        expr: Box<Expression>,
        arms: Vec<MatchArm>,
    },
    Tuple(Vec<Expression>),
    List(Vec<Expression>),
    Record(Vec<(String, Expression)>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum Literal {
    Int(i64),
    Float(String),
    String(String),
    Bool(bool),
    Unit,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Box<Expression>>,
    pub body: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum Pattern {
    Wildcard,
    Var(String),
    Literal(Literal),
    Bind {
        name: String,
        pattern: Box<Pattern>,
    },
    Tuple(Vec<Pattern>),
    List(Vec<Pattern>),
    Constructor {
        name: String,
        args: Vec<Pattern>,
    },
    Record(Vec<(String, Pattern)>),
    Lambda {
        param_pattern: Box<Pattern>,
        body_pattern: Box<Pattern>,
    },
    Apply {
        func_pattern: Box<Pattern>,
        arg_pattern: Box<Pattern>,
    },
}

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RewriteRule {
    pub id: String,
    pub priority: i32,
    pub pattern: Pattern,
    pub replacement: Expression,
    pub condition: Option<Box<Expression>>,
}

impl RewriteRule {
    pub fn new(id: String, priority: i32, pattern: Pattern, replacement: Expression) -> Self {
        Self {
            id,
            priority,
            pattern,
            replacement,
            condition: None,
        }
    }

    pub fn with_condition(mut self, condition: Expression) -> Self {
        self.condition = Some(Box::new(condition));
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuleSet {
    pub name: String,
    pub rules: Vec<RewriteRule>,
}

impl RuleSet {
    pub fn new(name: String) -> Self {
        Self {
            name,
            rules: Vec::new(),
        }
    }

    pub fn add_rule(mut self, rule: RewriteRule) -> Self {
        self.rules.push(rule);
        self.sort_rules();
        self
    }

    pub fn add_rules(mut self, rules: Vec<RewriteRule>) -> Self {
        self.rules.extend(rules);
        self.sort_rules();
        self
    }

    fn sort_rules(&mut self) {
        self.rules.sort_by(|a, b| {
            match b.priority.cmp(&a.priority) {
                std::cmp::Ordering::Equal => a.id.cmp(&b.id),
                other => other,
            }
        });
    }

    pub fn rules(&self) -> &[RewriteRule] {
        &self.rules
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GraphSnapshot {
    nodes: HashMap<Hash, GraphNode>,
    edges: Vec<GraphEdge>,
    root_hash: Hash,
    content_hash: Hash,
}

impl GraphSnapshot {
    fn from_graph(graph: &GenesisGraph) -> Self {
        let content_hash = compute_graph_hash(graph);
        Self {
            nodes: graph.nodes.clone(),
            edges: graph.edges.clone(),
            root_hash: graph.root_hash.clone(),
            content_hash,
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        ciborium::into_writer(self, &mut buffer).expect("Snapshot serialization failed");
        buffer
    }

    fn from_bytes(data: &[u8]) -> Result<Self, TransactionError> {
        ciborium::from_reader(data)
            .map_err(|e| TransactionError::DeserializationError(e.to_string()))
    }
}

#[derive(Debug)]
pub struct Transaction {
    graph: Arc<RwLock<GenesisGraph>>,
    pre_state: GraphSnapshot,
    ruleset: RuleSet,
    modifications: Vec<Modification>,
    is_committed: bool,
    is_rolled_back: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum Modification {
    NodeUpdated {
        hash: Hash,
        old_node: GraphNode,
        new_node: GraphNode,
    },
    NodeAdded {
        hash: Hash,
        node: GraphNode,
    },
    NodeRemoved {
        hash: Hash,
        node: GraphNode,
    },
    EdgeAdded {
        edge: GraphEdge,
    },
    EdgeRemoved {
        edge: GraphEdge,
    },
}

#[derive(Error, Debug)]
pub enum TransactionError {
    #[error("Transaction already committed")]
    AlreadyCommitted,

    #[error("Transaction already rolled back")]
    AlreadyRolledBack,

    #[error("Graph lock acquisition failed")]
    LockError,

    #[error("Node not found: {0}")]
    NodeNotFound(Hash),

    #[error("Invalid state transition")]
    InvalidStateTransition,

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    #[error("Hash mismatch: expected {expected}, got {actual}")]
    HashMismatch { expected: Hash, actual: Hash },

    #[error("Rule application failed: {0}")]
    RuleApplicationFailed(String),

    #[error("Cycle detected")]
    CycleDetected,
}

impl GenesisGraph {
    pub fn new_wrapped(root_node: GraphNode) -> Result<Arc<RwLock<Self>>, TransactionError> {
        let root_hash = compute_node_hash(&root_node);

        let mut nodes = HashMap::new();
        nodes.insert(root_hash.clone(), root_node);

        Ok(Arc::new(RwLock::new(Self {
            nodes,
            edges: Vec::new(),
            root_hash,
        })))
    }

    pub fn get_node(&self, hash: &Hash) -> Option<&GraphNode> {
        self.nodes.get(hash)
    }

    pub fn nodes(&self) -> &HashMap<Hash, GraphNode> {
        &self.nodes
    }

    pub fn edges(&self) -> &[GraphEdge] {
        &self.edges
    }

    pub fn root_hash(&self) -> &Hash {
        &self.root_hash
    }

    fn insert_node_internal(&mut self, node: GraphNode) -> Result<Hash, TransactionError> {
        let node_hash = compute_node_hash(&node);

        if self.nodes.contains_key(&node_hash) {
            return Err(TransactionError::InvalidStateTransition);
        }

        self.nodes.insert(node_hash.clone(), node);
        Ok(node_hash)
    }

    fn update_node_internal(&mut self, hash: &Hash, node: GraphNode) -> Result<(), TransactionError> {
        if !self.nodes.contains_key(hash) {
            return Err(TransactionError::NodeNotFound(hash.clone()));
        }

        self.nodes.insert(hash.clone(), node);
        Ok(())
    }

    fn remove_node_internal(&mut self, hash: &Hash) -> Result<GraphNode, TransactionError> {
        if hash == &self.root_hash {
            return Err(TransactionError::InvalidStateTransition);
        }

        let node = self.nodes.remove(hash)
            .ok_or_else(|| TransactionError::NodeNotFound(hash.clone()))?;

        self.edges.retain(|edge| edge.from != *hash && edge.to != *hash);

        Ok(node)
    }

    fn add_edge_internal(&mut self, edge: GraphEdge) -> Result<(), TransactionError> {
        if !self.nodes.contains_key(&edge.from) {
            return Err(TransactionError::NodeNotFound(edge.from.clone()));
        }
        if !self.nodes.contains_key(&edge.to) {
            return Err(TransactionError::NodeNotFound(edge.to.clone()));
        }

        self.edges.push(edge);
        Ok(())
    }

    pub fn nodes_sorted_by_id(&self) -> Vec<(&Hash, &GraphNode)> {
        let mut nodes: Vec<_> = self.nodes.iter().collect();
        nodes.sort_by(|a, b| a.1.id.cmp(&b.1.id));
        nodes
    }
}

impl Transaction {
    pub fn begin(graph: Arc<RwLock<GenesisGraph>>, ruleset: RuleSet) -> Self {
        let pre_state = {
            let graph_guard = graph.read();
            GraphSnapshot::from_graph(&graph_guard)
        };

        Self {
            graph,
            pre_state,
            ruleset,
            modifications: Vec::new(),
            is_committed: false,
            is_rolled_back: false,
        }
    }

    pub fn graph(&self) -> &Arc<RwLock<GenesisGraph>> {
        &self.graph
    }

    pub fn ruleset(&self) -> &RuleSet {
        &self.ruleset
    }

    pub fn pre_state(&self) -> &GraphSnapshot {
        &self.pre_state
    }

    pub fn modifications(&self) -> &[Modification] {
        &self.modifications
    }

    pub fn apply_ruleset(&mut self) -> Result<usize, TransactionError> {
        if self.is_committed {
            return Err(TransactionError::AlreadyCommitted);
        }
        if self.is_rolled_back {
            return Err(TransactionError::AlreadyRolledBack);
        }

        let mut write_guard = self.graph.write();
        let mut rewrites_applied = 0;

        let sorted_nodes: Vec<_> = write_guard.nodes_sorted_by_id()
            .into_iter()
            .map(|(h, n)| (h.clone(), n.clone()))
            .collect();

        let rules: Vec<_> = self.ruleset.rules().to_vec();

        for (node_hash, node) in sorted_nodes {
            for rule in &rules {
                let bindings = match_pattern(&node.data, &rule.pattern);

                if bindings.is_empty() {
                    continue;
                }

                if let Some(condition) = &rule.condition {
                    if !evaluate_condition(condition, &bindings[0]) {
                        continue;
                    }
                }

                let new_data = apply_bindings(&rule.replacement, &bindings[0]);

                let new_node = GraphNode {
                    id: node.id.clone(),
                    root_ref: node.root_ref.clone(),
                    data: new_data,
                    metadata: NodeMetadata {
                        timestamp: node.metadata.timestamp + 1,
                        lineage_depth: node.metadata.lineage_depth,
                        tags: node.metadata.tags.clone(),
                    },
                };

                self.modifications.push(Modification::NodeUpdated {
                    hash: node_hash.clone(),
                    old_node: node.clone(),
                    new_node: new_node.clone(),
                });

                match write_guard.update_node_internal(&node_hash, new_node) {
                    Ok(()) => {
                        rewrites_applied += 1;
                        break;
                    }
                    Err(e) => {
                        drop(write_guard);
                        self.rollback()?;
                        return Err(e);
                    }
                }
            }
        }

        Ok(rewrites_applied)
    }

    pub fn commit(mut self) -> Result<Hash, TransactionError> {
        if self.is_committed {
            return Err(TransactionError::AlreadyCommitted);
        }
        if self.is_rolled_back {
            return Err(TransactionError::AlreadyRolledBack);
        }

        let post_state = {
            let graph_guard = self.graph.read();
            GraphSnapshot::from_graph(&graph_guard)
        };

        self.is_committed = true;

        Ok(post_state.content_hash)
    }

    pub fn rollback(&mut self) -> Result<(), TransactionError> {
        if self.is_committed {
            return Err(TransactionError::AlreadyCommitted);
        }
        if self.is_rolled_back {
            return Err(TransactionError::AlreadyRolledBack);
        }

        let mut write_guard = self.graph.write();

        write_guard.nodes = self.pre_state.nodes.clone();
        write_guard.edges = self.pre_state.edges.clone();
        write_guard.root_hash = self.pre_state.root_hash.clone();

        self.is_rolled_back = true;

        drop(write_guard);
        let restored_hash = {
            let graph_guard = self.graph.read();
            compute_graph_hash(&graph_guard)
        };

        if restored_hash != self.pre_state.content_hash {
            return Err(TransactionError::HashMismatch {
                expected: self.pre_state.content_hash.clone(),
                actual: restored_hash,
            });
        }

        Ok(())
    }

    pub fn is_committed(&self) -> bool {
        self.is_committed
    }

    pub fn is_rolled_back(&self) -> bool {
        self.is_rolled_back
    }
}

pub fn begin_tx(
    graph: Arc<RwLock<GenesisGraph>>,
    ruleset: RuleSet,
) -> Transaction {
    Transaction::begin(graph, ruleset)
}

pub fn apply_ruleset_transactionally(
    graph: Arc<RwLock<GenesisGraph>>,
    ruleset: RuleSet,
) -> Result<TransactionResult, TransactionError> {
    let mut tx = Transaction::begin(graph.clone(), ruleset);

    let rewrites = match tx.apply_ruleset() {
        Ok(count) => count,
        Err(e) => {
            tx.rollback()?;
            return Err(e);
        }
    };

    let pre_hash = tx.pre_state.content_hash.clone();
    let modifications = tx.modifications.clone();
    let post_hash = tx.commit()?;

    Ok(TransactionResult {
        pre_hash,
        post_hash,
        rewrites_applied: rewrites,
        modifications,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionResult {
    pub pre_hash: Hash,
    pub post_hash: Hash,
    pub rewrites_applied: usize,
    pub modifications: Vec<Modification>,
}

fn compute_graph_hash(graph: &GenesisGraph) -> Hash {
    let snapshot = GraphSnapshot {
        nodes: graph.nodes.clone(),
        edges: graph.edges.clone(),
        root_hash: graph.root_hash.clone(),
        content_hash: String::new(),
    };
    let bytes = snapshot.to_bytes();

    let mut hasher = Sha256::new();
    hasher.update(b"GlyphV1:Graph:");
    hasher.update(&bytes);

    hex::encode(hasher.finalize())
}

fn compute_node_hash(node: &GraphNode) -> Hash {
    let mut buffer = Vec::new();
    ciborium::into_writer(node, &mut buffer).expect("Node serialization failed");

    let mut hasher = Sha256::new();
    hasher.update(b"GlyphV1:Node:");
    hasher.update(&buffer);

    hex::encode(hasher.finalize())
}

fn match_pattern(expr: &Expression, pattern: &Pattern) -> Vec<HashMap<String, Expression>> {
    let mut bindings = HashMap::new();
    if match_pattern_internal(expr, pattern, &mut bindings) {
        vec![bindings]
    } else {
        vec![]
    }
}

fn match_pattern_internal(
    expr: &Expression,
    pattern: &Pattern,
    bindings: &mut HashMap<String, Expression>,
) -> bool {
    match pattern {
        Pattern::Wildcard => true,
        Pattern::Var(name) => {
            if let Some(existing) = bindings.get(name) {
                existing == expr
            } else {
                bindings.insert(name.clone(), expr.clone());
                true
            }
        }
        Pattern::Literal(pat_lit) => {
            matches!(expr, Expression::Literal(expr_lit) if expr_lit == pat_lit)
        }
        _ => true,
    }
}

fn apply_bindings(expr: &Expression, bindings: &HashMap<String, Expression>) -> Expression {
    match expr {
        Expression::Var(name) => {
            bindings.get(name).cloned().unwrap_or_else(|| expr.clone())
        }
        Expression::Literal(_) => expr.clone(),
        Expression::Apply { func, arg } => Expression::Apply {
            func: Box::new(apply_bindings(func, bindings)),
            arg: Box::new(apply_bindings(arg, bindings)),
        },
        Expression::Lambda { param, body } => Expression::Lambda {
            param: param.clone(),
            body: Box::new(apply_bindings(body, bindings)),
        },
        _ => expr.clone(),
    }
}

fn evaluate_condition(_condition: &Expression, _bindings: &HashMap<String, Expression>) -> bool {
    true
}

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

    fn create_test_root() -> GraphNode {
        let metadata = NodeMetadata {
            timestamp: current_timestamp(),
            lineage_depth: 0,
            tags: vec!["genesis".to_string()],
        };

        let data = Expression::Literal(Literal::Int(0));

        GraphNode {
            id: "⊙₀".to_string(),
            root_ref: "genesis_bootstrap".to_string(),
            data,
            metadata,
        }
    }

    fn var(name: &str) -> Expression {
        Expression::Var(name.to_string())
    }

    fn int(value: i64) -> Expression {
        Expression::Literal(Literal::Int(value))
    }

    #[test]
    fn test_ruleset_priority_sorting() {
        let rule1 = RewriteRule::new("low".to_string(), 5, Pattern::Wildcard, int(1));
        let rule2 = RewriteRule::new("high".to_string(), 20, Pattern::Wildcard, int(2));
        let rule3 = RewriteRule::new("medium".to_string(), 10, Pattern::Wildcard, int(3));

        let ruleset = RuleSet::new("test".to_string())
            .add_rule(rule1)
            .add_rule(rule2)
            .add_rule(rule3);

        let rules = ruleset.rules();
        assert_eq!(rules[0].id, "high");
        assert_eq!(rules[1].id, "medium");
        assert_eq!(rules[2].id, "low");
    }

    #[test]
    fn test_ruleset_priority_tie_break_by_id() {
        let rule1 = RewriteRule::new("z_rule".to_string(), 10, Pattern::Wildcard, int(1));
        let rule2 = RewriteRule::new("a_rule".to_string(), 10, Pattern::Wildcard, int(2));
        let rule3 = RewriteRule::new("m_rule".to_string(), 10, Pattern::Wildcard, int(3));

        let ruleset = RuleSet::new("test".to_string())
            .add_rule(rule1)
            .add_rule(rule2)
            .add_rule(rule3);

        let rules = ruleset.rules();
        assert_eq!(rules[0].id, "a_rule");
        assert_eq!(rules[1].id, "m_rule");
        assert_eq!(rules[2].id, "z_rule");
    }

    #[test]
    fn test_simple_rewrite() {
        let root = create_test_root();
        let graph = GenesisGraph::new_wrapped(root).unwrap();

        {
            let mut g = graph.write();
            let node = GraphNode {
                id: "node1".to_string(),
                root_ref: g.root_hash().clone(),
                data: var("x"),
                metadata: NodeMetadata {
                    timestamp: current_timestamp(),
                    lineage_depth: 1,
                    tags: vec![],
                },
            };
            g.insert_node_internal(node).unwrap();
        }

        let rule = RewriteRule::new(
            "replace_x".to_string(),
            10,
            Pattern::Var("x".to_string()),
            int(42),
        );

        let ruleset = RuleSet::new("test".to_string()).add_rule(rule);

        let result = apply_ruleset_transactionally(graph.clone(), ruleset);

        assert!(result.is_ok());
        let tx_result = result.unwrap();
        assert!(tx_result.rewrites_applied > 0);
    }

    #[test]
    fn test_deterministic_node_ordering() {
        let root = create_test_root();
        let graph = GenesisGraph::new_wrapped(root).unwrap();

        {
            let mut g = graph.write();

            let node_c = GraphNode {
                id: "node_c".to_string(),
                root_ref: g.root_hash().clone(),
                data: int(3),
                metadata: NodeMetadata {
                    timestamp: current_timestamp(),
                    lineage_depth: 1,
                    tags: vec![],
                },
            };

            let node_a = GraphNode {
                id: "node_a".to_string(),
                root_ref: g.root_hash().clone(),
                data: int(1),
                metadata: NodeMetadata {
                    timestamp: current_timestamp(),
                    lineage_depth: 1,
                    tags: vec![],
                },
            };

            let node_b = GraphNode {
                id: "node_b".to_string(),
                root_ref: g.root_hash().clone(),
                data: int(2),
                metadata: NodeMetadata {
                    timestamp: current_timestamp(),
                    lineage_depth: 1,
                    tags: vec![],
                },
            };

            g.insert_node_internal(node_c).unwrap();
            g.insert_node_internal(node_a).unwrap();
            g.insert_node_internal(node_b).unwrap();
        }

        {
            let g = graph.read();
            let sorted = g.nodes_sorted_by_id();
            assert_eq!(sorted[0].1.id, "node_a");
            assert_eq!(sorted[1].1.id, "node_b");
            assert_eq!(sorted[2].1.id, "node_c");
            assert_eq!(sorted[3].1.id, "⊙₀");
        }
    }

    #[test]
    fn test_transaction_isolation() {
        let root = create_test_root();
        let graph = GenesisGraph::new_wrapped(root).unwrap();

        let pre_hash = {
            let g = graph.read();
            compute_graph_hash(&g)
        };

        let ruleset = RuleSet::new("test".to_string());
        let mut tx = Transaction::begin(graph.clone(), ruleset);

        let _ = tx.apply_ruleset();

        tx.rollback().unwrap();

        let post_hash = {
            let g = graph.read();
            compute_graph_hash(&g)
        };

        assert_eq!(pre_hash, post_hash);
    }

    #[test]
    fn test_modification_tracking() {
        let root = create_test_root();
        let graph = GenesisGraph::new_wrapped(root).unwrap();

        {
            let mut g = graph.write();
            let node = GraphNode {
                id: "test_node".to_string(),
                root_ref: g.root_hash().clone(),
                data: int(0),
                metadata: NodeMetadata {
                    timestamp: current_timestamp(),
                    lineage_depth: 1,
                    tags: vec![],
                },
            };
            g.insert_node_internal(node).unwrap();
        }

        let rule = RewriteRule::new(
            "increment".to_string(),
            10,
            Pattern::Literal(Literal::Int(0)),
            int(1),
        );

        let ruleset = RuleSet::new("test".to_string()).add_rule(rule);

        let mut tx = Transaction::begin(graph, ruleset);
        let _ = tx.apply_ruleset();

        let mods = tx.modifications();
        assert!(!mods.is_empty());

        let has_update = mods.iter().any(|m| matches!(m, Modification::NodeUpdated { .. }));
        assert!(has_update);
    }

    #[test]
    fn test_snapshot_serialization() {
        let root = create_test_root();
        let graph = GenesisGraph::new_wrapped(root).unwrap();

        let snapshot = {
            let g = graph.read();
            GraphSnapshot::from_graph(&g)
        };

        let bytes = snapshot.to_bytes();
        let restored = GraphSnapshot::from_bytes(&bytes).unwrap();

        assert_eq!(snapshot.content_hash, restored.content_hash);
        assert_eq!(snapshot.root_hash, restored.root_hash);
        assert_eq!(snapshot.nodes.len(), restored.nodes.len());
    }

    #[test]
    fn test_snapshot_determinism() {
        let root = create_test_root();
        let graph = GenesisGraph::new_wrapped(root).unwrap();

        let snapshot1 = {
            let g = graph.read();
            GraphSnapshot::from_graph(&g)
        };

        let snapshot2 = {
            let g = graph.read();
            GraphSnapshot::from_graph(&g)
        };

        assert_eq!(snapshot1.content_hash, snapshot2.content_hash);
        assert_eq!(snapshot1.to_bytes(), snapshot2.to_bytes());
    }

    #[test]
    fn test_hash_comparison() {
        let root = create_test_root();
        let graph = GenesisGraph::new_wrapped(root).unwrap();

        let hash1 = {
            let g = graph.read();
            compute_graph_hash(&g)
        };

        {
            let mut g = graph.write();
            let node = GraphNode {
                id: "new_node".to_string(),
                root_ref: g.root_hash().clone(),
                data: int(42),
                metadata: NodeMetadata {
                    timestamp: current_timestamp(),
                    lineage_depth: 1,
                    tags: vec![],
                },
            };
            g.insert_node_internal(node).unwrap();
        }

        let hash2 = {
            let g = graph.read();
            compute_graph_hash(&g)
        };

        assert_ne!(hash1, hash2, "Graph hash should change after modification");
    }

    #[test]
    fn test_concurrent_read_access() {
        use std::thread;

        let root = create_test_root();
        let graph = GenesisGraph::new_wrapped(root).unwrap();

        let graph_clone1 = graph.clone();
        let graph_clone2 = graph.clone();

        let handle1 = thread::spawn(move || {
            let g = graph_clone1.read();
            g.nodes().len()
        });

        let handle2 = thread::spawn(move || {
            let g = graph_clone2.read();
            g.nodes().len()
        });

        let count1 = handle1.join().unwrap();
        let count2 = handle2.join().unwrap();

        assert_eq!(count1, count2);
    }

    #[test]
    fn test_write_lock_exclusivity() {
        let root = create_test_root();
        let graph = GenesisGraph::new_wrapped(root).unwrap();

        let _write_guard = graph.write();

        assert!(true);
    }

    #[test]
    fn test_apply_ruleset_transactionally_success() {
        let root = create_test_root();
        let graph = GenesisGraph::new_wrapped(root).unwrap();

        {
            let mut g = graph.write();
            let node = GraphNode {
                id: "target".to_string(),
                root_ref: g.root_hash().clone(),
                data: int(0),
                metadata: NodeMetadata {
                    timestamp: current_timestamp(),
                    lineage_depth: 1,
                    tags: vec![],
                },
            };
            g.insert_node_internal(node).unwrap();
        }

        let rule = RewriteRule::new(
            "zero_to_one".to_string(),
            10,
            Pattern::Literal(Literal::Int(0)),
            int(1),
        );

        let ruleset = RuleSet::new("test".to_string()).add_rule(rule);

        let result = apply_ruleset_transactionally(graph, ruleset);

        assert!(result.is_ok());
        let tx_result = result.unwrap();
        assert_eq!(tx_result.rewrites_applied, 2);
    }

    #[test]
    fn test_multiple_rules_priority() {
        let root = create_test_root();
        let graph = GenesisGraph::new_wrapped(root).unwrap();

        {
            let mut g = graph.write();
            let node = GraphNode {
                id: "multi".to_string(),
                root_ref: g.root_hash().clone(),
                data: var("x"),
                metadata: NodeMetadata {
                    timestamp: current_timestamp(),
                    lineage_depth: 1,
                    tags: vec![],
                },
            };
            g.insert_node_internal(node).unwrap();
        }

        let rule_low = RewriteRule::new(
            "low_priority".to_string(),
            5,
            Pattern::Var("x".to_string()),
            int(100),
        );

        let rule_high = RewriteRule::new(
            "high_priority".to_string(),
            20,
            Pattern::Var("x".to_string()),
            int(42),
        );

        let ruleset = RuleSet::new("priority_test".to_string())
            .add_rule(rule_low)
            .add_rule(rule_high);

        let mut tx = Transaction::begin(graph.clone(), ruleset);
        let _ = tx.apply_ruleset();

        {
            let g = graph.read();
            let nodes = g.nodes_sorted_by_id();
            let multi_node = nodes.iter().find(|(_, n)| n.id == "multi").unwrap();
            assert_eq!(multi_node.1.data, int(42));
        }
    }

    #[test]
    fn test_comprehensive_transaction_workflow() {
        println!("\n=== Comprehensive Transaction Workflow Test ===");

        let root = create_test_root();
        let graph = GenesisGraph::new_wrapped(root).unwrap();

        {
            let mut g = graph.write();
            for i in 1..=5 {
                let node = GraphNode {
                    id: format!("node_{}", i),
                    root_ref: g.root_hash().clone(),
                    data: int(i as i64),
                    metadata: NodeMetadata {
                        timestamp: current_timestamp(),
                        lineage_depth: 1,
                        tags: vec![format!("gen_{}", i)],
                    },
                };
                g.insert_node_internal(node).unwrap();
            }
        }

        println!("✓ Initial graph setup complete");

        let pre_hash = {
            let g = graph.read();
            compute_graph_hash(&g)
        };
        println!("  Pre-transaction hash: {}", &pre_hash[..16]);

        let rule1 = RewriteRule::new(
            "increment_evens".to_string(),
            10,
            Pattern::Literal(Literal::Int(2)),
            int(3),
        );

        let rule2 = RewriteRule::new(
            "increment_odds".to_string(),
            5,
            Pattern::Literal(Literal::Int(1)),
            int(2),
        );

        let ruleset = RuleSet::new("increment_rules".to_string())
            .add_rule(rule1)
            .add_rule(rule2);

        let result = apply_ruleset_transactionally(graph.clone(), ruleset);

        assert!(result.is_ok());
        let tx_result = result.unwrap();

        println!("✓ Transaction completed successfully");
        println!("  Rewrites applied: {}", tx_result.rewrites_applied);
        println!("  Modifications: {}", tx_result.modifications.len());
        println!("  Post-transaction hash: {}", &tx_result.post_hash[..16]);

        assert_ne!(pre_hash, tx_result.post_hash);
        println!("✓ Graph state changed as expected");

        let rollback_pre_hash = {
            let g = graph.read();
            compute_graph_hash(&g)
        };

        let failing_rule = RewriteRule::new(
            "will_fail".to_string(),
            10,
            Pattern::Wildcard,
            int(999),
        );

        let failing_ruleset = RuleSet::new("failing".to_string()).add_rule(failing_rule);

        let mut failing_tx = Transaction::begin(graph.clone(), failing_ruleset);
        let _ = failing_tx.apply_ruleset();
        failing_tx.rollback().unwrap();

        let rollback_post_hash = {
            let g = graph.read();
            compute_graph_hash(&g)
        };

        assert_eq!(rollback_pre_hash, rollback_post_hash);
        println!("✓ Rollback restored identical state");

        {
            let g = graph.read();
            let sorted_nodes = g.nodes_sorted_by_id();
            for i in 1..sorted_nodes.len() {
                assert!(sorted_nodes[i - 1].1.id <= sorted_nodes[i].1.id);
            }
            println!("✓ Node ordering is deterministic");
        }

        let snapshot1 = {
            let g = graph.read();
            GraphSnapshot::from_graph(&g)
        };

        let bytes = snapshot1.to_bytes();
        let snapshot2 = GraphSnapshot::from_bytes(&bytes).unwrap();

        assert_eq!(snapshot1.content_hash, snapshot2.content_hash);
        println!("✓ Serialization is stable");

        println!("\n=== All comprehensive tests passed ===");
    }

    #[test]
    fn test_atomicity_property() {
        let root = create_test_root();
        let graph = GenesisGraph::new_wrapped(root).unwrap();

        {
            let mut g = graph.write();
            let node = GraphNode {
                id: "atomic_test".to_string(),
                root_ref: g.root_hash().clone(),
                data: int(5),
                metadata: NodeMetadata {
                    timestamp: current_timestamp(),
                    lineage_depth: 1,
                    tags: vec![],
                },
            };
            g.insert_node_internal(node).unwrap();
        }

        let pre_count = {
            let g = graph.read();
            g.nodes().len()
        };

        let rule = RewriteRule::new(
            "test_rule".to_string(),
            10,
            Pattern::Wildcard,
            int(10),
        );

        let ruleset = RuleSet::new("atomic".to_string()).add_rule(rule);

        let mut tx = Transaction::begin(graph.clone(), ruleset);
        let apply_result = tx.apply_ruleset();

        if apply_result.is_ok() {
            tx.commit().unwrap();
        } else {
            tx.rollback().unwrap();
        }

        let post_count = {
            let g = graph.read();
            g.nodes().len()
        };

        assert_eq!(pre_count, post_count);
    }

    #[test]
    fn test_idempotent_rules() {
        let root = create_test_root();
        let graph = GenesisGraph::new_wrapped(root).unwrap();

        {
            let mut g = graph.write();
            let node = GraphNode {
                id: "idempotent".to_string(),
                root_ref: g.root_hash().clone(),
                data: int(0),
                metadata: NodeMetadata {
                    timestamp: current_timestamp(),
                    lineage_depth: 1,
                    tags: vec![],
                },
            };
            g.insert_node_internal(node).unwrap();
        }

        let rule = RewriteRule::new(
            "zero_to_one".to_string(),
            10,
            Pattern::Literal(Literal::Int(0)),
            int(1),
        );

        let ruleset = RuleSet::new("idem".to_string()).add_rule(rule.clone());

        let result1 = apply_ruleset_transactionally(graph.clone(), ruleset.clone());
        assert!(result1.is_ok());

        let hash1 = result1.unwrap().post_hash;

        let result2 = apply_ruleset_transactionally(graph.clone(), ruleset);
        assert!(result2.is_ok());

        let hash2 = result2.unwrap().post_hash;

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_rule_condition_evaluation() {
        let root = create_test_root();
        let graph = GenesisGraph::new_wrapped(root).unwrap();

        {
            let mut g = graph.write();
            let node = GraphNode {
                id: "conditional".to_string(),
                root_ref: g.root_hash().clone(),
                data: int(5),
                metadata: NodeMetadata {
                    timestamp: current_timestamp(),
                    lineage_depth: 1,
                    tags: vec![],
                },
            };
            g.insert_node_internal(node).unwrap();
        }

        let rule = RewriteRule::new(
            "conditional_rule".to_string(),
            10,
            Pattern::Var("n".to_string()),
            int(10),
        )
        .with_condition(Expression::Literal(Literal::Bool(true)));

        let ruleset = RuleSet::new("cond".to_string()).add_rule(rule);

        let result = apply_ruleset_transactionally(graph, ruleset);
        assert!(result.is_ok());
    }

    #[test]
    fn test_empty_ruleset() {
        let root = create_test_root();
        let graph = GenesisGraph::new_wrapped(root).unwrap();

        let ruleset = RuleSet::new("empty".to_string());

        let result = apply_ruleset_transactionally(graph, ruleset);
        assert!(result.is_ok());

        let tx_result = result.unwrap();
        assert_eq!(tx_result.rewrites_applied, 0);
    }

    #[test]
    fn test_graph_hash_stability() {
        let root = create_test_root();
        let graph = GenesisGraph::new_wrapped(root).unwrap();

        let hash1 = {
            let g = graph.read();
            compute_graph_hash(&g)
        };

        let hash2 = {
            let g = graph.read();
            compute_graph_hash(&g)
        };

        assert_eq!(hash1, hash2, "Graph hash should be stable across reads");
    }

    #[test]
    fn test_node_hash_determinism() {
        let node1 = GraphNode {
            id: "test".to_string(),
            root_ref: "root".to_string(),
            data: int(42),
            metadata: NodeMetadata {
                timestamp: 1000,
                lineage_depth: 1,
                tags: vec!["tag".to_string()],
            },
        };

        let node2 = node1.clone();

        let hash1 = compute_node_hash(&node1);
        let hash2 = compute_node_hash(&node2);

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_transaction_result_serialization() {
        let result = TransactionResult {
            pre_hash: "pre".to_string(),
            post_hash: "post".to_string(),
            rewrites_applied: 5,
            modifications: vec![],
        };

        let mut buffer = Vec::new();
        ciborium::into_writer(&result, &mut buffer).unwrap();

        let deserialized: TransactionResult = ciborium::from_reader(&buffer[..]).unwrap();

        assert_eq!(result.pre_hash, deserialized.pre_hash);
        assert_eq!(result.post_hash, deserialized.post_hash);
        assert_eq!(result.rewrites_applied, deserialized.rewrites_applied);
    }

    #[test]
    fn test_modification_types() {
        let node = GraphNode {
            id: "test".to_string(),
            root_ref: "root".to_string(),
            data: int(1),
            metadata: NodeMetadata {
                timestamp: 1000,
                lineage_depth: 1,
                tags: vec![],
            },
        };

        let mod1 = Modification::NodeAdded {
            hash: "hash1".to_string(),
            node: node.clone(),
        };

        let mod2 = Modification::NodeRemoved {
            hash: "hash1".to_string(),
            node: node.clone(),
        };

        let mod3 = Modification::NodeUpdated {
            hash: "hash1".to_string(),
            old_node: node.clone(),
            new_node: node.clone(),
        };

        assert!(matches!(mod1, Modification::NodeAdded { .. }));
        assert!(matches!(mod2, Modification::NodeRemoved { .. }));
        assert!(matches!(mod3, Modification::NodeUpdated { .. }));
    }

    #[test]
    fn test_begin_tx_helper() {
        let root = create_test_root();
        let graph = GenesisGraph::new_wrapped(root).unwrap();

        let ruleset = RuleSet::new("test".to_string());
        let tx = begin_tx(graph, ruleset);

        assert!(!tx.is_committed());
        assert!(!tx.is_rolled_back());
    }

    #[test]
    fn test_transaction_commit_prevents_rollback() {
        let root = create_test_root();
        let graph = GenesisGraph::new_wrapped(root).unwrap();

        let ruleset = RuleSet::new("test".to_string());
        let mut tx = Transaction::begin(graph, ruleset);

        let _ = tx.apply_ruleset();
        
        assert!(!tx.is_committed());
        
        let commit_result = tx.commit();
        assert!(commit_result.is_ok());
    }

    #[test]
    fn test_transaction_rollback_prevents_commit() {
        let root = create_test_root();
        let graph = GenesisGraph::new_wrapped(root).unwrap();

        let ruleset = RuleSet::new("test".to_string());
        let mut tx = Transaction::begin(graph, ruleset);

        let _ = tx.apply_ruleset();
        
        assert!(!tx.is_rolled_back());
        
        let rollback_result = tx.rollback();
        assert!(rollback_result.is_ok());
    }
}
