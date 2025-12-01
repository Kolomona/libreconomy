use libreconomy::{Agent, AgentIdAllocator, Inventory, Needs, Skills, Wallet};
use specs::prelude::*;

fn main() {
	let mut world = World::new();
	world.register::<Needs>();
	world.register::<Inventory>();
	world.register::<Wallet>();
	world.register::<Skills>();
	world.register::<Agent>();

	// Insert the AgentId allocator resource
	world.insert(AgentIdAllocator::new());

	// Allocate a unique AgentId from the resource
	let id = {
		let mut alloc = world.write_resource::<AgentIdAllocator>();
		alloc.allocate().expect("allocate AgentId")
	};

	let agent = world.create_entity()
		.with(Needs { thirst: 0.5, hunger: 0.8 })
		.with(Inventory { items: std::collections::HashMap::new() })
		.with(Wallet { currency: 100.0 })
		.with(Skills { skills: std::collections::HashMap::new() })
		.with(Agent { id })
		.build();

	println!("Created agent: {:?}", agent);
}

