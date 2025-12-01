#!/bin/bash
set -euo pipefail

# Build release
cargo build --release

# Generate C header
cbindgen --config cbindgen.toml --crate libreconomy --output libreconomy.h

# Generate uniffi bindings (auto if tooling present; graceful fallback)
if command -v uniffi-bindgen >/dev/null 2>&1; then
	echo "uniffi-bindgen found: $(uniffi-bindgen --version || echo unknown version)"
		# Generate bindings from compiled library metadata
		LIB_PATH="target/release/liblibreconomy.so"
		if [ ! -f "$LIB_PATH" ]; then
			echo "Error: Compiled library not found at $LIB_PATH"
			exit 1
		fi
		OUT_DIR="dist"
				uniffi-bindgen generate --library --crate libreconomy --language python --out-dir "$OUT_DIR" "$LIB_PATH"
				uniffi-bindgen generate --library --crate libreconomy --language kotlin --out-dir "$OUT_DIR" "$LIB_PATH"
				uniffi-bindgen generate --library --crate libreconomy --language swift --out-dir "$OUT_DIR" "$LIB_PATH"
		echo "uniffi bindings generated successfully."
else
	echo "Error: uniffi-bindgen CLI not found. Please install a compatible version."
	exit 1
fi

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
