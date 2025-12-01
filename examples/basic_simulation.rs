use libreconomy::{
	Agent, AgentIdAllocator, Inventory, Needs, Skills, Wallet,
	create_agent, create_agent_with_needs, create_agent_with_wallet, create_agent_custom,
	MIN_NEEDS, MAX_NEEDS
};
use specs::prelude::*;

fn main() {
	println!("=== libreconomy Basic Simulation (Phases 1-4) ===\n");

	let mut world = World::new();
	world.register::<Agent>();
	world.register::<Needs>();
	world.register::<Inventory>();
	world.register::<Wallet>();
	world.register::<Skills>();

	// Insert the AgentId allocator resource
	world.insert(AgentIdAllocator::new());

	println!("--- Phase 3: Agent Creation Functions ---");
	
	// Create agent with default components (Phase 3)
	let agent1 = create_agent(&mut world);
	println!("Created agent1 (defaults): {:?}", agent1);
	{
		let agents = world.read_storage::<Agent>();
		let needs = world.read_storage::<Needs>();
		let wallet = world.read_storage::<Wallet>();
		println!("  ID: {:?}", agents.get(agent1).unwrap().id);
		println!("  Needs: thirst={:.1}, hunger={:.1}", needs.get(agent1).unwrap().thirst, needs.get(agent1).unwrap().hunger);
		println!("  Wallet: {:.1}", wallet.get(agent1).unwrap().currency);
	}

	// Create agent with custom needs (very thirsty)
	let agent2 = create_agent_with_needs(&mut world, Needs::new(90.0, 40.0));
	println!("\nCreated agent2 (thirsty): {:?}", agent2);
	{
		let agents = world.read_storage::<Agent>();
		let needs = world.read_storage::<Needs>();
		println!("  ID: {:?}", agents.get(agent2).unwrap().id);
		println!("  Needs: thirst={:.1}, hunger={:.1}", needs.get(agent2).unwrap().thirst, needs.get(agent2).unwrap().hunger);
	}

	// Create agent with custom wallet (wealthy)
	let agent3 = create_agent_with_wallet(&mut world, Wallet::new(1000.0));
	println!("\nCreated agent3 (wealthy): {:?}", agent3);
	{
		let agents = world.read_storage::<Agent>();
		let wallet = world.read_storage::<Wallet>();
		println!("  ID: {:?}", agents.get(agent3).unwrap().id);
		println!("  Wallet: {:.1}", wallet.get(agent3).unwrap().currency);
	}

	// Create fully custom agent with inventory
	let mut custom_inventory = Inventory::default();
	custom_inventory.add("water", 5);
	custom_inventory.add("food", 3);
	let agent4 = create_agent_custom(
		&mut world,
		Needs::new(20.0, 30.0), // satisfied
		custom_inventory,
		Wallet::new(500.0),
	);
	println!("\nCreated agent4 (custom): {:?}", agent4);
	{
		let agents = world.read_storage::<Agent>();
		let needs = world.read_storage::<Needs>();
		let inventory = world.read_storage::<Inventory>();
		let wallet = world.read_storage::<Wallet>();
		println!("  ID: {:?}", agents.get(agent4).unwrap().id);
		println!("  Needs: thirst={:.1}, hunger={:.1}", needs.get(agent4).unwrap().thirst, needs.get(agent4).unwrap().hunger);
		println!("  Inventory: water={}, food={}", inventory.get(agent4).unwrap().quantity("water"), inventory.get(agent4).unwrap().quantity("food"));
		println!("  Wallet: {:.1}", wallet.get(agent4).unwrap().currency);
	}

	println!("\n--- Phase 2: Component Operations ---");
	
	// Demonstrate Needs clamping with out-of-bounds values
	let test_id = {
		let mut alloc = world.write_resource::<AgentIdAllocator>();
		alloc.allocate().expect("allocate AgentId")
	};
	let test_agent = world.create_entity()
		.with(Agent { id: test_id })
		.with(Needs::new(-1.0, 200.0)) // intentionally out of bounds
		.with(Inventory::default())
		.with(Wallet::new(-10.0)) // negative becomes zero
		.with(Skills::default())
		.build();

	println!("\nTest agent with out-of-bounds values:");
	let needs = world.read_storage::<Needs>().get(test_agent).unwrap().clone();
	println!("  Needs after clamp -> thirst: {:.1} (min {}), hunger: {:.1} (max {})", 
		needs.thirst, MIN_NEEDS, needs.hunger, MAX_NEEDS);

	// Demonstrate Wallet operations
	{
		let mut wallets = world.write_storage::<Wallet>();
		let w = wallets.get_mut(test_agent).unwrap();
		println!("  Wallet initial (from -10.0): {:.1}", w.currency);
		w.deposit(5.0);
		println!("  After deposit(5.0): {:.1}", w.currency);
		let taken = w.withdraw(10.0);
		println!("  After withdraw(10.0) -> withdrew: {:.1}, balance: {:.1}", taken, w.currency);
	}

	// Demonstrate Inventory operations
	{
		let mut invs = world.write_storage::<Inventory>();
		let inv = invs.get_mut(test_agent).unwrap();
		inv.add("water", 2);
		println!("  Inventory after add(water, 2): water={}", inv.quantity("water"));
		let removed = inv.remove("water", 3);
		println!("  After remove(water, 3) -> removed: {}, left: {}", removed, inv.quantity("water"));
	}

	println!("\n--- Phase 4: Agent Removal ---");

	// Remove agent2 (thirsty) and verify removal
	use libreconomy::remove_agent;
	println!("Removing agent2 (thirsty): {:?}", agent2);
	remove_agent(&mut world, agent2);
	let agents = world.read_storage::<Agent>();
	if agents.get(agent2).is_none() {
		println!("  agent2 successfully removed from world.");
	} else {
		println!("  ERROR: agent2 still exists!");
	}

	println!("\n=== Simulation Complete ===");
}

