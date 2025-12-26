//! WASM bindings for libreconomy
//!
//! This module provides WebAssembly bindings for use in browsers and JavaScript environments.
//! All WASM functionality is feature-gated behind the `wasm` feature.

#[cfg(feature = "wasm")]
pub mod world;

#[cfg(feature = "wasm")]
pub mod decision;

#[cfg(feature = "wasm")]
pub use world::WasmWorld;

#[cfg(feature = "wasm")]
pub use decision::WasmDecisionMaker;
