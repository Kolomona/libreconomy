//! Event types for libreconomy
//!
//! This module defines events that occur during simulation, such as transactions,
//! interactions, and reputation updates.

use crate::AgentId;
use serde::{Deserialize, Serialize};

/// Outcome of an interaction or transaction
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Outcome {
    /// Positive outcome with weight (increases alpha in reputation)
    Positive(f32),
    /// Negative outcome with weight (increases beta in reputation)
    Negative(f32),
    /// Neutral outcome (no reputation change)
    Neutral,
}

impl Outcome {
    /// Get the weight for reputation updates
    /// Positive returns the weight, negative returns negative weight, neutral returns 0
    pub fn weight(&self) -> f32 {
        match self {
            Outcome::Positive(w) => *w,
            Outcome::Negative(w) => -*w,
            Outcome::Neutral => 0.0,
        }
    }

    /// Check if this is a positive outcome
    pub fn is_positive(&self) -> bool {
        matches!(self, Outcome::Positive(_))
    }

    /// Check if this is a negative outcome
    pub fn is_negative(&self) -> bool {
        matches!(self, Outcome::Negative(_))
    }

    /// Check if this is neutral
    pub fn is_neutral(&self) -> bool {
        matches!(self, Outcome::Neutral)
    }
}

/// A transaction event between two agents
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransactionEvent {
    /// First agent involved in the transaction
    pub agent1: AgentId,
    /// Second agent involved in the transaction
    pub agent2: AgentId,
    /// Item exchanged (if any)
    pub item: Option<String>,
    /// Price paid (if any)
    pub price: Option<f32>,
    /// Outcome of the transaction
    pub outcome: Outcome,
    /// Tick when transaction occurred
    pub tick: u64,
}

impl TransactionEvent {
    /// Create a new transaction event
    pub fn new(
        agent1: AgentId,
        agent2: AgentId,
        item: Option<String>,
        price: Option<f32>,
        outcome: Outcome,
        tick: u64,
    ) -> Self {
        Self {
            agent1,
            agent2,
            item,
            price,
            outcome,
            tick,
        }
    }

    /// Create a successful trade event
    pub fn successful_trade(
        buyer: AgentId,
        seller: AgentId,
        item: String,
        price: f32,
        tick: u64,
    ) -> Self {
        Self::new(
            buyer,
            seller,
            Some(item),
            Some(price),
            Outcome::Positive(1.0),
            tick,
        )
    }

    /// Create a failed trade event
    pub fn failed_trade(
        buyer: AgentId,
        seller: AgentId,
        item: String,
        weight: f32,
        tick: u64,
    ) -> Self {
        Self::new(
            buyer,
            seller,
            Some(item),
            None,
            Outcome::Negative(weight),
            tick,
        )
    }

    /// Create a positive interaction event (no trade)
    pub fn positive_interaction(agent1: AgentId, agent2: AgentId, weight: f32, tick: u64) -> Self {
        Self::new(agent1, agent2, None, None, Outcome::Positive(weight), tick)
    }

    /// Create a negative interaction event (no trade)
    pub fn negative_interaction(agent1: AgentId, agent2: AgentId, weight: f32, tick: u64) -> Self {
        Self::new(agent1, agent2, None, None, Outcome::Negative(weight), tick)
    }
}

/// Collection of transaction events for batch processing
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TransactionLog {
    events: Vec<TransactionEvent>,
}

impl TransactionLog {
    /// Create a new empty transaction log
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
        }
    }

    /// Add a transaction event to the log
    pub fn add(&mut self, event: TransactionEvent) {
        self.events.push(event);
    }

    /// Get all events in the log
    pub fn events(&self) -> &[TransactionEvent] {
        &self.events
    }

    /// Clear all events from the log
    pub fn clear(&mut self) {
        self.events.clear();
    }

    /// Get the number of events in the log
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Check if the log is empty
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Drain all events from the log
    pub fn drain(&mut self) -> Vec<TransactionEvent> {
        std::mem::take(&mut self.events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_outcome_weight() {
        assert_eq!(Outcome::Positive(2.5).weight(), 2.5);
        assert_eq!(Outcome::Negative(1.5).weight(), -1.5);
        assert_eq!(Outcome::Neutral.weight(), 0.0);
    }

    #[test]
    fn test_outcome_checks() {
        assert!(Outcome::Positive(1.0).is_positive());
        assert!(!Outcome::Positive(1.0).is_negative());
        assert!(!Outcome::Positive(1.0).is_neutral());

        assert!(Outcome::Negative(1.0).is_negative());
        assert!(!Outcome::Negative(1.0).is_positive());

        assert!(Outcome::Neutral.is_neutral());
        assert!(!Outcome::Neutral.is_positive());
    }

    #[test]
    fn test_transaction_event_creation() {
        let agent1 = AgentId(100);
        let agent2 = AgentId(200);

        let event = TransactionEvent::successful_trade(
            agent1,
            agent2,
            "water".to_string(),
            10.0,
            1000,
        );

        assert_eq!(event.agent1, agent1);
        assert_eq!(event.agent2, agent2);
        assert_eq!(event.item, Some("water".to_string()));
        assert_eq!(event.price, Some(10.0));
        assert!(event.outcome.is_positive());
        assert_eq!(event.tick, 1000);
    }

    #[test]
    fn test_failed_trade() {
        let agent1 = AgentId(100);
        let agent2 = AgentId(200);

        let event = TransactionEvent::failed_trade(
            agent1,
            agent2,
            "water".to_string(),
            2.0,
            1000,
        );

        assert!(event.outcome.is_negative());
        assert_eq!(event.outcome.weight(), -2.0);
        assert_eq!(event.price, None);
    }

    #[test]
    fn test_transaction_log() {
        let mut log = TransactionLog::new();
        assert!(log.is_empty());
        assert_eq!(log.len(), 0);

        let event = TransactionEvent::positive_interaction(
            AgentId(1),
            AgentId(2),
            1.0,
            100,
        );
        log.add(event.clone());

        assert!(!log.is_empty());
        assert_eq!(log.len(), 1);
        assert_eq!(log.events().len(), 1);
        assert_eq!(log.events()[0], event);

        log.clear();
        assert!(log.is_empty());
    }

    #[test]
    fn test_transaction_log_drain() {
        let mut log = TransactionLog::new();

        log.add(TransactionEvent::positive_interaction(
            AgentId(1),
            AgentId(2),
            1.0,
            100,
        ));
        log.add(TransactionEvent::positive_interaction(
            AgentId(3),
            AgentId(4),
            1.0,
            200,
        ));

        assert_eq!(log.len(), 2);

        let events = log.drain();
        assert_eq!(events.len(), 2);
        assert!(log.is_empty());
    }
}
