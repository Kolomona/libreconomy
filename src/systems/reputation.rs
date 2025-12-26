//! Reputation update system
//!
//! This system processes transaction events and updates agent reputation knowledge.

use crate::events::TransactionLog;
use crate::{Agent, ReputationKnowledge};
use specs::prelude::*;

/// System that processes transaction events and updates reputation
///
/// This system reads transaction events from the TransactionLog resource
/// and updates the ReputationKnowledge components for both agents involved.
/// Updates are symmetric: both agents update their view of each other.
///
/// # Algorithm
///
/// For each transaction event:
/// 1. Get agent1's view of agent2
/// 2. Match outcome:
///    - Positive(w) → view.alpha += w
///    - Negative(w) → view.beta += w
///    - Neutral → no change
/// 3. Update last_interaction_tick and interaction_count
/// 4. Symmetric update for agent2's view of agent1
///
/// # Example
///
/// ```
/// use libreconomy::*;
/// use specs::prelude::*;
///
/// let mut world = World::new();
/// world.register::<ReputationKnowledge>();
/// world.insert(TransactionLog::new());
///
/// let mut system = ReputationUpdateSystem;
/// system.run_now(&world);
/// ```
pub struct ReputationUpdateSystem;

impl<'a> System<'a> for ReputationUpdateSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Agent>,
        WriteStorage<'a, ReputationKnowledge>,
        Write<'a, TransactionLog>,
    );

    fn run(
        &mut self,
        (entities, agents, mut reputation_storage, mut transaction_log): Self::SystemData,
    ) {
        // Drain events from the log (process and clear)
        let events = transaction_log.drain();

        for event in events {
            let weight = event.outcome.weight();

            // Find entities for both agents
            let mut agent1_entity = None;
            let mut agent2_entity = None;

            for (entity, agent) in (&entities, &agents).join() {
                if agent.id == event.agent1 {
                    agent1_entity = Some(entity);
                }
                if agent.id == event.agent2 {
                    agent2_entity = Some(entity);
                }
                if agent1_entity.is_some() && agent2_entity.is_some() {
                    break;
                }
            }

            // Update agent1's view of agent2
            if let Some(entity1) = agent1_entity {
                if let Some(rep1) = reputation_storage.get_mut(entity1) {
                    rep1.update_reputation(event.agent2, weight, event.tick);
                }
            }

            // Symmetric update: agent2's view of agent1
            if let Some(entity2) = agent2_entity {
                if let Some(rep2) = reputation_storage.get_mut(entity2) {
                    rep2.update_reputation(event.agent1, weight, event.tick);
                }
            }
        }
    }
}

/// System that applies reputation decay over time
///
/// This system applies temporal decay to all reputation views based on
/// the current simulation tick. Scores decay towards neutral (0.5) over time.
///
/// # Parameters
///
/// - `decay_rate`: Rate of decay per tick (typically 0.0001 - 0.001)
/// - `current_tick`: Current simulation tick
///
/// # Example
///
/// ```
/// use libreconomy::*;
/// use specs::prelude::*;
///
/// let mut world = World::new();
/// world.register::<ReputationKnowledge>();
/// world.insert(ReputationDecayConfig { decay_rate: 0.0001 });
/// world.insert(CurrentTick(1000));
///
/// let mut system = ReputationDecaySystem;
/// system.run_now(&world);
/// ```
pub struct ReputationDecaySystem;

/// Configuration for reputation decay
#[derive(Debug, Clone, Copy)]
pub struct ReputationDecayConfig {
    /// Rate of decay per tick (0.0001 - 0.001 typical)
    pub decay_rate: f32,
}

impl Default for ReputationDecayConfig {
    fn default() -> Self {
        Self {
            decay_rate: 0.0001,
        }
    }
}

/// Current simulation tick
#[derive(Debug, Clone, Copy, Default)]
pub struct CurrentTick(pub u64);

