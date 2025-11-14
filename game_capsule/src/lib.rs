//! Game Capsule Storage Integration for CapsuleOS
//!
//! This module provides specialized capsule storage for game generation
//! using GΛLYPH expressions and immutable graph patterns.

use capsule_core::{Capsule, CapsuleMetadata, SignatureBlock, CanonicalSerialize, ContentAddressable};
use capsule_core::{canonical_cbor, compute_content_hash_with_prefix, generate_keypair, SigningKey};
use genesis_graph::{GenesisGraph, GraphNode, Expression, NodeMetadata, EdgeType};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;
use uuid::Uuid;

// ============================================================================
// Core Data Structures
// ============================================================================

/// Game manifest containing all game data
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameManifest {
    pub id: String,
    pub title: String,
    pub story: String,
    pub rules: serde_json::Value,
    pub code: String,
    pub balance: f64,
    pub genre: Option<String>,
    pub theme: Option<String>,
    pub created_at: u64,
    pub llm_outputs: Vec<LLMOutput>,
}

/// Output from a single LLM in the orchestration process
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LLMOutput {
    pub llm_name: String,
    pub llm_role: LLMRole,
    pub glyph_expression: String,  // GΛLYPH λ-expression
    pub processed_at: u64,
    pub confidence: Option<f64>,
}

/// Role of each LLM in the game generation process
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LLMRole {
    Narrative,    // Phi-3 - Story and narrative
    Mechanics,    // Gemma-2B - Game mechanics
    Assets,       // TinyLlama - Asset descriptions
    Balance,      // Qwen-0.5B - Balance testing
}

/// Game capsule containing game manifest and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameCapsule {
    pub metadata: GameCapsuleMetadata,
    pub manifest: GameManifest,
    pub signature_block: SignatureBlock,
}

/// Metadata specific to game capsules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameCapsuleMetadata {
    pub capsule_id: String,
    pub version: String,
    pub created_at: u64,
    pub parent_cid: Option<String>,
    pub genesis_cid: String,  // Root graph CID
    pub game_cid: String,     // Content-addressable game ID
    pub user_id: Option<String>,
    pub tags: Vec<String>,
}

/// Game storage engine for managing game capsules
pub struct GameStorage {
    signing_key: SigningKey,
    genesis_graph: GenesisGraph,
    game_cache: HashMap<String, GameCapsule>,
}

/// Errors that can occur during game capsule operations
#[derive(Error, Debug)]
pub enum GameStorageError {
    #[error("Serialization failed: {0}")]
    SerializationError(String),

    #[error("Invalid game manifest: {0}")]
    InvalidManifest(String),

    #[error("Game not found: {0}")]
    GameNotFound(String),

    #[error("Parent game not found: {0}")]
    ParentNotFound(String),

    #[error("Cryptographic error: {0}")]
    CryptoError(String),

    #[error("Graph error: {0}")]
    GraphError(String),

    #[error("Invalid GΛLYPH expression: {0}")]
    InvalidGlyphExpression(String),
}

// ============================================================================
// GameManifest Implementation
// ============================================================================

impl GameManifest {
    /// Create a new game manifest from LLM outputs
    pub fn new(
        title: String,
        story: String,
        rules: serde_json::Value,
        code: String,
        balance: f64,
        llm_outputs: Vec<LLMOutput>,
    ) -> Result<Self, GameStorageError> {
        // Validate inputs
        if title.trim().is_empty() {
            return Err(GameStorageError::InvalidManifest("Title cannot be empty".to_string()));
        }

        if balance < 0.0 || balance > 1.0 {
            return Err(GameStorageError::InvalidManifest("Balance must be between 0.0 and 1.0".to_string()));
        }

        if llm_outputs.len() != 4 {
            return Err(GameStorageError::InvalidManifest("Expected exactly 4 LLM outputs".to_string()));
        }

        // Validate LLM roles
        let required_roles = [LLMRole::Narrative, LLMRole::Mechanics, LLMRole::Assets, LLMRole::Balance];
        for required_role in &required_roles {
            if !llm_outputs.iter().any(|output| output.llm_role == *required_role) {
                return Err(GameStorageError::InvalidManifest(
                    format!("Missing LLM output for role: {:?}", required_role)
                ));
            }
        }

        Ok(Self {
            id: Uuid::new_v4().to_string(),
            title,
            story,
            rules,
            code,
            balance,
            genre: None,
            theme: None,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            llm_outputs,
        })
    }

