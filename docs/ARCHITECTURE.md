# libreconomy Architecture: Library/Application Separation

## Overview

libreconomy is designed as a **pure economic simulation library** that integrates into applications (games, simulations, research tools). This document defines the architectural boundaries between libreconomy and calling applications.

### Core Design Philosophy

**libreconomy handles:**
- Economic logic and decision-making
- Agent behavior and utility calculations
- Market mechanisms and price discovery
- Knowledge, reputation, and learning systems
- Production, labor, and trading protocols

**Your application handles:**
- Spatial world management (positions, coordinates)
- Proximity queries ("who is nearby?")
- Pathfinding and movement
- Rendering and visualization
- Game loop timing

**The interface between them:**
- Your app implements query traits to provide world context
- libreconomy returns decision outputs for your app to execute
- Both share the ECS World via the specs crate

---

## Responsibility Matrix

| Concern | libreconomy (Library) | Application (Your Code) |
|---------|----------------------|------------------------|
| **Agent State** | Needs, Inventory, Wallet, Skills, Knowledge, Employment, Preferences | ✗ |
| **Economic Decisions** | Utility calculations, trade decisions, partner selection | ✗ |
| **Spatial Data** | ✗ | Agent positions, world coordinates, grids/tiles |
| **Proximity Queries** | ✗ | "Which agents are nearby?" |
| **Pathfinding** | ✗ | Navigation, movement execution |
| **Item Definitions** | Item types, need satisfaction values, base items (water, food) | Custom item registration |
| **ResourceSource Entities** | Component definition, economic behavior | Placement in world, spatial queries |
| **Trading** | Negotiation logic, transaction execution | Bringing agents into contact |
| **Production** | Recipes, skill requirements, transformation logic | ✗ |
| **Labor** | Job matching, wage determination | ✗ |
| **Reputation** | Trust scoring, rumor propagation | ✗ |
| **Rendering** | ✗ | Graphics, UI, visual feedback |
| **Time Management** | ✗ | Game loop, tick rate, when to call library |

---

## Integration Workflow

```
┌─────────────────────────────────────────────────────────────┐
│                      YOUR APPLICATION                        │
│  ┌────────────┐  ┌──────────┐  ┌────────────┐              │
│  │   World    │  │  Render  │  │ Pathfinding│              │
│  │  Manager   │  │  System  │  │   System   │              │
│  │ (2D/3D/    │  │          │  │            │              │
│  │  Graph)    │  │          │  │            │              │
│  └─────┬──────┘  └──────────┘  └──────┬─────┘              │
│        │                                │                    │
│        │ implements                     │ executes           │
│        ↓                                ↓                    │
│  ┌──────────────────────────────────────────────────┐       │
│  │         WorldQuery Trait                         │       │
│  │  • get_nearby_agents(AgentId) → Vec<AgentId>    │       │
│  │  • get_nearby_resources(AgentId) → Vec<Entity>  │       │
│  │  • can_interact(AgentId, AgentId) → bool        │       │
│  └────────────────┬─────────────────────────────────┘       │
└───────────────────┼─────────────────────────────────────────┘
                    │ provides context
                    ↓
┌─────────────────────────────────────────────────────────────┐
│                      LIBRECONOMY                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │   Decision   │  │    Market    │  │  Production  │      │
│  │    System    │  │    System    │  │    System    │      │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘      │
│         │                  │                  │              │
│         └──────────────────┼──────────────────┘              │
│                            │ returns                         │
│                            ↓                                 │
│  ┌──────────────────────────────────────────────────┐       │
│  │         Decision Outputs                         │       │
│  │  • Intent("seek water", urgency: 0.9)           │       │
│  │  • Action(trade with agent #42)                 │       │
│  │  • Transaction(immediate trade executed)        │       │
│  └──────────────┬───────────────────────────────────┘       │
└─────────────────┼─────────────────────────────────────────┘
                  │
                  ↓ app executes
           ┌──────────────┐
           │ Move agent   │
           │ Update UI    │
           │ Play effects │
           └──────────────┘
```

