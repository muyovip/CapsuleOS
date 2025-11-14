#!/bin/bash
# GΛLYPH validation script for CapsuleOS Rust engine
set -e
echo "🔍 Validating GΛLYPH files in CapsuleOS..."
# Find all .glyph files
GLYPH_FILES=$(find . -name "*.glyph" -type f)
if [ -z "$GLYPH_FILES" ]; then
    echo "ℹ️  No .glyph files found in CapsuleOS repository"
    exit 0
fi
VALIDATION_FAILED=0
for glyph_file in $GLYPH_FILES; do
    echo "📝 Validating $glyph_file"
    # Check if file is readable
    if [ ! -r "$glyph_file" ]; then
        echo "❌ Cannot read $glyph_file"
        VALIDATION_FAILED=1
        continue
    fi
    # Validate syntax using our parser
    if cargo run --bin glyph_parser --validate "$glyph_file"; then
        echo "✅ $glyph_file validated successfully"
    else
        echo "❌ GΛLYPH validation failed for $glyph_file"
        VALIDATION_FAILED=1
    fi
done
if [ $VALIDATION_FAILED -eq 0 ]; then
    echo "🎉 All GΛLYPH files in CapsuleOS validated successfully!"
    exit 0
else
    echo "💥 Some GΛLYPH files failed validation"
    exit 1
fi
