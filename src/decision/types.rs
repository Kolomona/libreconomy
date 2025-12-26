// Decision output types for libreconomy
//
// These types represent the outputs of the decision-making system.
// They form a hierarchy from high-level intentions to specific actions
// to completed transactions.

use crate::agent::AgentId;
use serde::{Deserialize, Serialize};

/// High-level intent representing what an agent wants to achieve
///
/// Intents are abstract goals that may require multiple steps to fulfill.
/// The application is responsible for translating intents into concrete
/// actions and movement.
///
/// # Examples
///
/// ```rust
/// use libreconomy::decision::Intent;
///
/// // Agent needs water
/// let intent = Intent::SeekItem {
///     item_type: "water".to_string(),
///     urgency: 0.8,
/// };
///
/// // Agent needs rest
/// let rest_intent = Intent::Rest;
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Intent {
    /// Seek a specific item type (food, water, materials, etc.)
    SeekItem {
        /// Type of item to seek (e.g., "water", "food", "wood")
        item_type: String,
        /// How urgently the item is needed (0.0 to 1.0)
        urgency: f32,
    },

    /// Find employment matching given skills
    FindWork {
        /// List of skills the agent can offer
        skill_types: Vec<String>,
    },

    /// Seek trade opportunities
    SeekTrade {
        /// True if buying, false if selling
        buying: bool,
        /// Type of item to buy/sell
        item_type: String,
    },

    /// Rest/sleep to recover tiredness
    Rest,

    /// Wander aimlessly (exploration or no pressing needs)
    Wander,
}

impl Intent {
    /// Check if this is a survival-critical intent
    pub fn is_critical(&self) -> bool {
        match self {
            Intent::SeekItem { urgency, .. } => *urgency > 0.7,
            Intent::Rest => false,
            Intent::FindWork { .. } => false,
            Intent::SeekTrade { .. } => false,
            Intent::Wander => false,
        }
    }

    /// Get the intent type as a string (for debugging/logging)
    pub fn intent_type(&self) -> &str {
        match self {
            Intent::SeekItem { .. } => "SeekItem",
            Intent::FindWork { .. } => "FindWork",
            Intent::SeekTrade { .. } => "SeekTrade",
            Intent::Rest => "Rest",
            Intent::Wander => "Wander",
        }
    }
}

/// Specific action targeting a particular agent
///
/// Actions are more concrete than intents - they specify exactly what
/// to do and with whom. The application executes these actions.
///
/// # Examples
///
/// ```rust
/// use libreconomy::decision::{Action, ActionType};
/// use libreconomy::agent::AgentId;
///
/// // Initiate trade with agent 42
/// let action = Action {
///     target_agent: AgentId(42),
///     action_type: ActionType::InitiateTrade {
///         item: "wheat".to_string(),
///         offer_price: 50.0,
///     },
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Action {
    /// The agent being targeted
    pub target_agent: AgentId,
    /// What action to perform
    pub action_type: ActionType,
}

/// Types of actions that can be performed
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ActionType {
    /// Initiate a trade negotiation
    InitiateTrade {
        /// Item to trade
        item: String,
        /// Proposed price
        offer_price: f32,
    },

    /// Hunt/attack another agent (for food)
    Hunt {
        /// Target agent to hunt
        target: AgentId,
    },

    /// Consume a resource at a location
    Consume {
        /// World coordinates of the resource
        resource_location: (f32, f32),
    },

    /// Accept employment offer
    AcceptEmployment {
        /// Proposed wage
        wage: f32,
    },

    /// Ask for information (prices, locations, etc.)
    AskForInformation {
        /// What kind of information is being requested
        query_type: String,
    },
}

/// Completed transaction between two agents
///
/// Transactions represent completed economic interactions. They are
/// generated when an Action is successfully executed.
///
/// # Examples
///
/// ```rust
/// use libreconomy::decision::Transaction;
/// use libreconomy::agent::AgentId;
///
/// // Successful wheat trade
/// let transaction = Transaction {
///     buyer: AgentId(1),
///     seller: AgentId(2),
///     item: "wheat".to_string(),
///     quantity: 10,
///     price: 50.0,
///     success: true,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Transaction {
    /// Agent buying the item
    pub buyer: AgentId,
    /// Agent selling the item
    pub seller: AgentId,
    /// Item being traded
    pub item: String,
    /// Quantity of items
    pub quantity: u32,
    /// Price per item
    pub price: f32,
    /// Whether the transaction completed successfully
    pub success: bool,
}

impl Transaction {
    /// Calculate total transaction value
    pub fn total_value(&self) -> f32 {
        self.quantity as f32 * self.price
    }

    /// Check if this transaction represents a successful trade
    pub fn is_successful(&self) -> bool {
        self.success && self.quantity > 0
    }
}

/// Decision output from the decision-making system
///
/// This enum wraps the three levels of decision outputs:
/// - Intent: High-level goals
/// - Action: Specific targeted actions
/// - Transaction: Completed interactions
///
/// The decision-maker returns this enum, allowing different decision
/// systems to operate at different levels of abstraction.
///
/// # Examples
///
/// ```rust
/// use libreconomy::decision::{DecisionOutput, Intent};
///
/// // Decision system returns an intent
/// let decision = DecisionOutput::Intent(Intent::Rest);
///
/// match decision {
///     DecisionOutput::Intent(intent) => {
///         println!("Agent decided to: {}", intent.intent_type());
///     }
///     DecisionOutput::Action(action) => {
///         // Execute specific action
///     }
///     DecisionOutput::Transaction(transaction) => {
///         // Record completed transaction
///     }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DecisionOutput {
    /// High-level intent to achieve a goal
    Intent(Intent),
    /// Specific action targeting an agent
    Action(Action),
    /// Completed transaction
    Transaction(Transaction),
}

