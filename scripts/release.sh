#!/bin/bash
set -euo pipefail

# Load environment from .env if present (project root)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ENV_FILE="$SCRIPT_DIR/../.env"
if [ -f "$ENV_FILE" ]; then
	set -a
	# shellcheck disable=SC1090
	. "$ENV_FILE"
	set +a
	echo "Loaded environment from $ENV_FILE"
	if [ -n "${GODOT_CPP_DIR:-}" ]; then
		echo "GODOT_CPP_DIR=${GODOT_CPP_DIR}"
	fi
fi

# Build release
cargo build --release

# Run FFI completeness tests (verify API exports before generating artifacts)
echo "Running FFI completeness tests..."
cargo test --test ffi_completeness --quiet
echo "FFI completeness verified ✓"

# Build Rust API documentation (no dependencies)
echo "Generating Rust documentation..."
cargo doc --no-deps
echo "Documentation generated in target/doc/"

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

# Package documentation
if [ -d "target/doc" ]; then
	echo "Packaging documentation..."
	rm -rf dist/docs
	mkdir -p dist/docs
	cp -r target/doc/* dist/docs/
else
	echo "Warning: target/doc not found; documentation may have failed to build."
fi

# Print summary
ls -lh dist/
echo "Release artifacts are in dist/"

# -------------------------------------------------------------
# Godot 4 scaffold (optional auto-build if environment present)
# -------------------------------------------------------------

echo "Preparing Godot 4 scaffold in dist/godot4..."

# Create structure
G4_DIR="dist/godot4"
mkdir -p "$G4_DIR/lib" "$G4_DIR/bridge" "$G4_DIR/build"

# Copy Rust artifacts
cp target/release/liblibreconomy.* "$G4_DIR/lib/" 2>/dev/null || true
cp libreconomy.h "$G4_DIR/lib/"

# Emit bridge sources
cat > "$G4_DIR/bridge/libreconomy_bridge.cpp" <<'CPP'
#include <godot_cpp/classes/ref_counted.hpp>
#include <godot_cpp/core/class_db.hpp>
#include <godot_cpp/godot.hpp>
#include <gdextension_interface.h>
#include "libreconomy.h"
using namespace godot;

/// Wrapper for libreconomy World with full agent management API
class LibreWorld : public RefCounted {
	GDCLASS(LibreWorld, RefCounted);
private:
	WorldHandle* world_ptr = nullptr;
	
protected:
	static void _bind_methods() {
		ClassDB::bind_method(D_METHOD("create_agent"), &LibreWorld::create_agent);
		ClassDB::bind_method(D_METHOD("create_agent_with_needs", "thirst", "hunger"), &LibreWorld::create_agent_with_needs);
		ClassDB::bind_method(D_METHOD("create_agent_with_wallet", "currency"), &LibreWorld::create_agent_with_wallet);
		ClassDB::bind_method(D_METHOD("create_agent_full", "thirst", "hunger", "currency"), &LibreWorld::create_agent_full);
		ClassDB::bind_method(D_METHOD("remove_agent", "entity_id"), &LibreWorld::remove_agent);
		ClassDB::bind_method(D_METHOD("get_agent_count"), &LibreWorld::get_agent_count);
		ClassDB::bind_method(D_METHOD("get_version"), &LibreWorld::get_version);
	}
	
public:
	LibreWorld() {
		world_ptr = create_world();
	}
	
	~LibreWorld() {
		if (world_ptr) {
			destroy_world(world_ptr);
			world_ptr = nullptr;
		}
	}
	
	String get_version() const {
		const uint8_t *v = libreconomy_version();
		return String(reinterpret_cast<const char*>(v));
	}
	
	int64_t create_agent() {
		if (!world_ptr) return 0;
		return (int64_t)create_agent_default(world_ptr);
	}
	
	int64_t create_agent_with_needs(double thirst, double hunger) {
		if (!world_ptr) return 0;
		return (int64_t)::create_agent_with_needs(world_ptr, thirst, hunger);
	}
	
	int64_t create_agent_with_wallet(double currency) {
		if (!world_ptr) return 0;
		return (int64_t)::create_agent_with_wallet(world_ptr, currency);
	}
	
	int64_t create_agent_full(double thirst, double hunger, double currency) {
		if (!world_ptr) return 0;
		return (int64_t)::create_agent_full(world_ptr, thirst, hunger, currency);
	}
	
	bool remove_agent(int64_t entity_id) {
		if (!world_ptr) return false;
		return ::remove_agent(world_ptr, (uint64_t)entity_id) != 0;
	}
	
	int64_t get_agent_count() const {
		if (!world_ptr) return 0;
		return (int64_t)::get_agent_count(world_ptr);
	}
};

void initialize_libreconomy_module(ModuleInitializationLevel p_level) {
	if (p_level != MODULE_INITIALIZATION_LEVEL_SCENE) { return; }
	ClassDB::register_class<LibreWorld>();
}

void uninitialize_libreconomy_module(ModuleInitializationLevel p_level) {
	if (p_level != MODULE_INITIALIZATION_LEVEL_SCENE) { return; }
}

extern "C" {
GDExtensionBool GDE_EXPORT libreconomy_library_init(GDExtensionInterfaceGetProcAddress p_get_proc_address,
												   GDExtensionClassLibraryPtr p_library,
												   GDExtensionInitialization *r_initialization) {
	GDExtensionBinding::InitObject init_obj(p_get_proc_address, p_library, r_initialization);
	init_obj.register_initializer(initialize_libreconomy_module);
	init_obj.register_terminator(uninitialize_libreconomy_module);
	init_obj.set_minimum_library_initialization_level(MODULE_INITIALIZATION_LEVEL_SCENE);
	return init_obj.init();
}
}
CPP

cat > "$G4_DIR/bridge/libreconomy.gdextension" <<'GDE'
[configuration]
entry_symbol = "libreconomy_library_init"
compatibility_minimum = 4.1

[libraries]
linux.debug.x86_64 = "res://addons/libreconomy/libreconomy_gdextension.so"
linux.release.x86_64 = "res://addons/libreconomy/libreconomy_gdextension.so"
GDE

cat > "$G4_DIR/bridge/SConstruct" <<'SCONS'
import os, glob
from SCons.Script import Environment, Exit

env = Environment(ENV=os.environ)

godot_cpp_dir = os.environ.get('GODOT_CPP_DIR')
if not godot_cpp_dir:
	print('Warning: GODOT_CPP_DIR not set. Skipping auto-build. See README in dist/godot4 for instructions.')
	Exit(0)

# Basic compile/link flags for a GDExtension on Linux
env.Append(CXXFLAGS=['-std=c++17', '-fPIC', '-O2'])
env.Append(CPPDEFINES=['NDEBUG'])

# Include + link search paths
env.Append(CPPPATH=[
	f"{godot_cpp_dir}/include",
	f"{godot_cpp_dir}/include/godot_cpp",
	f"{godot_cpp_dir}/gen/include",
	f"{godot_cpp_dir}/gdextension",
	"../lib",
])
env.Append(LIBPATH=[f"{godot_cpp_dir}/bin", "../lib"])  # godot-cpp + libreconomy lib path

# Try to find a godot-cpp static library in bin
candidates = []
candidates += glob.glob(os.path.join(godot_cpp_dir, 'bin', 'libgodot-cpp.*release*.x86_64*.a'))
candidates += glob.glob(os.path.join(godot_cpp_dir, 'bin', 'libgodot-cpp.*x86_64*.a'))
candidates += glob.glob(os.path.join(godot_cpp_dir, 'bin', 'libgodot-cpp*.a'))
godot_cpp_lib = candidates[0] if candidates else None

if not godot_cpp_lib:
	print('Error: Could not find libgodot-cpp static library in GODOT_CPP_DIR/bin')
	Exit(1)

# Link against libreconomy using explicit path as File node
this_dir = os.getcwd()
libre_path = os.path.abspath(os.path.join(this_dir, '..', 'lib', 'liblibreconomy.so'))
env.Append(LIBS=[File(libre_path)])

# Add godot-cpp static library as a File node so SCons links it properly
env.Append(LIBS=[File(godot_cpp_lib)])

# Set rpath so the bridge finds liblibreconomy.so at runtime when colocated
env.Append(LINKFLAGS=['-Wl,-rpath,$ORIGIN'])

sources = ['libreconomy_bridge.cpp']
target = env.SharedLibrary(target='../build/libreconomy_gdextension', source=sources)
Default(target)
SCONS

# README for users
cat > "$G4_DIR/README.md" <<'MD'
# Godot 4 Integration Bundle

This folder contains everything needed to integrate `libreconomy` with Godot 4.5.1 on Linux x86_64.

Contents:
- lib/: Rust shared library (`liblibreconomy.*`) and header (`libreconomy.h`)
- bridge/: C++ GDExtension wrapper sources (`libreconomy_bridge.cpp`, `libreconomy.gdextension`, `SConstruct`)
- build/: output location for the compiled GDExtension (`libreconomy_gdextension.so`)

Quick start:
1. Ensure you have godot-cpp compiled for your Godot v4.5.1, and set `GODOT_CPP_DIR` to its location.
2. Install SCons (`scons` in PATH).
3. From this folder, run:
   scons -C bridge
4. Copy `build/libreconomy_gdextension.so` and `lib/liblibreconomy.so` (plus `libreconomy.h` if needed) into your Godot project under `res://native/libreconomy/`.
5. Add `bridge/libreconomy.gdextension` to your project and reference the built .so as shown.

Optional:
- If `GODOT_CPP_DIR` is not set, the build step is skipped. Use the scaffold to build manually later.
- For other platforms/versions, use these sources as a starting point.
MD

echo "Godot 4 scaffold prepared at $G4_DIR."

# Optional: attempt to auto-build the bridge if dependencies are present
if [ -n "${GODOT_CPP_DIR:-}" ] && command -v scons >/dev/null 2>&1; then
	echo "Auto-building Godot bridge with SCons..."
	if (cd "$G4_DIR/bridge" && scons); then
		echo "Godot bridge built: $G4_DIR/build/libreconomy_gdextension.so"
	else
		echo "Warning: Godot bridge build failed. You can build manually: scons -C $G4_DIR/bridge"
	fi
else
	echo "Skipping auto-build (missing GODOT_CPP_DIR or scons). Build manually: scons -C $G4_DIR/bridge"
fi
