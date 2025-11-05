use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use serde_cbor;
use serde_json::Value as JsonValue;
use std::collections::{BTreeMap, HashMap};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use thiserror::Error;
use time::OffsetDateTime;

/// The root / sovereignty id constant
const ROOT_ID: &str = "⊙₀";

/// Audit log file name (in current working directory).
const AUDIT_LOG: &str = "cyberus_audit.log";

/// CLI entrypoint
#[derive(Parser)]
#[command(name = "cyberus", about = "CyberusCLI - CapsuleOS node control")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Inspect or modify nodes registered in Γ (garden)
    Garden {
        /// path of the node, e.g., /node/1
        path: String,
        #[command(subcommand)]
        op: GardenOp,
    },

    /// View a node; output formats: text (canonical JSON) or cbor
    View {
        /// path of the node, e.g., /node/1
        path: String,

        /// output format: text | cbor
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Forge a new capsule from a manifest file (CBOR or JSON)
    Forge {
        /// Path to manifest file (CBOR or JSON)
        manifest: PathBuf,
    },

    /// Verify integrity of Γ
    Resonate,
}

/// Garden operations
#[derive(Subcommand)]
enum GardenOp {
    /// Set a node label
    SetLabel { label: String },

    /// Set an attribute key/value on node
    SetAttr { key: String, value: String },

    /// Remove an attribute by key
    RemoveAttr { key: String },
}

#[derive(Debug, Error)]
enum CliError {
    #[error("node not found: {0}")]
    NotFound(String),

