//! C FFI for component access
//!
//! Functions for reading and writing agent components from C/C++

use specs::prelude::*;
use crate::agent::components::{Needs, Inventory, Wallet};
use super::WorldHandle;
use std::ffi::CStr;
use std::os::raw::c_char;

/// Get agent needs
///
/// # Arguments
/// * `world` - World handle
/// * `entity_id` - Entity ID
/// * `out_thirst` - Output pointer for thirst value
/// * `out_hunger` - Output pointer for hunger value
/// * `out_tiredness` - Output pointer for tiredness value
///
/// # Returns
/// 1 on success, 0 if entity doesn't exist or doesn't have Needs component
///
/// # Safety
/// All output pointers must be valid. The world handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn get_needs(
    world: *mut WorldHandle,
    entity_id: u64,
    out_thirst: *mut f32,
    out_hunger: *mut f32,
    out_tiredness: *mut f32,
) -> i32 {
    if world.is_null() || out_thirst.is_null() || out_hunger.is_null() || out_tiredness.is_null() {
        return 0;
    }

    let world_ref = &*(world as *const World);
    let entity = world_ref.entities().entity(entity_id as u32);

    if !world_ref.entities().is_alive(entity) {
        return 0;
    }

    let needs_storage = world_ref.read_storage::<Needs>();

    match needs_storage.get(entity) {
        Some(needs) => {
            *out_thirst = needs.thirst;
            *out_hunger = needs.hunger;
            *out_tiredness = needs.tiredness;
            1
        }
        None => 0,
    }
}

/// Set agent needs
///
/// # Arguments
/// * `world` - World handle
/// * `entity_id` - Entity ID
/// * `thirst` - Thirst value (clamped to 0-100)
/// * `hunger` - Hunger value (clamped to 0-100)
/// * `tiredness` - Tiredness value (clamped to 0-100)
///
/// # Returns
/// 1 on success, 0 if entity doesn't exist or doesn't have Needs component
///
/// # Safety
/// The world handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn set_needs(
    world: *mut WorldHandle,
    entity_id: u64,
    thirst: f32,
    hunger: f32,
    tiredness: f32,
) -> i32 {
    if world.is_null() {
        return 0;
    }

    let world_ref = &mut *(world as *mut World);
    let entity = world_ref.entities().entity(entity_id as u32);

    if !world_ref.entities().is_alive(entity) {
        return 0;
    }

    let mut needs_storage = world_ref.write_storage::<Needs>();

    match needs_storage.get_mut(entity) {
        Some(needs) => {
            *needs = Needs::new(thirst, hunger, tiredness);
            1
        }
        None => 0,
    }
}

/// Get inventory item quantity
///
/// # Arguments
/// * `world` - World handle
/// * `entity_id` - Entity ID
/// * `item_id` - Item ID (null-terminated C string)
///
/// # Returns
/// Item quantity, or 0 if entity doesn't exist, doesn't have Inventory, or doesn't have the item
///
/// # Safety
/// The world handle and item_id must be valid. item_id must be a null-terminated string.
#[no_mangle]
pub unsafe extern "C" fn get_inventory_item(
    world: *mut WorldHandle,
    entity_id: u64,
    item_id: *const c_char,
) -> u32 {
    if world.is_null() || item_id.is_null() {
        return 0;
    }

    let item_str = match CStr::from_ptr(item_id).to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };

    let world_ref = &*(world as *const World);
    let entity = world_ref.entities().entity(entity_id as u32);

    if !world_ref.entities().is_alive(entity) {
        return 0;
    }

    let inventory_storage = world_ref.read_storage::<Inventory>();

    match inventory_storage.get(entity) {
        Some(inventory) => inventory.quantity(item_str),
        None => 0,
    }
}

