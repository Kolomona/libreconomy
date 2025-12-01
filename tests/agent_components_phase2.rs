//! Phase 2 component tests: Needs, Inventory, Wallet

use libreconomy::agent::components::{Inventory, Needs, Wallet, MAX_NEEDS, MIN_NEEDS};

#[test]
fn needs_new_clamps_values() {
    let n = Needs::new(-5.0, 150.0);
    assert!(n.thirst >= MIN_NEEDS && n.thirst <= MAX_NEEDS);
    assert!(n.hunger >= MIN_NEEDS && n.hunger <= MAX_NEEDS);
    assert_eq!(n.thirst, MIN_NEEDS);
    assert_eq!(n.hunger, MAX_NEEDS);
}

#[test]
fn needs_clamp_in_place() {
    let mut n = Needs { thirst: -1.0, hunger: 200.0 };
    n.clamp();
    assert_eq!(n.thirst, MIN_NEEDS);
    assert_eq!(n.hunger, MAX_NEEDS);
}

#[test]
fn inventory_add_and_remove_is_safe() {
    let mut inv = Inventory::default();
    assert_eq!(inv.quantity("water"), 0);

    inv.add("water", 3);
    assert_eq!(inv.quantity("water"), 3);

    let removed = inv.remove("water", 2);
    assert_eq!(removed, 2);
    assert_eq!(inv.quantity("water"), 1);

    // removing more than available only removes what's there
    let removed2 = inv.remove("water", 10);
    assert_eq!(removed2, 1);
    assert_eq!(inv.quantity("water"), 0);
}

#[test]
fn wallet_deposit_withdraw_non_negative() {
    let mut w = Wallet::new(-10.0);
    assert_eq!(w.currency, 0.0);

    w.deposit(5.0);
    assert_eq!(w.currency, 5.0);

    let got = w.withdraw(10.0);
    assert_eq!(got, 5.0);
    assert_eq!(w.currency, 0.0);

    w.deposit(-2.0); // ignored
    assert_eq!(w.currency, 0.0);

    let got2 = w.withdraw(-1.0); // ignored
    assert_eq!(got2, 0.0);
    assert_eq!(w.currency, 0.0);
}