---

## WorldQuery Trait: Your Application's Interface

### Trait Definition

```rust
use specs::Entity;
use crate::AgentId;

/// Trait that applications implement to provide world context to libreconomy.
///
/// libreconomy queries the application for spatial information without knowing
/// the details of your world representation (2D grid, 3D space, graph, etc.).
pub trait WorldQuery {
    /// Get agents within interaction range of the specified agent.
    ///
    /// Applications define "nearby" based on their spatial model:
    /// - 2D/3D games: agents within radius
    /// - Grid-based: agents in adjacent tiles
    /// - Graph-based: agents connected by edges
    /// - Social networks: agents in relationship graph
    ///
    /// `max_count` is a hint for performance (library may not need all nearby agents)
    fn get_nearby_agents(&self, agent: AgentId, max_count: usize) -> Vec<AgentId>;

    /// Get resource sources within interaction range of the agent.
    ///
    /// resource_type examples: "water_well", "farm", "mine", "shop"
    /// Returns Entity IDs that have ResourceSource components
    fn get_nearby_resources(&self, agent: AgentId, resource_type: &str) -> Vec<Entity>;

    /// Check if two agents can currently interact.
    ///
    /// Applications may consider:
    /// - Distance/proximity
    /// - Line of sight
    /// - Obstacles/barriers
    /// - Current state (busy, stunned, etc.)
    fn can_interact(&self, agent1: AgentId, agent2: AgentId) -> bool;
}
```

### Implementation Example: 2D Grid World

```rust
use std::collections::HashMap;

struct GridWorld {
    // Map from AgentId to (x, y) position
    agent_positions: HashMap<AgentId, (i32, i32)>,
    // Map from (x, y) to list of ResourceSource entities at that position
    resources: HashMap<(i32, i32), Vec<(Entity, String)>>, // (entity, type)
    interaction_range: f32,
}

impl WorldQuery for GridWorld {
    fn get_nearby_agents(&self, agent: AgentId, max_count: usize) -> Vec<AgentId> {
        let Some(&pos) = self.agent_positions.get(&agent) else {
            return Vec::new();
        };

        let mut nearby: Vec<(AgentId, f32)> = self.agent_positions
            .iter()
            .filter(|(&other_id, _)| other_id != agent)
            .map(|(&other_id, &other_pos)| {
                let dist = distance(pos, other_pos);
                (other_id, dist)
            })
            .filter(|(_, dist)| *dist <= self.interaction_range)
            .collect();

        // Sort by distance, closest first
        nearby.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        nearby.truncate(max_count);
        nearby.into_iter().map(|(id, _)| id).collect()
    }

    fn get_nearby_resources(&self, agent: AgentId, resource_type: &str) -> Vec<Entity> {
        let Some(&pos) = self.agent_positions.get(&agent) else {
            return Vec::new();
        };

        let mut nearby_resources = Vec::new();

        // Check positions within range
        for dx in -self.interaction_range as i32..=self.interaction_range as i32 {
            for dy in -self.interaction_range as i32..=self.interaction_range as i32 {
                let check_pos = (pos.0 + dx, pos.1 + dy);
                if let Some(resources) = self.resources.get(&check_pos) {
                    for (entity, rtype) in resources {
                        if rtype == resource_type {
                            nearby_resources.push(*entity);
                        }
                    }
                }
            }
        }

        nearby_resources
    }

    fn can_interact(&self, agent1: AgentId, agent2: AgentId) -> bool {
        let Some(&pos1) = self.agent_positions.get(&agent1) else { return false; };
        let Some(&pos2) = self.agent_positions.get(&agent2) else { return false; };

        distance(pos1, pos2) <= self.interaction_range
    }
}

fn distance(a: (i32, i32), b: (i32, i32)) -> f32 {
    (((a.0 - b.0).pow(2) + (a.1 - b.1).pow(2)) as f32).sqrt()
}
```

