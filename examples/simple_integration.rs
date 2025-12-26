//! Simple Integration Example
//!
//! This example demonstrates how to integrate libreconomy into a game or simulation.
//! It shows the separation of concerns between libreconomy (economic logic) and your
//! application (spatial logic).
//!
//! Note: This example shows the intended architecture. Some systems (WorldQuery trait,
//! Decision types, ItemRegistry) are designed but not yet implemented in libreconomy.
//! This serves as a template for future integration once those systems are complete.

use libreconomy::*;
use specs::prelude::*;
use std::collections::HashMap;

// ============================================================================
// APPLICATION CODE: Your spatial world implementation
// ============================================================================

/// 2D position component (your application's responsibility)
#[derive(Debug, Clone, Copy)]
struct Position {
    x: i32,
    y: i32,
}

/// Simple 2D grid world (your application's responsibility)
struct GridWorld {
    /// Agent positions in the world
    agent_positions: HashMap<Entity, Position>,
    /// Resource source positions
    resource_positions: HashMap<Entity, (Position, String)>,  // (pos, resource_type)
    /// Interaction range
    interaction_range: f32,
}

impl GridWorld {
    fn new() -> Self {
        Self {
            agent_positions: HashMap::new(),
            resource_positions: HashMap::new(),
            interaction_range: 3.0,
        }
    }

    fn place_agent(&mut self, entity: Entity, x: i32, y: i32) {
        self.agent_positions.insert(entity, Position { x, y });
    }

    fn place_resource(&mut self, entity: Entity, x: i32, y: i32, resource_type: String) {
        self.resource_positions.insert(entity, (Position { x, y }, resource_type));
    }

    fn distance(&self, pos1: Position, pos2: Position) -> f32 {
        (((pos1.x - pos2.x).pow(2) + (pos1.y - pos2.y).pow(2)) as f32).sqrt()
    }
}

// ============================================================================
// INTEGRATION: WorldQuery trait implementation
// (This trait will be defined in libreconomy once the interfaces are implemented)
// ============================================================================

// TODO: Uncomment when WorldQuery trait is implemented in libreconomy
/*
impl WorldQuery for GridWorld {
    fn get_nearby_agents(&self, agent_entity: Entity, max_count: usize) -> Vec<Entity> {
        let Some(&agent_pos) = self.agent_positions.get(&agent_entity) else {
            return Vec::new();
        };

        // Find nearby agents sorted by distance
        let mut nearby: Vec<(Entity, f32)> = self.agent_positions
            .iter()
            .filter(|(&entity, _)| entity != agent_entity)
            .map(|(&entity, &pos)| (entity, self.distance(agent_pos, pos)))
            .filter(|(_, dist)| *dist <= self.interaction_range)
            .collect();

        nearby.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        nearby.truncate(max_count);
        nearby.into_iter().map(|(entity, _)| entity).collect()
    }

    fn get_nearby_resources(&self, agent_entity: Entity, resource_type: &str) -> Vec<Entity> {
        let Some(&agent_pos) = self.agent_positions.get(&agent_entity) else {
            return Vec::new();
        };

        // Find nearby resource sources of the specified type
        self.resource_positions
            .iter()
            .filter(|(_, (_, rtype))| rtype == resource_type)
            .map(|(entity, (pos, _))| (*entity, self.distance(agent_pos, *pos)))
            .filter(|(_, dist)| *dist <= self.interaction_range)
            .map(|(entity, _)| entity)
            .collect()
    }

    fn can_interact(&self, agent1: Entity, agent2: Entity) -> bool {
        let Some(&pos1) = self.agent_positions.get(&agent1) else { return false; };
        let Some(&pos2) = self.agent_positions.get(&agent2) else { return false; };
        self.distance(pos1, pos2) <= self.interaction_range
    }
}
*/

// ============================================================================
// SIMULATION SETUP
// ============================================================================

