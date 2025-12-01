use libreconomy::agent::components::*;
use specs::prelude::*;

fn main() {
	let mut world = World::new();
	world.register::<Needs>();
	world.register::<Inventory>();
	world.register::<Wallet>();
	world.register::<Skills>();

	let agent = world.create_entity()
		.with(Needs { thirst: 0.5, hunger: 0.8 })
		.with(Inventory { items: std::collections::HashMap::new() })
		.with(Wallet { currency: 100.0 })
		.with(Skills { skills: std::collections::HashMap::new() })
		.build();

	println!("Created agent: {:?}", agent);
}

