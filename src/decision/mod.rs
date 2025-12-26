//! Decision-making trait and implementations

pub mod types;
pub mod utility_maximizer;

pub use types::{Intent, Action, ActionType, Transaction, DecisionOutput};
pub use utility_maximizer::{UtilityMaximizer, DecisionThresholds, UtilityWeights};

use crate::world_query::WorldQuery;
use specs::prelude::*;

/// Trait for agent decision-making in the economy
///
/// Implement this trait to create custom decision-making logic for agents.
/// The library provides [`UtilityMaximizer`] as the default implementation.
pub trait DecisionMaker {
    /// Make a decision for the given agent
    ///
    /// # Arguments
    ///
    /// * `agent` - The entity to make a decision for
    /// * `world` - The ECS world containing all components
    /// * `world_query` - Application-provided spatial query interface
    ///
    /// # Returns
    ///
    /// A DecisionOutput containing the chosen action/intent
    fn decide(
        &self,
        agent: Entity,
        world: &World,
        world_query: &dyn WorldQuery,
    ) -> DecisionOutput;
}

impl DecisionMaker for UtilityMaximizer {
    fn decide(
        &self,
        agent: Entity,
        world: &World,
        world_query: &dyn WorldQuery,
    ) -> DecisionOutput {
        UtilityMaximizer::decide(self, agent, world, world_query)
    }
}