    /// Set genre and theme
    pub fn with_genre_theme(mut self, genre: Option<String>, theme: Option<String>) -> Self {
        self.genre = genre;
        self.theme = theme;
        self
    }

    /// Get the merged GΛLYPH expression (λgame)
    pub fn get_merged_glyph_expression(&self) -> Result<String, GameStorageError> {
        // Find the expression from the Balance LLM (should contain merged result)
        let balance_output = self.llm_outputs.iter()
            .find(|output| output.llm_role == LLMRole::Balance)
            .ok_or_else(|| GameStorageError::InvalidManifest("Missing Balance LLM output".to_string()))?;

        // Validate it's a proper λgame expression
        if !balance_output.glyph_expression.starts_with("λgame") {
            return Err(GameStorageError::InvalidGlyphExpression(
                "Balance LLM output should contain λgame expression".to_string()
            ));
        }

        Ok(balance_output.glyph_expression.clone())
    }
}

// ============================================================================
// GameCapsule Implementation
// ============================================================================

impl GameCapsule {
    /// Create a new game capsule
    pub fn new(
        manifest: GameManifest,
        parent_cid: Option<String>,
        genesis_cid: String,
        user_id: Option<String>,
        signing_key: &SigningKey,
    ) -> Result<Self, GameStorageError> {
        let game_cid = Self::compute_game_cid(&manifest)?;

        let metadata = GameCapsuleMetadata {
            capsule_id: Uuid::new_v4().to_string(),
            version: "1.0.0".to_string(),
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            parent_cid,
            genesis_cid,
            game_cid: game_cid.clone(),
            user_id,
            tags: vec!["game".to_string(), "generated".to_string()],
        };

        // Sign the capsule
        let signature_block = Self::sign_capsule(&metadata, &manifest, signing_key)?;

        Ok(Self {
            metadata,
            manifest,
            signature_block,
        })
    }

    /// Compute content-addressable CID for the game
    fn compute_game_cid(manifest: &GameManifest) -> Result<String, GameStorageError> {
        // Serialize manifest for CID computation
        let cbor_data = canonical_cbor(manifest)
            .map_err(|e| GameStorageError::SerializationError(e.to_string()))?;

        // Compute CID with GameV1 prefix
        Ok(compute_content_hash_with_prefix("GameV1", &cbor_data))
    }

    /// Sign the capsule content
    fn sign_capsule(
        metadata: &GameCapsuleMetadata,
        manifest: &GameManifest,
        signing_key: &SigningKey,
    ) -> Result<SignatureBlock, GameStorageError> {
        // Create unsigned capsule for signing
        #[derive(Serialize)]
        struct UnsignedGameCapsule<'a> {
            metadata: &'a GameCapsuleMetadata,
            manifest: &'a GameManifest,
        }

        let unsigned = UnsignedGameCapsule { metadata, manifest };

        let cbor_data = canonical_cbor(&unsigned)
            .map_err(|e| GameStorageError::SerializationError(e.to_string()))?;

        let content_hash = compute_content_hash_with_prefix("GameCapsuleV1", &cbor_data);

        // Sign the content hash
        let signature = signing_key.sign(content_hash.as_bytes());

        Ok(SignatureBlock {
            public_key: signing_key.verifying_key().to_bytes(),
            signature: signature.to_bytes(),
            content_hash,
        })
    }

    /// Verify the capsule signature and integrity
    pub fn verify(&self) -> Result<bool, GameStorageError> {
        // Recreate unsigned capsule
        #[derive(Serialize)]
        struct UnsignedGameCapsule<'a> {
            metadata: &'a GameCapsuleMetadata,
            manifest: &'a GameManifest,
        }

        let unsigned = UnsignedGameCapsule {
            metadata: &self.metadata,
            manifest: &self.manifest,
        };

        let cbor_data = canonical_cbor(&unsigned)
            .map_err(|e| GameStorageError::SerializationError(e.to_string()))?;

        let computed_hash = compute_content_hash_with_prefix("GameCapsuleV1", &cbor_data);

        // Verify content hash
        if computed_hash != self.signature_block.content_hash {
            return Ok(false);
        }

        // Verify cryptographic signature
        use ed25519_dalek::{Verifier, VerifyingKey, Signature};

        let public_key = VerifyingKey::from_bytes(&self.signature_block.public_key)
            .map_err(|e| GameStorageError::CryptoError(e.to_string()))?;

        let signature = Signature::try_from(&self.signature_block.signature[..])
            .map_err(|e| GameStorageError::CryptoError(e.to_string()))?;

        Ok(public_key.verify(self.signature_block.content_hash.as_bytes(), &signature).is_ok())
    }

    /// Get the game CID
    pub fn game_cid(&self) -> &str {
        &self.metadata.game_cid
    }

    /// Get the parent CID if this is an evolution
    pub fn parent_cid(&self) -> Option<&str> {
        self.metadata.parent_cid.as_deref()
    }

    /// Get the genesis graph CID
    pub fn genesis_cid(&self) -> &str {
        &self.metadata.genesis_cid
    }
}

