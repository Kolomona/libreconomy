# libreconomy User Guide

This guide will help you get started with libreconomy and understand the core concepts.

## Table of Contents

- [Quick Start](#quick-start)
- [Core Concepts](#core-concepts)
- [Creating Agents](#creating-agents)
- [Working with Components](#working-with-components)
- [Querying the World](#querying-the-world)
- [Agent Lifecycle](#agent-lifecycle)
- [Common Patterns](#common-patterns)

## Quick Start

### Installation

Add libreconomy to your `Cargo.toml`:

```toml
[dependencies]
libreconomy = "0.0.1"
specs = "0.18"
```

### Your First Simulation

```rust
use libreconomy::*;
use specs::prelude::*;

fn main() {
    // 1. Create a new ECS world
    let mut world = World::new();
    
    // 2. Register all component types
    world.register::<Agent>();
    world.register::<Needs>();
    world.register::<Inventory>();
    world.register::<Wallet>();
    
    // 3. Insert the AgentId allocator resource
    world.insert(AgentIdAllocator::new());
    
    // 4. Create agents
    let agent1 = create_agent(&mut world);
    let agent2 = create_agent(&mut world);
    
    // 5. Query and use components
    let needs_storage = world.read_storage::<Needs>();
    let needs = needs_storage.get(agent1).unwrap();
    println!("Agent 1 - Thirst: {}, Hunger: {}", needs.thirst, needs.hunger);
}
```

## Core Concepts

### Entity-Component-System (ECS)

libreconomy uses the ECS pattern via the `specs` library:

- **Entities**: Unique identifiers for agents (like database row IDs)
- **Components**: Data attached to entities (Needs, Inventory, Wallet)
- **Systems**: Logic that processes components (coming in future updates)

### The World

The `World` is your simulation container. It holds:
- All entities and their components
- Resources like `AgentIdAllocator`
- Registered component types

### Agents

An agent is represented by:
- An `Entity` (from specs)
- An `Agent` component (contains unique `AgentId`)
- Core components: `Needs`, `Inventory`, `Wallet`

## Creating Agents

### Default Agent

The simplest way to create an agent:

```rust
let agent = create_agent(&mut world);
```

This creates an agent with:
- Thirst: 50.0
- Hunger: 50.0
- Empty inventory
- Currency: 100.0

### Custom Needs

Create an agent with specific needs:

```rust
let thirsty_agent = create_agent_with_needs(
    &mut world,
    Needs::new(90.0, 30.0) // Very thirsty, not very hungry
);
```

### Custom Wallet

Create a wealthy agent:

```rust
let rich_agent = create_agent_with_wallet(
    &mut world,
    Wallet::new(1000.0)
);
```

### Fully Custom Agent

Create an agent with all custom components:

```rust
let mut inventory = Inventory::default();
inventory.add("water", 5);
inventory.add("food", 3);

let custom_agent = create_agent_custom(
    &mut world,
    Needs::new(20.0, 30.0),
    inventory,
    Wallet::new(500.0)
);
```

## Working with Components

### Reading Components

To read component data:

```rust
// Get read-only access to a component storage
let needs_storage = world.read_storage::<Needs>();
let needs = needs_storage.get(agent).unwrap();
println!("Thirst: {}", needs.thirst);
```

### Modifying Components

To modify component data:

```rust
// Get mutable access to a component storage
let mut wallet_storage = world.write_storage::<Wallet>();
let wallet = wallet_storage.get_mut(agent).unwrap();

// Deposit currency
wallet.deposit(50.0);

// Withdraw currency (returns actual amount withdrawn)
let withdrawn = wallet.withdraw(30.0);
```

### Inventory Operations

```rust
let mut inv_storage = world.write_storage::<Inventory>();
let inv = inv_storage.get_mut(agent).unwrap();

// Add items
inv.add("water", 3);

// Check quantity
let water_qty = inv.quantity("water");

// Remove items (returns amount actually removed)
let removed = inv.remove("water", 2);
```

### Needs Management

```rust
let mut needs_storage = world.write_storage::<Needs>();
let needs = needs_storage.get_mut(agent).unwrap();

// Modify needs
needs.thirst += 10.0;
needs.hunger += 5.0;

// Clamp to valid range [0.0, 100.0]
needs.clamp();
```

## Querying the World

### Iterate Over All Agents

```rust
let agents = world.read_storage::<Agent>();
let needs = world.read_storage::<Needs>();
let entities = world.entities();

// Join entities with their components
for (_entity, agent, needs) in (&entities, &agents, &needs).join() {
    println!("Agent {} - Thirst: {}, Hunger: {}", 
        agent.id.0, needs.thirst, needs.hunger);
}
```

### Filter Agents by Condition

```rust
let agents = world.read_storage::<Agent>();
let needs = world.read_storage::<Needs>();
let entities = world.entities();

// Find thirsty agents (thirst > 80.0)
for (entity, agent, needs) in (&entities, &agents, &needs).join() {
    if needs.thirst > 80.0 {
        println!("Agent {} is very thirsty!", agent.id.0);
    }
}
```

### Count Agents

```rust
let agents = world.read_storage::<Agent>();
let entities = world.entities();
let count = (&entities, &agents).join().count();
println!("Total agents: {}", count);
```

## Agent Lifecycle

### Creating an Agent

Agents are created with the `create_agent*` functions. Each agent automatically receives a unique ID.

### Removing an Agent

To remove an agent and all its components:

```rust
remove_agent(&mut world, agent);

// Verify removal
assert!(!world.entities().is_alive(agent));
```

**Note**: Removed entities are not immediately reused. The ECS maintains generation counters to prevent use-after-free bugs.

## Common Patterns

### Setup Pattern

A typical simulation setup:

```rust
fn setup_world() -> World {
    let mut world = World::new();
    
    // Register all components
    world.register::<Agent>();
    world.register::<Needs>();
    world.register::<Inventory>();
    world.register::<Wallet>();
    
    // Insert resources
    world.insert(AgentIdAllocator::new());
    
    world
}
```

### Batch Agent Creation

Create multiple agents efficiently:

```rust
let agents: Vec<Entity> = (0..100)
    .map(|_| create_agent(&mut world))
    .collect();
```

### Transaction Pattern

Modify multiple components atomically:

```rust
{
    let mut wallets = world.write_storage::<Wallet>();
    let mut inventories = world.write_storage::<Inventory>();
    
    // Agent 1 pays Agent 2 for water
    let wallet1 = wallets.get_mut(agent1).unwrap();
    let withdrawn = wallet1.withdraw(10.0);
    
    let wallet2 = wallets.get_mut(agent2).unwrap();
    wallet2.deposit(withdrawn);
    
    // Transfer water
    let inv1 = inventories.get_mut(agent1).unwrap();
    inv1.add("water", 1);
    
    let inv2 = inventories.get_mut(agent2).unwrap();
    inv2.remove("water", 1);
}
```

### Safe Component Access

Always check if an entity has a component:

```rust
let needs_storage = world.read_storage::<Needs>();
if let Some(needs) = needs_storage.get(agent) {
    println!("Thirst: {}", needs.thirst);
} else {
    println!("Agent has no Needs component");
}
```

## Next Steps

- Run the `basic_simulation` example: `cargo run --example basic_simulation`
- Read the full API documentation: `cargo doc --open`
- See the test suite for more examples: `cargo test`
- Check out `docs/api/FFI.md` for using libreconomy from other languages

## Getting Help

- Read the API docs: `cargo doc --open`
- Check the examples in the `examples/` directory
- Look at the test files in `tests/` for usage patterns
- Report issues on GitHub
