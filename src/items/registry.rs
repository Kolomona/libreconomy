//! Item registry and type definitions
//!
//! This module defines the item system used for need satisfaction and economic exchanges.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Types of needs that items can satisfy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NeedType {
    Thirst,
    Hunger,
    Tiredness,
}

/// Item type definition with need satisfaction properties
///
/// # Example
/// ```rust
/// use libreconomy::{ItemType, NeedType};
/// use std::collections::HashMap;
///
/// let mut satisfies = HashMap::new();
/// satisfies.insert(NeedType::Thirst, -30.0);
///
/// let water = ItemType {
///     id: "water".to_string(),
///     satisfies,
///     consumable: true,
/// };
///
/// assert_eq!(water.satisfies.get(&NeedType::Thirst), Some(&-30.0));
/// assert!(water.consumable);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ItemType {
    pub id: String,
    /// Maps need types to satisfaction values (negative = reduces need)
    pub satisfies: HashMap<NeedType, f32>,
    pub consumable: bool,
}

impl ItemType {
    /// Create a new item type
    pub fn new(id: String, satisfies: HashMap<NeedType, f32>, consumable: bool) -> Self {
        Self {
            id,
            satisfies,
            consumable,
        }
    }

    /// Get how much this item satisfies a particular need
    /// Returns 0.0 if the item doesn't affect this need
    pub fn satisfaction_for(&self, need: NeedType) -> f32 {
        *self.satisfies.get(&need).unwrap_or(&0.0)
    }

    /// Check if this item satisfies a particular need
    pub fn satisfies_need(&self, need: NeedType) -> bool {
        self.satisfies.contains_key(&need) && self.satisfaction_for(need) != 0.0
    }
}

/// Registry of all item types in the simulation
///
/// # Example
/// ```rust
/// use libreconomy::ItemRegistry;
///
/// let registry = ItemRegistry::with_defaults();
///
/// // Query default items
/// let water = registry.get("water").unwrap();
/// assert!(water.consumable);
///
/// let food = registry.get("food").unwrap();
/// assert!(food.consumable);
/// ```
#[derive(Debug, Clone, Default)]
pub struct ItemRegistry {
    items: HashMap<String, ItemType>,
}

