# CapsuleOS - Meta-Operating System Toolchain

## Project Overview
CapsuleOS is a new meta-operating system with a cryptographic foundation and custom language (GΛLYPH). This repository contains the core Rust implementation of three foundational components:

1. **capsule_core** - Cryptographic foundation for the Root Capsule (⊙₀)
2. **glyph_lexer** - Tokenizer/lexer for the GΛLYPH language
3. **glyph_parser** - Recursive descent parser and AST for GΛLYPH

## Recent Changes

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

### capsule_core (4/4 tests ✓)
- Ed25519 cryptographic signing (ed25519-dalek 2.1)
- SHA-256 hashing with GlyphV1 prefix
- Canonical CBOR serialization (deterministic encoding)
- Root Capsule creation (⊙₀) with signature verification
- Zero-knowledge proof generation and validation

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
- `serde_cbor = "0.11"` - CBOR encoding
- `sha2 = "0.10"` - SHA-256 hashing
- `rand = "0.8"` - Random number generation

### glyph_lexer
- `unicode-xid = "0.2"` - Unicode identifier validation

### glyph_parser
- `serde = { version = "1.0", features = ["derive"] }` - Serialization
- `ciborium = "0.2"` - CBOR serialization
- `thiserror = "1.0"` - Error handling

## Testing
All tests must run with `--test-threads=1` for deterministic validation:
```bash
cargo test --workspace -- --test-threads=1
```

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
- **Deterministic behavior**: All cryptographic operations are deterministic for reproducibility
- **Error handling**: Comprehensive error types for verification failures
