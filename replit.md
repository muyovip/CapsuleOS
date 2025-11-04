# CapsuleOS - Meta-Operating System Toolchain

## Project Overview
CapsuleOS is a new meta-operating system with a cryptographic foundation and custom language (GΛLYPH). This repository contains the core Rust implementation of two foundational components:

1. **capsule_core** - Cryptographic foundation for the Root Capsule (⊙₀)
2. **glyph_lexer** - Tokenizer/lexer for the GΛLYPH language

## Recent Changes

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

## Dependencies
- `ed25519-dalek = "2.1"` - Ed25519 signatures
- `serde = { version = "1", features = ["derive"] }` - Serialization
- `serde_cbor = "0.11"` - CBOR encoding
- `sha2 = "0.10"` - SHA-256 hashing
- `rand = "0.8"` - Random number generation
- `unicode-xid = "0.2"` - Unicode identifier validation

## Testing
All tests must run with `--test-threads=1` for deterministic validation:
```bash
cargo test --workspace -- --test-threads=1
```

## Design Decisions
- **Comment tokens**: Emits canonicalized Comment tokens rather than stripping them completely
- **Deterministic behavior**: All cryptographic operations and lexing are deterministic for reproducibility
- **Position tracking**: Lexer maintains accurate byte-offset spans for all tokens
- **Error handling**: Comprehensive ParseError with position information
