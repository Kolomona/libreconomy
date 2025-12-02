//! C FFI layer for cross-language integration
//! 
//! This module provides C-compatible exports for the libreconomy API.
//! Functions use opaque pointers to wrap the ECS World.

use specs::prelude::*;
use crate::agent::components::{Agent, Needs, Inventory, Wallet};
use crate::agent::identity::AgentIdAllocator;
use crate::agent::creation;

/// Opaque handle to an ECS World for C FFI
#[repr(C)]
pub struct WorldHandle {
    _private: [u8; 0],
}

/// Create a new World and return an opaque handle
/// Caller must call destroy_world to free memory
#[no_mangle]
pub extern "C" fn create_world() -> *mut WorldHandle {
    let mut world = World::new();
    world.register::<Agent>();
    world.register::<Needs>();
    world.register::<Inventory>();
    world.register::<Wallet>();
    world.insert(AgentIdAllocator::new());
    
    Box::into_raw(Box::new(world)) as *mut WorldHandle
}

/// Destroy a World created by create_world
/// 
/// # Safety
/// The handle must be valid and not used after this call
#[no_mangle]
pub unsafe extern "C" fn destroy_world(world: *mut WorldHandle) {
    if !world.is_null() {
        drop(Box::from_raw(world as *mut World));
    }
}

/// Create an agent with default components
/// Returns the entity ID as u64
#[no_mangle]
pub unsafe extern "C" fn create_agent_default(world: *mut WorldHandle) -> u64 {
    if world.is_null() {
        return 0;
    }
    let world_ref = &mut *(world as *mut World);
    let entity = creation::create_agent(world_ref);
    entity.id() as u64
}

/// Create an agent with custom needs (thirst, hunger) and default inventory/wallet
/// Returns the entity ID as u64
#[no_mangle]
pub unsafe extern "C" fn create_agent_with_needs(
    world: *mut WorldHandle,
    thirst: f64,
    hunger: f64,
) -> u64 {
    if world.is_null() {
        return 0;
    }
    let world_ref = &mut *(world as *mut World);
    let needs = Needs::new(thirst as f32, hunger as f32);
    let entity = creation::create_agent_with_needs(world_ref, needs);
    entity.id() as u64
}

/// Create an agent with custom wallet and default needs/inventory
/// Returns the entity ID as u64
#[no_mangle]
pub unsafe extern "C" fn create_agent_with_wallet(
    world: *mut WorldHandle,
    currency: f64,
) -> u64 {
    if world.is_null() {
        return 0;
    }
    let world_ref = &mut *(world as *mut World);
    let wallet = Wallet::new(currency as f32);
    let entity = creation::create_agent_with_wallet(world_ref, wallet);
    entity.id() as u64
}

/// Create an agent with fully custom components
/// Returns the entity ID as u64
#[no_mangle]
pub unsafe extern "C" fn create_agent_full(
    world: *mut WorldHandle,
    thirst: f64,
    hunger: f64,
    currency: f64,
) -> u64 {
    if world.is_null() {
        return 0;
    }
    let world_ref = &mut *(world as *mut World);
    let needs = Needs::new(thirst as f32, hunger as f32);
    let inventory = Inventory::default();
    let wallet = Wallet::new(currency as f32);
    let entity = creation::create_agent_custom(world_ref, needs, inventory, wallet);
    entity.id() as u64
}

/// Remove an agent from the world by entity ID
/// Returns 1 on success, 0 on failure
#[no_mangle]
pub unsafe extern "C" fn remove_agent(world: *mut WorldHandle, entity_id: u64) -> i32 {
    if world.is_null() {
        return 0;
    }
    let world_ref = &mut *(world as *mut World);
    
    // Convert u64 back to Entity
    // Note: Entity contains generation info; this is simplified
    let entity = world_ref.entities().entity(entity_id as u32);
    
    if !world_ref.entities().is_alive(entity) {
        return 0;
    }
    
    creation::remove_agent(world_ref, entity);
    1
}

/// Get the total number of agents in the world
#[no_mangle]
pub unsafe extern "C" fn get_agent_count(world: *mut WorldHandle) -> u64 {
    if world.is_null() {
        return 0;
    }
    let world_ref = &*(world as *mut World);
    let agents = world_ref.read_storage::<Agent>();
    agents.count() as u64
}
