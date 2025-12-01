# libreconomy

A cross-platform, agent-based economy simulator library for games and applications.

## Key Features
- **Agent-based simulation** using Entity-Component-System (ECS) architecture
- **Core agent components**: Needs (thirst, hunger), Inventory, Wallet
- **Unique agent identity** with safe ID allocation
- **Agent lifecycle management** including creation and removal
- **FFI support** for C/C++ (via cbindgen) and Python/Swift/Kotlin (via uniffi)
- Subjective value modeling (planned)
- Market, labor, and production systems (planned)

## Build Instructions
```sh
cargo build --release
```

## Using uniffi for Language Bindings

This project uses uniffi's proc-macro approach for generating language bindings. Functions are exported using the `#[uniffi::export]` attribute in the Rust code.

1. **Build the dynamic library:**
   ```sh
   cargo build --release
   ```
   The shared library will be in `target/release/liblibreconomy.so` (Linux), `liblibreconomy.dylib` (macOS), or `liblibreconomy.dll` (Windows).

2. **Generate bindings from the compiled library:**
   ```sh
   uniffi-bindgen generate --library target/release/liblibreconomy.so --language python --out-dir dist
   uniffi-bindgen generate --library target/release/liblibreconomy.so --language kotlin --out-dir dist
   uniffi-bindgen generate --library target/release/liblibreconomy.so --language swift --out-dir dist
   ```
   Or simply run:
   ```sh
   bash scripts/release.sh
   ```
   This generates bindings for Python, Kotlin, and Swift in the `dist/` directory.

3. **Example usage in Python:**
   ```python
   import libreconomy
   print(libreconomy.libreconomy_version())
   print(libreconomy.get_agent_count())
   ```

See `uniffi.toml` for configuration. The API is defined using `#[uniffi::export]` attributes in the Rust source code.

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

## Current Implementation

The following features are implemented and tested:

- **Agent Entity System**: Unique agent IDs with overflow-safe allocation
- **Core Components**:
  - `Needs`: Tracks agent needs (thirst, hunger) with automatic clamping
  - `Inventory`: Item storage with safe add/remove operations
  - `Wallet`: Currency balance with non-negative guarantees
- **Agent Creation**: Multiple creation functions for different configurations
- **Agent Removal**: Clean deletion of agents and their components
- **ECS Integration**: Full registration and query support for all components

## Examples

Run the basic simulation example to see agents in action:

```bash
cargo run --example basic_simulation
```

This demonstrates agent creation, component manipulation, and ECS queries.

## Status

Early development. Core agent systems are functional. API may change as additional systems are added.

## Roadmap

- Market systems (trading, pricing)
- Labor systems (employment, gigs, contracts)
- Production systems (crafting, resource transformation)
- Decision-making traits and implementations
- More agent components (skills, knowledge, preferences)

## Documentation

- **User Guide**: See [`docs/GUIDE.md`](docs/GUIDE.md) for tutorials and common patterns
- **API Reference**: Run `cargo doc --open` to view full API documentation
- **FFI Integration**: See [`docs/api/FFI.md`](docs/api/FFI.md) for using from C/C++, Python, Swift, or Kotlin

## Testing

This project uses test-driven development. Run tests with:

```bash
cargo test
```

Run benchmarks with:

```bash
cargo bench
```