/// Add item to inventory
///
/// # Arguments
/// * `world` - World handle
/// * `entity_id` - Entity ID
/// * `item_id` - Item ID (null-terminated C string)
/// * `quantity` - Quantity to add
///
/// # Returns
/// 1 on success, 0 if entity doesn't exist or doesn't have Inventory component
///
/// # Safety
/// The world handle and item_id must be valid. item_id must be a null-terminated string.
#[no_mangle]
pub unsafe extern "C" fn add_inventory_item(
    world: *mut WorldHandle,
    entity_id: u64,
    item_id: *const c_char,
    quantity: u32,
) -> i32 {
    if world.is_null() || item_id.is_null() {
        return 0;
    }

    let item_str = match CStr::from_ptr(item_id).to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };

    let world_ref = &mut *(world as *mut World);
    let entity = world_ref.entities().entity(entity_id as u32);

    if !world_ref.entities().is_alive(entity) {
        return 0;
    }

    let mut inventory_storage = world_ref.write_storage::<Inventory>();

    match inventory_storage.get_mut(entity) {
        Some(inventory) => {
            inventory.add(item_str, quantity);
            1
        }
        None => 0,
    }
}

/// Remove item from inventory
///
/// # Arguments
/// * `world` - World handle
/// * `entity_id` - Entity ID
/// * `item_id` - Item ID (null-terminated C string)
/// * `quantity` - Quantity to remove
///
/// # Returns
/// Amount actually removed (may be less than requested if not enough in inventory)
///
/// # Safety
/// The world handle and item_id must be valid. item_id must be a null-terminated string.
#[no_mangle]
pub unsafe extern "C" fn remove_inventory_item(
    world: *mut WorldHandle,
    entity_id: u64,
    item_id: *const c_char,
    quantity: u32,
) -> u32 {
    if world.is_null() || item_id.is_null() {
        return 0;
    }

    let item_str = match CStr::from_ptr(item_id).to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };

    let world_ref = &mut *(world as *mut World);
    let entity = world_ref.entities().entity(entity_id as u32);

    if !world_ref.entities().is_alive(entity) {
        return 0;
    }

    let mut inventory_storage = world_ref.write_storage::<Inventory>();

    match inventory_storage.get_mut(entity) {
        Some(inventory) => inventory.remove(item_str, quantity),
        None => 0,
    }
}

/// Get wallet currency amount
///
/// # Arguments
/// * `world` - World handle
/// * `entity_id` - Entity ID
/// * `out_currency` - Output pointer for currency value
///
/// # Returns
/// 1 on success, 0 if entity doesn't exist or doesn't have Wallet component
///
/// # Safety
/// The world handle and out_currency pointer must be valid.
#[no_mangle]
pub unsafe extern "C" fn get_wallet(
    world: *mut WorldHandle,
    entity_id: u64,
    out_currency: *mut f32,
) -> i32 {
    if world.is_null() || out_currency.is_null() {
        return 0;
    }

    let world_ref = &*(world as *const World);
    let entity = world_ref.entities().entity(entity_id as u32);

    if !world_ref.entities().is_alive(entity) {
        return 0;
    }

    let wallet_storage = world_ref.read_storage::<Wallet>();

    match wallet_storage.get(entity) {
        Some(wallet) => {
            *out_currency = wallet.currency;
            1
        }
        None => 0,
    }
}

/// Deposit currency to wallet
///
/// # Arguments
/// * `world` - World handle
/// * `entity_id` - Entity ID
/// * `amount` - Amount to deposit (negative values ignored)
///
/// # Returns
/// 1 on success, 0 if entity doesn't exist or doesn't have Wallet component
///
/// # Safety
/// The world handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn deposit_wallet(
    world: *mut WorldHandle,
    entity_id: u64,
    amount: f32,
) -> i32 {
    if world.is_null() {
        return 0;
    }

    let world_ref = &mut *(world as *mut World);
    let entity = world_ref.entities().entity(entity_id as u32);

    if !world_ref.entities().is_alive(entity) {
        return 0;
    }

    let mut wallet_storage = world_ref.write_storage::<Wallet>();

    match wallet_storage.get_mut(entity) {
        Some(wallet) => {
            wallet.deposit(amount);
            1
        }
        None => 0,
    }
}