// ============================================================================
// GameStorage Implementation
// ============================================================================

impl GameStorage {
    /// Create a new game storage instance
    pub fn new() -> Result<Self, GameStorageError> {
        let signing_key = generate_keypair();

        // Create genesis graph
        let root_node = genesis_graph::create_root_node();
        let genesis_graph = GenesisGraph::new(root_node)
            .map_err(|e| GameStorageError::GraphError(e.to_string()))?;

        Ok(Self {
            signing_key,
            genesis_graph,
            game_cache: HashMap::new(),
        })
    }

    /// Create game storage with existing signing key
    pub fn with_signing_key(signing_key: SigningKey) -> Result<Self, GameStorageError> {
        let root_node = genesis_graph::create_root_node();
        let genesis_graph = GenesisGraph::new(root_node)
            .map_err(|e| GameStorageError::GraphError(e.to_string()))?;

        Ok(Self {
            signing_key,
            genesis_graph,
            game_cache: HashMap::new(),
        })
    }

    /// Store a new game capsule
    pub fn store_game(
        &mut self,
        manifest: GameManifest,
        parent_cid: Option<String>,
        user_id: Option<String>,
    ) -> Result<String, GameStorageError> {
        let genesis_cid = self.genesis_graph.root_hash().clone();

        let capsule = GameCapsule::new(
            manifest,
            parent_cid,
            genesis_cid,
            user_id,
            &self.signing_key,
        )?;

        let game_cid = capsule.game_cid().to_string();

        // Store in cache
        self.game_cache.insert(game_cid.clone(), capsule);

        // Store in genesis graph as GΛLYPH expression
        self.store_in_genesis_graph(&game_cid)?;

        Ok(game_cid)
    }

    /// Store game reference in genesis graph
    fn store_in_genesis_graph(&mut self, game_cid: &str) -> Result<(), GameStorageError> {
        // Create a node representing the game in the graph
        let glyph_expression = format!("λgame.{}", game_cid);

        let node_data = Expression::Lambda {
            param: "game".to_string(),
            body: Box::new(Expression::Var(game_cid.to_string())),
        };

        let node = GraphNode {
            id: format!("game_{}", game_cid),
            root_ref: self.genesis_graph.root_hash().clone(),
            data: node_data,
            metadata: NodeMetadata {
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                lineage_depth: 1,
                tags: vec!["game".to_string(), "λgame".to_string()],
            },
        };

        self.genesis_graph.insert_node(node)
            .map_err(|e| GameStorageError::GraphError(e.to_string()))?;

        Ok(())
    }

    /// Retrieve a game capsule by CID
    pub fn get_game(&self, game_cid: &str) -> Result<&GameCapsule, GameStorageError> {
        self.game_cache.get(game_cid)
            .ok_or_else(|| GameStorageError::GameNotFound(game_cid.to_string()))
    }

    /// Evolve an existing game
    pub fn evolve_game(
        &mut self,
        parent_cid: &str,
        new_manifest: GameManifest,
        user_id: Option<String>,
    ) -> Result<String, GameStorageError> {
        // Verify parent exists
        if !self.game_cache.contains_key(parent_cid) {
            return Err(GameStorageError::ParentNotFound(parent_cid.to_string()));
        }

        self.store_game(new_manifest, Some(parent_cid.to_string()), user_id)
    }

