//! Minimal example: setting up a World and agents
use specs::prelude::*;
use libreconomy::agent::components::*;

fn main() {
    // Create a new ECS World
    let mut world = World::new();

    // Register components
    world.register::<Needs>();
    world.register::<Inventory>();
    world.register::<Wallet>();
    world.register::<Skills>();

    // Create agent entities
    let agent1 = world.create_entity()
        .with(Needs { thirst: 0.5, hunger: 0.8 })
        .with(Inventory { items: std::collections::HashMap::new() })
        .with(Wallet { currency: 100.0 })
        .with(Skills { skills: std::collections::HashMap::new() })
        .build();

    let agent2 = world.create_entity()
        .with(Needs { thirst: 0.2, hunger: 0.4 })
        .with(Inventory { items: std::collections::HashMap::new() })
        .with(Wallet { currency: 50.0 })
        .with(Skills { skills: std::collections::HashMap::new() })
        .build();

    // TODO: Add systems and run simulation loop
    println!("Created agents: {:?}, {:?}", agent1, agent2);
}
