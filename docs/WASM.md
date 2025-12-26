# WASM Build Guide

This guide explains how to build and use libreconomy in WebAssembly environments (browsers, Node.js).

## Prerequisites

1. **Install wasm-pack**:
   ```bash
   cargo install wasm-pack
   ```

2. **Install wasm32 target** (if not already installed):
   ```bash
   rustup target add wasm32-unknown-unknown
   ```

## Building

Use the provided build script:

```bash
./scripts/build-wasm.sh [target]
```

Available targets:
- `web` (default) - For direct browser use
- `bundler` - For webpack, rollup, etc.
- `nodejs` - For Node.js environments

The build output will be in the `pkg/` directory with:
- `libreconomy.js` - JavaScript bindings
- `libreconomy_bg.wasm` - WebAssembly binary
- `libreconomy.d.ts` - TypeScript type definitions

## Usage Examples

### Basic World Management

```javascript
import init, { WasmWorld } from './pkg/libreconomy.js';

// Initialize the WASM module
await init();

// Create a world
const world = new WasmWorld();

// Create agents
const agent1 = world.create_agent();
const agent2 = world.create_agent_with_needs(80.0, 60.0, 40.0);
const agent3 = world.create_agent_with_wallet(500.0);
const agent4 = world.create_agent_full(70.0, 50.0, 30.0, 200.0);

console.log(`Total agents: ${world.get_agent_count()}`);
```

### Reading and Modifying Agent Components

```javascript
// Get agent needs
const needs = world.get_needs(agent1);
console.log(`Thirst: ${needs.thirst}, Hunger: ${needs.hunger}`);

// Set agent needs
world.set_needs(agent1, 50.0, 50.0, 50.0);

// Get inventory
const inventory = world.get_inventory(agent1);
console.log(inventory.items);

// Add/remove items
world.add_item(agent1, "water", 5);
const removed = world.remove_item(agent1, "water", 2);

// Get wallet
const wallet = world.get_wallet(agent1);
console.log(`Currency: ${wallet.currency}`);

// Deposit/withdraw
world.deposit(agent1, 50.0);
const withdrawn = world.withdraw(agent1, 30.0);
```

### Resource Sources

```javascript
// Create a grass patch (renewable resource)
const grassPatch = world.create_resource_source(
  "plant",           // resource type
  "grass",          // item produced
  1.0,              // regeneration rate per tick
  100               // initial stock
);

// Create a water source
const waterSource = world.create_resource_source(
  "water",
  "water",
  0.5,
  200
);

// Harvest from a resource
const harvested = world.harvest_resource(grassPatch, 30);
console.log(`Harvested ${harvested} grass`);

// Regenerate all resources (call this each tick)
world.regenerate_resources();
```

### Item Registry

```javascript
// Get items that satisfy a need
const thirstItems = world.get_items_for_need("thirst");
console.log(thirstItems); // ["water"]

const hungerItems = world.get_items_for_need("hunger");
console.log(hungerItems); // ["food", "grass", "rabbit_meat"]

// Get how much an item satisfies a need
const waterSatisfaction = world.get_item_satisfaction("water", "thirst");
console.log(waterSatisfaction); // -30.0 (reduces thirst by 30)

const grassSatisfaction = world.get_item_satisfaction("grass", "hunger");
console.log(grassSatisfaction); // -15.0
```

### Decision System

The decision system requires implementing a WorldQuery interface to provide spatial information:

