//! libreconomy: Cross-platform agent-based economy simulator library
//!
//! This library provides modular systems for simulating economic agents, markets, labor, and production.
//! Designed for games and applications, with FFI support for integration in other languages.

pub use agent::components::*;
#[no_mangle]
pub extern "C" fn libreconomy_version() -> *const u8 {
    b"libreconomy 0.0.1\0".as_ptr()
}
// TODO: Add core simulation systems and API
// TODO: Add more FFI functions for simulation control

pub use uniffi_macros::export;
uniffi_macros::include_scaffolding!("libreconomy");

#[export]
pub fn libreconomy_version() -> String {
    "0.0.1".to_string()
}

#[export]
pub fn get_agent_count() -> u32 {
    0 // Stub for TDD
}
