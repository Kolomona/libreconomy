//! Property-based tests for libreconomy
//!
//! These tests use proptest to verify invariants across random inputs.

use libreconomy::*;
use proptest::prelude::*;

// Test that reputation scores always stay in [0, 1] bounds
proptest! {
    #[test]
    fn reputation_score_in_bounds(
        alpha in 0.0f32..1000.0,
        beta in 0.0f32..1000.0
    ) {
        let view = ReputationView::with_prior(alpha.max(0.01), beta.max(0.01));
        let score = view.score();
        prop_assert!(score >= 0.0 && score <= 1.0);
    }

    #[test]
    fn reputation_score_with_decay_in_bounds(
        alpha in 1.0f32..100.0,
        beta in 1.0f32..100.0,
        ticks in 0u64..10000,
        decay_rate in 0.0f32..0.01
    ) {
        let mut view = ReputationView::with_prior(alpha, beta);
        view.last_interaction_tick = 0;
        let score = view.score_with_decay(ticks, decay_rate);
        prop_assert!(score >= 0.0 && score <= 1.0);
    }

    #[test]
    fn needs_always_clamped(
        thirst in -100.0f32..200.0,
        hunger in -100.0f32..200.0,
        tiredness in -100.0f32..200.0
    ) {
        let needs = Needs::new(thirst, hunger, tiredness);
        prop_assert!(needs.thirst >= MIN_NEEDS && needs.thirst <= MAX_NEEDS);
        prop_assert!(needs.hunger >= MIN_NEEDS && needs.hunger <= MAX_NEEDS);
        prop_assert!(needs.tiredness >= MIN_NEEDS && needs.tiredness <= MAX_NEEDS);
    }

    #[test]
    fn wallet_never_negative(
        initial in -1000.0f32..1000.0,
        deposit in 0.0f32..500.0,
        withdraw in 0.0f32..500.0
    ) {
        let mut wallet = Wallet::new(initial);
        prop_assert!(wallet.currency >= 0.0);

        wallet.deposit(deposit);
        prop_assert!(wallet.currency >= 0.0);

        wallet.withdraw(withdraw);
        prop_assert!(wallet.currency >= 0.0);
    }

    #[test]
    fn inventory_saturating_operations(
        initial in 0u32..100,
        add1 in 0u32..1000,
        add2 in 0u32..1000,
        remove in 0u32..500
    ) {
        let mut inv = Inventory::default();
        inv.set_quantity("test", initial);

        inv.add("test", add1);
        let qty1 = inv.quantity("test");
        prop_assert!(qty1 >= initial);

        inv.add("test", add2);
        let qty2 = inv.quantity("test");
        prop_assert!(qty2 >= qty1);

        let removed = inv.remove("test", remove);
        let qty3 = inv.quantity("test");
        prop_assert!(removed <= qty2);
        prop_assert!(qty3 == qty2.saturating_sub(removed));
    }

    #[test]
    fn positive_updates_increase_reputation(
        weight in 0.1f32..10.0
    ) {
        let mut view = ReputationView::new();
        let score_before = view.score();

        view.update(weight, 100);
        let score_after = view.score();

        prop_assert!(score_after > score_before);
    }

    #[test]
    fn negative_updates_decrease_reputation(
        weight in 0.1f32..10.0
    ) {
        let mut view = ReputationView::new();
        let score_before = view.score();

        view.update(-weight, 100);
        let score_after = view.score();

        prop_assert!(score_after < score_before);
    }

    #[test]
    fn reputation_confidence_increases_with_interactions(
        updates in 1usize..20
    ) {
        let mut view = ReputationView::new();
        let confidence_before = view.confidence();

        for i in 0..updates {
            view.update(if i % 2 == 0 { 1.0 } else { -1.0 }, i as u64 * 100);
        }

        let confidence_after = view.confidence();
        prop_assert!(confidence_after > confidence_before);
        prop_assert_eq!(view.interaction_count, updates as u32);
    }

    #[test]
    fn resource_source_harvest_never_exceeds_stock(
        initial_stock in 0u32..1000,
        harvest_amount in 0u32..2000
    ) {
        let mut resource = ResourceSource::new(
            "test".to_string(),
            "item".to_string(),
            0.0,
            initial_stock
        );

        let harvested = resource.harvest(harvest_amount);

        prop_assert!(harvested <= initial_stock);
        prop_assert!(harvested <= harvest_amount);
        prop_assert_eq!(resource.current_stock, initial_stock.saturating_sub(harvested));
    }

    #[test]
    fn resource_regeneration_adds_to_stock(
        initial_stock in 0u32..500,
        regen_rate in 0.0f32..50.0,
        ticks in 1usize..10
    ) {
        let mut resource = ResourceSource::new(
            "test".to_string(),
            "item".to_string(),
            regen_rate,
            initial_stock
        );

        for _ in 0..ticks {
            resource.regenerate();
        }

        // Regeneration truncates to u32, so only rates >= 1.0 will add stock
        if regen_rate >= 1.0 {
            prop_assert!(resource.current_stock > initial_stock);
        } else {
            // Rates < 1.0 get truncated to 0, so no change
            prop_assert_eq!(resource.current_stock, initial_stock);
        }
    }
}
