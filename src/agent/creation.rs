//! Agent creation and lifecycle management
//! Functions for creating and managing agent entities with their components

use specs::prelude::*;
use super::components::{Agent, Needs, Inventory, Wallet};
use super::identity::AgentIdAllocator;

/// Default starting needs for a new agent (mid-range)
const DEFAULT_THIRST: f32 = 50.0;
const DEFAULT_HUNGER: f32 = 50.0;

/// Default starting currency for a new agent
const DEFAULT_CURRENCY: f32 = 100.0;

/// Create a new agent with default components
///
/// # Default Components
/// - Needs: thirst=50.0, hunger=50.0
/// - Inventory: empty
/// - Wallet: currency=100.0
///
/// # Panics
/// Panics if AgentIdAllocator resource is not registered in the world
/// Panics if required component types are not registered
pub fn create_agent(world: &mut World) -> Entity {
    let needs = Needs::new(DEFAULT_THIRST, DEFAULT_HUNGER);
    let inventory = Inventory::default();
    let wallet = Wallet::new(DEFAULT_CURRENCY);
    
    create_agent_custom(world, needs, inventory, wallet)
}

/// Create a new agent with custom needs and default inventory/wallet
///
/// # Arguments
/// * `world` - ECS world to create the agent in
/// * `needs` - Custom needs component
///
/// # Panics
/// Panics if AgentIdAllocator resource is not registered in the world
/// Panics if required component types are not registered
pub fn create_agent_with_needs(world: &mut World, needs: Needs) -> Entity {
    let inventory = Inventory::default();
    let wallet = Wallet::new(DEFAULT_CURRENCY);
    
    create_agent_custom(world, needs, inventory, wallet)
}

/// Create a new agent with custom wallet and default needs/inventory
///
/// # Arguments
/// * `world` - ECS world to create the agent in
/// * `wallet` - Custom wallet component
///
/// # Panics
/// Panics if AgentIdAllocator resource is not registered in the world
/// Panics if required component types are not registered
pub fn create_agent_with_wallet(world: &mut World, wallet: Wallet) -> Entity {
    let needs = Needs::new(DEFAULT_THIRST, DEFAULT_HUNGER);
    let inventory = Inventory::default();
    
    create_agent_custom(world, needs, inventory, wallet)
}

/// Create a new agent with fully custom components
///
/// # Arguments
/// * `world` - ECS world to create the agent in
/// * `needs` - Custom needs component
/// * `inventory` - Custom inventory component
/// * `wallet` - Custom wallet component
///
/// # Panics
/// Panics if AgentIdAllocator resource is not registered in the world
/// Panics if required component types are not registered
/// Panics if AgentId allocation fails (overflow)
pub fn create_agent_custom(
    world: &mut World,
    needs: Needs,
    inventory: Inventory,
    wallet: Wallet,
) -> Entity {
    // Allocate unique AgentId
    let agent_id = {
        let mut allocator = world.write_resource::<AgentIdAllocator>();
        allocator.allocate().expect("AgentId allocation failed")
    };

    // Create entity with all components
    world
        .create_entity()
        .with(Agent { id: agent_id })
        .with(needs)
        .with(inventory)
        .with(wallet)
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use super::super::identity::AgentId;

    #[test]
    fn test_create_agent_assigns_components() {
        // Arrange
        let mut world = World::new();
        world.register::<Agent>();
        world.register::<Needs>();
        world.register::<Inventory>();
        world.register::<Wallet>();
        world.insert(AgentIdAllocator::new());

        // Act
        let entity = create_agent(&mut world);

        // Assert - entity exists
        assert!(world.entities().is_alive(entity));

        // Assert - has Agent component
        let agents = world.read_storage::<Agent>();
        let agent = agents.get(entity).expect("should have Agent component");
        assert_eq!(agent.id, AgentId(1));

        // Assert - has Needs component with defaults
        let needs_storage = world.read_storage::<Needs>();
        let needs = needs_storage.get(entity).expect("should have Needs component");
        assert_eq!(needs.thirst, DEFAULT_THIRST);
        assert_eq!(needs.hunger, DEFAULT_HUNGER);

        // Assert - has Inventory component (empty)
        let inventory_storage = world.read_storage::<Inventory>();
        let inventory = inventory_storage.get(entity).expect("should have Inventory component");
        assert_eq!(inventory.items.len(), 0);

        // Assert - has Wallet component with default
        let wallet_storage = world.read_storage::<Wallet>();
        let wallet = wallet_storage.get(entity).expect("should have Wallet component");
        assert_eq!(wallet.currency, DEFAULT_CURRENCY);
    }

    #[test]
    fn test_create_multiple_agents_unique_ids() {
        // Arrange
        let mut world = World::new();
        world.register::<Agent>();
        world.register::<Needs>();
        world.register::<Inventory>();
        world.register::<Wallet>();
        world.insert(AgentIdAllocator::new());

        // Act
        let e1 = create_agent(&mut world);
        let e2 = create_agent(&mut world);
        let e3 = create_agent(&mut world);

        // Assert - all exist
        assert!(world.entities().is_alive(e1));
        assert!(world.entities().is_alive(e2));
        assert!(world.entities().is_alive(e3));

        // Assert - unique IDs
        let agents = world.read_storage::<Agent>();
        let id1 = agents.get(e1).unwrap().id;
        let id2 = agents.get(e2).unwrap().id;
        let id3 = agents.get(e3).unwrap().id;

        assert_eq!(id1, AgentId(1));
        assert_eq!(id2, AgentId(2));
        assert_eq!(id3, AgentId(3));
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

        // Act
        let entity = create_agent_with_needs(&mut world, custom_needs);

        // Assert
        let needs_storage = world.read_storage::<Needs>();
        let needs = needs_storage.get(entity).unwrap();
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

        // Act
        let entity = create_agent_with_wallet(&mut world, custom_wallet);

        // Assert
        let wallet_storage = world.read_storage::<Wallet>();
        let wallet = wallet_storage.get(entity).unwrap();
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

        // Act
        let entity = create_agent_custom(
            &mut world,
            custom_needs,
            custom_inventory.clone(),
            custom_wallet,
        );

        // Assert - needs
        let needs_storage = world.read_storage::<Needs>();
        let needs = needs_storage.get(entity).unwrap();
        assert_eq!(needs.thirst, 80.0);
        assert_eq!(needs.hunger, 30.0);

        // Assert - inventory
        let inventory_storage = world.read_storage::<Inventory>();
        let inventory = inventory_storage.get(entity).unwrap();
        assert_eq!(inventory.quantity("water"), 5);
        assert_eq!(inventory.quantity("food"), 3);

        // Assert - wallet
        let wallet_storage = world.read_storage::<Wallet>();
        let wallet = wallet_storage.get(entity).unwrap();
        assert_eq!(wallet.currency, 250.0);

        // Assert - agent id
        let agents = world.read_storage::<Agent>();
        let agent = agents.get(entity).unwrap();
        assert_eq!(agent.id, AgentId(1));
    }
}