/// Withdraw currency from wallet
///
/// # Arguments
/// * `world` - World handle
/// * `entity_id` - Entity ID
/// * `amount` - Amount to withdraw
/// * `out_withdrawn` - Output pointer for amount actually withdrawn
///
/// # Returns
/// 1 on success, 0 if entity doesn't exist or doesn't have Wallet component
///
/// # Safety
/// The world handle and out_withdrawn pointer must be valid.
#[no_mangle]
pub unsafe extern "C" fn withdraw_wallet(
    world: *mut WorldHandle,
    entity_id: u64,
    amount: f32,
    out_withdrawn: *mut f32,
) -> i32 {
    if world.is_null() || out_withdrawn.is_null() {
        return 0;
    }

    let world_ref = &mut *(world as *mut World);
    let entity = world_ref.entities().entity(entity_id as u32);

    if !world_ref.entities().is_alive(entity) {
        return 0;
    }

    let mut wallet_storage = world_ref.write_storage::<Wallet>();

    match wallet_storage.get_mut(entity) {
        Some(wallet) => {
            *out_withdrawn = wallet.withdraw(amount);
            1
        }
        None => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{create_world, create_agent_default};
    use std::ffi::CString;

    #[test]
    fn test_get_set_needs() {
        unsafe {
            let world = create_world();
            let agent_id = create_agent_default(world);

            let mut thirst: f32 = 0.0;
            let mut hunger: f32 = 0.0;
            let mut tiredness: f32 = 0.0;

            // Get initial needs
            assert_eq!(
                get_needs(world, agent_id, &mut thirst, &mut hunger, &mut tiredness),
                1
            );
            assert_eq!(thirst, 50.0);
            assert_eq!(hunger, 50.0);
            assert_eq!(tiredness, 50.0);

            // Set new needs
            assert_eq!(set_needs(world, agent_id, 80.0, 60.0, 40.0), 1);

            // Verify changed
            assert_eq!(
                get_needs(world, agent_id, &mut thirst, &mut hunger, &mut tiredness),
                1
            );
            assert_eq!(thirst, 80.0);
            assert_eq!(hunger, 60.0);
            assert_eq!(tiredness, 40.0);

            crate::destroy_world(world);
        }
    }

    #[test]
    fn test_inventory_operations() {
        unsafe {
            let world = create_world();
            let agent_id = create_agent_default(world);

            let item_water = CString::new("water").unwrap();

            // Initially empty
            assert_eq!(get_inventory_item(world, agent_id, item_water.as_ptr()), 0);

            // Add items
            assert_eq!(
                add_inventory_item(world, agent_id, item_water.as_ptr(), 10),
                1
            );
            assert_eq!(get_inventory_item(world, agent_id, item_water.as_ptr()), 10);

            // Remove some
            let removed = remove_inventory_item(world, agent_id, item_water.as_ptr(), 3);
            assert_eq!(removed, 3);
            assert_eq!(get_inventory_item(world, agent_id, item_water.as_ptr()), 7);

            // Remove more than available
            let removed = remove_inventory_item(world, agent_id, item_water.as_ptr(), 20);
            assert_eq!(removed, 7);
            assert_eq!(get_inventory_item(world, agent_id, item_water.as_ptr()), 0);

            crate::destroy_world(world);
        }
    }

    #[test]
    fn test_wallet_operations() {
        unsafe {
            let world = create_world();
            let agent_id = create_agent_default(world);

            let mut currency: f32 = 0.0;

            // Get initial wallet
            assert_eq!(get_wallet(world, agent_id, &mut currency), 1);
            assert_eq!(currency, 100.0);

            // Deposit
            assert_eq!(deposit_wallet(world, agent_id, 50.0), 1);
            assert_eq!(get_wallet(world, agent_id, &mut currency), 1);
            assert_eq!(currency, 150.0);

            // Withdraw
            let mut withdrawn: f32 = 0.0;
            assert_eq!(withdraw_wallet(world, agent_id, 30.0, &mut withdrawn), 1);
            assert_eq!(withdrawn, 30.0);
            assert_eq!(get_wallet(world, agent_id, &mut currency), 1);
            assert_eq!(currency, 120.0);

            // Withdraw more than available
            assert_eq!(withdraw_wallet(world, agent_id, 200.0, &mut withdrawn), 1);
            assert_eq!(withdrawn, 120.0);
            assert_eq!(get_wallet(world, agent_id, &mut currency), 1);
            assert_eq!(currency, 0.0);

            crate::destroy_world(world);
        }
    }
}
