# CapsuleOS - Meta-Operating System Toolchain

## Project Overview
CapsuleOS is a new meta-operating system with a cryptographic foundation and custom language (GΛLYPH). This repository contains the core Rust implementation of five foundational components:

1. **capsule_core** - Cryptographic foundation for the Root Capsule (⊙₀) with content-addressable hashing
2. **glyph_lexer** - Tokenizer/lexer for the GΛLYPH language
3. **glyph_parser** - Recursive descent parser and AST for GΛLYPH
4. **genesis_graph** - Content-addressable DAG for cryptographic lineage and dependencies
5. **glyph_engine** - Pattern matching engine for GΛLYPH expressions

## Recent Changes

### 2025-11-05: Capture-Avoiding Substitution Engine (Work Order 7) ✓
Created complete capture-avoiding substitution engine in glyph_engine crate:

**Core Features:**
- Fresh name generation (gensym) with atomic counter for deterministic testing
- Free variables analysis with scope tracking
- Alpha renaming (α-conversion) to avoid variable capture
- Capture-avoiding substitution for all expression types
- Pattern variable extraction
- Multiple simultaneous substitutions
- Well-formedness checking (no unbound variables)

**Substitution Functions:**
- `substitute()`: Main entry point for capture-avoiding substitution expr[var := replacement]
- `substitute_internal()`: Recursive substitution with bound variable tracking
- `substitute_match_arm()`: Special handling for match arms with pattern capture avoidance
- `substitute_many()`: Parallel substitution of multiple variables (deterministic ordering)
- `substitute_pattern()`: Rename variables in patterns

**Helper Functions:**
- `gensym()`: Generate unique fresh variable names with format "prefix$N"
- `reset_gensym()`: Reset counter for deterministic testing
- `free_vars()`: Compute set of free variables in expression
- `pattern_variables()`: Extract all variables bound by a pattern
- `alpha_rename()`: Rename bound variable to avoid capture (returns new name + renamed expr)
- `alpha_rename_pattern()`: Rename pattern variables
- `is_well_formed()`: Check for unbound variables (closed terms only)

**Key Implementation Details:**
- Variable shadowing: bound variables block substitution in their scope
- Capture avoidance: automatic α-renaming when replacement would be captured
- Lambda handling: renames parameter if it appears in replacement's free vars
- Let binding handling: same capture avoidance as lambda
- Match expression handling: pattern variables shadow in guard and body
- Deterministic substitution: uses avoid sets to ensure consistent fresh names

**Test Coverage (47 tests):**
- Gensym uniqueness and freshness
- Free variables computation (simple, lambda, mixed scopes)
- Variable substitution (var, apply, tuple, list, record)
- Lambda substitution (free vars, shadowing, capture avoidance)
- Let binding substitution (value, body, shadowing, capture)
- Match expression substitution (expr, pattern shadowing, capture avoidance, guards)
- Alpha renaming (simple, conflict avoidance)
- Pattern variable extraction (simple, tuple, bind, constructor, record)
- Well-formedness checking (closed/open terms, preservation)
- Multiple substitutions (deterministic ordering)
- Property tests (determinism, idempotence, commutativity, α-equivalence preservation)
- Comprehensive integration tests (Church numerals, nested structures, complex scenarios)

**Test Status**: All 47 substitute tests passing (89 total glyph_engine tests)

### 2025-11-04: Pattern Matching Engine (Work Order 6) ✓
Created complete glyph_engine crate with pattern matching functionality for GΛLYPH expressions:

**Core Features:**
- Complete pattern matching engine with structural and bind support
- Pattern types: Wildcard, Variable, Literal, Bind (x@P), Tuple, List, Record, Constructor, Lambda, Apply
- Variable consistency checking prevents invalid bindings (pattern variable shadowing)
- Constructor pattern unwinding handles both Apply and LinearApply nodes
- Deterministic binding serialization using BTreeMap
- Helper functions: match_any_pattern, match_pattern_many, matches, pattern_variables

