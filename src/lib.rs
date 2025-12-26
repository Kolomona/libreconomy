//! libreconomy: Cross-platform agent-based economy simulator library
//!
//! This library provides modular systems for simulating economic agents using an Entity-Component-System (ECS) architecture.
//! Designed for games and applications, with FFI support for integration in other languages.
//!
//! # Quick Start
//!
//! ```rust
//! use libreconomy::*;
//! use specs::prelude::*;
//!
//! // Create a new ECS world
//! let mut world = World::new();
//! 
//! // Register components
//! world.register::<Agent>();
//! world.register::<Needs>();
//! world.register::<Inventory>();
//! world.register::<Wallet>();
//! 
//! // Insert the AgentId allocator resource
//! world.insert(AgentIdAllocator::new());
//! 
//! // Create an agent with default components
//! let agent = create_agent(&mut world);
//! 
//! // Query the agent's components
//! let needs_storage = world.read_storage::<Needs>();
//! let needs = needs_storage.get(agent).unwrap();
//! println!("Thirst: {}, Hunger: {}", needs.thirst, needs.hunger);
//! ```
//!
//! # Core Concepts
//!
//! - **Agents**: Autonomous entities with unique IDs
//! - **Components**: Data attached to agents (Needs, Inventory, Wallet, Skills, Knowledge, Reputation)
//! - **ECS World**: Container for all entities and components
//! - **Systems**: Logic that operates on components (ReputationUpdateSystem, ReputationDecaySystem)
//! - **Events**: Transaction events that trigger reputation updates
//! - **Decision Making**: Utility-based AI with species-aware behavior
//!
//! # FFI Support
//!
//! This library supports Foreign Function Interface (FFI) for multiple languages:
//! - C/C++ via cbindgen
//! - Python, Swift, Kotlin via uniffi
//!
//! For detailed FFI documentation, see `docs/api/FFI.md`

pub mod agent;
pub mod decision;
pub mod events;
pub mod ffi;
pub mod items;
pub mod systems;
pub mod world_query;

#[cfg(feature = "wasm")]
pub mod wasm;

pub use agent::components::*;
pub use agent::identity::{AgentId, AgentIdAllocator, AgentIdError};
pub use agent::creation::{create_agent, create_agent_with_needs, create_agent_with_wallet, create_agent_custom, remove_agent};
pub use decision::{Intent, Action, ActionType, Transaction, DecisionOutput, DecisionMaker, UtilityMaximizer, DecisionThresholds, UtilityWeights};
pub use events::{Outcome, TransactionEvent, TransactionLog};
pub use items::{ItemRegistry, ItemType, NeedType};
pub use systems::{ReputationUpdateSystem, ReputationDecaySystem, ReputationDecayConfig, CurrentTick};
pub use world_query::{WorldQuery, ResourceLocation};

// C FFI exports
pub use ffi::{
    WorldHandle, create_world, destroy_world, create_agent_default,
    create_agent_with_needs as ffi_create_agent_with_needs,
    create_agent_with_wallet as ffi_create_agent_with_wallet,
    create_agent_full, remove_agent as ffi_remove_agent,
    get_agent_count as ffi_get_agent_count,
    // Component access
    get_needs, set_needs,
    get_inventory_item, add_inventory_item, remove_inventory_item,
    get_wallet, deposit_wallet, withdraw_wallet,
};

#[export_name = "libreconomy_version"]
pub extern "C" fn libreconomy_version_c() -> *const u8 {
    b"libreconomy 0.0.1\0".as_ptr()
}
// TODO: Add core simulation systems and API

// uniffi scaffolding (must be at crate root)
#[cfg(feature = "uniffi")]
uniffi::setup_scaffolding!();
