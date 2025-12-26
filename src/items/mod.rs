//! Item system for libreconomy
//!
//! This module provides item definitions and need satisfaction mappings.
//! Items can satisfy various agent needs (thirst, hunger, tiredness) and
//! have properties like consumability.

pub mod registry;

pub use registry::{ItemRegistry, ItemType, NeedType};
