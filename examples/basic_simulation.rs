use libreconomy::{Agent, AgentIdAllocator, Inventory, Needs, Skills, Wallet, MIN_NEEDS, MAX_NEEDS};
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

	// Create an Agent with Phase 2 helpers
	let agent = world.create_entity()
		.with(Needs::new(-1.0, 200.0)) // intentionally out of bounds to demonstrate clamping
		.with(Inventory::default())
		.with(Wallet::new(-10.0)) // negative becomes zero
		.with(Skills::default())
		.with(Agent { id })
		.build();

	println!("Created agent: {:?}", agent);

	// Demonstrate Needs clamping
	let needs = world.read_storage::<Needs>().get(agent).unwrap().clone();
	println!("Needs after clamp -> thirst: {:.1} (min {}), hunger: {:.1} (max {})", needs.thirst, MIN_NEEDS, needs.hunger, MAX_NEEDS);

	// Demonstrate Wallet operations
	{
		let mut wallets = world.write_storage::<Wallet>();
		let w = wallets.get_mut(agent).unwrap();
		w.deposit(5.0);
		let taken = w.withdraw(10.0);
		println!("Wallet -> balance: {:.1}, withdrew: {:.1}", w.currency, taken);
	}

	// Demonstrate Inventory operations
	{
		let mut invs = world.write_storage::<Inventory>();
		let inv = invs.get_mut(agent).unwrap();
		inv.add("water", 2);
		let removed = inv.remove("water", 3);
		println!("Inventory -> water left: {}, removed: {}", inv.quantity("water"), removed);
	}
}