fn setup_world_and_agents() -> (World, GridWorld) {
    // Setup ECS world
    let mut world = World::new();

    // Register libreconomy components
    world.register::<Agent>();
    world.register::<Needs>();
    world.register::<Inventory>();
    world.register::<Wallet>();
    world.register::<Skills>();
    world.register::<Knowledge>();
    world.register::<Employment>();
    world.register::<Preferences>();

    // Insert libreconomy resources
    world.insert(AgentIdAllocator::new());

    // Create application's spatial world
    let mut grid_world = GridWorld::new();

    // Create some agents with different needs
    println!("Creating agents...");

    // Agent 1: Very thirsty agent at (5, 5)
    let agent1 = create_agent_with_needs(&mut world, Needs::new(90.0, 30.0));
    grid_world.place_agent(agent1, 5, 5);
    println!("  Agent 1 created at (5, 5) - Very thirsty!");

    // Agent 2: Hungry agent at (8, 8)
    let agent2 = create_agent_with_needs(&mut world, Needs::new(20.0, 85.0));
    grid_world.place_agent(agent2, 8, 8);
    println!("  Agent 2 created at (8, 8) - Very hungry!");

    // Agent 3: Satisfied agent with water to sell at (6, 7)
    let agent3 = create_agent(&mut world);
    {
        let mut inventories = world.write_storage::<Inventory>();
        if let Some(inv) = inventories.get_mut(agent3) {
            inv.add("water", 10);
        }
    }
    grid_world.place_agent(agent3, 6, 7);
    println!("  Agent 3 created at (6, 7) - Has water to sell!");

    // Create resource sources
    // TODO: Uncomment when ResourceSource component is implemented
    /*
    println!("\nCreating resource sources...");

    // Water well at (7, 5)
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
    grid_world.place_resource(well, 7, 5, "water_well".to_string());
    println!("  Water well created at (7, 5)");

    // Farm at (10, 10)
    let farm = world.create_entity()
        .with(ResourceSource {
            resource_type: "farm".to_string(),
            item_produced: "food".to_string(),
            regeneration_rate: 0.5,
            current_stock: 50,
            max_stock: 100,
            requires_skill: Some("farming".to_string()),
        })
        .build();
    grid_world.place_resource(farm, 10, 10, "farm".to_string());
    println!("  Farm created at (10, 10)");
    */

    (world, grid_world)
}

// ============================================================================
// GAME LOOP SIMULATION
// ============================================================================

fn print_agent_status(world: &World) {
    println!("\n=== Agent Status ===");
    let agents = world.read_storage::<Agent>();
    let needs = world.read_storage::<Needs>();
    let inventories = world.read_storage::<Inventory>();
    let wallets = world.read_storage::<Wallet>();

    for (agent, need, inv, wallet) in (&agents, &needs, &inventories, &wallets).join() {
        println!(
            "Agent {}: Thirst={:.1}, Hunger={:.1}, Water={}, Money={:.1}",
            agent.id.0,
            need.thirst,
            need.hunger,
            inv.quantity("water"),
            wallet.currency
        );
    }
}

fn simulate_decision_making(_world: &World, _grid_world: &GridWorld) {
    println!("\n=== Decision Making Phase ===");
    println!("NOTE: Full decision-making system not yet implemented.");
    println!("Once implemented, agents would:");
    println!("  1. Evaluate their needs (thirst, hunger)");
    println!("  2. Query nearby agents via WorldQuery");
    println!("  3. Return DecisionOutput (Intent, Action, or Transaction)");
    println!();

    // TODO: Uncomment when Decision types are implemented
    /*
    let decision_maker = UtilityMaximizationDecisionMaker::new();
    let entities = world.entities();
    let agents = world.read_storage::<Agent>();

    for (entity, _agent) in (&entities, &agents).join() {
        let decision = decision_maker.decide(entity, world, grid_world);

        match decision {
            DecisionOutput::Intent(Intent::SeekItem { item_type, urgency }) => {
                println!("  Agent {} wants to seek {} (urgency: {:.2})", entity.id(), item_type, urgency);
                // Application would find nearest source and pathfind
                let sources = grid_world.get_nearby_resources(entity, &item_type);
                if let Some(source) = sources.first() {
                    println!("    → Found {} nearby, pathfinding...", item_type);
                }
            }

            DecisionOutput::Action(Action { target_agent, action_type }) => {
                println!("  Agent {} wants to perform action with Agent {}", entity.id(), target_agent);
                // Application would move agent into range
                if grid_world.can_interact(entity, target_agent) {
                    println!("    → In range, executing action");
                } else {
                    println!("    → Moving into range...");
                }
            }

            DecisionOutput::Transaction(txn) => {
                if txn.success {
                    println!("  ✓ Transaction completed: {} sold {} to {}",
                        txn.seller, txn.item, txn.buyer);
                }
            }
        }
    }
    */
}

fn run_simple_simulation() {
    println!("╔═══════════════════════════════════════════════════════╗");
    println!("║   libreconomy Simple Integration Example             ║");
    println!("║   Demonstrates library/application separation        ║");
    println!("╚═══════════════════════════════════════════════════════╝\n");

    let (mut world, grid_world) = setup_world_and_agents();

    println!("\n=== Initial State ===");
    print_agent_status(&world);

    // Run a few simulation ticks
    for tick in 1..=5 {
        println!("\n╭─────────────────────────────────────────╮");
        println!("│  Tick {}                                │", tick);
        println!("╰─────────────────────────────────────────╯");

        // 1. Update libreconomy systems
        // Currently only need decay system is implemented
        // TODO: This will be replaced with proper system execution once systems are implemented
        {
            let mut needs = world.write_storage::<Needs>();
            for need in (&mut needs).join() {
                need.thirst = (need.thirst + 2.0).min(100.0);
                need.hunger = (need.hunger + 1.5).min(100.0);
                need.clamp();
            }
        }

        // 2. Get decisions from agents (future)
        simulate_decision_making(&world, &grid_world);

        // 3. Execute decisions in your world (future)
        // This is where your application would:
        // - Move agents based on Intents
        // - Execute Actions when agents are in range
        // - Show UI feedback for Transactions

        // 4. Show current state
        print_agent_status(&world);
    }

    println!("\n╔═══════════════════════════════════════════════════════╗");
    println!("║   Integration Lessons                                 ║");
    println!("╚═══════════════════════════════════════════════════════╝");
    println!();
    println!("✓ libreconomy manages economic state (Needs, Inventory, Wallet)");
    println!("✓ Your app manages spatial state (Position in GridWorld)");
    println!("✓ Both share the same ECS World (specs)");
    println!();
    println!("Next steps:");
    println!("  1. Implement WorldQuery trait for your world");
    println!("  2. Handle DecisionOutput types in your game loop");
    println!("  3. Add pathfinding for Intent execution");
    println!("  4. Add UI feedback for Transaction results");
    println!();
    println!("See docs/ARCHITECTURE.md for complete integration guide!");
    println!();
}