    /// Get evolution lineage for a game
    pub fn get_evolution_lineage(&self, game_cid: &str) -> Result<Vec<String>, GameStorageError> {
        let mut lineage = Vec::new();
        let mut current_cid = Some(game_cid.to_string());

        while let Some(cid) = current_cid {
            lineage.push(cid.clone());

            let capsule = self.game_cache.get(&cid)
                .ok_or_else(|| GameStorageError::GameNotFound(cid.clone()))?;

            current_cid = capsule.parent_cid().map(|s| s.to_string());
        }

        Ok(lineage)
    }

    /// List all games for a user
    pub fn list_user_games(&self, user_id: &str) -> Vec<&GameCapsule> {
        self.game_cache.values()
            .filter(|capsule| capsule.metadata.user_id.as_ref() == Some(&user_id.to_string()))
            .collect()
    }

    /// Get genesis graph CID
    pub fn genesis_cid(&self) -> &str {
        self.genesis_graph.root_hash()
    }

    /// Get signing key public key
    pub fn public_key(&self) -> [u8; 32] {
        self.signing_key.verifying_key().to_bytes()
    }
}

// ============================================================================
// Traits and Implementations
// ============================================================================

impl CanonicalSerialize for GameManifest {
    fn canonical_serialize(&self) -> Vec<u8> {
        canonical_cbor(self).expect("GameManifest serialization should not fail")
    }
}

impl ContentAddressable for GameManifest {
    fn content_hash(&self) -> String {
        let ser = self.canonical_serialize();
        compute_content_hash_with_prefix("GameV1", &ser)
    }
}

impl CanonicalSerialize for GameCapsule {
    fn canonical_serialize(&self) -> Vec<u8> {
        canonical_cbor(self).expect("GameCapsule serialization should not fail")
    }
}

