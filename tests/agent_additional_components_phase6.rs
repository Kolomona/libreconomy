//! Phase 6: Additional Agent Components Tests (Skills, Knowledge, Employment, Preferences)

use pretty_assertions::assert_eq;
use specs::prelude::*;

use libreconomy::{
    Agent, AgentIdAllocator,
    Skills, Knowledge, Employment, Preferences, UtilityFunctionType,
};

#[test]
fn skills_component_defaults_and_ecs_attachment() {
    // Arrange
    let mut world = World::new();
    world.register::<Agent>();
    world.register::<Skills>();
    world.insert(AgentIdAllocator::new());

    // Create entity with Skills
    let mut skills = Skills::default();
    assert_eq!(skills.skills.len(), 0);
    skills.skills.insert("trading".into(), 2);
    skills.skills.insert("farming".into(), 5);

    let id = { let mut a = world.write_resource::<AgentIdAllocator>(); a.allocate().unwrap() };
    let entity = world
        .create_entity()
        .with(Agent { id })
        .with(skills.clone())
        .build();

    // Assert: exists and skills stored
    assert!(world.entities().is_alive(entity));
    let s_store = world.read_storage::<Skills>();
    let s = s_store.get(entity).expect("Skills component missing");
    assert_eq!(s.skills.get("trading"), Some(&2));
    assert_eq!(s.skills.get("farming"), Some(&5));
    assert_eq!(s.skills.len(), 2);
}

#[test]
fn knowledge_component_learning_updates_prices() {
    // Arrange
    let mut k = Knowledge::default();
    assert_eq!(k.known_prices.len(), 0);

    // Act: learn a price, then update it
    libreconomy::agent::components::LearningSystem::update(&mut k, "water", 3.5);
    libreconomy::agent::components::LearningSystem::update(&mut k, "water", 4.0);
    libreconomy::agent::components::LearningSystem::update(&mut k, "food", 2.25);

    // Assert
    assert_eq!(k.known_prices.get("water"), Some(&4.0));
    assert_eq!(k.known_prices.get("food"), Some(&2.25));
    assert_eq!(k.known_prices.len(), 2);
}

#[test]
fn employment_component_defaults_and_basic_usage() {
    // Default
    let mut e = Employment::default();
    assert_eq!(e.job_status, None);
    assert_eq!(e.employer, None);
    assert!(e.employees.is_empty());

    // Assign values
    e.job_status = Some("employed".into());
    e.employer = Some("Acme Inc".into());
    e.employees.push("WorkerA".into());
    e.employees.push("WorkerB".into());

    assert_eq!(e.job_status.as_deref(), Some("employed"));
    assert_eq!(e.employer.as_deref(), Some("Acme Inc"));
    assert_eq!(e.employees.len(), 2);
    assert_eq!(e.employees[0], "WorkerA");
    assert_eq!(e.employees[1], "WorkerB");
}

#[test]
fn preferences_component_construction_and_values() {
    let p1 = Preferences { utility_function: UtilityFunctionType::Linear, risk_tolerance: 0.25 };
    assert!(matches!(p1.utility_function, UtilityFunctionType::Linear));
    assert_eq!(p1.risk_tolerance, 0.25);

    let p2 = Preferences { utility_function: UtilityFunctionType::Exponential, risk_tolerance: 0.75 };
    assert!(matches!(p2.utility_function, UtilityFunctionType::Exponential));
    assert_eq!(p2.risk_tolerance, 0.75);

    let p3 = Preferences { utility_function: UtilityFunctionType::Custom("logit".into()), risk_tolerance: 0.5 };
    match &p3.utility_function {
        UtilityFunctionType::Custom(s) => assert_eq!(s, "logit"),
        _ => panic!("Expected Custom utility function"),
    }
    assert_eq!(p3.risk_tolerance, 0.5);
}

#[test]
fn ecs_can_attach_all_additional_components() {
    let mut world = World::new();
    world.register::<Agent>();
    world.register::<Skills>();
    world.register::<Knowledge>();
    world.register::<Employment>();
    world.register::<Preferences>();
    world.insert(AgentIdAllocator::new());

    let id = { let mut a = world.write_resource::<AgentIdAllocator>(); a.allocate().unwrap() };

    let entity = world.create_entity()
        .with(Agent { id })
        .with(Skills::default())
        .with(Knowledge::default())
        .with(Employment::default())
        .with(Preferences { utility_function: UtilityFunctionType::Linear, risk_tolerance: 0.3 })
        .build();

    assert!(world.entities().is_alive(entity));

    {
        let s_store = world.read_storage::<Skills>();
        let s = s_store.get(entity).unwrap();
        assert_eq!(s.skills.len(), 0);
    }
    {
        let k_store = world.read_storage::<Knowledge>();
        let k = k_store.get(entity).unwrap();
        assert!(k.known_prices.is_empty());
    }
    {
        let e_store = world.read_storage::<Employment>();
        let e = e_store.get(entity).unwrap();
        assert!(e.employees.is_empty());
    }
    {
        let p_store = world.read_storage::<Preferences>();
        let p = p_store.get(entity).unwrap();
        assert!(matches!(p.utility_function, UtilityFunctionType::Linear));
        assert_eq!(p.risk_tolerance, 0.3);
    }
}