impl DecisionOutput {
    /// Extract the Intent if this is an Intent output
    pub fn as_intent(&self) -> Option<&Intent> {
        match self {
            DecisionOutput::Intent(intent) => Some(intent),
            _ => None,
        }
    }

    /// Extract the Action if this is an Action output
    pub fn as_action(&self) -> Option<&Action> {
        match self {
            DecisionOutput::Action(action) => Some(action),
            _ => None,
        }
    }

    /// Extract the Transaction if this is a Transaction output
    pub fn as_transaction(&self) -> Option<&Transaction> {
        match self {
            DecisionOutput::Transaction(tx) => Some(tx),
            _ => None,
        }
    }

    /// Check if this is a critical decision (requires immediate action)
    pub fn is_critical(&self) -> bool {
        match self {
            DecisionOutput::Intent(intent) => intent.is_critical(),
            DecisionOutput::Action(_) => true, // Actions are always immediate
            DecisionOutput::Transaction(_) => false, // Transactions are completed
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intent_seek_item() {
        let intent = Intent::SeekItem {
            item_type: "water".to_string(),
            urgency: 0.9,
        };

        assert_eq!(intent.intent_type(), "SeekItem");
        assert!(intent.is_critical());
    }

    #[test]
    fn test_intent_rest_not_critical() {
        let intent = Intent::Rest;
        assert_eq!(intent.intent_type(), "Rest");
        assert!(!intent.is_critical());
    }

    #[test]
    fn test_intent_wander() {
        let intent = Intent::Wander;
        assert_eq!(intent.intent_type(), "Wander");
        assert!(!intent.is_critical());
    }

    #[test]
    fn test_action_initiate_trade() {
        let action = Action {
            target_agent: AgentId(42),
            action_type: ActionType::InitiateTrade {
                item: "wheat".to_string(),
                offer_price: 50.0,
            },
        };

        assert_eq!(action.target_agent, AgentId(42));
    }

    #[test]
    fn test_action_hunt() {
        let action = Action {
            target_agent: AgentId(10),
            action_type: ActionType::Hunt {
                target: AgentId(10),
            },
        };

        assert_eq!(action.target_agent, AgentId(10));
    }

    #[test]
    fn test_transaction_total_value() {
        let tx = Transaction {
            buyer: AgentId(1),
            seller: AgentId(2),
            item: "wheat".to_string(),
            quantity: 10,
            price: 5.0,
            success: true,
        };

        assert_eq!(tx.total_value(), 50.0);
        assert!(tx.is_successful());
    }

    #[test]
    fn test_transaction_failed() {
        let tx = Transaction {
            buyer: AgentId(1),
            seller: AgentId(2),
            item: "wheat".to_string(),
            quantity: 0,
            price: 5.0,
            success: false,
        };

        assert_eq!(tx.total_value(), 0.0);
        assert!(!tx.is_successful());
    }

    #[test]
    fn test_decision_output_intent() {
        let decision = DecisionOutput::Intent(Intent::Rest);

        assert!(decision.as_intent().is_some());
        assert!(decision.as_action().is_none());
        assert!(decision.as_transaction().is_none());
        assert!(!decision.is_critical());
    }

    #[test]
    fn test_decision_output_action() {
        let action = Action {
            target_agent: AgentId(1),
            action_type: ActionType::InitiateTrade {
                item: "wood".to_string(),
                offer_price: 10.0,
            },
        };
        let decision = DecisionOutput::Action(action.clone());

        assert!(decision.as_action().is_some());
        assert_eq!(decision.as_action().unwrap(), &action);
        assert!(decision.is_critical()); // Actions are critical
    }

    #[test]
    fn test_decision_output_transaction() {
        let tx = Transaction {
            buyer: AgentId(1),
            seller: AgentId(2),
            item: "wheat".to_string(),
            quantity: 10,
            price: 5.0,
            success: true,
        };
        let decision = DecisionOutput::Transaction(tx.clone());

        assert!(decision.as_transaction().is_some());
        assert_eq!(decision.as_transaction().unwrap(), &tx);
        assert!(!decision.is_critical()); // Transactions are completed
    }

    #[test]
    fn test_intent_serialization() {
        let intent = Intent::SeekItem {
            item_type: "water".to_string(),
            urgency: 0.8,
        };

        let json = serde_json::to_string(&intent).unwrap();
        let deserialized: Intent = serde_json::from_str(&json).unwrap();

        assert_eq!(intent, deserialized);
    }

    #[test]
    fn test_action_serialization() {
        let action = Action {
            target_agent: AgentId(42),
            action_type: ActionType::Hunt {
                target: AgentId(99),
            },
        };

        let json = serde_json::to_string(&action).unwrap();
        let deserialized: Action = serde_json::from_str(&json).unwrap();

        assert_eq!(action, deserialized);
    }

    #[test]
    fn test_transaction_serialization() {
        let tx = Transaction {
            buyer: AgentId(1),
            seller: AgentId(2),
            item: "wheat".to_string(),
            quantity: 10,
            price: 5.0,
            success: true,
        };

        let json = serde_json::to_string(&tx).unwrap();
        let deserialized: Transaction = serde_json::from_str(&json).unwrap();

        assert_eq!(tx, deserialized);
    }

    #[test]
    fn test_decision_output_serialization() {
        let decision = DecisionOutput::Intent(Intent::Wander);

        let json = serde_json::to_string(&decision).unwrap();
        let deserialized: DecisionOutput = serde_json::from_str(&json).unwrap();

        assert_eq!(decision, deserialized);
    }
}
