//! WASM world wrapper
//!
//! Provides JavaScript-friendly interface to the ECS World.

use wasm_bindgen::prelude::*;
use specs::prelude::*;
use serde_wasm_bindgen;

use crate::{
    Agent, Needs, Inventory, Wallet, ResourceSource, SpeciesComponent, Species,
    AgentIdAllocator, create_agent, create_agent_with_needs,
    create_agent_with_wallet, create_agent_custom, remove_agent,
    ItemRegistry, NeedType, EnergyComponent,
};

/// WASM wrapper for the ECS World
///
/// This provides a JavaScript-friendly interface to the libreconomy simulation.
///
/// # Example (JavaScript)
/// ```javascript
/// import init, { WasmWorld } from './libreconomy.js';
///
/// await init();
/// const world = WasmWorld.new();
/// const agentId = world.create_agent();
/// const needs = world.get_needs(agentId);
/// console.log(needs.thirst, needs.hunger, needs.tiredness);
/// ```
#[wasm_bindgen]
pub struct WasmWorld {
    world: World,
    item_registry: ItemRegistry,
}

#[wasm_bindgen]
impl WasmWorld {
    /// Create a new WASM world with all components registered
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let mut world = World::new();

        // Register all components
        world.register::<Agent>();
        world.register::<Needs>();
        world.register::<EnergyComponent>();
        world.register::<Inventory>();
        world.register::<Wallet>();
        world.register::<ResourceSource>();
        world.register::<SpeciesComponent>();

        // Insert AgentId allocator resource
        world.insert(AgentIdAllocator::new());

        // Create default item registry
        let item_registry = ItemRegistry::with_defaults();