// ============================================================================
// MAIN
// ============================================================================

fn main() {
    run_simple_simulation();
}

// ============================================================================
// REFERENCE: Integration Patterns
// ============================================================================

/// This section shows the patterns you'll use once the full API is implemented.
/// Copy these patterns into your own application code.

#[allow(dead_code)]
mod integration_patterns {
    use super::*;

    /// Pattern 1: Implementing WorldQuery
    ///
    /// Your application implements this trait to provide spatial context
    /// to libreconomy without exposing your internal world representation.
    #[allow(unused_variables)]
    fn example_worldquery_implementation() {
        // See the GridWorld impl WorldQuery block above (commented out)
        // This shows how to implement the trait for a 2D grid world.
        //
        // You can implement it for any world type:
        // - 3D continuous space
        // - Graph-based networks
        // - Tile-based maps
        // - Abstract connection graphs
    }

    /// Pattern 2: Setting up ItemRegistry
    ///
    /// Register items that agents can use to satisfy needs.
    #[allow(unused_variables)]
    fn example_item_registry() {
        // TODO: Uncomment when ItemRegistry is implemented
        /*
        let mut registry = ItemRegistry::with_defaults();

        // Override default water
        registry.register(ItemType {
            id: "water".to_string(),
            name: "Purified Water".to_string(),
            satisfies: [(NeedType::Thirst, -50.0)].into(),
            consumable: true,
            durability: None,
            stack_size: 5,
        });

        // Add custom item
        registry.register(ItemType {
            id: "healing_potion".to_string(),
            name: "Healing Potion".to_string(),
            satisfies: [(NeedType::Health, -80.0)].into(),
            consumable: true,
            durability: None,
            stack_size: 3,
        });
        */
    }

    /// Pattern 3: Handling Decision Outputs
    ///
    /// Process decisions from libreconomy in your game loop.
    #[allow(unused_variables)]
    fn example_decision_handling(world: &World, grid_world: &GridWorld) {
        // TODO: Uncomment when Decision types are implemented
        /*
        let decision_maker = UtilityMaximizationDecisionMaker::new();

        // For each agent
        for (entity, agent) in (&world.entities(), &world.read_storage::<Agent>()).join() {
            let decision = decision_maker.decide(entity, world, grid_world);

            match decision {
                // High-level intent: app finds target and pathfinds
                DecisionOutput::Intent(intent) => {
                    match intent {
                        Intent::SeekItem { item_type, urgency } => {
                            let sources = grid_world.get_nearby_resources(entity, &item_type);
                            // Your pathfinding code here
                        }
                        Intent::FindWork { skill_types } => {
                            // Find employers with your job matching system
                        }
                        _ => {}
                    }
                }

                // Specific action: app moves agent to target
                DecisionOutput::Action(Action { target_agent, action_type }) => {
                    if grid_world.can_interact(entity, target_agent) {
                        // Execute immediately
                        // libreconomy::execute_action(world, entity, target_agent, action_type);
                    } else {
                        // Queue for execution after movement
                        // your_pathfinding.move_to_agent(entity, target_agent);
                    }
                }

                // Immediate transaction: library handled it, show UI
                DecisionOutput::Transaction(txn) => {
                    if txn.success {
                        // your_ui.show_message("Trade completed!");
                        // your_audio.play_sound("coin_clink");
                    }
                }
            }
        }
        */
    }

    /// Pattern 4: Complete Game Loop
    ///
    /// Structure of a typical game loop with libreconomy integration.
    #[allow(unused_variables)]
    fn example_game_loop(world: &mut World, grid_world: &mut GridWorld) {
        loop {
            // 1. Update libreconomy systems
            // run_libreconomy_systems(world);

            // 2. Get decisions from agents
            // let decisions = get_all_agent_decisions(world, grid_world);

            // 3. Execute decisions in your world
            // for (agent, decision) in decisions {
            //     execute_decision(world, grid_world, agent, decision);
            // }

            // 4. Update your game systems
            // update_movement(grid_world);
            // update_animations(world);

            // 5. Render
            // render_frame(world, grid_world);

            // 6. Sleep/vsync
            // thread::sleep(Duration::from_millis(16)); // 60 FPS
        }
    }
}