**Data Types:**
- `Expression`: Literal, Var, Lambda, Apply, LinearApply, Let, Match, Tuple, List, Record
- `Literal`: Int, Float, String, Bool, Unit
- `Pattern`: 10 pattern types with full structural matching
- `Bindings`: BTreeMap<String, Expression> for deterministic ordering
- `MatchResult`: Vec<Bindings> supporting multiple match results

**Pattern Matching Semantics:**
- Wildcard pattern: matches anything, binds nothing
- Variable pattern: matches anything, binds to name with consistency checking
- Literal pattern: exact value matching for all literal types
- Bind pattern (x@P): binds value to name AND matches nested pattern
- Structural patterns: recursive matching for Tuple, List, Record
- Constructor patterns: zero-arg and multi-arg with curried application unwinding
- Lambda/Application patterns: matches expression structure

**Key Implementation Details:**
- Variable shadowing prevention: rejects patterns binding same variable to different values
- Constructor unwinding: handles nested Apply and LinearApply chains
- Partial record matching: pattern matches subset of record fields
- Deterministic serialization: BTreeMap ensures stable CBOR output
- Helper functions for OR semantics, bulk matching, and variable extraction

**Test Coverage (42 tests):**
- All pattern types (wildcard, variable, literal, bind, structural, constructor)
- LinearApply constructor patterns (single-arg, multi-arg, mixed chains)
- Variable shadowing prevention
- Deterministic binding order and serialization
- Round-trip property verification
- Idempotence testing
- Comprehensive structural matching scenarios

**Test Status**: All 42 glyph_engine tests passing

### 2025-11-04: GenesisGraph DAG Implementation (Work Order 5) ✓
Created complete genesis_graph crate with cryptographic DAG functionality:

**Core Features:**
- Content-addressable directed acyclic graph (DAG) with cryptographic lineage tracking
- Root node (⊙₀) creation with backward-compatible hash validation
- Node insertion with root reference enforcement
- Edge linking with automatic cycle detection (DFS-based)
- Topological sorting via Kahn's algorithm
- Lineage path tracking from root via BFS traversal
- Node deletion with automatic edge cleanup
- Canonical CBOR serialization (deterministic, order-independent)

**Data Structures:**
- `GraphNode`: ID, root reference, Expression data, metadata (timestamp, lineage_depth, tags)
- `GraphEdge`: from/to hashes with edge types (Dependency, Derivation, Reference)
- `GenesisGraph`: nodes HashMap, edges Vec, root_hash
- `GraphError`: 7 error types for comprehensive validation

**Hash Computation:**
- Root node hash: `GlyphV1:Root:` prefix, computed with empty `root_ref` for determinism
- Node hash: `GlyphV1:Node:` prefix, includes all node fields
- Backward compatible: accepts both empty and pre-hashed root_ref values
- Internal normalization: stores root nodes with empty `root_ref` for consistency