        Self { world, item_registry }
    }

    /// Create an agent with default components
    /// Returns the entity ID as u32
    pub fn create_agent(&mut self) -> u32 {
        let entity = create_agent(&mut self.world);
        entity.id() as u32
    }

    /// Create an agent with custom needs
    /// Returns the entity ID as u32
    pub fn create_agent_with_needs(&mut self, thirst: f32, hunger: f32, tiredness: f32) -> u32 {
        let needs = Needs::new(thirst, hunger, tiredness);
        let entity = create_agent_with_needs(&mut self.world, needs);
        entity.id() as u32
    }

    /// Create an agent with custom wallet
    /// Returns the entity ID as u32
    pub fn create_agent_with_wallet(&mut self, currency: f32) -> u32 {
        let wallet = Wallet::new(currency);
        let entity = create_agent_with_wallet(&mut self.world, wallet);
        entity.id() as u32
    }

    /// Create an agent with fully custom components
    /// Returns the entity ID as u32
    pub fn create_agent_full(
        &mut self,
        thirst: f32,
        hunger: f32,
        tiredness: f32,
        currency: f32,
    ) -> u32 {
        let needs = Needs::new(thirst, hunger, tiredness);
        let inventory = Inventory::default();
        let wallet = Wallet::new(currency);
        let entity = create_agent_custom(&mut self.world, needs, inventory, wallet);
        entity.id() as u32
    }

    /// Remove an agent from the world
    /// Returns true on success
    ///
    /// # Safety
    /// The entity_id must be valid and currently alive.
    /// Calling this with an invalid or already-deleted entity ID may panic.
    pub fn remove_agent(&mut self, entity_id: u32) -> bool {
        // Note: entity() will panic if the entity doesn't exist
        // This is a limitation of the specs API
        let entity = self.world.entities().entity(entity_id);
        remove_agent(&mut self.world, entity);
        true
    }

    /// Get the total number of agents
    pub fn get_agent_count(&self) -> u32 {
        let agents = self.world.read_storage::<Agent>();
        agents.count() as u32
    }

    /// Get agent needs as JSON
    /// Returns null if entity doesn't exist
    pub fn get_needs(&self, entity_id: u32) -> JsValue {
        let entity = self.world.entities().entity(entity_id);
        let needs_storage = self.world.read_storage::<Needs>();

        match needs_storage.get(entity) {
            Some(needs) => serde_wasm_bindgen::to_value(needs).unwrap_or(JsValue::NULL),
            None => JsValue::NULL,
        }
    }

    /// Set agent needs
    /// Returns true on success, false if entity doesn't exist
    pub fn set_needs(&mut self, entity_id: u32, thirst: f32, hunger: f32, tiredness: f32) -> bool {
        let entity = self.world.entities().entity(entity_id);
        let mut needs_storage = self.world.write_storage::<Needs>();

        match needs_storage.get_mut(entity) {
            Some(needs) => {
                *needs = Needs::new(thirst, hunger, tiredness);
                true
            }
            None => false,
        }
    }

    /// Get agent energy as JSON
    /// Returns null if entity doesn't exist
    pub fn get_energy(&self, entity_id: u32) -> JsValue {
        let entity = self.world.entities().entity(entity_id);
        let energy_storage = self.world.read_storage::<EnergyComponent>();

        match energy_storage.get(entity) {
            Some(energy) => serde_wasm_bindgen::to_value(energy).unwrap_or(JsValue::NULL),
            None => JsValue::NULL,
        }
    }

    /// Set agent energy
    /// Returns true on success, false if entity doesn't exist
    pub fn set_energy(&mut self, entity_id: u32, current: f32, max: f32) -> bool {
        let entity = self.world.entities().entity(entity_id);
        let mut energy_storage = self.world.write_storage::<EnergyComponent>();

        match energy_storage.get_mut(entity) {
            Some(energy) => {
                *energy = EnergyComponent::new(current, max);
                true
            }
            None => {
                // If entity doesn't have energy component, add it
                energy_storage.insert(entity, EnergyComponent::new(current, max)).ok();
                true
            }
        }
    }

    /// Get agent inventory as JSON
    /// Returns null if entity doesn't exist
    pub fn get_inventory(&self, entity_id: u32) -> JsValue {
        let entity = self.world.entities().entity(entity_id);
        let inventory_storage = self.world.read_storage::<Inventory>();

        match inventory_storage.get(entity) {
            Some(inventory) => serde_wasm_bindgen::to_value(inventory).unwrap_or(JsValue::NULL),
            None => JsValue::NULL,
        }
    }

    /// Add an item to agent's inventory
    /// Returns true on success, false if entity doesn't exist
    pub fn add_item(&mut self, entity_id: u32, item_id: &str, quantity: u32) -> bool {
        let entity = self.world.entities().entity(entity_id);
        let mut inventory_storage = self.world.write_storage::<Inventory>();

        match inventory_storage.get_mut(entity) {
            Some(inventory) => {
                inventory.add(item_id, quantity);
                true
            }
            None => false,
        }
    }

    /// Remove an item from agent's inventory
    /// Returns amount actually removed
    pub fn remove_item(&mut self, entity_id: u32, item_id: &str, quantity: u32) -> u32 {
        let entity = self.world.entities().entity(entity_id);
        let mut inventory_storage = self.world.write_storage::<Inventory>();

        match inventory_storage.get_mut(entity) {
            Some(inventory) => inventory.remove(item_id, quantity),
            None => 0,
        }
    }

    /// Get agent wallet as JSON
    /// Returns null if entity doesn't exist
    pub fn get_wallet(&self, entity_id: u32) -> JsValue {
        let entity = self.world.entities().entity(entity_id);
        let wallet_storage = self.world.read_storage::<Wallet>();

        match wallet_storage.get(entity) {
            Some(wallet) => serde_wasm_bindgen::to_value(wallet).unwrap_or(JsValue::NULL),
            None => JsValue::NULL,
        }
    }

    /// Deposit currency to agent's wallet
    /// Returns true on success, false if entity doesn't exist
    pub fn deposit(&mut self, entity_id: u32, amount: f32) -> bool {
        let entity = self.world.entities().entity(entity_id);
        let mut wallet_storage = self.world.write_storage::<Wallet>();

        match wallet_storage.get_mut(entity) {
            Some(wallet) => {
                wallet.deposit(amount);
                true
            }
            None => false,
        }
    }

    /// Withdraw currency from agent's wallet
    /// Returns amount actually withdrawn
    pub fn withdraw(&mut self, entity_id: u32, amount: f32) -> f32 {
        let entity = self.world.entities().entity(entity_id);
        let mut wallet_storage = self.world.write_storage::<Wallet>();

        match wallet_storage.get_mut(entity) {
            Some(wallet) => wallet.withdraw(amount),
            None => 0.0,
        }
    }

    /// Create a resource source entity
    /// Returns the entity ID as u32
    pub fn create_resource_source(
        &mut self,
        resource_type: &str,
        item_produced: &str,
        regeneration_rate: f32,
        initial_stock: u32,
    ) -> u32 {
        let resource_source = ResourceSource::new(
            resource_type.to_string(),
            item_produced.to_string(),
            regeneration_rate,
            initial_stock,
        );

        let entity = self.world
            .create_entity()
            .with(resource_source)
            .build();

        entity.id() as u32
    }

    /// Get resource source as JSON
    /// Returns null if entity doesn't exist or isn't a resource source
    pub fn get_resource_source(&self, entity_id: u32) -> JsValue {
        let entity = self.world.entities().entity(entity_id);
        let resource_storage = self.world.read_storage::<ResourceSource>();

        match resource_storage.get(entity) {
            Some(resource) => serde_wasm_bindgen::to_value(resource).unwrap_or(JsValue::NULL),
            None => JsValue::NULL,
        }
    }

    /// Harvest from a resource source
    /// Returns amount actually harvested
    pub fn harvest_resource(&mut self, entity_id: u32, amount: u32) -> u32 {
        let entity = self.world.entities().entity(entity_id);
        let mut resource_storage = self.world.write_storage::<ResourceSource>();

        match resource_storage.get_mut(entity) {
            Some(resource) => resource.harvest(amount),
            None => 0,
        }
    }

    /// Regenerate all resource sources
    pub fn regenerate_resources(&mut self) {
        let mut resource_storage = self.world.write_storage::<ResourceSource>();

        for resource in (&mut resource_storage).join() {
            resource.regenerate();
        }
    }

    /// Get items that satisfy a specific need type
    /// Returns JSON array of item IDs
    pub fn get_items_for_need(&self, need_type: &str) -> JsValue {
        let need = match need_type {
            "thirst" => NeedType::Thirst,
            "hunger" => NeedType::Hunger,
            "tiredness" => NeedType::Tiredness,
            _ => return JsValue::NULL,
        };

        let items = self.item_registry.items_satisfying(need);
        serde_wasm_bindgen::to_value(&items).unwrap_or(JsValue::NULL)
    }

    /// Get item satisfaction value for a need
    /// Returns 0.0 if item doesn't exist or doesn't affect this need
    pub fn get_item_satisfaction(&self, item_id: &str, need_type: &str) -> f32 {
        let need = match need_type {
            "thirst" => NeedType::Thirst,
            "hunger" => NeedType::Hunger,
            "tiredness" => NeedType::Tiredness,
            _ => return 0.0,
        };

        self.item_registry
            .get(item_id)
            .map(|item| item.satisfaction_for(need))
            .unwrap_or(0.0)
    }

    /// Create a rabbit agent
    /// Returns the entity ID as u32
    pub fn create_rabbit(&mut self) -> u32 {
        let needs = Needs::new(50.0, 50.0, 50.0);
        let inventory = Inventory::default();
        let wallet = Wallet::new(0.0);
        let species = SpeciesComponent::rabbit();

        let entity = create_agent_custom(&mut self.world, needs, inventory, wallet);

        // Add species component
        let mut species_storage = self.world.write_storage::<SpeciesComponent>();
        species_storage.insert(entity, species).ok();

        entity.id() as u32
    }

    /// Create a human agent
    /// Returns the entity ID as u32
    pub fn create_human(&mut self) -> u32 {
        let needs = Needs::new(50.0, 50.0, 50.0);
        let inventory = Inventory::default();
        let wallet = Wallet::new(0.0);
        let species = SpeciesComponent::human();

        let entity = create_agent_custom(&mut self.world, needs, inventory, wallet);

        // Add species component
        let mut species_storage = self.world.write_storage::<SpeciesComponent>();
        species_storage.insert(entity, species).ok();

        entity.id() as u32
    }

    /// Get agent species type as string
    /// Returns "Human", "Rabbit", "Custom", or null if no species component
    pub fn get_species(&self, entity_id: u32) -> JsValue {
        let entity = self.world.entities().entity(entity_id);
        let species_storage = self.world.read_storage::<SpeciesComponent>();

        match species_storage.get(entity) {
            Some(species_comp) => {
                let species_str = match species_comp.species {
                    Species::Human => "Human",
                    Species::Rabbit => "Rabbit",
                    Species::Custom(_) => "Custom",
                };
                JsValue::from_str(species_str)
            }
            None => JsValue::NULL,
        }
    }

    /// Check if agent can eat a specific plant item
    pub fn can_eat_plant(&self, entity_id: u32, plant_id: &str) -> bool {
        let entity = self.world.entities().entity(entity_id);
        let species_storage = self.world.read_storage::<SpeciesComponent>();

        match species_storage.get(entity) {
            Some(species_comp) => species_comp.diet.can_eat_plant(plant_id),
            None => false,
        }
    }

    /// Check if agent can hunt a specific species
    /// species_name: "Human", "Rabbit", etc.
    pub fn can_hunt(&self, entity_id: u32, prey_name: &str) -> bool {
        let entity = self.world.entities().entity(entity_id);
        let species_storage = self.world.read_storage::<SpeciesComponent>();

        let prey_species = match prey_name {
            "Human" => Species::Human,
            "Rabbit" => Species::Rabbit,
            _ => return false,
        };

        match species_storage.get(entity) {
            Some(species_comp) => species_comp.diet.can_hunt(prey_species),
            None => false,
        }
    }
}

