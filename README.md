# libreconomy

A cross-platform, agent-based economy simulator library for games and applications.

## Key Features

- **Agent-based simulation** using Entity-Component-System (ECS) architecture
- **Comprehensive agent components**: Needs, Energy, Species, Skills, Knowledge, Employment, Preferences, Reputation
- **Utility-based decision making** with configurable thresholds and weights
- **Species-aware behavior** with diet types (Herbivore, Carnivore, Omnivore) and terrain penalties
- **Reputation system** using Beta distribution for trust modeling
- **Resource management** with renewable/non-renewable sources and regeneration
- **WebAssembly support** with TypeScript definitions for web integration
- **Multi-language FFI** for C/C++, Python, Swift, and Kotlin via uniffi
- **Event logging** for transaction tracking and outcome analysis
- **Unique agent identity** with overflow-safe ID allocation

## Architecture

libreconomy is a **pure economic simulation library** - it handles economic logic, agent decision-making, and trading protocols, but **not** spatial/physical simulation.

**Your application provides:**
- Spatial world management (positions, coordinates)
- Proximity queries via the `WorldQuery` trait
- Pathfinding and movement execution
- Rendering and game loop

**libreconomy provides:**
- Economic decisions based on agent needs and preferences
- Utility-based decision making with species-aware behavior
- Trading, employment, and resource management systems
- Price discovery and knowledge learning
- Reputation tracking and trust modeling
- Item definitions and need satisfaction

**See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for complete integration guide, patterns, and examples.**

## Build Instructions

```sh
cargo build --release
```

### Build with WASM support:
```sh
./scripts/build-wasm.sh
```

This generates WebAssembly bindings for web, bundler, and Node.js targets in `pkg/`.

## Examples

### libreterra-p5js (Complete Web Example)

A fully functional web-based simulation demonstrating all libreconomy features:

```sh
cd examples/libreterra-p5js
# Open index.html in a browser (requires a local server)
python -m http.server 8000
# Navigate to http://localhost:8000
```

**Features:**
- 10,000 × 10,000 pixel procedurally generated world
- Multiple species (Humans, Rabbits) with distinct behaviors
- Terrain system (Water, Grass, Rocky, Dirt) with traversability penalties
- Full needs-driven behavior (hunger, thirst, tiredness, energy)
- Age system with lifespan and max energy curves
- Decision-making system with utility maximization
- Interactive camera controls (pan, zoom, follow entities)
- Real-time statistics and entity info display
- Hotkeys for spawning/killing entities and speed control

**Systems implemented:**
- Camera system with viewport culling
- Terrain generation using Perlin noise
- Needs decay with state-based multipliers
- Movement with sliding collision and terrain costs
- Consumption system (eating, drinking, sleeping)
- Age system with health-based lifespan modifiers
- Spatial hashing for O(1) proximity queries
- Decision system with WASM integration

### Rust Examples

Run individual examples to explore specific features:

```bash
# Basic agent creation and components
cargo run --example basic_simulation

# Integration patterns with game loops
cargo run --example simple_integration

# Decision-making demonstrations
cargo run --example minimal_decision

# Reputation system usage
cargo run --example reputation_tracking
```

For integration patterns, see:
- `examples/simple_integration.rs` - Complete example showing WorldQuery implementation
- `examples/libreterra-p5js/` - Full-featured web simulation
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) - Detailed integration guide with code examples

## Current Implementation

The following features are **fully implemented and tested**:

### Core Agent Systems
- **Agent Entity System**: Unique agent IDs with overflow-safe allocation
- **Agent Lifecycle**: Creation with various configurations and clean removal

### Agent Components
- **Needs**: Tracks thirst, hunger, and tiredness with automatic clamping
- **Energy**: Current/max energy tracking with restoration and depletion
- **Species**: Species type with diet definitions (Herbivore, Carnivore, Omnivore)
- **Inventory**: Item storage with safe add/remove operations
- **Wallet**: Currency balance with non-negative guarantees
- **Skills**: Key-value skill levels for agent capabilities
- **Knowledge**: Observed prices and known trade partners
- **Employment**: Job status, employer tracking, employee management
- **Preferences**: Utility functions and risk tolerance settings
- **ResourceSource**: Renewable/non-renewable resources with regeneration rates
- **ReputationView**: Beta distribution-based reputation (alpha/beta parameters)
- **ReputationKnowledge**: First-hand reputation observations

### Decision-Making System
- **UtilityMaximizer**: Full utility-based decision maker with configurable weights
- **DecisionThresholds**: Urgency thresholds for needs (high_hunger, high_thirst, etc.)
- **UtilityWeights**: Importance weights for survival, comfort, and efficiency
- **Species-Aware**: Decisions consider diet type and species capabilities
- **Intent System**: High-level goals (SeekItem, FindWork, SeekTrade, Rest, Wander)
- **Action System**: Directed actions toward specific agents (Trade, Hunt, Consume)
- **Transaction Tracking**: Completed economic exchanges with outcomes

### Reputation System
- **Beta Distribution Model**: Sophisticated trust modeling using conjugate priors
- **ReputationUpdateSystem**: Updates reputation based on transaction outcomes
- **ReputationDecaySystem**: Temporal decay of reputation confidence
- **First-Hand Knowledge**: Direct observation tracking separate from hearsay

