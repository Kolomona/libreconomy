# libreconomy

A cross-platform, agent-based economy simulator library for games and applications.

## Key Features
- Agent-based simulation (ECS)
- Subjective value modeling
- Modular systems: market, labor, production, decision-making
- Supports FFI (C/C++, Python, Swift, Kotlin) via cbindgen and uniffi
- Planned language bindings for easy integration

## Build Instructions
```sh
cargo build --release
```

## Using uniffi for Language Bindings

1. **Build the dynamic library:**
   ```sh
   cargo build --release
   ```
   The shared library will be in `target/release/liblibreconomy.so` (Linux), `liblibreconomy.dylib` (macOS), or `liblibreconomy.dll` (Windows).

2. **Generate bindings with uniffi:**
   ```sh
   uniffi-bindgen generate src/libreconomy.udl --language python
   uniffi-bindgen generate src/libreconomy.udl --language kotlin
   uniffi-bindgen generate src/libreconomy.udl --language swift
   ```
   This creates bindings for Python, Kotlin, and Swift in the respective output directories.

3. **Example usage in Python:**
   ```python
   import libreconomy
   print(libreconomy.libreconomy_version())
   print(libreconomy.get_agent_count())
   ```

See `uniffi.toml` for configuration and `src/libreconomy.udl` for the API definition.

## Using in Godot

1. **Build the dynamic library:**
   ```sh
   cargo build --release
   ```
   The shared library will be in `target/release/liblibreconomy.so` (Linux), `liblibreconomy.dylib` (macOS), or `liblibreconomy.dll` (Windows).

2. **Generate C header with cbindgen:**
   ```sh
   cbindgen --config cbindgen.toml --crate libreconomy --output libreconomy.h
   ```
   This creates a C header file for FFI integration.

3. **In Godot:**
   - Use GDNative or GDExtension to load the shared library.
   - Use the generated header to call FFI functions (e.g., `libreconomy_version`).
   - Write Godot scripts to interact with the library via FFI.

4. **Example FFI function:**
   ```c
   const char* version = libreconomy_version();
   printf("libreconomy version: %s\n", version);
   ```

## Status
Early development. API and features subject to change.

## Directory Structure
See source tree for module layout. Each system is modular and pluggable.

## TODO
- Implement core simulation systems
- Add more examples
- Expand FFI support

## Testing

This project uses test-driven development. Run tests with:

```bash
cargo test
```

Run benchmarks with:

```bash
cargo bench
```