---

## Decision Output Types: Library → Application

libreconomy returns three types of decisions that your application executes:

### 1. Intents (High-Level Goals)

**When**: Agent decides it wants something but needs to find it
**App responsibility**: Find appropriate target, pathfind to it

```rust
pub enum Intent {
    /// Agent wants to acquire an item to satisfy needs
    SeekItem {
        item_type: String,  // e.g., "water", "food"
        urgency: f32,       // 0.0-1.0, how desperate
    },

    /// Agent wants employment
    FindWork {
        skill_types: Vec<String>,  // skills agent can offer
    },

    /// Agent wants to trade (buy or sell)
    SeekTrade {
        buying: bool,
        item_type: String,
    },

    /// Agent is satisfied, do nothing
    Rest,
}
```

**Application handling example:**

```rust
match decision_output {
    DecisionOutput::Intent(Intent::SeekItem { item_type, urgency }) => {
        // App finds nearest water source
        let sources = world_query.get_nearby_resources(agent_id, &item_type);
        if let Some(nearest) = sources.first() {
            // App pathfinds agent to the source
            pathfinding_system.navigate_to(agent_entity, *nearest);
        }
    }
    _ => {}
}
```

### 2. Actions (Specific Targets)

**When**: Agent has identified a specific partner/target
**App responsibility**: Move agent into interaction range

```rust
pub struct Action {
    pub target_agent: AgentId,
    pub action_type: ActionType,
}

pub enum ActionType {
    /// Initiate trade negotiation
    InitiateTrade {
        item: String,
        offer_price: f32,
    },

    /// Accept a job offer
    AcceptEmployment {
        wage: f32,
    },

    /// Request information from another agent
    AskForInformation {
        query_type: String,
    },
}
```

**Application handling example:**

```rust
match decision_output {
    DecisionOutput::Action(Action { target_agent, action_type }) => {
        // Check if agents can already interact
        if world_query.can_interact(agent_id, target_agent) {
            // Already in range, execute immediately
            libreconomy::execute_action(world, agent_id, target_agent, action_type);
        } else {
            // App moves agent toward target
            pathfinding_system.navigate_to_agent(agent_entity, target_agent);
            // Store pending action to execute when in range
            pending_actions.insert(agent_entity, action_type);
        }
    }
    _ => {}
}
```

### 3. Transactions (Immediate Execution)

**When**: Agents are already in contact and library handles everything
**App responsibility**: Update UI, play effects (optional)

```rust
pub struct Transaction {
    pub buyer: AgentId,
    pub seller: AgentId,
    pub item: String,
    pub quantity: u32,
    pub price: f32,
    pub success: bool,
}
```

**Application handling example:**

```rust
match decision_output {
    DecisionOutput::Transaction(txn) => {
        // Library already updated inventories/wallets
        // App just provides feedback
        if txn.success {
            ui_system.show_message(&format!(
                "Agent {} bought {} from Agent {}",
                txn.buyer, txn.item, txn.seller
            ));
            audio_system.play_sound("coin_clink");
        }
    }
    _ => {}
}
```

---

## Item System: Registration and Customization

### Library-Provided Default Items

libreconomy includes common items that 80% of simulations need:

```rust
// Basic survival items
"water"      → satisfies thirst (-30.0)
"food"       → satisfies hunger (-25.0)
"meal"       → satisfies both thirst (-10.0) and hunger (-40.0)

// Resources
"wood"       → crafting material
"stone"      → crafting material
"ore"        → crafting material

// Tools
"axe"        → enables wood production
"pickaxe"    → enables mining
```

### ItemRegistry System

