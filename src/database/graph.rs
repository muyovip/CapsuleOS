use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub node_type: String,
    pub properties: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub edge_type: String,
    pub properties: HashMap<String, String>,
}

impl Node {
    pub fn new(id: String, node_type: String) -> Self {
        Self {
            id,
            node_type,
            properties: HashMap::new(),
        }
    }
}

impl Edge {
    pub fn new(from: String, to: String, edge_type: String) -> Self {
        Self {
            from,
            to,
            edge_type,
            properties: HashMap::new(),
        }
    }
}
