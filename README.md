# CapsuleOS Build Toolchain

Core logic and toolchain for the CapsuleOS meta-operating system. This project contains the compiler, graph database, and command-line interface implemented in Rust.

## Project Structure

```
capsule-os-build/
├── src/
│   ├── main.rs              # CLI entry point
│   ├── cli/                 # Command-line interface module
│   │   └── mod.rs
│   ├── compiler/            # Compiler components
│   │   ├── mod.rs
│   │   ├── lexer.rs         # Lexical analysis
│   │   ├── parser.rs        # Syntax parsing
│   │   └── codegen.rs       # Code generation
│   └── database/            # Graph database
│       ├── mod.rs
│       ├── graph.rs         # Graph data structures
│       └── query_engine.rs  # Query execution
├── Cargo.toml               # Project manifest
├── rustfmt.toml             # Code formatting configuration
└── .clippy.toml             # Linting configuration
```

## Components

### 1. Compiler
The compiler module implements a three-stage compilation pipeline:
- **Lexer**: Tokenizes source code into lexical tokens
- **Parser**: Builds an Abstract Syntax Tree (AST) from tokens
- **Code Generator**: Produces bytecode from the AST

### 2. Graph Database
A lightweight graph database designed for the meta-OS:
- Node and edge-based data model
- JSON-based persistence
- Query engine for graph traversal

### 3. CLI
Command-line interface for interacting with all components:
- Compilation commands
- Database operations
- System information queries

## Building

### Prerequisites
- Rust 1.88.0 or later
- Cargo (comes with Rust)

### Build Commands

```bash
# Build the project
cargo build

# Build with optimizations
cargo build --release

# Run the CLI
cargo run -- --help

# Run tests
cargo test

# Format code
cargo fmt

# Run linter
cargo clippy
```

## Usage

### Compile a Source File
```bash
cargo run -- compile --input source.caps --output output.capsule
```

### Initialize a Database
```bash
cargo run -- database init my_database.json
```

### Query the Database
```bash
cargo run -- database query "MATCH (n) RETURN n"
```

### Display System Information
```bash
cargo run -- info
```

## Development

### Code Formatting
The project uses `rustfmt` for consistent code formatting. Configuration is in `rustfmt.toml`.

```bash
cargo fmt
```

### Linting
The project uses `clippy` for additional linting. Configuration is in `.clippy.toml`.

```bash
cargo clippy
```

### Dependencies
- `clap`: Command-line argument parsing
- `serde`: Serialization/deserialization
- `anyhow`: Error handling
- `thiserror`: Custom error types
- `tokio`: Async runtime

## License

MIT