```rust
use std::collections::HashMap;

pub struct ItemRegistry {
    items: HashMap<String, ItemType>,
}

pub struct ItemType {
    pub id: String,
    pub name: String,
    pub satisfies: HashMap<NeedType, f32>,  // How much each need is reduced
    pub consumable: bool,                    // Destroyed on use?
    pub durability: Option<u32>,             // For tools
    pub stack_size: u32,                     // Max stack size
}

pub enum NeedType {
    Thirst,
    Hunger,
    // Apps can extend with custom needs
}

impl ItemRegistry {
    /// Create registry with default items
    pub fn with_defaults() -> Self {
        let mut registry = Self { items: HashMap::new() };

        // Register built-in items
        registry.register(ItemType {
            id: "water".to_string(),
            name: "Water".to_string(),
            satisfies: [(NeedType::Thirst, -30.0)].into(),
            consumable: true,
            durability: None,
            stack_size: 10,
        });

        // ... more defaults

        registry
    }

    /// Register a custom item (can override defaults)
    pub fn register(&mut self, item: ItemType) -> Result<(), ItemError> {
        if item.id.is_empty() {
            return Err(ItemError::EmptyId);
        }
        self.items.insert(item.id.clone(), item);
        Ok(())
    }

    /// Get item definition
    pub fn get(&self, id: &str) -> Option<&ItemType> {
        self.items.get(id)
    }
}
```

### Customizing Items

```rust
// Example: Override water to be more effective
let mut registry = ItemRegistry::with_defaults();

registry.register(ItemType {
    id: "water".to_string(),
    name: "Purified Water".to_string(),
    satisfies: [(NeedType::Thirst, -50.0)].into(),  // More effective!
    consumable: true,
    durability: None,
    stack_size: 5,
});

// Example: Add custom fantasy item
registry.register(ItemType {
    id: "mana_potion".to_string(),
    name: "Mana Potion".to_string(),
    satisfies: [(NeedType::Custom("mana".to_string()), -100.0)].into(),
    consumable: true,
    durability: None,
    stack_size: 3,
});
```

---

## ResourceSource Component

libreconomy defines ResourceSource as an ECS component that your app places in the world.

### Component Definition

```rust
use specs::{Component, VecStorage};

/// Component for entities that provide resources
#[derive(Component)]
#[storage(VecStorage)]
pub struct ResourceSource {
    pub resource_type: String,      // "water_well", "farm", "mine"
    pub item_produced: String,       // "water", "food", "ore"
    pub regeneration_rate: f32,      // Items per tick
    pub current_stock: u32,          // Available now
    pub max_stock: u32,              // Capacity
    pub requires_skill: Option<String>, // e.g., "mining"
}
```

### Application Usage

```rust
// Your app creates and places resource sources in the world
fn setup_world(world: &mut World) {
    // Register the component
    world.register::<ResourceSource>();

    // Create a water well at position (10, 5)
    let well = world.create_entity()
        .with(ResourceSource {
            resource_type: "water_well".to_string(),
            item_produced: "water".to_string(),
            regeneration_rate: 1.0,
            current_stock: 50,
            max_stock: 100,
            requires_skill: None,
        })
        .build();

    // Your app tracks its position
    your_world.place_resource(well, (10, 5));
}
```

### Library Interaction

When an agent decides to seek water (Intent::SeekItem), your app uses the WorldQuery to find nearby water_wells and moves the agent there. Once in range, the library handles the actual resource extraction.

---

## Complete Integration Example

### Minimal Game Loop

