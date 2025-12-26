//! uniffi implementation for Python/Kotlin/Swift bindings
//!
//! Provides an OOP-style wrapper around the C FFI for cleaner language bindings.

use specs::prelude::*;
use crate::agent::components::{Needs as NeedsComponent, Inventory as InventoryComponent, Wallet as WalletComponent, Agent};
use crate::agent::identity::AgentIdAllocator;
use crate::agent::creation;
use std::sync::{Arc, Mutex};

/// World wrapper for uniffi bindings
///
/// Wraps the ECS World with thread-safe access for Python/Kotlin/Swift.
pub struct World {
    world: Arc<Mutex<specs::World>>,
}

impl World {
    /// Create a new world with all components registered
    pub fn new() -> Self {
        let mut world = specs::World::new();

        // Register all components
        world.register::<Agent>();
        world.register::<NeedsComponent>();
        world.register::<InventoryComponent>();
        world.register::<WalletComponent>();

        // Insert AgentId allocator resource
        world.insert(AgentIdAllocator::new());

        Self {
            world: Arc::new(Mutex::new(world)),
        }
    }

    /// Create an agent with default components
    pub fn create_agent(&self) -> u64 {
        let mut world = self.world.lock().unwrap();
        let entity = creation::create_agent(&mut world);
        entity.id() as u64
    }

    /// Create an agent with custom needs
    pub fn create_agent_with_needs(&self, thirst: f32, hunger: f32, tiredness: f32) -> u64 {
        let mut world = self.world.lock().unwrap();
        let needs = NeedsComponent::new(thirst, hunger, tiredness);
        let entity = creation::create_agent_with_needs(&mut world, needs);
        entity.id() as u64
    }

    /// Create an agent with custom wallet
    pub fn create_agent_with_wallet(&self, currency: f32) -> u64 {
        let mut world = self.world.lock().unwrap();
        let wallet = WalletComponent::new(currency);
        let entity = creation::create_agent_with_wallet(&mut world, wallet);
        entity.id() as u64
    }

    /// Create an agent with fully custom components
    pub fn create_agent_full(
        &self,
        thirst: f32,
        hunger: f32,
        tiredness: f32,
        currency: f32,
    ) -> u64 {
        let mut world = self.world.lock().unwrap();
        let needs = NeedsComponent::new(thirst, hunger, tiredness);
        let inventory = InventoryComponent::default();
        let wallet = WalletComponent::new(currency);
        let entity = creation::create_agent_custom(&mut world, needs, inventory, wallet);
        entity.id() as u64
    }

    /// Remove an agent from the world
    /// Returns true on success
    pub fn remove_agent(&self, entity_id: u64) -> bool {
        let mut world = self.world.lock().unwrap();
        let entity = world.entities().entity(entity_id as u32);
        if !world.entities().is_alive(entity) {
            return false;
        }
        creation::remove_agent(&mut world, entity);
        true
    }

    /// Get the total number of agents
    pub fn get_agent_count(&self) -> u64 {
        let world = self.world.lock().unwrap();
        let agents = world.read_storage::<Agent>();
        agents.count() as u64
    }

    /// Get agent needs
    /// Returns None if entity doesn't exist or doesn't have Needs component
    pub fn get_needs(&self, entity_id: u64) -> Option<Needs> {
        let world = self.world.lock().unwrap();
        let entity = world.entities().entity(entity_id as u32);

        if !world.entities().is_alive(entity) {
            return None;
        }

        let needs_storage = world.read_storage::<NeedsComponent>();
        needs_storage.get(entity).map(|needs| Needs {
            thirst: needs.thirst,
            hunger: needs.hunger,
            tiredness: needs.tiredness,
        })
    }

    /// Set agent needs
    /// Returns true on success
    pub fn set_needs(&self, entity_id: u64, thirst: f32, hunger: f32, tiredness: f32) -> bool {
        let world = self.world.lock().unwrap();
        let entity = world.entities().entity(entity_id as u32);

        if !world.entities().is_alive(entity) {
            return false;
        }

        let mut needs_storage = world.write_storage::<NeedsComponent>();
        match needs_storage.get_mut(entity) {
            Some(needs) => {
                *needs = NeedsComponent::new(thirst, hunger, tiredness);
                true
            }
            None => false,
        }
    }