impl ContentAddressable for GameCapsule {
    fn content_hash(&self) -> String {
        let ser = self.canonical_serialize();
        compute_content_hash_with_prefix("GameCapsuleV1", &ser)
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Create a game manifest from orchestration result
pub fn create_game_manifest_from_orchestration(
    title: String,
    story: String,
    rules: serde_json::Value,
    code: String,
    balance: f64,
    glyph_expression: &str,  // The merged λgame expression
    llm_outputs: Vec<LLMOutput>,
) -> Result<GameManifest, GameStorageError> {
    let mut manifest = GameManifest::new(title, story, rules, code, balance, llm_outputs)?;

    // Validate that the merged expression matches the balance output
    if let Some(balance_output) = manifest.llm_outputs.iter()
        .find(|output| output.llm_role == LLMRole::Balance) {
        if balance_output.glyph_expression != glyph_expression {
            return Err(GameStorageError::InvalidGlyphExpression(
                "Provided glyph expression doesn't match balance LLM output".to_string()
            ));
        }
    }

    Ok(manifest)
}

/// Validate a GΛLYPH expression
pub fn validate_glyph_expression(expression: &str) -> Result<(), GameStorageError> {
    // Basic validation for λgame expressions
    if !expression.starts_with("λgame") {
        return Err(GameStorageError::InvalidGlyphExpression(
            "Expression must start with λgame".to_string()
        ));
    }

    // Check for balanced parentheses and proper structure
    let mut paren_count = 0;
    for char in expression.chars() {
        match char {
            '(' => paren_count += 1,
            ')' => paren_count -= 1,
            _ => {}
        }
        if paren_count < 0 {
            return Err(GameStorageError::InvalidGlyphExpression(
                "Unbalanced parentheses".to_string()
            ));
        }
    }

    if paren_count != 0 {
        return Err(GameStorageError::InvalidGlyphExpression(
            "Unbalanced parentheses".to_string()
        ));
    }

    Ok(())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_llm_outputs() -> Vec<LLMOutput> {
        vec![
            LLMOutput {
                llm_name: "Phi-3".to_string(),
                llm_role: LLMRole::Narrative,
                glyph_expression: "λnarrative.story=\"Space adventure\"".to_string(),
                processed_at: 1234567890,
                confidence: Some(0.95),
            },
            LLMOutput {
                llm_name: "Gemma-2B".to_string(),
                llm_role: LLMRole::Mechanics,
                glyph_expression: "λmechanics.turn_based=true".to_string(),
                processed_at: 1234567891,
                confidence: Some(0.88),
            },
            LLMOutput {
                llm_name: "TinyLlama".to_string(),
                llm_role: LLMRole::Assets,
                glyph_expression: "λassets.sprite_size=32x32".to_string(),
                processed_at: 1234567892,
                confidence: Some(0.92),
            },
            LLMOutput {
                llm_name: "Qwen-0.5B".to_string(),
                llm_role: LLMRole::Balance,
                glyph_expression: "λgame.(λnarrative.story=\"Space adventure\" λmechanics.turn_based=true λassets.sprite_size=32x32)".to_string(),
                processed_at: 1234567893,
                confidence: Some(0.90),
            },
        ]
    }

    #[test]
    fn test_game_manifest_creation() {
        let llm_outputs = create_test_llm_outputs();
        let manifest = GameManifest::new(
            "Test Game".to_string(),
            "A test story".to_string(),
            serde_json::json!({"turn_based": true}),
            "function game() {}".to_string(),
            0.85,
            llm_outputs,
        ).unwrap();

        assert_eq!(manifest.title, "Test Game");
        assert_eq!(manifest.llm_outputs.len(), 4);
        assert!(manifest.balance > 0.0 && manifest.balance <= 1.0);
    }

    #[test]
    fn test_game_capsule_creation_and_verification() {
        let llm_outputs = create_test_llm_outputs();
        let manifest = GameManifest::new(
            "Test Game".to_string(),
            "A test story".to_string(),
            serde_json::json!({"turn_based": true}),
            "function game() {}".to_string(),
            0.85,
            llm_outputs,
        ).unwrap();

        let signing_key = generate_keypair();
        let capsule = GameCapsule::new(
            manifest,
            None,
            "genesis_cid".to_string(),
            Some("user123".to_string()),
            &signing_key,
        ).unwrap();

        // Verify capsule
        let is_valid = capsule.verify().unwrap();
        assert!(is_valid, "Capsule should be valid");

        // Check CID format
        assert!(capsule.game_cid().starts_with("GameV1:"));
    }

    #[test]
    fn test_game_storage_operations() {
        let mut storage = GameStorage::new().unwrap();

        let llm_outputs = create_test_llm_outputs();
        let manifest = GameManifest::new(
            "Storage Test Game".to_string(),
            "A story for storage test".to_string(),
            serde_json::json!({"test": true}),
            "function test() {}".to_string(),
            0.75,
            llm_outputs,
        ).unwrap();

        // Store game
        let game_cid = storage.store_game(
            manifest.clone(),
            None,
            Some("user456".to_string()),
        ).unwrap();

        // Retrieve game
        let retrieved = storage.get_game(&game_cid).unwrap();
        assert_eq!(retrieved.manifest.title, "Storage Test Game");
        assert_eq!(retrieved.metadata.user_id, Some("user456".to_string()));

        // Test evolution
        let evolved_manifest = GameManifest::new(
            "Evolved Game".to_string(),
            "An evolved story".to_string(),
            serde_json::json!({"evolved": true}),
            "function evolved() {}".to_string(),
            0.80,
            create_test_llm_outputs(),
        ).unwrap();

        let evolved_cid = storage.evolve_game(
            &game_cid,
            evolved_manifest,
            Some("user456".to_string()),
        ).unwrap();

        // Check lineage
        let lineage = storage.get_evolution_lineage(&evolved_cid).unwrap();
        assert_eq!(lineage.len(), 2);
        assert_eq!(lineage[1], game_cid);
    }

    #[test]
    fn test_glyph_expression_validation() {
        let valid_expr = "λgame.(λnarrative.story=\"test\" λmechanics.turn_based=true)";
        assert!(validate_glyph_expression(valid_expr).is_ok());

        let invalid_expr = "game.(invalid structure";
        assert!(validate_glyph_expression(invalid_expr).is_err());
    }

    #[test]
    fn test_error_handling() {
        // Test empty title
        let result = GameManifest::new(
            "".to_string(),
            "Story".to_string(),
            serde_json::json!({}),
            "code".to_string(),
            0.5,
            create_test_llm_outputs(),
        );
        assert!(matches!(result, Err(GameStorageError::InvalidManifest(_))));

        // Test invalid balance
        let result = GameManifest::new(
            "Title".to_string(),
            "Story".to_string(),
            serde_json::json!({}),
            "code".to_string(),
            1.5,  // Invalid balance > 1.0
            create_test_llm_outputs(),
        );
        assert!(matches!(result, Err(GameStorageError::InvalidManifest(_))));
    }
}