```javascript
import { WasmWorld, WasmDecisionMaker } from './pkg/libreconomy.js';

// Create world and decision maker
const world = new WasmWorld();
const decisionMaker = WasmDecisionMaker.new();

// Or with custom configuration
const customDecisionMaker = WasmDecisionMaker.withConfig(
  90.0,  // critical_thirst
  70.0,  // high_thirst
  90.0,  // critical_hunger
  70.0,  // high_hunger
  90.0,  // critical_tiredness
  70.0,  // high_tiredness
  1.0,   // survival_weight
  0.5,   // comfort_weight
  0.3,   // efficiency_weight
  50.0   // search_radius
);

// Implement WorldQuery interface
// This provides spatial information from your game/simulation
const worldQuery = {
  // Return array of nearby agent IDs
  getNearbyAgents(agentId, maxCount) {
    // Your spatial query logic here
    // Example: query spatial hash, grid, etc.
    return []; // Array of u32 agent IDs
  },

  // Return array of nearby resources
  getNearbyResources(agentId, resourceType, maxRadius) {
    // Your spatial query logic here
    // Return objects with: { x: number, y: number, distance: number }
    return [];
  },

  // Check if two agents can interact
  canInteract(agent1Id, agent2Id) {
    // Your interaction logic here
    // Example: check distance, line of sight, etc.
    return true;
  }
};

// Create an agent and make a decision
const agentId = world.create_agent_with_needs(80.0, 50.0, 30.0);
const decision = decisionMaker.decide(world, agentId, worldQuery);

console.log(decision);
// Example output:
// {
//   Intent: {
//     SeekItem: { item_type: "water", urgency: 0.8 }
//   }
// }
```

### Decision Output Types

The decision system returns one of three types:

1. **Intent** - High-level goal:
   ```javascript
   { Intent: { SeekItem: { item_type: "water", urgency: 0.8 } } }
   { Intent: { FindWork: { skill_types: ["farming"] } } }
   { Intent: { Rest: null } }
   { Intent: { Wander: null } }
   ```

2. **Action** - Directed action toward another agent:
   ```javascript
   {
     Action: {
       target_agent: 123,
       action_type: { Hunt: null }
     }
   }
   ```

3. **Transaction** - Economic exchange:
   ```javascript
   {
     Transaction: {
       buyer: 456,
       seller: 789,
       item: "water",
       price: 5.0
     }
   }
   ```

## Integration with p5.js (libreterra example)

```javascript
// sketch.js
let world;
let decisionMaker;
let wasmModule;

async function setup() {
  // Load WASM module
  wasmModule = await import('./pkg/libreconomy.js');
  await wasmModule.default(); // Initialize WASM

  const { WasmWorld, WasmDecisionMaker } = wasmModule;

  world = new WasmWorld();
  decisionMaker = new WasmDecisionMaker();

  // Create some agents
  for (let i = 0; i < 10; i++) {
    world.create_agent();
  }
}

function draw() {
  // Your simulation loop
  // Make decisions for agents
  // Update positions based on decisions
  // Render
}
```

## Performance Considerations

- **Batch Operations**: Minimize calls across the WASM boundary. Batch operations when possible.
- **Memory**: JavaScript objects returned from WASM are serialized. Cache them if used frequently.
- **Resource Sources**: Call `regenerate_resources()` once per tick, not per resource.
- **Agent Count**: Decision-making scales with agent count and spatial complexity.

## TypeScript Support

TypeScript definitions are automatically generated in `pkg/libreconomy.d.ts`:

```typescript
import init, { WasmWorld, WasmDecisionMaker } from './pkg/libreconomy';

async function main() {
  await init();
  const world = new WasmWorld();
  const agent: number = world.create_agent();

  // TypeScript will provide autocompletion and type checking
  const needs = world.get_needs(agent);
  if (needs) {
    console.log(`Thirst: ${needs.thirst}`);
  }
}
```

## Build Options

### Development Build
```bash
./scripts/build-wasm.sh web
```

### Production Build (Optimized)
```bash
wasm-pack build --target web --features wasm --release
```

Add to `Cargo.toml` for maximum optimization:
```toml
[profile.release]
opt-level = "z"     # Optimize for size
lto = true          # Link-time optimization
codegen-units = 1   # Single codegen unit
```

## Debugging

### Enable Debug Symbols
```bash
wasm-pack build --target web --features wasm --dev
```

### Console Logging
WASM panics will appear in the browser console. For custom logging, use `console_error_panic_hook`:

```toml
# Add to Cargo.toml
[dependencies]
console_error_panic_hook = { version = "0.1", optional = true }
```

## Browser Compatibility

Requires browsers with WebAssembly support:
- Chrome 57+
- Firefox 52+
- Safari 11+
- Edge 16+

## Next Steps

- See [examples/libreterra-p5js](../examples/libreterra-p5js) for a complete integration
- Read [API documentation](./api/API.md) for detailed component information
- Check [CONTRIBUTING.md](../CONTRIBUTING.md) for development guidelines
