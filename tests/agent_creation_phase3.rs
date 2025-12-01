//! Phase 3: Agent Creation Logic Tests
//! TDD tests for agent creation functionality

use pretty_assertions::assert_eq;
use specs::prelude::*;

use libreconomy::{Agent, AgentId, AgentIdAllocator, Needs, Inventory, Wallet};

#[test]
fn test_remove_agent_from_world() {
    // Arrange
    let mut world = World::new();
    world.register::<Agent>();
    world.register::<Needs>();
    world.register::<Inventory>();
    world.register::<Wallet>();
    world.insert(AgentIdAllocator::new());

    // Create agent
    let entity = libreconomy::create_agent(&mut world);
    assert!(world.entities().is_alive(entity));

    // Remove agent (should use new function)
    // This will fail until implemented
    libreconomy::remove_agent(&mut world, entity);

    // Assert: Entity no longer exists
    assert!(!world.entities().is_alive(entity));
    // Assert: Components are removed
    let agents = world.read_storage::<Agent>();
    assert!(agents.get(entity).is_none());
    let needs = world.read_storage::<Needs>();
    assert!(needs.get(entity).is_none());
    let inventory = world.read_storage::<Inventory>();
    assert!(inventory.get(entity).is_none());
    let wallet = world.read_storage::<Wallet>();
    assert!(wallet.get(entity).is_none());
}

#[test]
fn test_create_agent_with_default_components() {
    // Arrange
    let mut world = World::new();
    world.register::<Agent>();
    world.register::<Needs>();
    world.register::<Inventory>();
    world.register::<Wallet>();
    world.insert(AgentIdAllocator::new());

    // Act: Create agent with default components
    let entity = libreconomy::create_agent(&mut world);

    // Assert: Entity exists
    assert!(world.entities().is_alive(entity));

    // Assert: Has Agent component with valid ID
    let agents = world.read_storage::<Agent>();
    let agent = agents.get(entity).expect("Agent component should exist");
    assert_eq!(agent.id, AgentId(1));

    // Assert: Has Needs component with default values
    let needs_storage = world.read_storage::<Needs>();
    let needs = needs_storage.get(entity).expect("Needs component should exist");
    assert_eq!(needs.thirst, 50.0);
    assert_eq!(needs.hunger, 50.0);

    // Assert: Has Inventory component (empty)
    let inventory_storage = world.read_storage::<Inventory>();
    let inventory = inventory_storage.get(entity).expect("Inventory component should exist");
    assert_eq!(inventory.items.len(), 0);

    // Assert: Has Wallet component with starting balance
    let wallet_storage = world.read_storage::<Wallet>();
    let wallet = wallet_storage.get(entity).expect("Wallet component should exist");
    assert_eq!(wallet.currency, 100.0);
}

#[test]
fn test_create_multiple_agents_have_unique_ids() {
    // Arrange
    let mut world = World::new();
    world.register::<Agent>();
    world.register::<Needs>();
    world.register::<Inventory>();
    world.register::<Wallet>();
    world.insert(AgentIdAllocator::new());

    // Act: Create multiple agents
    let entity1 = libreconomy::create_agent(&mut world);
    let entity2 = libreconomy::create_agent(&mut world);
    let entity3 = libreconomy::create_agent(&mut world);

    // Assert: All entities exist
    assert!(world.entities().is_alive(entity1));
    assert!(world.entities().is_alive(entity2));
    assert!(world.entities().is_alive(entity3));

    // Assert: Each has unique AgentId
    let agents = world.read_storage::<Agent>();
    let id1 = agents.get(entity1).unwrap().id;
    let id2 = agents.get(entity2).unwrap().id;
    let id3 = agents.get(entity3).unwrap().id;

    assert_eq!(id1, AgentId(1));
    assert_eq!(id2, AgentId(2));
    assert_eq!(id3, AgentId(3));
    assert!(id1 != id2 && id2 != id3 && id1 != id3);
}

#[test]
fn test_create_agent_with_custom_needs() {
    // Arrange
    let mut world = World::new();
    world.register::<Agent>();
    world.register::<Needs>();
    world.register::<Inventory>();
    world.register::<Wallet>();
    world.insert(AgentIdAllocator::new());

    let custom_needs = Needs::new(75.0, 25.0);

    // Act: Create agent with custom needs
    let entity = libreconomy::create_agent_with_needs(&mut world, custom_needs);

    // Assert: Entity has custom needs
    let needs_storage = world.read_storage::<Needs>();
    let needs = needs_storage.get(entity).expect("Needs component should exist");
    assert_eq!(needs.thirst, 75.0);
    assert_eq!(needs.hunger, 25.0);
}

#[test]
fn test_create_agent_with_custom_wallet() {
    // Arrange
    let mut world = World::new();
    world.register::<Agent>();
    world.register::<Needs>();
    world.register::<Inventory>();
    world.register::<Wallet>();
    world.insert(AgentIdAllocator::new());

    let custom_wallet = Wallet::new(500.0);

    // Act: Create agent with custom wallet
    let entity = libreconomy::create_agent_with_wallet(&mut world, custom_wallet);

    // Assert: Entity has custom wallet
    let wallet_storage = world.read_storage::<Wallet>();
    let wallet = wallet_storage.get(entity).expect("Wallet component should exist");
    assert_eq!(wallet.currency, 500.0);
}

#[test]
fn test_create_agent_fully_custom() {
    // Arrange
    let mut world = World::new();
    world.register::<Agent>();
    world.register::<Needs>();
    world.register::<Inventory>();
    world.register::<Wallet>();
    world.insert(AgentIdAllocator::new());

    let custom_needs = Needs::new(80.0, 30.0);
    let custom_wallet = Wallet::new(250.0);
    let mut custom_inventory = Inventory::default();
    custom_inventory.add("water", 5);
    custom_inventory.add("food", 3);

    // Act: Create fully customized agent
    let entity = libreconomy::create_agent_custom(
        &mut world,
        custom_needs,
        custom_inventory.clone(),
        custom_wallet,
    );

    // Assert: Entity has all custom components
    let needs_storage = world.read_storage::<Needs>();
    let needs = needs_storage.get(entity).unwrap();
    assert_eq!(needs.thirst, 80.0);
    assert_eq!(needs.hunger, 30.0);

    let inventory_storage = world.read_storage::<Inventory>();
    let inventory = inventory_storage.get(entity).unwrap();
    assert_eq!(inventory.quantity("water"), 5);
    assert_eq!(inventory.quantity("food"), 3);

    let wallet_storage = world.read_storage::<Wallet>();
    let wallet = wallet_storage.get(entity).unwrap();
    assert_eq!(wallet.currency, 250.0);

    let agents = world.read_storage::<Agent>();
    let agent = agents.get(entity).unwrap();
    assert_eq!(agent.id, AgentId(1));
}