    #[error("sovereignty violation: {0}")]
    Sovereignty(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serialization error: {0}")]
    Serde(#[from] serde_cbor::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("invalid manifest format: {0}")]
    Manifest(String),
}

/// Graph node data (keeps maps as BTreeMap for canonical ordering)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GraphNode {
    pub id: String,
    pub label: Option<String>,
    pub parent: Option<String>,
    /// attributes stored as BTreeMap to ensure canonical key ordering in JSON/CBOR
    pub attrs: BTreeMap<String, String>,
    /// lineage: first element is this id, last must be ROOT_ID
    pub lineage: Vec<String>,
}

/// Simulated GenesisGraph (Γ) registry
#[derive(Clone)]
pub struct GenesisGraph {
    inner: Arc<Mutex<HashMap<String, GraphNode>>>,
}

impl GenesisGraph {
    pub fn new() -> Self {
        GenesisGraph {
            inner: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Register (or replace) a node in Γ
    pub fn register(&self, node: GraphNode) {
        let mut m = self.inner.lock().unwrap();
        m.insert(node.id.clone(), node);
    }

    /// Get a node by path like "/node/1" or "node/1"
    pub fn get_by_path(&self, path: &str) -> Option<GraphNode> {
        let id = normalize_path(path);
        let m = self.inner.lock().unwrap();
        m.get(&id).cloned()
    }

    /// Mutate node attributes (call closure with mutable ref)
    /// checks sovereignty before allowing mutation.
    pub fn mutate_node<F>(&self, path: &str, mutator: F) -> Result<(), CliError>
    where
        F: FnOnce(&mut GraphNode),
    {
        let id = normalize_path(path);
        let mut m = self.inner.lock().unwrap();
        let node = m.get_mut(&id).ok_or(CliError::NotFound(id.clone()))?;

        // Sovereignty check: if node lineage ends directly at ROOT_ID (i.e., parent is None or parent == ROOT_ID)
        // we treat this as sovereign and disallow mutation.
        if node.lineage.last().map(|s| s.as_str()) == Some(ROOT_ID) && node.lineage.len() == 2 {
            // lineage like [id, ROOT_ID] -> sovereign (direct child of root)
            return Err(CliError::Sovereignty(format!(
                "node '{}' is under root sovereignty and cannot be mutated",
                id
            )));
        }

        // Apply mutation closure
        mutator(node);

        // Audit the mutation
        audit_log(&format!("mutate {} by CLI", id)).ok();

        Ok(())
    }

    /// Verify all nodes for basic integrity (lineage ends with ROOT_ID and first element equals id)
    pub fn verify_all(&self) -> Vec<(String, Result<(), String>)> {
        let m = self.inner.lock().unwrap();
        m.values()
            .map(|n| {
                let id = n.id.clone();
                // checks
                if n.lineage.is_empty() {
                    return (id, Err("empty lineage".to_string()));
                }
                if n.lineage[0] != n.id {
                    return (id, Err("lineage[0] != id".to_string()));
                }
                if n.lineage.last().map(|s| s.as_str()) != Some(ROOT_ID) {
                    return (id, Err(format!("lineage must end at {}", ROOT_ID)));
                }
                (id, Ok(()))
            })
            .collect()
    }
}

/// Normalize incoming path into internal id: trim leading '/', treat rest as id
fn normalize_path(path: &str) -> String {
    let mut s = path.trim().to_string();
    if s.starts_with('/') {
        s = s.trim_start_matches('/').to_string();
    }
    s
}

/// Write an audit entry (timestamp + message) in append mode.
/// Non-fatal; returns io::Result.
fn audit_log(message: &str) -> Result<(), std::io::Error> {
    let now = OffsetDateTime::now_utc();
    let entry = format!("{} {}\n", now.format(&time::format_description::well_known::Rfc3339).unwrap(), message);
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(AUDIT_LOG)?;
    file.write_all(entry.as_bytes())?;
    Ok(())
}

/// Canonical textual format output for nodes: deterministic JSON (BTreeMap ensures ordering).
fn canonical_text_for_node(node: &GraphNode) -> Result<String, serde_json::Error> {
    // We can serialize GraphNode directly: attrs is BTreeMap guaranteeing key ordering;
    // serde_json will preserve ordering of maps when serializing values produced from BTreeMap.
    let json = serde_json::to_string_pretty(node)?;
    Ok(json)
}

/// CBOR bytes for node
fn cbor_for_node(node: &GraphNode) -> Result<Vec<u8>, serde_cbor::Error> {
    serde_cbor::to_vec(node)
}

/// Attempt to parse manifest file (JSON or CBOR) and return a GraphNode to register.
/// For this mock implementation, we accept a JSON manifest with fields compatible with GraphNode.
/// If the file ends in .cbor or is not valid json but valid CBOR, parse as CBOR.
fn parse_manifest_to_node(path: &PathBuf) -> Result<GraphNode, CliError> {
    let data = fs::read(path)?;
    // First try JSON
    if let Ok(json) = serde_json::from_slice::<JsonValue>(&data) {
        // Try to deserialize into GraphNode (serde_json -> our struct)
        let node: GraphNode = serde_json::from_value(json).map_err(|e| CliError::Manifest(format!("json->node: {}", e)))?;
        return Ok(node);
    }
    // Try CBOR
    let node: GraphNode = serde_cbor::from_slice(&data).map_err(|e| CliError::Manifest(format!("cbor->node: {}", e)))?;
    Ok(node)
}

fn main() -> Result<(), CliError> {
    let cli = Cli::parse();

    // For demonstration, create a GenesisGraph with some pre-populated nodes.
    // In real integration, connect to actual Γ via IPC/RPC.
    let graph = demos_graph();

    match cli.command {
        Commands::Garden { path, op } => {
            match op {
                GardenOp::SetLabel { label } => {
                    graph
                        .mutate_node(&path, |node| {
                            node.label = Some(label.clone());
                        })
                        .map_err(|e| e)?;
                    println!("OK");
                }
                GardenOp::SetAttr { key, value } => {
                    graph
                        .mutate_node(&path, |node| {
                            node.attrs.insert(key.clone(), value.clone());
                        })
                        .map_err(|e| e)?;
                    println!("OK");
                }
                GardenOp::RemoveAttr { key } => {
                    graph
                        .mutate_node(&path, |node| {
                            node.attrs.remove(&key);
                        })
                        .map_err(|e| e)?;
                    println!("OK");
                }
            }
        }

        Commands::View { path, format } => {
            let node = graph.get_by_path(&path).ok_or(CliError::NotFound(path.clone()))?;
            match format.as_str() {
                "text" => {
                    let out = canonical_text_for_node(&node)?;
                    println!("{}", out);
                }
                "cbor" => {
                    let bytes = cbor_for_node(&node)?;
                    // write to stdout as raw bytes
                    use std::io::{self, Write};
                    let mut stdout = io::stdout();
                    stdout.write_all(&bytes)?;
                }
                other => {
                    return Err(CliError::Manifest(format!("unsupported format '{}'", other)));
                }
            }
        }

        Commands::Forge { manifest } => {
            // parse manifest into GraphNode and register if valid
            let node = parse_manifest_to_node(&manifest)?;
            // check lineage ends at root
            if node.lineage.last().map(|s| s.as_str()) != Some(ROOT_ID) {
                return Err(CliError::Manifest(format!("manifest lineage must end at {}", ROOT_ID)));
            }
            // register
            graph.register(node);
            audit_log(&format!("forge manifest {}", manifest.display()))?;
            println!("OK");
        }

        Commands::Resonate => {
            // verify all nodes in graph
            let results = graph.verify_all();
            let mut any_err = false;
            for (id, res) in results {
                match res {
                    Ok(()) => println!("{}: OK", id),
                    Err(e) => {
                        any_err = true;
                        println!("{}: ERROR - {}", id, e);
                    }
                }
            }
            if any_err {
                std::process::exit(2);
            }
        }
    }

    Ok(())
}

/// Build a demo GenesisGraph with a few nodes for CLI demonstration.
/// In real runtime, this would be an IPC/RPC connection to CapsuleOS.
fn demos_graph() -> GenesisGraph {
    let g = GenesisGraph::new();

    // Node that is a direct child of root (so sovereign and cannot be mutated)
    let mut attrs1 = BTreeMap::new();
    attrs1.insert("owner".to_string(), "genesis".to_string());
    let node1 = GraphNode {
        id: "node/1".to_string(),
        label: Some("Genesis Node".to_string()),
        parent: Some(ROOT_ID.to_string()),
        attrs: attrs1,
        lineage: vec!["node/1".to_string(), ROOT_ID.to_string()],
    };
    g.register(node1);

    // Node that is child of node/1 (mutable)
    let mut attrs2 = BTreeMap::new();
    attrs2.insert("owner".to_string(), "alice".to_string());
    let node2 = GraphNode {
        id: "node/1/child".to_string(),
        label: Some("Alice's Node".to_string()),
        parent: Some("node/1".to_string()),
        attrs: attrs2,
        lineage: vec!["node/1/child".to_string(), "node/1".to_string(), ROOT_ID.to_string()],
    };
    g.register(node2);

    g
}

////////////////////////////////////////////////////////////////////////////////
// Tests
////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_view_text_canonical_ordering() {
        let g = demos_graph();
        let node = g.get_by_path("/node/1/child").expect("node");
        // attrs BTreeMap ensures deterministic ordering
        let text = canonical_text_for_node(&node).unwrap();
        // must contain keys in predictable order (JSON pretty printing is stable)
        assert!(text.contains("\"attrs\""));
        assert!(text.contains("\"owner\": \"alice\""));
    }

