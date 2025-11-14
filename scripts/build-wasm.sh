#!/bin/bash
# WASM build script for CapsuleOS Rust engine
set -e
echo "🔨 Building CapsuleOS Rust engine for WASM..."
# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "📦 Installing wasm-pack..."
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
fi
# Add WASM target if not already present
rustup target add wasm32-unknown-unknown
# Build the main workspace
echo "🏗️  Building workspace..."
cargo build --release
# Build GΛLYPH parser for WASM
echo "🎮 Building GΛLYPH parser for web..."
cd glyph_parser
wasm-pack build --target web --out-dir pkg/wasm --release
# Validate WASM output
echo "✅ Validating WASM output..."
find pkg/wasm -name "*.wasm" -exec wasm-validate {} \;
# Copy to a common location
mkdir -p ../dist/wasm
cp -r pkg/wasm/* ../dist/wasm/
echo "🚀 WASM build complete!"
echo "📦 WASM files available in dist/wasm/"