### Item and Resource Systems
- **ItemRegistry**: Central registry of items with types and need satisfaction values
- **Base Items**: Water, food, grass, rabbit meat, and more
- **Resource Sources**: Renewable resources with regeneration logic

### Event and Transaction Systems
- **TransactionEvent**: Full transaction logging with outcomes (Positive/Negative/Neutral)
- **TransactionLog**: Batch event collection and processing
- **Event-Driven Updates**: Reputation and knowledge updates from events

### Integration Support
- **ECS Integration**: Full registration and query support for all components
- **WorldQuery Trait**: Standardized interface for spatial queries
- **CurrentTick**: Global tick counter for time-based operations

## WebAssembly Integration

libreconomy provides comprehensive WASM support for web applications:

### Build and Use

1. **Build the WASM module:**
   ```sh
   ./scripts/build-wasm.sh
   ```
   This generates `pkg/libreconomy.js`, `libreconomy_bg.wasm`, and TypeScript definitions.

2. **Use in JavaScript/TypeScript:**
   ```javascript
   import init, { WasmWorld, WasmDecisionMaker } from './pkg/libreconomy.js';

   await init();
   const world = new WasmWorld();
   const agentId = world.create_agent();

   // Set agent needs
   world.set_needs(agentId, 50, 30, 20);

   // Make decisions
   const decisionMaker = new WasmDecisionMaker();
   const decision = decisionMaker.decide_libreterra(agentId, world, worldQuery);
   ```

3. **TypeScript support:**
   Auto-generated TypeScript definitions provide full type safety.

**See [docs/WASM.md](docs/WASM.md) for complete WASM integration guide.**

## Using uniffi for Language Bindings

This project uses uniffi's proc-macro approach for generating language bindings.

1. **Build the dynamic library:**
   ```sh
   cargo build --release --features uniffi
   ```
   The shared library will be in `target/release/liblibreconomy.so` (Linux), `liblibreconomy.dylib` (macOS), or `liblibreconomy.dll` (Windows).

2. **Generate bindings:**
   ```sh
   bash scripts/release.sh
   ```
   This generates bindings for Python, Kotlin, and Swift in the `dist/` directory.

3. **Example usage in Python:**
   ```python
   import libreconomy
   print(libreconomy.libreconomy_version())

   world = libreconomy.create_world()
   agent_id = libreconomy.create_agent(world)
   ```

See `uniffi.toml` for configuration. The API is defined using `#[uniffi::export]` attributes in the Rust source code.

## Using in Godot

1. **Build the dynamic library:**
   ```sh
   cargo build --release
   ```

2. **Generate C header with cbindgen:**
   ```sh
   cbindgen --config cbindgen.toml --crate libreconomy --output libreconomy.h
   ```

3. **In Godot:**
   - Use GDExtension to load the shared library
   - Use the generated header to call FFI functions
   - Write Godot scripts to interact with the library

**See [docs/integrations/godot-how-to.md](docs/integrations/godot-how-to.md) for detailed Godot integration guide.**

## Roadmap

### ✅ Implemented
- Agent entity system with unique IDs
- Comprehensive agent components (Needs, Energy, Species, Skills, Knowledge, Employment, Preferences)
- Utility-based decision making with configurable thresholds
- Reputation system with Beta distribution
- Item registry and resource sources
- Transaction logging and event system
- WASM integration with TypeScript support
- Multi-language FFI (Python, Swift, Kotlin, C/C++)
- Complete web example (libreterra-p5js)

### 🚧 In Progress
- Market systems (trading protocols, price discovery)
- Labor systems (employment matching, contracts)
- Production systems (crafting, resource transformation)

### 📋 Planned
- Advanced preference modeling (risk curves, time preference)
- Negotiation protocols
- Information asymmetry and signaling
- Social networks and reputation propagation
- Multi-commodity trading
- Complex production chains
- Learning and adaptation algorithms

## Documentation

- **User Guide**: See [`docs/GUIDE.md`](docs/GUIDE.md) for tutorials and common patterns
- **Architecture**: See [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) for integration guide
- **WASM Guide**: See [`docs/WASM.md`](docs/WASM.md) for WebAssembly integration
- **API Reference**: Run `cargo doc --open` to view full API documentation
- **FFI Integration**: See [`docs/api/FFI.md`](docs/api/FFI.md) for C/C++, Python, Swift, or Kotlin
- **Godot Integration**: See [`docs/integrations/godot-how-to.md`](docs/integrations/godot-how-to.md)
- **Testing Guide**: See [`docs/TESTING.md`](docs/TESTING.md) for testing guidelines

## Testing

This project uses test-driven development. Run tests with:

```bash
cargo test
```

Run benchmarks with:

```bash
cargo bench
```

## Project Statistics

- **Rust source code**: ~5,500 lines
- **Web example (libreterra-p5js)**: ~5,000 lines of JavaScript
- **Examples**: 4 Rust examples + 1 complete web simulation
- **Documented files**: 27 files in examples/libreterra-p5js
- **Agent components**: 11 comprehensive components
- **Decision outputs**: Intent, Action, and Transaction types

## Status

**Active development.** Core agent systems, decision-making, and reputation are production-ready. Market and labor systems are in progress. API is stabilizing but may change as additional systems are added.

## License

See LICENSE file for details.

## Contributing

Contributions welcome! Please see CONTRIBUTING.md for guidelines (coming soon).
