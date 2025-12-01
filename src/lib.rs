//! libreconomy: Cross-platform agent-based economy simulator library
//!
//! This library provides modular systems for simulating economic agents, markets, labor, and production.
//! Designed for games and applications, with FFI support for integration in other languages.

pub mod agent;
pub use agent::components::*;
pub use agent::identity::{AgentId, AgentIdAllocator, AgentIdError};
pub use agent::creation::{create_agent, create_agent_with_needs, create_agent_with_wallet, create_agent_custom};
#[export_name = "libreconomy_version"]
pub extern "C" fn libreconomy_version_c() -> *const u8 {
    b"libreconomy 0.0.1\0".as_ptr()
}
// TODO: Add core simulation systems and API
// TODO: Add more FFI functions for simulation control

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
