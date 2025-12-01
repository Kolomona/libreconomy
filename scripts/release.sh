#!/bin/bash
set -e

# Build release
cargo build --release

# Generate C header
cbindgen --config cbindgen.toml --crate libreconomy --output libreconomy.h

# Generate uniffi bindings
uniffi-bindgen generate src/libreconomy.udl --language python
uniffi-bindgen generate src/libreconomy.udl --language kotlin
uniffi-bindgen generate src/libreconomy.udl --language swift

# Package artifacts
mkdir -p dist
cp target/release/liblibreconomy.* dist/
cp libreconomy.h dist/
cp src/libreconomy.udl dist/
cp -r python/ dist/ 2>/dev/null || true
cp -r kotlin/ dist/ 2>/dev/null || true
cp -r swift/ dist/ 2>/dev/null || true

# Print summary
ls -lh dist/
echo "Release artifacts are in dist/"
