#[test]
fn phase5_register_and_query_agent_components() {
    use libreconomy::{Agent, Needs, Inventory, Wallet, AgentIdAllocator};
    use specs::prelude::*;

    // Create ECS world and register agent/core components
    let mut world = World::new();
    world.register::<Agent>();
    world.register::<Needs>();
    world.register::<Inventory>();
    world.register::<Wallet>();
    world.insert(AgentIdAllocator::new());

    // Create several agents
    let agent1 = libreconomy::create_agent(&mut world);
    let agent2 = libreconomy::create_agent_with_needs(&mut world, Needs::new(80.0, 20.0));
    let agent3 = libreconomy::create_agent_with_wallet(&mut world, Wallet::new(500.0));

    // Query all Agent entities
    let agents = world.read_storage::<Agent>();
    let entities = world.entities();
    let mut found_ids: Vec<u64> = (&entities, &agents).join().map(|(_e, a)| a.id.0).collect();
    found_ids.sort();
    assert_eq!(found_ids, vec![1, 2, 3]);

    // Query Needs for agent2
    let needs_storage = world.read_storage::<Needs>();
    let needs2 = needs_storage.get(agent2).unwrap();
    assert_eq!(needs2.thirst, 80.0);
    assert_eq!(needs2.hunger, 20.0);

    // Query Wallet for agent3
    let wallet_storage = world.read_storage::<Wallet>();
    let wallet3 = wallet_storage.get(agent3).unwrap();
    assert_eq!(wallet3.currency, 500.0);

    // Query Inventory for agent1 (should be empty)
    let inventory_storage = world.read_storage::<Inventory>();
    let inv1 = inventory_storage.get(agent1).unwrap();
    assert_eq!(inv1.items.len(), 0);
}
use pretty_assertions::assert_eq;
use specs::prelude::*;

use libreconomy::{Agent, AgentId, AgentIdAllocator};

#[test]
fn agent_id_allocator_uniqueness_and_ordering() {
    // Arrange
    let mut alloc = AgentIdAllocator::new();

    // Act
    let a1 = alloc.allocate().expect("first id");
    let a2 = alloc.allocate().expect("second id");
    let a3 = alloc.allocate().expect("third id");

    // Assert
    assert_eq!(a1, AgentId(1));
    assert_eq!(a2, AgentId(2));
    assert_eq!(a3, AgentId(3));
    assert!(a1 != a2 && a2 != a3 && a1 != a3);
}

#[test]
fn ecs_can_register_agent_component_and_store_entities() {
    // Arrange
    let mut world = World::new();
    world.register::<Agent>();
    world.insert(AgentIdAllocator::new());

    // Allocate two unique ids using the resource allocator
    let id1 = {
        let mut alloc = world.write_resource::<AgentIdAllocator>();
        alloc.allocate().unwrap()
    };

    let id2 = {
        let mut alloc = world.write_resource::<AgentIdAllocator>();
        alloc.allocate().unwrap()
    };

    // Act: create two Agent entities
    let e1 = world.create_entity().with(Agent { id: id1 }).build();
    let e2 = world.create_entity().with(Agent { id: id2 }).build();

    // Assert: storage contains two entries with distinct ids
    let storage = world.read_storage::<Agent>();
    let entities = world.entities();

    // Count
    let count = (&entities, &storage).join().count();
    assert_eq!(count, 2);

    // Collect ids to verify uniqueness and association
    let mut ids: Vec<AgentId> = (&entities, &storage)
        .join()
        .map(|(_e, a)| a.id)
        .collect();
    ids.sort_by_key(|id| id.0);
    assert_eq!(ids, vec![AgentId(1), AgentId(2)]);

    // Ensure created entities exist
    assert!(entities.is_alive(e1));
    assert!(entities.is_alive(e2));
}
