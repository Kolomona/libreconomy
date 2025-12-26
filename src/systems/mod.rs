//! ECS Systems for libreconomy
//!
//! This module contains systems that process game logic each tick.

pub mod reputation;

pub use reputation::{
    ReputationUpdateSystem, ReputationDecaySystem, ReputationDecayConfig, CurrentTick,
};