    /// Get inventory item quantity
    /// Returns 0 if entity doesn't exist or doesn't have the item
    pub fn get_inventory_item(&self, entity_id: u64, item_id: String) -> u32 {
        let world = self.world.lock().unwrap();
        let entity = world.entities().entity(entity_id as u32);

        if !world.entities().is_alive(entity) {
            return 0;
        }

        let inventory_storage = world.read_storage::<InventoryComponent>();
        inventory_storage
            .get(entity)
            .map(|inv| inv.quantity(&item_id))
            .unwrap_or(0)
    }

    /// Add item to inventory
    /// Returns true on success
    pub fn add_inventory_item(&self, entity_id: u64, item_id: String, quantity: u32) -> bool {
        let world = self.world.lock().unwrap();
        let entity = world.entities().entity(entity_id as u32);

        if !world.entities().is_alive(entity) {
            return false;
        }

        let mut inventory_storage = world.write_storage::<InventoryComponent>();
        match inventory_storage.get_mut(entity) {
            Some(inventory) => {
                inventory.add(&item_id, quantity);
                true
            }
            None => false,
        }
    }

    /// Remove item from inventory
    /// Returns amount actually removed
    pub fn remove_inventory_item(&self, entity_id: u64, item_id: String, quantity: u32) -> u32 {
        let world = self.world.lock().unwrap();
        let entity = world.entities().entity(entity_id as u32);

        if !world.entities().is_alive(entity) {
            return 0;
        }

        let mut inventory_storage = world.write_storage::<InventoryComponent>();
        inventory_storage
            .get_mut(entity)
            .map(|inv| inv.remove(&item_id, quantity))
            .unwrap_or(0)
    }

    /// Get wallet
    /// Returns None if entity doesn't exist or doesn't have Wallet component
    pub fn get_wallet(&self, entity_id: u64) -> Option<Wallet> {
        let world = self.world.lock().unwrap();
        let entity = world.entities().entity(entity_id as u32);

        if !world.entities().is_alive(entity) {
            return None;
        }

        let wallet_storage = world.read_storage::<WalletComponent>();
        wallet_storage.get(entity).map(|wallet| Wallet {
            currency: wallet.currency,
        })
    }

    /// Deposit currency to wallet
    /// Returns true on success
    pub fn deposit_wallet(&self, entity_id: u64, amount: f32) -> bool {
        let world = self.world.lock().unwrap();
        let entity = world.entities().entity(entity_id as u32);

        if !world.entities().is_alive(entity) {
            return false;
        }

        let mut wallet_storage = world.write_storage::<WalletComponent>();
        match wallet_storage.get_mut(entity) {
            Some(wallet) => {
                wallet.deposit(amount);
                true
            }
            None => false,
        }
    }

    /// Withdraw currency from wallet
    /// Returns amount actually withdrawn
    pub fn withdraw_wallet(&self, entity_id: u64, amount: f32) -> f32 {
        let world = self.world.lock().unwrap();
        let entity = world.entities().entity(entity_id as u32);

        if !world.entities().is_alive(entity) {
            return 0.0;
        }

        let mut wallet_storage = world.write_storage::<WalletComponent>();
        wallet_storage
            .get_mut(entity)
            .map(|wallet| wallet.withdraw(amount))
            .unwrap_or(0.0)
    }
}

impl std::fmt::Debug for World {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "World {{ agent_count: {} }}", self.get_agent_count())
    }
}

/// Needs dictionary for uniffi
pub struct Needs {
    pub thirst: f32,
    pub hunger: f32,
    pub tiredness: f32,
}

/// Inventory dictionary for uniffi
/// Note: HashMap is handled through methods, not as a direct field
pub struct Inventory {}

/// Wallet dictionary for uniffi
pub struct Wallet {
    pub currency: f32,
}

// Namespace functions
pub fn libreconomy_version() -> String {
    "0.0.1".to_string()
}

pub fn get_agent_count() -> u32 {
    // Placeholder - in a real implementation, this would track global agent count
    // For now, users should use World::get_agent_count() instead
    0
}
