#!/bin/bash
# Build script for WASM package using wasm-pack
#
# Usage:
#   ./scripts/build-wasm.sh [target]
#
# Targets:
#   web      - Build for web (default)
#   bundler  - Build for bundlers (webpack, etc.)
#   nodejs   - Build for Node.js
#
# Output will be in pkg/ directory

set -e

TARGET=${1:-web}

echo "Building libreconomy WASM package for target: $TARGET"

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "Error: wasm-pack is not installed"
    echo "Install it with: cargo install wasm-pack"
    exit 1
fi

# Build the WASM package
wasm-pack build \
    --target "$TARGET" \
    --features wasm

echo "✓ Build complete!"
echo "Output: pkg/"
echo ""
echo "Files generated:"
ls -lh pkg/

# Copy to libreterra example
if [ -d "examples/libreterra-p5js" ]; then
    echo ""
    echo "Copying to libreterra-p5js..."
    mkdir -p examples/libreterra-p5js/pkg
    cp -r pkg/* examples/libreterra-p5js/pkg/
    echo "✓ Copied to examples/libreterra-p5js/pkg/"
fi

echo ""
echo "To use in a web project:"
echo "  import init, { WasmWorld } from './pkg/libreconomy.js';"
echo "  await init();"
echo "  const world = new WasmWorld();"
