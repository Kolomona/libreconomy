//! Agent ECS components
use specs::prelude::{Component, VecStorage};
use super::identity::AgentId;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Maximum bound for needs values for performance-friendly f32 usage
pub const MAX_NEEDS: f32 = 100.0;
/// Minimum bound for needs values
pub const MIN_NEEDS: f32 = 0.0;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Needs {
    pub thirst: f32,
    pub hunger: f32,
}

impl Needs {
    /// Creates a new Needs clamped to [MIN_NEEDS, MAX_NEEDS]
    pub fn new(thirst: f32, hunger: f32) -> Self {
        fn clamp(v: f32) -> f32 { v.max(MIN_NEEDS).min(MAX_NEEDS) }
        Self { thirst: clamp(thirst), hunger: clamp(hunger) }
    }

    /// Clamp the current needs in place.
    pub fn clamp(&mut self) {
        self.thirst = self.thirst.max(MIN_NEEDS).min(MAX_NEEDS);
        self.hunger = self.hunger.max(MIN_NEEDS).min(MAX_NEEDS);
    }
}

impl Component for Needs {
    type Storage = VecStorage<Self>;
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Inventory {
    pub items: HashMap<String, u32>, // item_id -> quantity
}

impl Inventory {
    /// Get quantity for an item, returns 0 if missing.
    pub fn quantity(&self, item_id: &str) -> u32 {
        *self.items.get(item_id).unwrap_or(&0)
    }

    /// Set absolute quantity (negative not allowed, zero removes entry).
    pub fn set_quantity(&mut self, item_id: &str, quantity: u32) {
        if quantity == 0 {
            self.items.remove(item_id);
        } else {
            self.items.insert(item_id.to_string(), quantity);
        }
    }

    /// Add quantity safely, saturating at u32::MAX.
    pub fn add(&mut self, item_id: &str, delta: u32) {
        if delta == 0 { return; }
        let current = self.quantity(item_id);
        let new_qty = current.saturating_add(delta);
        self.set_quantity(item_id, new_qty);
    }

    /// Remove up to delta; returns removed amount.
    pub fn remove(&mut self, item_id: &str, delta: u32) -> u32 {
        if delta == 0 { return 0; }
        let current = self.quantity(item_id);
        let removed = current.min(delta);
        self.set_quantity(item_id, current.saturating_sub(removed));
        removed
    }
}

impl Component for Inventory {
    type Storage = VecStorage<Self>;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Wallet {
    pub currency: f32,
}

impl Wallet {
    /// Create a new wallet with non-negative balance
    pub fn new(currency: f32) -> Self {
        Self { currency: currency.max(0.0) }
    }

    /// Deposit non-negative amount; negative is treated as zero.
    pub fn deposit(&mut self, amount: f32) {
        if amount <= 0.0 { return; }
        self.currency += amount;
    }

    /// Withdraw up to amount, not allowing negative balance; returns withdrawn.
    pub fn withdraw(&mut self, amount: f32) -> f32 {
        if amount <= 0.0 { return 0.0; }
        let withdrawn = self.currency.min(amount);
        self.currency -= withdrawn;
        withdrawn
    }
}

impl Component for Wallet {
    type Storage = VecStorage<Self>;
}

/// Marker/data component designating an entity as an Agent with a unique id
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Agent {
    pub id: AgentId,
}

impl Component for Agent {
    type Storage = VecStorage<Self>;
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Skills {
    pub skills: HashMap<String, u32>, // skill_id -> level
}
impl Component for Skills {
    type Storage = VecStorage<Self>;
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Knowledge {
    pub known_prices: std::collections::HashMap<String, f32>,
    pub trade_partners: Vec<String>,
}
impl Component for Knowledge {
    type Storage = VecStorage<Self>;
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Employment {
    pub job_status: Option<String>,
    pub employer: Option<String>,
    pub employees: Vec<String>,
}
impl Component for Employment {
    type Storage = VecStorage<Self>;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UtilityFunctionType {
    Linear,
    Exponential,
    Custom(String), // Placeholder for custom logic
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Preferences {
    pub utility_function: UtilityFunctionType,
    pub risk_tolerance: f32,
}

impl Component for Preferences {
    type Storage = VecStorage<Self>;
}

pub struct NeedDecaySystem;
impl NeedDecaySystem {
    pub fn tick(needs: &mut Needs) {
        needs.thirst += 0.01;
        needs.hunger += 0.01;
    }
}

pub struct LearningSystem;
impl LearningSystem {
    pub fn update(knowledge: &mut Knowledge, item: &str, price: f32) {
        knowledge.known_prices.insert(item.to_string(), price);
    }
}

pub struct NegotiationSystem;
impl NegotiationSystem {
    pub fn negotiate() -> bool {
        true // Placeholder
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_needs_creation() {
        let needs = Needs { thirst: 1.0, hunger: 2.0 };
        assert_eq!(needs.thirst, 1.0);
        assert_eq!(needs.hunger, 2.0);
    }

    #[test]
    fn test_needs_boundaries() {
        let needs = Needs { thirst: 0.0, hunger: 100.0 };
        assert!(needs.thirst >= 0.0 && needs.thirst <= 100.0);
        assert!(needs.hunger >= 0.0 && needs.hunger <= 100.0);
    }
}
