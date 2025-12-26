//! Minimal decision-making example
//!
//! Demonstrates how to use libreconomy's decision system with a simple WorldQuery implementation.

use libreconomy::*;
use specs::prelude::*;

/// Simple WorldQuery implementation for testing
struct SimpleWorldQuery {
    water_at: Option<(f32, f32)>,  // (x, y) position
    food_at: Option<(f32, f32)>,
}

impl WorldQuery for SimpleWorldQuery {
    fn get_nearby_agents(&self, _agent: AgentId, _max_count: usize) -> Vec<AgentId> {
        // No other agents in this simple example
        Vec::new()
    }

    fn get_nearby_resources(
        &self,
        _agent: AgentId,
        resource_type: &str,
        max_radius: f32,
    ) -> Vec<ResourceLocation> {
        match resource_type {
            "water" => {
                if let Some((x, y)) = self.water_at {
                    let distance = (x * x + y * y).sqrt();
                    if distance <= max_radius {
                        vec![ResourceLocation::new(x, y, distance)]
                    } else {
                        Vec::new()
                    }
                } else {
                    Vec::new()
                }
            }
            "food" | "grass" => {
                if let Some((x, y)) = self.food_at {
                    let distance = (x * x + y * y).sqrt();
                    if distance <= max_radius {
                        vec![ResourceLocation::new(x, y, distance)]
                    } else {
                        Vec::new()
                    }
                } else {
                    Vec::new()
                }
            }
            _ => Vec::new(),
        }
    }

    fn can_interact(&self, _agent1: AgentId, _agent2: AgentId) -> bool {
        true
    }
}

fn main() {
    println!("=== Libreconomy Decision System Example ===\n");

    // Create ECS world
    let mut world = World::new();
    world.register::<Agent>();
    world.register::<Needs>();
    world.register::<Inventory>();
    world.register::<Wallet>();
    world.register::<SpeciesComponent>();
    world.insert(AgentIdAllocator::new());

    println!("1. Creating agents with different needs...");

    // Create a thirsty human
    let thirsty_human = {
        let needs = Needs::new(85.0, 30.0, 20.0); // Very thirsty
        let inventory = Inventory::default();
        let wallet = Wallet::new(100.0);
        create_agent_custom(&mut world, needs, inventory, wallet)
    };
    println!("   - Thirsty human (thirst: 85.0, hunger: 30.0)");

    // Create a hungry rabbit
    let hungry_rabbit = {
        let needs = Needs::new(20.0, 80.0, 30.0); // Very hungry
        let inventory = Inventory::default();
        let wallet = Wallet::new(50.0);
        let entity = create_agent_custom(&mut world, needs, inventory, wallet);

        // Add species component
        let mut species_storage = world.write_storage::<SpeciesComponent>();
        species_storage.insert(entity, SpeciesComponent::rabbit()).unwrap();
        entity
    };
    println!("   - Hungry rabbit (thirst: 20.0, hunger: 80.0)");

    // Create tired agent
    let tired_agent = {
        let needs = Needs::new(30.0, 40.0, 90.0); // Very tired
        let inventory = Inventory::default();
        let wallet = Wallet::new(75.0);
        create_agent_custom(&mut world, needs, inventory, wallet)
    };
    println!("   - Tired agent (thirst: 30.0, hunger: 40.0, tiredness: 90.0)\n");

    // Create decision maker
    let decision_maker = UtilityMaximizer::default();

    // Scenario 1: Water nearby
    println!("2. Scenario: Water source at (100, 100)");
    let world_query = SimpleWorldQuery {
        water_at: Some((100.0, 100.0)),
        food_at: None,
    };

    let decision = decision_maker.decide(thirsty_human, &world, &world_query);
    println!("   Thirsty human decision: {:?}\n", decision);

    // Scenario 2: Food nearby (grass for rabbit)
    println!("3. Scenario: Grass at (150, 150)");
    let world_query = SimpleWorldQuery {
        water_at: None,
        food_at: Some((150.0, 150.0)),
    };

    let decision = decision_maker.decide(hungry_rabbit, &world, &world_query);
    println!("   Hungry rabbit decision: {:?}", decision);
    match decision {
        DecisionOutput::Intent(Intent::SeekItem { item_type, urgency }) => {
            println!("   -> Seeking {} (urgency: {:.2})", item_type, urgency);
            if item_type == "grass" {
                println!("   ✓ Rabbit correctly seeks grass (herbivore diet)\n");
            }
        }
        _ => println!("   Unexpected decision\n"),
    }

    // Scenario 3: No resources, very tired
    println!("4. Scenario: No resources nearby");
    let world_query = SimpleWorldQuery {
        water_at: None,
        food_at: None,
    };

    let decision = decision_maker.decide(tired_agent, &world, &world_query);
    println!("   Tired agent decision: {:?}", decision);
    match decision {
        DecisionOutput::Intent(Intent::Rest) => {
            println!("   ✓ Agent correctly chooses to rest when tired\n");
        }
        _ => println!(),
    }

    // Demonstrate reputation system
    println!("5. Reputation System:");
    let mut reputation = ReputationKnowledge::new();

    let partner1 = AgentId(100);
    let partner2 = AgentId(101);

    // Positive interactions with partner1
    for i in 0..5 {
        reputation.update_reputation(partner1, 1.0, i * 100);
    }

    // Mixed interactions with partner2
    reputation.update_reputation(partner2, 1.0, 100);
    reputation.update_reputation(partner2, -2.0, 200);

    println!("   Partner 1 reputation: {:.2} (5 positive interactions)", reputation.get_score(partner1));
    println!("   Partner 2 reputation: {:.2} (1 positive, 1 negative)", reputation.get_score(partner2));
    println!("   Partner 1 trusted (>0.7)? {}", reputation.is_trusted(partner1, 0.7));
    println!("   Partner 2 trusted (>0.7)? {}\n", reputation.is_trusted(partner2, 0.7));

    // Item registry
    println!("6. Item Registry:");
    let registry = ItemRegistry::with_defaults();

    println!("   Items that satisfy hunger: {:?}", registry.items_satisfying(NeedType::Hunger));
    println!("   Items that satisfy thirst: {:?}", registry.items_satisfying(NeedType::Thirst));
    println!("   Water satisfies thirst by: {:.0}", registry.get("water").unwrap().satisfaction_for(NeedType::Thirst));
    println!("   Grass satisfies hunger by: {:.0}\n", registry.get("grass").unwrap().satisfaction_for(NeedType::Hunger));

    println!("=== Example Complete ===");
}
