use criterion::{black_box, criterion_group, criterion_main, Criterion};
use libreconomy::*;
use specs::prelude::*;

fn benchmark_agent_creation(c: &mut Criterion) {
    c.bench_function("create 100 agents", |b| {
        b.iter(|| {
            let mut world = World::new();
            world.register::<agent::components::Needs>();
            world.register::<agent::components::Inventory>();
            world.register::<agent::components::Wallet>();
            world.register::<agent::components::Skills>();
            for _ in 0..100 {
                world.create_entity()
                    .with(agent::components::Needs { thirst: black_box(0.5), hunger: black_box(0.8) })
                    .with(agent::components::Inventory { items: std::collections::HashMap::new() })
                    .with(agent::components::Wallet { currency: black_box(100.0) })
                    .with(agent::components::Skills { skills: std::collections::HashMap::new() })
                    .build();
            }
        });
    });
}

criterion_group!(benches, benchmark_agent_creation);
criterion_main!(benches);