**Key Design Decisions:**
- Root nodes have empty `root_ref` (they're the genesis, no parent)
- Validation accepts legacy root nodes with pre-computed `root_ref` for compatibility
- Cycle detection prevents DAG corruption (no circular dependencies)
- Self-loops forbidden to maintain acyclic invariant
- Canonical serialization uses BTreeMap for nodes and sorted edges

**Test Status**: All 18 genesis_graph tests passing (comprehensive coverage)

### 2025-11-04: Content-Addressable Hashing (Work Order 4) ✓
Integrated content-addressable hashing functionality into capsule_core:

**New Features:**
- `ContentAddressable` trait for computing prefixed content hashes
- `CanonicalSerialize` trait for deterministic CBOR serialization
- Domain types: `Glyph`, `Expression`, `GraphNode`, `GlyphRef`, `ExpressionRef`
- `compute_content_hash_with_prefix()` function returning "prefix:hexhash" format
- Type-specific hash prefixes: GlyphV1, ExprV1, NodeV1

**Implementation Details:**
- Uses `serde_cbor::to_vec()` for deterministic CBOR serialization
- Preserves backward compatibility with existing Root Capsule functionality
- Legacy `compute_content_hash()` still works for existing code
- All domain types implement both traits with proper prefixes
- Deterministic hashing: identical input always produces identical hash

**Test Status**: All 10 capsule_core tests passing (4 original + 6 new Work Order 4 tests)

### 2025-11-04: GΛLYPH Parser Implementation (Work Order 3) ✓
Implemented complete recursive descent parser for the GΛLYPH language:

**Features:**
- Complete AST with Expression, Literal, Pattern, and MatchArm types
- Lexer with support for: literals, keywords, lambda (λ), linear arrow (⊸), comments (#), negative numbers
- Recursive descent parser with proper precedence handling
- Pattern matching with wildcards, variables, literals, tuples, and constructors
- Guard expressions in match arms
- Canonical CBOR serialization/deserialization
- Public API: `parse(input: &str) -> Result<Expression, ParseError>`

**Key Implementation Details:**
- Parse chain: parse_expression → parse_let → parse_match → parse_lambda → parse_application → parse_primary
- Match subjects restricted to primary expressions to avoid ambiguity with match arm braces
- Record literals fully supported as function arguments: `f { x: 1 }`
- Lambda bodies use full expression parsing to support nested let/match expressions

**Test Status**: All 36 tests passing (120 comprehensive test cases)

### 2025-11-04: Lexer Bug Fixes - All Tests Passing ✓
Fixed three critical position tracking bugs in the glyph_lexer:

1. **Float literal detection** - Fixed lookahead logic to properly detect float literals (e.g., "3.14")
   - Previously: Lexed as three tokens (integer, delimiter, integer)
   - Now: Correctly lexed as single FloatLiteral token
   - Root cause: `peek_ahead()` was looking from stale `cur_pos` instead of current peek position

2. **Block comment parsing** - Fixed nested comment detection
   - Previously: "/* comment */" falsely reported as "Unterminated block comment"
   - Now: Correctly parses all block comments including nested ones
   - Root cause: Same peek_ahead issue affecting `*/` detection

3. **Division operator recognition** - Fixed "/" at end of input
   - Previously: Trailing "/" incorrectly treated as line comment starter
   - Now: Correctly emitted as Operator("/")
   - Root cause: Lookahead checking for "//" before position update

**Solution**: Changed all lookahead checks to use `self.input[(pos + 1)..].chars().next()` instead of `peek_ahead()`, ensuring position-relative peeking.

**Test Status**: All 96 tests passing (92 lexer + 4 capsule_core)

## Project Architecture

### glyph_engine (89/89 tests ✓)

**Capture-Avoiding Substitution Engine (47 tests):**
- Complete capture-avoiding substitution with α-conversion
- Fresh name generation via atomic gensym counter
- Free variables analysis with scope tracking
- Alpha renaming for bound variables to avoid capture
- Pattern variable extraction from all pattern types
- Multiple simultaneous substitutions with deterministic ordering
- Well-formedness checking for closed terms

**Substitution Algorithm:**
- Main entry: `substitute(expr, var, replacement)` returns expr[var := replacement]
- Handles all expression types: Literal, Var, Lambda, Apply, LinearApply, Let, Match, Tuple, List, Record
- Automatic α-renaming when replacement would capture bound variables
- Special handling for match arms with pattern-bound variables
- Preserves well-formedness of closed terms
- Deterministic behavior for testing (gensym reset capability)

**glyph_engine (42 tests ✓)
**Pattern Matching Engine:**
- Complete pattern matcher for GΛLYPH expressions with structural and bind support
- Pattern types: Wildcard, Var, Literal, Bind, Tuple, List, Constructor, Record, Lambda, Apply
- Variable consistency checking prevents shadowing (same variable bound to different values)
- Constructor pattern unwinding for both Apply and LinearApply nodes
- Deterministic binding serialization using BTreeMap

**Core Functions:**
- `match_pattern()`: Main entry point returning MatchResult (Vec<Bindings>)
- `match_pattern_internal()`: Recursive matcher with binding accumulation
- `match_constructor()` / `match_constructor_application()`: Constructor pattern handling
- `match_any_pattern()`: OR semantics across multiple patterns
- `match_pattern_many()`: Bulk matching against multiple expressions
- `matches()`: Boolean check without bindings (faster)
- `pattern_variables()`: Extract all bound variables from pattern
- CBOR serialization helpers for MatchResult

**Dependencies:**
- `serde = { version = "1.0", features = ["derive"] }` - Serialization
- `ciborium = "0.2"` - CBOR encoding
- `proptest = "1.4"` - Property-based testing (dev dependency)

**Test Coverage:**
- All 10 pattern types with comprehensive scenarios
- LinearApply constructor patterns (single-arg, multi-arg, mixed chains)
- Variable shadowing prevention
- Deterministic binding order and serialization
- Round-trip property, idempotence testing
- Nested patterns, partial record matching
- Complex structural matching cases

### genesis_graph (18/18 tests ✓)
**GenesisGraph DAG:**
- Content-addressable directed acyclic graph (DAG)
- Root node creation with ⊙₀ symbol and backward-compatible validation
- Graph operations: insert, delete, link, query
- Cycle detection via depth-first search (DFS)
- Topological sort via Kahn's algorithm
- Lineage path tracking via breadth-first search (BFS)
- Canonical CBOR serialization with deterministic ordering
- Comprehensive error handling with 7 error types

**Dependencies:**
- `serde_cbor = "0.11"` - CBOR serialization
- `sha2 = "0.10"` - SHA-256 hashing
- `hex = "0.4"` - Hexadecimal encoding
- References capsule_core for Expression and ContentAddressable trait

**Test Coverage:**
- Root node creation and validation
- Graph creation with backward compatibility
- Node insertion with root reference enforcement
- Edge linking with cycle detection
- Node deletion with edge cleanup
- Topological sorting (valid DAG and cycle detection)
- Lineage path tracking from root
- Canonical serialization stability
- Comprehensive integration test (11 nodes, 10 edges)

### capsule_core (10/10 tests ✓)
**Root Capsule Functionality:**
- Ed25519 cryptographic signing (ed25519-dalek 2.1)
- SHA-256 hashing with GlyphV1 prefix
- Canonical CBOR serialization (deterministic encoding)
- Root Capsule creation (⊙₀) with signature verification
- Zero-knowledge proof generation and validation

**Content-Addressable Hashing (Work Order 4):**
- `ContentAddressable` and `CanonicalSerialize` traits
- Domain types: `Glyph`, `Expression`, `GraphNode` with references
- Type-specific hash prefixes (GlyphV1, ExprV1, NodeV1)
- Deterministic CBOR serialization via serde_cbor
- Comprehensive testing of hash determinism and prefix handling

### glyph_lexer (92/92 tests ✓)
- Unicode-aware tokenization (unicode-xid 0.2)
- Deterministic canonicalization (CRLF→LF, whitespace normalization)
- Complete token support:
  - Identifiers (Unicode XID_Start/Continue)
  - Numeric literals: hex (0x), binary (0b), octal (0o), decimal, float
  - String/char literals with Unicode escapes (\u{NNNN})
  - Nested block comments (/* ... */)
  - Line comments (//)
  - Operators (longest-match parsing)
  - Delimiters: ( ) { } [ ] ; , .
- Comprehensive error reporting with spans
- 92 unit tests covering all edge cases

### glyph_parser (36/36 tests ✓)
- Complete AST definition:
  - Expression: Literal, Var, Lambda, Apply, LinearApply, Let, Match, Tuple, List, Record
  - Literal: Int, Float, String, Bool, Unit
  - Pattern: Wildcard, Var, Literal, Tuple, Constructor
  - MatchArm: pattern + optional guard + body
- Lexer supporting GΛLYPH syntax:
  - Keywords: let, in, match, if, then, else, true, false
  - Special symbols: λ (lambda), ⊸ (linear arrow), # (comments)
  - All numeric types including negative numbers
  - String escapes: \n, \t, \r, \\, \"
- Recursive descent parser with precedence:
  - Let bindings with proper scoping
  - Pattern matching with guards
  - Lambda abstractions
  - Function application (both regular and linear)
  - Tuples, lists, and records
- Canonical CBOR serialization via ciborium
- 36 unit tests including 120 comprehensive cases
- All round-trip serialization tests passing

## Dependencies

### capsule_core
- `ed25519-dalek = "2.1"` - Ed25519 signatures
- `serde = { version = "1", features = ["derive"] }` - Serialization
- `ciborium = "0.2"` - CBOR encoding (Root Capsule)
- `serde_cbor = "0.11"` - CBOR encoding (Work Order 4)
- `serde_bytes = "0.11"` - Efficient byte serialization
- `sha2 = "0.10"` - SHA-256 hashing
- `hex = "0.4"` - Hexadecimal encoding
- `rand = "0.8"` - Random number generation

### glyph_lexer
- `unicode-xid = "0.2"` - Unicode identifier validation

### glyph_parser
- `serde = { version = "1.0", features = ["derive"] }` - Serialization
- `ciborium = "0.2"` - CBOR serialization
- `thiserror = "1.0"` - Error handling

### glyph_engine
- `serde = { version = "1.0", features = ["derive"] }` - Serialization
- `ciborium = "0.2"` - CBOR encoding
- `proptest = "1.4"` - Property-based testing (dev)

## Testing
All tests must run with `--test-threads=1` for deterministic validation:
```bash
cargo test --workspace -- --test-threads=1
```

**Current Test Results: 245 tests passing**
- capsule_core: 10 tests ✓
- genesis_graph: 18 tests ✓
- glyph_engine: 89 tests ✓ (42 pattern matching + 47 substitution)
- glyph_lexer: 92 tests ✓
- glyph_parser: 36 tests ✓

## Design Decisions

### glyph_lexer
- **Comment tokens**: Emits canonicalized Comment tokens rather than stripping them completely
- **Position tracking**: Maintains accurate byte-offset spans for all tokens
- **Deterministic behavior**: All lexing operations are deterministic for reproducibility

### glyph_parser
- **Match subject restriction**: Match subjects limited to primary expressions to avoid ambiguity with match arm braces. Complex subjects require parentheses: `match (f x) { ... }`
- **Record arguments**: Fully supported in function applications: `f { x: 1 }`  
- **Lambda body parsing**: Uses full expression parsing to support nested let/match expressions
- **Error handling**: Comprehensive ParseError and LexError types with descriptive messages
- **Canonical serialization**: CBOR-based deterministic serialization for AST persistence

### capsule_core
- **Deterministic behavior**: All cryptographic operations and content hashing are deterministic for reproducibility
- **Error handling**: Comprehensive error types for verification failures
- **Dual CBOR libraries**: Uses `ciborium` for Root Capsule (RFC 8949) and `serde_cbor` for Work Order 4 types
- **Hash prefixes**: Type-specific prefixes prevent hash collisions across domain types (GlyphV1:, ExprV1:, NodeV1:)
- **Backward compatibility**: Legacy `compute_content_hash()` preserved for existing Root Capsule code

### genesis_graph
- **Root node bootstrapping**: Root nodes stored with empty `root_ref` to avoid circular dependency
- **Backward compatibility**: Accepts both empty and pre-hashed `root_ref` values during graph creation
- **Deterministic hashing**: Root hash always computed with `root_ref = ""` for consistency
- **Cycle prevention**: DFS-based cycle detection prevents DAG corruption
- **Canonical serialization**: BTreeMap for nodes and sorted edges ensures deterministic CBOR output
- **Hash computation**: Uses `GlyphV1:Root:` and `GlyphV1:Node:` prefixes to distinguish node types

### glyph_engine
- **BTreeMap for Bindings**: Uses BTreeMap instead of HashMap to ensure deterministic serialization order
- **Variable consistency**: Prevents patterns binding the same variable to different values (shadowing check)
- **LinearApply support**: Constructor pattern matching handles both Apply and LinearApply nodes
- **Partial record matching**: Record patterns can match a subset of fields (structural subtyping)
- **Bind pattern semantics**: x@P binds the value to x AND matches the nested pattern P
- **Constructor unwinding**: Recursively unwraps nested Apply/LinearApply chains for multi-arg constructors
- **Atomic gensym**: Uses AtomicUsize for thread-safe fresh name generation with deterministic testing
- **Capture avoidance**: Automatic α-renaming prevents variable capture during substitution
- **Scope tracking**: Maintains bound variable sets to handle shadowing correctly
- **Match arm handling**: Special logic for pattern-bound variables in match expressions
