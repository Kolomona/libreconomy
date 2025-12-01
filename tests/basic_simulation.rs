use libreconomy::*;

use crate::agent::components::*;
use specs::prelude::*;

#[test]
fn test_create_world_with_agents() {
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

    assert!(world.entities().is_alive(agent));
    // TODO: Add more assertions for agent state
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
    use specs::prelude::*;

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