// Non-WASM-bindgen methods (for internal use)
impl WasmWorld {
    /// Get reference to the inner World (for decision system)
    pub(crate) fn get_world(&self) -> &World {
        &self.world
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_world_creation() {
        let world = WasmWorld::new();
        assert_eq!(world.get_agent_count(), 0);
    }

    #[test]
    fn test_create_agent() {
        let mut world = WasmWorld::new();
        let id = world.create_agent();
        assert_eq!(world.get_agent_count(), 1);
        // Entity ID should be valid (can be 0 or higher in specs)
        let _ = id; // Just verify it's created
    }

    // WASM-specific tests that use JsValue require wasm32 target
    #[cfg(target_arch = "wasm32")]
    #[test]
    fn test_needs_operations() {
        let mut world = WasmWorld::new();
        let id = world.create_agent_with_needs(80.0, 60.0, 40.0);

        // Get needs should work
        let needs_js = world.get_needs(id);
        assert!(!needs_js.is_null());

        // Set needs should work
        assert!(world.set_needs(id, 50.0, 50.0, 50.0));

        // Invalid entity should return false
        assert!(!world.set_needs(9999, 0.0, 0.0, 0.0));
    }

    // Non-WASM test for needs operations
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_needs_operations() {
        let mut world = WasmWorld::new();
        let id = world.create_agent_with_needs(80.0, 60.0, 40.0);

        // Set needs should work
        assert!(world.set_needs(id, 50.0, 50.0, 50.0));

        // Invalid entity should return false
        assert!(!world.set_needs(9999, 0.0, 0.0, 0.0));
    }

    #[test]
    fn test_inventory_operations() {
        let mut world = WasmWorld::new();
        let id = world.create_agent();

        // Add item
        assert!(world.add_item(id, "water", 5));

        // Remove item
        let removed = world.remove_item(id, "water", 3);
        assert_eq!(removed, 3);

        // Invalid entity
        assert!(!world.add_item(9999, "water", 1));
    }

    #[test]
    fn test_wallet_operations() {
        let mut world = WasmWorld::new();
        let id = world.create_agent_with_wallet(100.0);

        // Deposit
        assert!(world.deposit(id, 50.0));

        // Withdraw
        let withdrawn = world.withdraw(id, 30.0);
        assert_eq!(withdrawn, 30.0);

        // Invalid entity
        assert!(!world.deposit(9999, 10.0));
    }

    #[test]
    fn test_resource_source_operations() {
        let mut world = WasmWorld::new();
        let id = world.create_resource_source("plant", "grass", 1.0, 100);

        // Harvest
        let harvested = world.harvest_resource(id, 30);
        assert_eq!(harvested, 30);

        // Regenerate
        world.regenerate_resources();

        // Invalid entity
        let harvested = world.harvest_resource(9999, 10);
        assert_eq!(harvested, 0);
    }

    #[test]
    fn test_item_registry_queries() {
        let world = WasmWorld::new();

        // Get satisfaction value (doesn't require JsValue)
        let satisfaction = world.get_item_satisfaction("water", "thirst");
        assert_eq!(satisfaction, -30.0);

        // Invalid item
        let satisfaction = world.get_item_satisfaction("invalid_item", "thirst");
        assert_eq!(satisfaction, 0.0);

        // Invalid need type
        let satisfaction = world.get_item_satisfaction("water", "invalid");
        assert_eq!(satisfaction, 0.0);
    }

    #[test]
    fn test_remove_agent() {
        let mut world = WasmWorld::new();
        let id1 = world.create_agent();
        let id2 = world.create_agent();
        assert_eq!(world.get_agent_count(), 2);

        // Remove first agent
        assert!(world.remove_agent(id1));
        assert_eq!(world.get_agent_count(), 1);

        // Remove second agent
        assert!(world.remove_agent(id2));
        assert_eq!(world.get_agent_count(), 0);
    }
}