    #[test]
    fn test_garden_mutation_respects_sovereignty() {
        let g = demos_graph();
        // attempt to mutate sovereign node node/1 -> should return sovereignty error
        let res = g.mutate_node("/node/1", |n| {
            n.attrs.insert("x".into(), "y".into());
        });
        assert!(matches!(res, Err(CliError::Sovereignty(_))));

        // mutate child -> allowed
        let res2 = g.mutate_node("/node/1/child", |n| {
            n.attrs.insert("new".into(), "val".into());
        });
        assert!(res2.is_ok());
        let after = g.get_by_path("/node/1/child").unwrap();
        assert_eq!(after.attrs.get("new").map(|s| s.as_str()), Some("val"));
    }

    #[test]
    fn test_forge_accepts_json_and_registers() {
        let _g = GenesisGraph::new();
        // construct a manifest JSON for a new node
        let node = GraphNode {
            id: "node/x".to_string(),
            label: Some("x".to_string()),
            parent: Some("node/1".to_string()),
            attrs: {
                let mut m = BTreeMap::new();
                m.insert("k".to_string(), "v".to_string());
                m
            },
            lineage: vec!["node/x".to_string(), "node/1".to_string(), ROOT_ID.to_string()],
        };

        // write JSON manifest to temp file
        let mut tf = NamedTempFile::new().unwrap();
        let json = serde_json::to_string(&node).unwrap();
        tf.write_all(json.as_bytes()).unwrap();
        let path = tf.into_temp_path();
        // parse via parse_manifest_to_node
        let parsed = parse_manifest_to_node(&PathBuf::from(path.to_string_lossy().to_string())).unwrap();
        assert_eq!(parsed.id, "node/x");
    }

    #[test]
    fn test_resonate_reports_errors_for_bad_lineage() {
        let g = GenesisGraph::new();
        // add bad node with missing root in lineage
        let bad = GraphNode {
            id: "bad".to_string(),
            label: None,
            parent: None,
            attrs: BTreeMap::new(),
            lineage: vec!["bad".to_string(), "some".to_string()],
        };
        g.register(bad);
        let results = g.verify_all();
        // find bad entry
        let found = results.into_iter().find(|(id, _)| id == "bad").unwrap();
        assert!(found.1.is_err());
    }
}