impl<'a> System<'a> for ReputationDecaySystem {
    type SystemData = (
        WriteStorage<'a, ReputationKnowledge>,
        Read<'a, ReputationDecayConfig>,
        Read<'a, CurrentTick>,
    );

    fn run(
        &mut self,
        (mut reputation_storage, decay_config, current_tick): Self::SystemData,
    ) {
        for reputation in (&mut reputation_storage).join() {
            // Apply decay to all first-hand reputation views
            for view in reputation.first_hand.values_mut() {
                // Decay is applied by updating the view's effective score
                // The decay happens automatically when score_with_decay is called,
                // but we can also periodically rebalance alpha/beta to prevent overflow
                let current_score =
                    view.score_with_decay(current_tick.0, decay_config.decay_rate);

                // Only rebalance if we have significant time passed or high values
                let ticks_since =
                    current_tick.0.saturating_sub(view.last_interaction_tick);
                if ticks_since > 10000 || (view.alpha + view.beta) > 1000.0 {
                    // Rebalance to prevent overflow while preserving the decayed score
                    let total = view.alpha + view.beta;
                    let new_alpha = current_score * total;
                    let new_beta = (1.0 - current_score) * total;

                    // Normalize to reasonable values
                    let scale = 10.0 / total.max(10.0);
                    view.alpha = new_alpha * scale;
                    view.beta = new_beta * scale;
                    view.last_interaction_tick = current_tick.0;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::TransactionEvent;
    use crate::{Agent, AgentId, AgentIdAllocator};

    fn create_test_world() -> (World, Entity, Entity) {
        let mut world = World::new();
        world.register::<Agent>();
        world.register::<ReputationKnowledge>();
        world.insert(TransactionLog::new());
        world.insert(AgentIdAllocator::new());

        // Create two agents
        let agent1 = world
            .create_entity()
            .with(Agent { id: AgentId(1) })
            .with(ReputationKnowledge::new())
            .build();

        let agent2 = world
            .create_entity()
            .with(Agent { id: AgentId(2) })
            .with(ReputationKnowledge::new())
            .build();

        (world, agent1, agent2)
    }

    #[test]
    fn test_reputation_update_system_positive() {
        let (mut world, agent1, agent2) = create_test_world();

        // Add positive transaction event
        {
            let mut log = world.write_resource::<TransactionLog>();
            log.add(TransactionEvent::positive_interaction(
                AgentId(1),
                AgentId(2),
                1.0,
                100,
            ));
        }

        // Run system
        let mut system = ReputationUpdateSystem;
        system.run_now(&world);
        world.maintain();

        // Check that reputations were updated
        let reputation_storage = world.read_storage::<ReputationKnowledge>();

        // Agent 1's view of Agent 2 should be updated
        let rep1 = reputation_storage.get(agent1).unwrap();
        let score1 = rep1.get_score(AgentId(2));
        assert!(score1 > 0.5, "Score should be above neutral: {}", score1);

        // Agent 2's view of Agent 1 should be updated (symmetric)
        let rep2 = reputation_storage.get(agent2).unwrap();
        let score2 = rep2.get_score(AgentId(1));
        assert!(score2 > 0.5, "Score should be above neutral: {}", score2);
    }

    #[test]
    fn test_reputation_update_system_negative() {
        let (mut world, agent1, agent2) = create_test_world();

        // Add negative transaction event
        {
            let mut log = world.write_resource::<TransactionLog>();
            log.add(TransactionEvent::negative_interaction(
                AgentId(1),
                AgentId(2),
                2.0,
                100,
            ));
        }

        // Run system
        let mut system = ReputationUpdateSystem;
        system.run_now(&world);
        world.maintain();

        // Check that reputations were updated
        let reputation_storage = world.read_storage::<ReputationKnowledge>();

        // Both agents should have lower scores of each other
        let rep1 = reputation_storage.get(agent1).unwrap();
        let score1 = rep1.get_score(AgentId(2));
        assert!(score1 < 0.5, "Score should be below neutral: {}", score1);

        let rep2 = reputation_storage.get(agent2).unwrap();
        let score2 = rep2.get_score(AgentId(1));
        assert!(score2 < 0.5, "Score should be below neutral: {}", score2);
    }

    #[test]
    fn test_reputation_update_system_multiple_events() {
        let (mut world, agent1, _agent2) = create_test_world();

        // Add multiple events
        {
            let mut log = world.write_resource::<TransactionLog>();
            log.add(TransactionEvent::positive_interaction(
                AgentId(1),
                AgentId(2),
                1.0,
                100,
            ));
            log.add(TransactionEvent::positive_interaction(
                AgentId(1),
                AgentId(2),
                1.0,
                200,
            ));
            log.add(TransactionEvent::negative_interaction(
                AgentId(1),
                AgentId(2),
                0.5,
                300,
            ));
        }

        // Run system
        let mut system = ReputationUpdateSystem;
        system.run_now(&world);
        world.maintain();

        // Check interaction counts
        let reputation_storage = world.read_storage::<ReputationKnowledge>();
        let rep1 = reputation_storage.get(agent1).unwrap();

        // Should have view of agent 2
        let view = rep1.first_hand.get(&AgentId(2)).unwrap();
        assert_eq!(view.interaction_count, 3);

        // Score should still be positive (2 positive, 1 negative)
        assert!(view.score() > 0.5);
    }

    #[test]
    fn test_reputation_decay_system() {
        let (mut world, agent1, _agent2) = create_test_world();

        // Setup decay config
        world.insert(ReputationDecayConfig {
            decay_rate: 0.001,
        });
        world.insert(CurrentTick(1000));

        // Add a positive event at tick 0
        {
            let mut log = world.write_resource::<TransactionLog>();
            log.add(TransactionEvent::positive_interaction(
                AgentId(1),
                AgentId(2),
                5.0, // Strong positive
                0,
            ));
        }

        // Update reputation
        let mut update_system = ReputationUpdateSystem;
        update_system.run_now(&world);
        world.maintain();

        // Check initial score
        let initial_score = {
            let reputation_storage = world.read_storage::<ReputationKnowledge>();
            let rep1 = reputation_storage.get(agent1).unwrap();
            rep1.get_score(AgentId(2))
        };

        // Run decay system
        let mut decay_system = ReputationDecaySystem;
        decay_system.run_now(&world);
        world.maintain();

        // Check that view was rebalanced
        let reputation_storage = world.read_storage::<ReputationKnowledge>();
        let rep1 = reputation_storage.get(agent1).unwrap();
        let view = rep1.first_hand.get(&AgentId(2)).unwrap();

        // After rebalancing, alpha + beta should be normalized
        assert!(
            view.alpha + view.beta < 100.0,
            "Should be normalized: {} + {} = {}",
            view.alpha,
            view.beta,
            view.alpha + view.beta
        );

        // Score should be preserved (approximately)
        let new_score = view.score();
        assert!(
            (new_score - initial_score).abs() < 0.1,
            "Score should be preserved: {} vs {}",
            initial_score,
            new_score
        );
    }

    #[test]
    fn test_transaction_log_cleared_after_processing() {
        let (mut world, _agent1, _agent2) = create_test_world();

        // Add event
        {
            let mut log = world.write_resource::<TransactionLog>();
            log.add(TransactionEvent::positive_interaction(
                AgentId(1),
                AgentId(2),
                1.0,
                100,
            ));
            assert_eq!(log.len(), 1);
        }

        // Run system
        let mut system = ReputationUpdateSystem;
        system.run_now(&world);
        world.maintain();

        // Log should be empty after processing
        let log = world.read_resource::<TransactionLog>();
        assert_eq!(log.len(), 0);
    }
}
