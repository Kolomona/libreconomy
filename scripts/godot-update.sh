#!/bin/bash
set -euo pipefail

# Load environment from .env if present (project root)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ENV_FILE="$SCRIPT_DIR/../.env"

if [ ! -f "$ENV_FILE" ]; then
	echo "Error: .env file not found at $ENV_FILE"
	exit 1
fi

# Source environment variables
set -a
# shellcheck disable=SC1090
. "$ENV_FILE"
set +a

# Verify GODOT_TEST_PROJECT_DIR is set
if [ -z "${GODOT_TEST_PROJECT_DIR:-}" ]; then
	echo "Error: GODOT_TEST_PROJECT_DIR not set in .env"
	exit 1
fi

# Verify the Godot project directory exists
if [ ! -d "$GODOT_TEST_PROJECT_DIR" ]; then
	echo "Error: Godot project directory does not exist: $GODOT_TEST_PROJECT_DIR"
	exit 1
fi

# Source and destination paths
DIST_DIR="$SCRIPT_DIR/../dist/godot4"
TARGET_DIR="$GODOT_TEST_PROJECT_DIR/addons/libreconomy"

echo "Updating Godot project at: $GODOT_TEST_PROJECT_DIR"
echo "Source: $DIST_DIR"
echo "Target: $TARGET_DIR"

# Create target directory if it doesn't exist
mkdir -p "$TARGET_DIR"

# Copy the three required files
echo "Copying files..."

# 1. Copy the Rust library
if [ -f "$DIST_DIR/lib/liblibreconomy.so" ]; then
	cp "$DIST_DIR/lib/liblibreconomy.so" "$TARGET_DIR/"
	echo "  ✓ Copied liblibreconomy.so"
else
	echo "  ✗ Error: liblibreconomy.so not found in $DIST_DIR/lib/"
	exit 1
fi

# 2. Copy the Godot bridge
if [ -f "$DIST_DIR/build/libreconomy_gdextension.so" ]; then
	cp "$DIST_DIR/build/libreconomy_gdextension.so" "$TARGET_DIR/"
	echo "  ✓ Copied libreconomy_gdextension.so"
else
	echo "  ✗ Error: libreconomy_gdextension.so not found in $DIST_DIR/build/"
	exit 1
fi

# 3. Copy the GDExtension config
if [ -f "$DIST_DIR/bridge/libreconomy.gdextension" ]; then
	cp "$DIST_DIR/bridge/libreconomy.gdextension" "$TARGET_DIR/"
	echo "  ✓ Copied libreconomy.gdextension"
else
	echo "  ✗ Error: libreconomy.gdextension not found in $DIST_DIR/bridge/"
	exit 1
fi

# Verify files were copied
echo ""
echo "Files in $TARGET_DIR:"
ls -lh "$TARGET_DIR"

echo ""
echo "✓ Update complete!"
echo ""
echo "Next steps:"
echo "  1. Restart Godot editor to reload the extension"
echo "  2. Run your test script to verify functionality"