impl ItemRegistry {
    /// Create an empty item registry
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
        }
    }

    /// Create a registry with default items pre-loaded
    ///
    /// Default items:
    /// - `"water"` - Reduces Thirst by 30.0
    /// - `"food"` - Reduces Hunger by 25.0
    /// - `"grass"` - Reduces Hunger by 15.0 (for herbivores)
    /// - `"rabbit_meat"` - Reduces Hunger by 40.0 (for carnivores/omnivores)
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        registry.register_defaults();
        registry
    }

    /// Register default items in the registry
    fn register_defaults(&mut self) {
        // Water - satisfies thirst
        let mut water_satisfies = HashMap::new();
        water_satisfies.insert(NeedType::Thirst, -30.0);
        self.register(ItemType::new(
            "water".to_string(),
            water_satisfies,
            true,
        ));

        // Food - generic food satisfies hunger moderately
        let mut food_satisfies = HashMap::new();
        food_satisfies.insert(NeedType::Hunger, -25.0);
        self.register(ItemType::new(
            "food".to_string(),
            food_satisfies,
            true,
        ));

        // Grass - low nutrition plant food
        let mut grass_satisfies = HashMap::new();
        grass_satisfies.insert(NeedType::Hunger, -15.0);
        self.register(ItemType::new(
            "grass".to_string(),
            grass_satisfies,
            true,
        ));

        // Rabbit meat - high nutrition meat
        let mut rabbit_meat_satisfies = HashMap::new();
        rabbit_meat_satisfies.insert(NeedType::Hunger, -40.0);
        self.register(ItemType::new(
            "rabbit_meat".to_string(),
            rabbit_meat_satisfies,
            true,
        ));
    }

    /// Register a new item type
    ///
    /// # Example
    /// ```rust
    /// use libreconomy::{ItemRegistry, ItemType, NeedType};
    /// use std::collections::HashMap;
    ///
    /// let mut registry = ItemRegistry::new();
    ///
    /// let mut satisfies = HashMap::new();
    /// satisfies.insert(NeedType::Tiredness, -50.0);
    ///
    /// let bed = ItemType::new("bed".to_string(), satisfies, false);
    /// registry.register(bed);
    ///
    /// assert!(registry.get("bed").is_some());
    /// ```
    pub fn register(&mut self, item: ItemType) {
        self.items.insert(item.id.clone(), item);
    }

    /// Get an item type by ID
    pub fn get(&self, item_id: &str) -> Option<&ItemType> {
        self.items.get(item_id)
    }

    /// Get a mutable reference to an item type by ID
    pub fn get_mut(&mut self, item_id: &str) -> Option<&mut ItemType> {
        self.items.get_mut(item_id)
    }

    /// Check if an item is registered
    pub fn contains(&self, item_id: &str) -> bool {
        self.items.contains_key(item_id)
    }

    /// Remove an item type from the registry
    pub fn remove(&mut self, item_id: &str) -> Option<ItemType> {
        self.items.remove(item_id)
    }

    /// Get all item IDs
    pub fn item_ids(&self) -> Vec<String> {
        self.items.keys().cloned().collect()
    }

    /// Get the number of registered items
    pub fn count(&self) -> usize {
        self.items.len()
    }

    /// Find all items that satisfy a particular need
    ///
    /// # Example
    /// ```rust
    /// use libreconomy::{ItemRegistry, NeedType};
    ///
    /// let registry = ItemRegistry::with_defaults();
    /// let thirst_items = registry.items_satisfying(NeedType::Thirst);
    ///
    /// assert!(thirst_items.contains(&"water"));
    /// ```
    pub fn items_satisfying(&self, need: NeedType) -> Vec<&str> {
        self.items
            .values()
            .filter(|item| item.satisfies_need(need))
            .map(|item| item.id.as_str())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_item_type_creation() {
        let mut satisfies = HashMap::new();
        satisfies.insert(NeedType::Thirst, -30.0);

        let water = ItemType::new("water".to_string(), satisfies, true);

        assert_eq!(water.id, "water");
        assert_eq!(water.satisfaction_for(NeedType::Thirst), -30.0);
        assert_eq!(water.satisfaction_for(NeedType::Hunger), 0.0);
        assert!(water.consumable);
    }

    #[test]
    fn test_item_type_satisfies_need() {
        let mut satisfies = HashMap::new();
        satisfies.insert(NeedType::Hunger, -25.0);

        let food = ItemType::new("food".to_string(), satisfies, true);

        assert!(food.satisfies_need(NeedType::Hunger));
        assert!(!food.satisfies_need(NeedType::Thirst));
        assert!(!food.satisfies_need(NeedType::Tiredness));
    }

    #[test]
    fn test_registry_creation() {
        let registry = ItemRegistry::new();
        assert_eq!(registry.count(), 0);

        let with_defaults = ItemRegistry::with_defaults();
        assert_eq!(with_defaults.count(), 4);
    }

    #[test]
    fn test_registry_default_items() {
        let registry = ItemRegistry::with_defaults();

        // Water
        let water = registry.get("water").expect("water should exist");
        assert_eq!(water.satisfaction_for(NeedType::Thirst), -30.0);
        assert!(water.consumable);

        // Food
        let food = registry.get("food").expect("food should exist");
        assert_eq!(food.satisfaction_for(NeedType::Hunger), -25.0);
        assert!(food.consumable);

        // Grass
        let grass = registry.get("grass").expect("grass should exist");
        assert_eq!(grass.satisfaction_for(NeedType::Hunger), -15.0);
        assert!(grass.consumable);

        // Rabbit meat
        let rabbit_meat = registry.get("rabbit_meat").expect("rabbit_meat should exist");
        assert_eq!(rabbit_meat.satisfaction_for(NeedType::Hunger), -40.0);
        assert!(rabbit_meat.consumable);
    }

    #[test]
    fn test_registry_register_custom() {
        let mut registry = ItemRegistry::new();

        let mut satisfies = HashMap::new();
        satisfies.insert(NeedType::Tiredness, -50.0);

        let bed = ItemType::new("bed".to_string(), satisfies, false);
        registry.register(bed);

        assert!(registry.contains("bed"));
        assert_eq!(registry.count(), 1);

        let bed = registry.get("bed").unwrap();
        assert_eq!(bed.satisfaction_for(NeedType::Tiredness), -50.0);
        assert!(!bed.consumable);
    }

    #[test]
    fn test_registry_get_mut() {
        let mut registry = ItemRegistry::with_defaults();

        let water = registry.get_mut("water").unwrap();
        water.satisfies.insert(NeedType::Hunger, -5.0);

        let water = registry.get("water").unwrap();
        assert_eq!(water.satisfaction_for(NeedType::Thirst), -30.0);
        assert_eq!(water.satisfaction_for(NeedType::Hunger), -5.0);
    }

    #[test]
    fn test_registry_remove() {
        let mut registry = ItemRegistry::with_defaults();
        assert_eq!(registry.count(), 4);

        let removed = registry.remove("water");
        assert!(removed.is_some());
        assert_eq!(registry.count(), 3);
        assert!(!registry.contains("water"));
    }

    #[test]
    fn test_registry_item_ids() {
        let registry = ItemRegistry::with_defaults();
        let ids = registry.item_ids();

        assert_eq!(ids.len(), 4);
        assert!(ids.contains(&"water".to_string()));
        assert!(ids.contains(&"food".to_string()));
        assert!(ids.contains(&"grass".to_string()));
        assert!(ids.contains(&"rabbit_meat".to_string()));
    }

    #[test]
    fn test_items_satisfying() {
        let registry = ItemRegistry::with_defaults();

        let thirst_items = registry.items_satisfying(NeedType::Thirst);
        assert_eq!(thirst_items.len(), 1);
        assert!(thirst_items.contains(&"water"));

        let hunger_items = registry.items_satisfying(NeedType::Hunger);
        assert_eq!(hunger_items.len(), 3);
        assert!(hunger_items.contains(&"food"));
        assert!(hunger_items.contains(&"grass"));
        assert!(hunger_items.contains(&"rabbit_meat"));

        let tiredness_items = registry.items_satisfying(NeedType::Tiredness);
        assert_eq!(tiredness_items.len(), 0);
    }

    #[test]
    fn test_item_type_with_multiple_needs() {
        let mut satisfies = HashMap::new();
        satisfies.insert(NeedType::Hunger, -20.0);
        satisfies.insert(NeedType::Thirst, -10.0);

        let soup = ItemType::new("soup".to_string(), satisfies, true);

        assert!(soup.satisfies_need(NeedType::Hunger));
        assert!(soup.satisfies_need(NeedType::Thirst));
        assert!(!soup.satisfies_need(NeedType::Tiredness));

        assert_eq!(soup.satisfaction_for(NeedType::Hunger), -20.0);
        assert_eq!(soup.satisfaction_for(NeedType::Thirst), -10.0);
        assert_eq!(soup.satisfaction_for(NeedType::Tiredness), 0.0);
    }
}
