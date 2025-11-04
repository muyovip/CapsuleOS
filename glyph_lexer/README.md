# GΛLYPH Lexer

A deterministic, canonical lexical analyzer for the GΛLYPH programming language.

## Overview

The GΛLYPH lexer converts source text into a stream of tokens with the following guarantees:

- **Deterministic**: identical input always produces identical token streams
- **Canonical**: formatting differences (whitespace, comment placement) don't affect semantic tokens
- **Unicode-safe**: full Unicode support for identifiers using Unicode XID rules
- **Locale-agnostic**: no locale-dependent operations

## API

### Main Entry Points

```rust
use glyph_lexer::tokenize;

// Tokenize a string
let tokens = tokenize("let x = 42;").unwrap();

// Tokenize bytes (validates UTF-8)
use glyph_lexer::tokenize_bytes;
let tokens = tokenize_bytes(source_bytes).unwrap();
```

### Token Types

All tokens are categorized under `TokenKind`:

- **Identifier**: Variable/function names (Unicode-aware)
- **IntegerLiteral**: Decimal, hex (0x), binary (0b), octal (0o)
- **FloatLiteral**: Decimal notation with optional exponent
- **StringLiteral**: Double-quoted strings with escape sequences
- **CharLiteral**: Single-quoted characters
- **Operator**: +, -, ==, !=, ->, ::, etc.
- **Symbol**: Multi-character punctuation
- **Delimiter**: (), {}, [], ;, ,, .
- **Comment**: Preserved from // and /* */ with canonical formatting
- **Eof**: End of input marker

## Canonicalization Rules

The lexer enforces strict canonicalization to ensure deterministic token streams:

### 1. Line Endings

All line endings are normalized to LF (`\n`) before lexing:
- CRLF (`\r\n`) → LF (`\n`)
- CR (`\r`) → LF (`\n`)

### 2. Whitespace

- Non-semantic whitespace is consumed and not emitted as tokens
- Multiple spaces/tabs are equivalent to a single space
- Tokens are separated by whitespace but whitespace itself is not tokenized

### 3. Comments

Comments are emitted as `TokenKind::Comment` tokens with canonicalized content:
- Leading/trailing whitespace is trimmed
- Internal line endings normalized to `\n`
- Multiple consecutive spaces collapsed to single space (preserving `\n`)
- Nested block comments are fully supported

**Example:**
```rust
"//   comment   " → Comment("comment")
"/* outer /* nested */ */  → Comment("outer /* nested */")
```

### 4. Identifiers

- Follow Unicode XID identifier rules
- Case-preserving (no case folding)
- Underscores allowed anywhere
- Not locale-normalized

**Examples:** `hello`, `_private`, `π_radius`, `变量`

### 5. Numeric Literals

Raw source is preserved, but a canonical value is computed:
- Underscores removed: `1_000_000` → canonical `1000000`
- Hex digits lowercased: `0xFF` → canonical `0xff`

**Supported formats:**
- Decimal: `42`, `1_000_000`
- Hexadecimal: `0xFF`, `0xDEAD_BEEF`
- Binary: `0b1010`, `0b1111_0000`
- Octal: `0o755`, `0o644`
- Float: `3.14`, `1e10`, `1.2e-3`

### 6. String/Char Literals

Escape sequences are resolved to canonical codepoints:

**Supported escapes:**
- `\n` - newline
- `\t` - tab
- `\r` - carriage return
- `\\` - backslash
- `\"` - double quote
- `\'` - single quote
- `\0` - null character
- `\xHH` - hex escape (2 digits)
- `\u{HHHHHH}` - Unicode escape (1-6 hex digits)

### 7. Spans

All tokens include a `Span { start: usize, end: usize }` with byte offsets into the **normalized** source (after line ending conversion).

## Operators

Longest-match rule is applied to multi-character operators. Operators are recognized in this priority order:

```
::  ->  =>  ==  !=  <=  >=  &&  ||  
+=  -=  *=  /=  %=  <<  >>
+   -   *   /   %   <   >   =   !   &   |   ^   ~   ?   :
```

## Error Handling

All lexical errors return `ParseError::Lexical` with:
- Descriptive error message
- Optional span indicating error location

**Common errors:**
- Unterminated string/char literal
- Unterminated block comment
- Invalid escape sequence
- Invalid numeric literal
- Invalid UTF-8 input
- Unexpected character

## Example Usage

```rust
use glyph_lexer::{tokenize, TokenKind};

let source = r#"
    // Calculate factorial
    fn factorial(n) -> int {
        if n <= 1 {
            return 1;
        }
        return n * factorial(n - 1);
    }
"#;

let tokens = tokenize(source).unwrap();

for token in tokens {
    match token.kind {
        TokenKind::Identifier(name) => println!("Identifier: {}", name),
        TokenKind::IntegerLiteral { canonical_value, .. } => {
            println!("Integer: {}", canonical_value)
        }
        TokenKind::Comment(text) => println!("Comment: {}", text),
        _ => {}
    }
}
```

## Determinism Guarantees

The lexer produces identical token streams for semantically equivalent source:

```rust
let src1 = "let x=1; // comment\n";
let src2 = "let   x  = 1 ;/* comment */";

let tokens1 = tokenize(src1).unwrap();
let tokens2 = tokenize(src2).unwrap();

// After filtering comments, token streams are identical
```

## Testing

Run the comprehensive test suite with deterministic ordering:

```bash
cargo test --package glyph_lexer -- --test-threads=1
```

The test suite includes 50+ tests covering:
- Basic tokenization
- Unicode identifiers
- Numeric literals (all formats)
- String/char escapes
- Comments (nested blocks)
- Operators (longest-match)
- Whitespace normalization
- Complex source files
- Determinism validation
- Edge cases and errors

## Performance

- O(n) time complexity over input characters
- Single-pass lexing
- No recursion (safe for large inputs)
- Efficient Unicode handling with `unicode-xid` crate

## Dependencies

- `unicode-xid 0.2` - Unicode XID identifier rules

## License

MIT
