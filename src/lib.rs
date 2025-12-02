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
//! - **Components**: Data attached to agents (Needs, Inventory, Wallet)
//! - **ECS World**: Container for all entities and components
//! - **Systems**: Logic that operates on components (coming soon)
//!
//! # FFI Support
//!
//! This library supports Foreign Function Interface (FFI) for multiple languages:
//! - C/C++ via cbindgen
//! - Python, Swift, Kotlin via uniffi
//!
//! For detailed FFI documentation, see `docs/api/FFI.md`

pub mod agent;
pub mod ffi;

pub use agent::components::*;
pub use agent::identity::{AgentId, AgentIdAllocator, AgentIdError};
pub use agent::creation::{create_agent, create_agent_with_needs, create_agent_with_wallet, create_agent_custom, remove_agent};

// C FFI exports
pub use ffi::{WorldHandle, create_world, destroy_world, create_agent_default, 
              create_agent_with_needs as ffi_create_agent_with_needs,
              create_agent_with_wallet as ffi_create_agent_with_wallet,
              create_agent_full, remove_agent as ffi_remove_agent, 
              get_agent_count as ffi_get_agent_count};

#[export_name = "libreconomy_version"]
pub extern "C" fn libreconomy_version_c() -> *const u8 {
    b"libreconomy 0.0.1\0".as_ptr()
}
// TODO: Add core simulation systems and API

// Uniffi bindings are generated from proc-macro metadata via the CLI in release.
// Declare a component to collect exported items under a namespace for metadata.
uniffi::setup_scaffolding!();

#[uniffi::export]
pub fn libreconomy_version() -> String {
    "0.0.1".to_string()
}

#[uniffi::export]
pub fn get_agent_count() -> u32 {
    0
}