```rust
use libreconomy::*;
use specs::prelude::*;

fn main() {
    // 1. Setup ECS world
    let mut world = World::new();
    world.register::<Agent>();
    world.register::<Needs>();
    world.register::<Inventory>();
    world.register::<Wallet>();
    world.register::<ResourceSource>();
    world.insert(AgentIdAllocator::new());

    // 2. Setup your spatial world
    let mut grid_world = GridWorld::new(100, 100);

    // 3. Create agents
    let agent1 = create_agent_with_needs(&mut world, Needs::new(80.0, 30.0));
    let agent2 = create_agent(&mut world);

    // 4. Place agents in your world
    grid_world.place_agent(agent1, (10, 10));
    grid_world.place_agent(agent2, (15, 15));

    // 5. Create resource sources
    let well = world.create_entity()
        .with(ResourceSource {
            resource_type: "water_well".to_string(),
            item_produced: "water".to_string(),
            regeneration_rate: 1.0,
            current_stock: 100,
            max_stock: 100,
            requires_skill: None,
        })
        .build();
    grid_world.place_resource(well, (12, 12));

    // 6. Game loop
    for tick in 0..1000 {
        // Update libreconomy systems
        run_need_decay_system(&mut world);

        // Get decisions from libreconomy
        let decisions = get_agent_decisions(&world, &grid_world);

        // Execute decisions in your world
        for (agent, decision) in decisions {
            match decision {
                DecisionOutput::Intent(Intent::SeekItem { item_type, .. }) => {
                    let sources = grid_world.get_nearby_resources(agent, &item_type);
                    if let Some(source) = sources.first() {
                        pathfind_agent_to_resource(&mut grid_world, agent, *source);
                    }
                }
                DecisionOutput::Action(action) => {
                    execute_action(&mut world, &grid_world, agent, action);
                }
                DecisionOutput::Transaction(txn) => {
                    println!("Trade completed: {:?}", txn);
                }
            }
        }

        // Update your world (movement, rendering, etc.)
        grid_world.update();
    }
}
```

---

## Best Practices

### 1. Keep Concerns Separated

❌ **Don't** put position data in libreconomy components
❌ **Don't** implement pathfinding in libreconomy
❌ **Don't** query libreconomy for spatial information

✅ **Do** implement WorldQuery in your app
✅ **Do** let libreconomy handle all economic logic
✅ **Do** execute decisions in your app's spatial system

### 2. Use the ECS Properly

Both libreconomy and your app share the same specs World. Use it for:

- **libreconomy components**: Needs, Inventory, Wallet, Skills, etc.
- **Your components**: Position, Velocity, Sprite, Health, etc.
- **Shared entities**: Agents exist in both systems

### 3. Handle All Decision Types

Make sure your game loop handles all three decision output types:

```rust
match decision {
    DecisionOutput::Intent(intent) => {
        // Find targets, pathfind
    }
    DecisionOutput::Action(action) => {
        // Move agents into range, execute
    }
    DecisionOutput::Transaction(txn) => {
        // UI feedback only
    }
}
```

### 4. Customize Items Appropriately

Start with defaults, override as needed:

```rust
let mut registry = ItemRegistry::with_defaults();

// Override for your game's balance
registry.register(/* your custom water */);
```

---

## FAQ

**Q: Can I use libreconomy without spatial coordinates?**
A: Yes! Implement WorldQuery for a graph-based or abstract world. "Nearby" can mean anything.

**Q: Do I need to use all decision output types?**
A: No, but you should handle what you use. For simple simulations, you might only use Transactions.

**Q: Can I extend NeedType with custom needs?**
A: Yes, the design supports custom needs. (Implementation details TBD)

**Q: How do I integrate with Godot/Unity/Unreal?**
A: Use the C FFI bindings (cbindgen) or language-specific bindings (uniffi). Implement WorldQuery in your engine's scripting language.

**Q: What if my game has teleportation or portals?**
A: Your WorldQuery implementation defines "nearby" however you want. Agents connected by portals can be "nearby" even if spatially distant.

---

## Next Steps

1. Read the [User Guide](GUIDE.md) for ECS basics
2. See `examples/simple_integration.rs` for working code
3. Check [FFI.md](api/FFI.md) for C/Python/Swift integration
4. Review [overall-research.md](ai/plans/overall-research.md) for system designs

---

**Last Updated**: December 2025
**Status**: Design complete, implementation in progress
