# FFI (Foreign Function Interface) Documentation

This document describes how to use libreconomy from languages other than Rust.

## Table of Contents

- [Overview](#overview)
- [C/C++ Integration](#cc-integration)
- [Python Integration](#python-integration)
- [Swift Integration](#swift-integration)
- [Kotlin Integration](#kotlin-integration)
- [Building for FFI](#building-for-ffi)

## Overview

libreconomy provides two types of FFI support:

1. **C FFI** (via cbindgen): For C and C++ integration
2. **High-level bindings** (via uniffi): For Python, Swift, and Kotlin

### Architecture

- Rust code uses `#[uniffi::export]` attributes to expose functions
- uniffi generates bindings from the compiled library metadata
- cbindgen generates C headers from Rust source

## C/C++ Integration

### Building

Generate the C header file:

```bash
cargo build --release
cbindgen --config cbindgen.toml --crate libreconomy --output libreconomy.h
```

Or use the release script:

```bash
bash scripts/release.sh
```

### Using from C

```c
#include "libreconomy.h"
#include <stdio.h>

int main() {
    const char* version = libreconomy_version();
    printf("libreconomy version: %s\n", version);
    return 0;
}
```

### Linking

Linux:
```bash
gcc -o myapp main.c -L./target/release -llibreconomy
export LD_LIBRARY_PATH=./target/release:$LD_LIBRARY_PATH
./myapp
```

macOS:
```bash
gcc -o myapp main.c -L./target/release -llibreconomy
export DYLD_LIBRARY_PATH=./target/release:$DYLD_LIBRARY_PATH
./myapp
```

Windows:
```bash
gcc -o myapp.exe main.c -L./target/release -llibreconomy
# Copy liblibreconomy.dll to the same directory as myapp.exe
myapp.exe
```

### Using from C++

```cpp
#include "libreconomy.h"
#include <iostream>

int main() {
    const char* version = libreconomy_version();
    std::cout << "libreconomy version: " << version << std::endl;
    return 0;
}
```

### Godot Integration (GDNative/GDExtension)

1. Build the shared library
2. Copy to your Godot project's `lib/` directory
3. Create a GDNative/GDExtension configuration file
4. Load and call FFI functions from GDScript

Example GDScript:

```gdscript
extends Node

var libreconomy

func _ready():
    # Load the native library
    libreconomy = GDNative.new()
    libreconomy.library = load("res://lib/libreconomy.gdnlib")
    libreconomy.initialize()
    
    # Call FFI function
    var version = libreconomy.call_native("standard_varcall", "libreconomy_version", [])
    print("libreconomy version: ", version)
```

## Python Integration

### Building

Generate Python bindings:

```bash
cargo build --release
uniffi-bindgen generate --library target/release/liblibreconomy.so --language python --out-dir dist
```

Or use the release script:

```bash
bash scripts/release.sh
```

### Installation

Copy the generated files to your Python project:

```bash
cp dist/libreconomy.py your_project/
cp dist/liblibreconomy.so your_project/
```

Or add to your `sys.path`:

```python
import sys
sys.path.insert(0, '/path/to/libreconomy/dist')
```

### Usage

```python
import libreconomy

# Get library version
version = libreconomy.libreconomy_version()
print(f"libreconomy version: {version}")

# Get agent count
count = libreconomy.get_agent_count()
print(f"Agent count: {count}")
```

### Complete Example

```python
#!/usr/bin/env python3
import sys
import os

# Add dist directory to path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'dist'))

import libreconomy

def main():
    print("=== libreconomy Python Integration ===")
    
    # Display version
    version = libreconomy.libreconomy_version()
    print(f"Version: {version}")
    
    # Get agent count
    count = libreconomy.get_agent_count()
    print(f"Current agent count: {count}")

if __name__ == "__main__":
    main()
```

### Type Hints

The generated Python bindings include type hints:

```python
def libreconomy_version() -> str: ...
def get_agent_count() -> int: ...
```

## Swift Integration

### Building

Generate Swift bindings:

```bash
cargo build --release
uniffi-bindgen generate --library target/release/liblibreconomy.dylib --language swift --out-dir dist
```

Or use the release script:

```bash
bash scripts/release.sh
```

### Usage

```swift
import Foundation
import libreconomy

// Get library version
let version = libreconomyVersion()
print("libreconomy version: \(version)")

// Get agent count
let count = getAgentCount()
print("Agent count: \(count)")
```

### Xcode Integration

1. Add the generated `.swift` file to your Xcode project
2. Add `libreconomyFFI.h` and `libreconomyFFI.modulemap` to your bridging header
3. Link against the `.dylib` library
4. Add the library directory to your library search paths

### iOS Integration

For iOS, you'll need to build libreconomy for the target architecture:

```bash
# Install target
rustup target add aarch64-apple-ios

# Build for iOS
cargo build --release --target aarch64-apple-ios

# Generate bindings
uniffi-bindgen generate \
    --library target/aarch64-apple-ios/release/liblibreconomy.a \
    --language swift \
    --out-dir dist-ios
```

## Kotlin Integration

### Building

Generate Kotlin bindings:

```bash
cargo build --release
uniffi-bindgen generate --library target/release/liblibreconomy.so --language kotlin --out-dir dist
```

Or use the release script:

```bash
bash scripts/release.sh
```

### Usage

```kotlin
import libreconomy.*

fun main() {
    // Get library version
    val version = libreconomyVersion()
    println("libreconomy version: $version")
    
    // Get agent count
    val count = getAgentCount()
    println("Agent count: $count")
}
```

### Android Integration

1. Build libreconomy for Android targets:

```bash
# Install targets
rustup target add aarch64-linux-android
rustup target add armv7-linux-androideabi
rustup target add x86_64-linux-android

# Build for Android (requires NDK)
cargo build --release --target aarch64-linux-android
```

2. Add the `.so` files to your Android project's `jniLibs/` directory:

```
app/src/main/jniLibs/
├── arm64-v8a/
│   └── liblibreconomy.so
├── armeabi-v7a/
│   └── liblibreconomy.so
└── x86_64/
    └── liblibreconomy.so
```

3. Add the generated Kotlin file to your project's source directory

4. Use in your Android app:

```kotlin
class MainActivity : AppCompatActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        
        val version = libreconomyVersion()
        Log.d("libreconomy", "Version: $version")
    }
}
```

## Building for FFI

### One-Command Build

The easiest way to build all FFI bindings:

```bash
bash scripts/release.sh
```

This will:
1. Build the release library
2. Generate the C header
3. Generate Python, Swift, and Kotlin bindings
4. Package everything in the `dist/` directory

### Manual Build Steps

If you prefer to build manually:

1. **Build the library:**
   ```bash
   cargo build --release
   ```

2. **Generate C header:**
   ```bash
   cbindgen --config cbindgen.toml --crate libreconomy --output libreconomy.h
   ```

3. **Generate language bindings:**
   ```bash
   # Python
   uniffi-bindgen generate --library target/release/liblibreconomy.so --language python --out-dir dist
   
   # Swift (macOS)
   uniffi-bindgen generate --library target/release/liblibreconomy.dylib --language swift --out-dir dist
   
   # Kotlin
   uniffi-bindgen generate --library target/release/liblibreconomy.so --language kotlin --out-dir dist
   ```

### Platform-Specific Libraries

The library name varies by platform:

- **Linux**: `liblibreconomy.so`
- **macOS**: `liblibreconomy.dylib`
- **Windows**: `libreconomy.dll`

### Cross-Compilation

For cross-compilation, install the target and build:

```bash
# Example: Build for Windows from Linux
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
```

## Current API

### Available Functions

Currently, the FFI exposes these functions:

#### `libreconomy_version() -> String`

Returns the version string of the library.

**Example (Python):**
```python
version = libreconomy.libreconomy_version()
print(version)  # "0.0.1"
```

#### `get_agent_count() -> u32`

Returns the current number of agents in the simulation.

**Example (Python):**
```python
count = libreconomy.get_agent_count()
print(count)  # 0 (placeholder implementation)
```

### Future API

The FFI layer is being expanded to include:

- Agent creation functions
- Component access functions
- Simulation step functions
- Query functions

Stay tuned for updates!

## Troubleshooting

### Library Not Found

**Python:**
```python
# Make sure library is in the same directory as the .py file
# or use absolute paths
import os
import sys
sys.path.insert(0, os.path.abspath('path/to/dist'))
```

**C/C++:**
```bash
# Set library path
export LD_LIBRARY_PATH=/path/to/library:$LD_LIBRARY_PATH  # Linux
export DYLD_LIBRARY_PATH=/path/to/library:$DYLD_LIBRARY_PATH  # macOS
```

### Version Mismatches

Always regenerate bindings after recompiling the library:

```bash
cargo clean
bash scripts/release.sh
```

### Symbol Not Found

Ensure you're using the correct library for your platform and architecture.

## Best Practices

1. **Always use the release script** for consistent builds
2. **Version your bindings** along with your library
3. **Test on target platforms** before deployment
4. **Document platform requirements** for users
5. **Handle errors gracefully** in FFI code

## Additional Resources

- [uniffi documentation](https://mozilla.github.io/uniffi-rs/)
- [cbindgen documentation](https://github.com/eqrion/cbindgen)
- [Rust FFI guide](https://doc.rust-lang.org/nomicon/ffi.html)
