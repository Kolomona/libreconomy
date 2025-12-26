//! Reputation tracking example
//!
//! Demonstrates how to use libreconomy's reputation system with transaction events.

use libreconomy::*;
use specs::prelude::*;

fn main() {
    println!("=== Libreconomy Reputation System Example ===\n");

    // Create ECS world
    let mut world = World::new();
    world.register::<Agent>();
    world.register::<Needs>();
    world.register::<Inventory>();
    world.register::<Wallet>();
    world.register::<ReputationKnowledge>();
    world.insert(AgentIdAllocator::new());
    world.insert(TransactionLog::new());
    world.insert(CurrentTick(0));
    world.insert(ReputationDecayConfig::default());

    println!("1. Creating three trading agents...");

    // Create three agents
    let merchant = {
        let needs = Needs::new(30.0, 30.0, 20.0);
        let inventory = Inventory::default();
        let wallet = Wallet::new(1000.0);
        let entity = create_agent_custom(&mut world, needs, inventory, wallet);

        // Add reputation component
        let mut reputation_storage = world.write_storage::<ReputationKnowledge>();
        reputation_storage.insert(entity, ReputationKnowledge::new()).unwrap();
        entity
    };

    let customer1 = {
        let needs = Needs::new(30.0, 30.0, 20.0);
        let inventory = Inventory::default();
        let wallet = Wallet::new(500.0);
        let entity = create_agent_custom(&mut world, needs, inventory, wallet);

        let mut reputation_storage = world.write_storage::<ReputationKnowledge>();
        reputation_storage.insert(entity, ReputationKnowledge::new()).unwrap();
        entity
    };

    let customer2 = {
        let needs = Needs::new(30.0, 30.0, 20.0);
        let inventory = Inventory::default();
        let wallet = Wallet::new(500.0);
        let entity = create_agent_custom(&mut world, needs, inventory, wallet);

        let mut reputation_storage = world.write_storage::<ReputationKnowledge>();
        reputation_storage.insert(entity, ReputationKnowledge::new()).unwrap();
        entity
    };

    println!("   - Merchant (lots of currency)");
    println!("   - Customer 1 (moderate currency)");
    println!("   - Customer 2 (moderate currency)\n");

    // Get agent IDs
    let (merchant_id, customer1_id, customer2_id) = {
        let agents = world.read_storage::<Agent>();
        let m_id = agents.get(merchant).unwrap().id;
        let c1_id = agents.get(customer1).unwrap().id;
        let c2_id = agents.get(customer2).unwrap().id;
        (m_id, c1_id, c2_id)
    };

    println!("2. Simulating trading interactions...\n");

    // Scenario 1: Multiple successful trades with Customer 1
    println!("   Trade 1: Customer 1 buys water from Merchant (SUCCESS)");
    {
        let mut log = world.write_resource::<TransactionLog>();
        log.add(TransactionEvent::successful_trade(
            customer1_id,
            merchant_id,
            "water".to_string(),
            10.0,
            100,
        ));
    }

    println!("   Trade 2: Customer 1 buys food from Merchant (SUCCESS)");
    {
        let mut log = world.write_resource::<TransactionLog>();
        log.add(TransactionEvent::successful_trade(
            customer1_id,
            merchant_id,
            "food".to_string(),
            15.0,
            200,
        ));
    }

    println!("   Trade 3: Customer 1 buys water from Merchant (SUCCESS)");
    {
        let mut log = world.write_resource::<TransactionLog>();
        log.add(TransactionEvent::successful_trade(
            customer1_id,
            merchant_id,
            "water".to_string(),
            10.0,
            300,
        ));
    }

    // Scenario 2: Mixed interactions with Customer 2
    println!("   Trade 4: Customer 2 buys food from Merchant (SUCCESS)");
    {
        let mut log = world.write_resource::<TransactionLog>();
        log.add(TransactionEvent::successful_trade(
            customer2_id,
            merchant_id,
            "food".to_string(),
            15.0,
            400,
        ));
    }

    println!("   Trade 5: Customer 2 disputes quality (NEGATIVE)");
    {
        let mut log = world.write_resource::<TransactionLog>();
        log.add(TransactionEvent::negative_interaction(
            customer2_id,
            merchant_id,
            1.5, // Moderate negative weight
            500,
        ));
    }

    println!("   Trade 6: Customer 2 tries another purchase (FAILED)");
    {
        let mut log = world.write_resource::<TransactionLog>();
        log.add(TransactionEvent::failed_trade(
            customer2_id,
            merchant_id,
            "water".to_string(),
            2.0, // Stronger negative weight for failed trade
            600,
        ));
    }

    // Process all transactions
    println!("\n3. Processing reputation updates...");
    let mut reputation_system = ReputationUpdateSystem;
    reputation_system.run_now(&world);
    world.maintain();

    // Check reputation scores
    println!("\n4. Reputation Scores:\n");

    // Extract all needed data in scoped block
    let (merchant_view_c1, merchant_view_c2, merchant_trusted_c1, merchant_trusted_c2,
         c1_score, c1_confidence, c1_interactions, c1_alpha, c1_beta,
         c2_score, c2_confidence, c2_interactions, c2_alpha, c2_beta,
         merchant_trusted) = {
        let reputation_storage = world.read_storage::<ReputationKnowledge>();

        let merchant_rep = reputation_storage.get(merchant).unwrap();
        let mv1 = merchant_rep.get_score(customer1_id);
        let mv2 = merchant_rep.get_score(customer2_id);
        let mt1 = merchant_rep.is_trusted(customer1_id, 0.7);
        let mt2 = merchant_rep.is_trusted(customer2_id, 0.7);
        let mt = merchant_rep.get_most_trusted(2);

        let customer1_rep = reputation_storage.get(customer1).unwrap();
        let c1_view = customer1_rep.first_hand.get(&merchant_id).unwrap();
        let c1s = customer1_rep.get_score(merchant_id);
        let c1conf = c1_view.confidence();
        let c1int = c1_view.interaction_count;
        let c1a = c1_view.alpha;
        let c1b = c1_view.beta;

        let customer2_rep = reputation_storage.get(customer2).unwrap();
        let c2_view = customer2_rep.first_hand.get(&merchant_id).unwrap();
        let c2s = customer2_rep.get_score(merchant_id);
        let c2conf = c2_view.confidence();
        let c2int = c2_view.interaction_count;
        let c2a = c2_view.alpha;
        let c2b = c2_view.beta;

        (mv1, mv2, mt1, mt2, c1s, c1conf, c1int, c1a, c1b, c2s, c2conf, c2int, c2a, c2b, mt)
    };

    println!("   Merchant's perspective:");
    println!("     - Customer 1 score: {:.2} (3 successful trades)", merchant_view_c1);
    println!("     - Customer 2 score: {:.2} (1 success, 2 negative)", merchant_view_c2);
    println!("     - Trusted (>0.7): Customer 1={}, Customer 2={}",
        merchant_trusted_c1, merchant_trusted_c2
    );

    println!("\n   Customer 1's perspective:");
    println!("     - Merchant score: {:.2}", c1_score);
    println!("     - Confidence: {:.2} ({} interactions)", c1_confidence, c1_interactions);
    println!("     - Beta params: alpha={:.2}, beta={:.2}", c1_alpha, c1_beta);

    println!("\n   Customer 2's perspective:");
    println!("     - Merchant score: {:.2} (mixed experience)", c2_score);
    println!("     - Confidence: {:.2} ({} interactions)", c2_confidence, c2_interactions);
    println!("     - Beta params: alpha={:.2}, beta={:.2}", c2_alpha, c2_beta);

    // Demonstrate reputation-based decision making
    println!("\n5. Most Trusted Partners:\n");
    println!("   Merchant's top 2 trusted partners:");
    for (i, (agent_id, score)) in merchant_trusted.iter().enumerate() {
        let name = if *agent_id == customer1_id {
            "Customer 1"
        } else if *agent_id == customer2_id {
            "Customer 2"
        } else {
            "Unknown"
        };
        println!("     {}. {} (score: {:.2})", i + 1, name, score);
    }

    // Demonstrate temporal decay
    println!("\n6. Temporal Decay Simulation:\n");

    // Advance time significantly
    {
        let mut tick = world.write_resource::<CurrentTick>();
        tick.0 = 10000;
    }

    println!("   Time advanced to tick 10000 (from tick 600)");
    println!("   Original score (tick 300): {:.2}", c1_score);

    // Calculate decayed score
    let decayed_score = {
        let reputation_storage = world.read_storage::<ReputationKnowledge>();
        let customer1_rep = reputation_storage.get(customer1).unwrap();
        let c1_view = customer1_rep.first_hand.get(&merchant_id).unwrap();
        c1_view.score_with_decay(10000, 0.0001)
    };
    println!("   Decayed score (decay_rate=0.0001): {:.2}", decayed_score);
    println!("   Decay toward neutral (0.5) demonstrates memory fading");

    // Run decay system
    let mut decay_system = ReputationDecaySystem;
    decay_system.run_now(&world);
    world.maintain();

    // Check rebalanced values
    let (new_sum, new_score) = {
        let reputation_storage = world.read_storage::<ReputationKnowledge>();
        let customer1_rep = reputation_storage.get(customer1).unwrap();
        let c1_view = customer1_rep.first_hand.get(&merchant_id).unwrap();
        (c1_view.alpha + c1_view.beta, c1_view.score())
    };

    println!("   After decay system rebalancing:");
    println!("     - Alpha+Beta sum: {:.2} (normalized from higher values)", new_sum);
    println!("     - Score preserved: {:.2}", new_score);

    println!("\n=== Example Complete ===");
    println!("\nKey Takeaways:");
    println!("  - Positive interactions increase reputation (alpha parameter)");
    println!("  - Negative interactions decrease reputation (beta parameter)");
    println!("  - Score = alpha / (alpha + beta) ∈ [0, 1]");
    println!("  - Confidence = alpha + beta (higher = more certain)");
    println!("  - Temporal decay regresses scores toward neutral (0.5)");
    println!("  - Symmetric updates: both parties update their views");
}
