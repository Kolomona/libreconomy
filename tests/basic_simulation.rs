use libreconomy::*;

use specs::prelude::*;

#[test]
fn test_create_world_with_agents() {
    let mut world = World::new();
    world.register::<Needs>();
    world.register::<Inventory>();
    world.register::<Wallet>();
    world.register::<Skills>();

    let agent = world.create_entity()
        .with(Needs::new(0.5, 0.8))
        .with(Inventory::default())
        .with(Wallet::new(100.0))
        .with(Skills::default())
        .build();

    assert!(world.entities().is_alive(agent));
    // TODO: Add more assertions for agent state
}

#[test]
fn test_example_agent_creation_pattern() {
    // This mirrors the example usage and enforces TDD for the example path
    let mut world = World::new();
    world.register::<Needs>();
    world.register::<Inventory>();
    world.register::<Wallet>();
    world.register::<Skills>();
    world.register::<Agent>();

    // Insert allocator and allocate an AgentId
    world.insert(AgentIdAllocator::new());
    let id = {
        let mut alloc = world.write_resource::<AgentIdAllocator>();
        alloc.allocate().expect("allocate AgentId")
    };

    // Create one Agent entity with the allocated id (use helpers)
    let e = world
        .create_entity()
        .with(Needs::new(-1.0, 200.0))
        .with(Inventory::default())
        .with(Wallet::new(-10.0))
        .with(Skills::default())
        .with(Agent { id })
        .build();

    // Assert exactly one Agent in storage and id is positive
    let entities = world.entities();
    let storage = world.read_storage::<Agent>();
    let mut ids: Vec<AgentId> = (&entities, &storage).join().map(|(_e, a)| a.id).collect();
    assert_eq!(ids.len(), 1);
    assert!(ids.pop().unwrap().0 > 0);

    // Verify components behave per Phase 2 helpers
    {
        let needs_storage = world.read_storage::<Needs>();
        let needs = needs_storage.get(e).unwrap();
        assert_eq!(needs.thirst, MIN_NEEDS);
        assert_eq!(needs.hunger, MAX_NEEDS);
    }
    {
        let mut wallet_storage = world.write_storage::<Wallet>();
        let wallet = wallet_storage.get_mut(e).unwrap();
        assert_eq!(wallet.currency, 0.0);
        wallet.deposit(5.0);
        assert_eq!(wallet.currency, 5.0);
        let taken = wallet.withdraw(10.0);
        assert_eq!(taken, 5.0);
        assert_eq!(wallet.currency, 0.0);
    }
    {
        let mut inv_storage = world.write_storage::<Inventory>();
        let inv = inv_storage.get_mut(e).unwrap();
        assert_eq!(inv.quantity("water"), 0);
        inv.add("water", 2);
        assert_eq!(inv.quantity("water"), 2);
        let removed = inv.remove("water", 3);
        assert_eq!(removed, 2);
        assert_eq!(inv.quantity("water"), 0);
    }
}

#[test]
fn test_libreconomy_version_ffi() {
    extern "C" {
        fn libreconomy_version() -> *const u8;
    }
    unsafe {
        let ptr = libreconomy_version();
        let c_str = std::ffi::CStr::from_ptr(ptr as *const i8);
        assert_eq!(c_str.to_str().unwrap(), "libreconomy 0.0.1");
    }
}

#[cfg(test)]
mod ecs_missing_components_tests {
    use super::*;

    #[test]
    fn test_knowledge_component_creation() {
        // Should fail: Knowledge not implemented yet
        let _k = Knowledge { known_prices: std::collections::HashMap::new(), trade_partners: vec![] };
    }

    #[test]
    fn test_employment_component_creation() {
        // Should fail: Employment not implemented yet
        let _e = Employment { job_status: None, employer: None, employees: vec![] };
    }

    #[test]
    fn test_preferences_component_creation() {
        // Should fail: Preferences not implemented yet
        let _p = Preferences { utility_function: UtilityFunctionType::Linear, risk_tolerance: 0.5 };
    }

    #[test]
    fn test_need_decay_system() {
        let mut needs = Needs { thirst: 0.2, hunger: 0.3 };
        NeedDecaySystem::tick(&mut needs);
        assert!(needs.thirst > 0.2 && needs.hunger > 0.3);
    }

    #[test]
    fn test_learning_system() {
        let mut knowledge = Knowledge { known_prices: std::collections::HashMap::new(), trade_partners: vec![] };
        LearningSystem::update(&mut knowledge, "water", 1.5);
        assert_eq!(knowledge.known_prices.get("water"), Some(&1.5));
    }

    #[test]
    fn test_negotiation_system() {
        assert!(NegotiationSystem::negotiate());
    }
}

#[cfg(test)]
mod phase3_agent_creation_tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_create_agent_default() {
        // Test basic agent creation function
        let mut world = World::new();
        world.register::<Agent>();
        world.register::<Needs>();
        world.register::<Inventory>();
        world.register::<Wallet>();
        world.insert(AgentIdAllocator::new());

        let entity = create_agent(&mut world);

        assert!(world.entities().is_alive(entity));
        let agents = world.read_storage::<Agent>();
        let agent = agents.get(entity).unwrap();
        assert_eq!(agent.id, AgentId(1));
    }

    #[test]
    fn test_create_multiple_agents_showcase() {
        // Showcase creating multiple agents with different configurations
        let mut world = World::new();
        world.register::<Agent>();
        world.register::<Needs>();
        world.register::<Inventory>();
        world.register::<Wallet>();
        world.insert(AgentIdAllocator::new());

        // Create agent with defaults
        let agent1 = create_agent(&mut world);

        // Create agent with custom needs (thirsty)
        let agent2 = create_agent_with_needs(&mut world, Needs::new(90.0, 50.0));

        // Create agent with custom wallet (rich)
        let agent3 = create_agent_with_wallet(&mut world, Wallet::new(1000.0));

        // Create fully custom agent
        let mut inventory = Inventory::default();
        inventory.add("water", 10);
        inventory.add("food", 5);
        let agent4 = create_agent_custom(
            &mut world,
            Needs::new(20.0, 30.0),
            inventory,
            Wallet::new(500.0),
        );

        // Verify all agents exist and have unique IDs
        let agents = world.read_storage::<Agent>();
        assert_eq!(agents.get(agent1).unwrap().id, AgentId(1));
        assert_eq!(agents.get(agent2).unwrap().id, AgentId(2));
        assert_eq!(agents.get(agent3).unwrap().id, AgentId(3));
        assert_eq!(agents.get(agent4).unwrap().id, AgentId(4));

        // Verify custom properties
        let needs = world.read_storage::<Needs>();
        assert_eq!(needs.get(agent2).unwrap().thirst, 90.0);

        let wallets = world.read_storage::<Wallet>();
        assert_eq!(wallets.get(agent3).unwrap().currency, 1000.0);

        let inventories = world.read_storage::<Inventory>();
        assert_eq!(inventories.get(agent4).unwrap().quantity("water"), 10);
    }
}

#[cfg(test)]
mod godot_ffi_tests {
    #[test]
    fn test_godot_ffi_entrypoint() {
        extern "C" {
            fn libreconomy_version() -> *const u8;
        }
        unsafe {
            let ptr = libreconomy_version();
            let c_str = std::ffi::CStr::from_ptr(ptr as *const i8);
            assert_eq!(c_str.to_str().unwrap(), "libreconomy 0.0.1");
        }
    }
}
