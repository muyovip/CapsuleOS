#!/bin/bash

# Build script for GΛLYPH Parser WebAssembly module

set -e

echo "🔨 Building GΛLYPH Parser WebAssembly module..."

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "❌ wasm-pack not found. Installing..."
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
fi

# Build the WebAssembly package
echo "📦 Building WebAssembly package..."
wasm-pack build \
    --target web \
    --out-dir pkg \
    --scope capsuleos \
    --dev

echo "✅ WebAssembly build complete!"
echo ""
echo "📁 Generated files:"
echo "   - pkg/glyph_parser_wasm.js (JavaScript bindings)"
echo "   - pkg/glyph_parser_wasm_bg.wasm (WebAssembly binary)"
echo "   - pkg/glyph_parser_wasm.d.ts (TypeScript definitions)"
echo ""
echo "🚀 To use in your frontend:"
echo "   1. Copy the pkg directory to your frontend"
echo "   2. Import the module: import init from './pkg/glyph_parser_wasm.js'"
echo "   3. Initialize: const wasm = await init()"
echo "   4. Use the parser: const result = wasm.parse_expression('λx -> x')"