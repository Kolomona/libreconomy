//! Agent ECS components
use specs::prelude::{Component, VecStorage};
use super::identity::AgentId;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Needs {
    pub thirst: f32,
    pub hunger: f32,
    // TODO: Add more needs
}

impl Component for Needs {
    type Storage = VecStorage<Self>;
}

#[derive(Debug, Clone)]
pub struct Inventory {
    pub items: HashMap<String, u32>, // item_id -> quantity
}

impl Component for Inventory {
    type Storage = VecStorage<Self>;
}

#[derive(Debug, Clone)]
pub struct Wallet {
    pub currency: f32,
}

impl Component for Wallet {
    type Storage = VecStorage<Self>;
}

/// Marker/data component designating an entity as an Agent with a unique id
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Agent {
    pub id: AgentId,
}

impl Component for Agent {
    type Storage = VecStorage<Self>;
}

#[derive(Debug, Clone)]
pub struct Skills {
    pub skills: HashMap<String, u32>, // skill_id -> level
}

impl Component for Skills {
    type Storage = VecStorage<Self>;
}

#[derive(Debug, Clone)]
pub struct Knowledge {
    pub known_prices: std::collections::HashMap<String, f32>,
    pub trade_partners: Vec<String>,
}

impl Component for Knowledge {
    type Storage = VecStorage<Self>;
}

#[derive(Debug, Clone)]
pub struct Employment {
    pub job_status: Option<String>,
    pub employer: Option<String>,
    pub employees: Vec<String>,
}

impl Component for Employment {
    type Storage = VecStorage<Self>;
}

#[derive(Debug, Clone)]
pub enum UtilityFunctionType {
    Linear,
    Exponential,
    Custom(String), // Placeholder for custom logic
}

#[derive(Debug, Clone)]
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
