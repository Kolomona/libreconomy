# libreconomy Development Roadmap

# Agents System
- [x] Define agent entity structure
- [x] Implement core agent components (Needs, Inventory, Wallet, Skills, Knowledge, Employment, Preferences)
- [x] Implement agent creation logic
- [x] Implement agent lifecycle management (creation, removal)
- [x] Integrate agents with ECS world
- [x] Add unit tests for agent logic

- [ ] Market System (Phased Evolution)
	- [ ] Stage 0: Direct bilateral trade (baseline)
		- [ ] Basic MatchingSystem (neighbor scan + negotiation)
		- [ ] Emit TransactionEvent for successful trades
		- [ ] Unit tests for direct matching & negotiation trigger
	- [ ] Stage 1: Intent Registry
		- [ ] Define Offer component & OfferIndex (bounded ring buffers, expiry)
		- [ ] Expiry + LRU pruning logic
		- [ ] Matching prefers indexed offers before neighbor scanning
		- [ ] Unit tests for offer posting/matching & pruning
	- [ ] Stage 2: Venue Activation
		- [ ] Venue component & activation metrics collection
		- [ ] Threshold evaluation (search attempts, failed match rate)
		- [ ] petgraph edges (agent ↔ venue) for network analysis
		- [ ] Integration tests for dynamic venue enabling/disabling
	- [ ] Stage 3: Order Book & Escrow
		- [ ] Bounded OrderBook per item (capacity limits)
		- [ ] Escrow/SettlementSystem for low-trust trades
		- [ ] Price indicators (median/EMA) feeding PriceMemory
		- [ ] Unit & integration tests (settlement atomicity, indicator correctness)
	- [ ] Stage 4: Advanced Features
		- [ ] Fees & latency modeling (optional)
		- [ ] Specialized venue rules (min reputation, caps)
		- [ ] Performance profiling (embedded constraints)
		- [ ] Property tests for pruning & order invariants
	- [ ] Gating & Config
		- [ ] Implement MarketConfig flags (intents_enabled, venues_enabled, order_book_enabled, escrow_enabled)
		- [ ] Dynamic enable/disable logic & metrics thresholds tests
		- [ ] Documentation of phased activation & deactivation criteria

- [ ] Labor System
	- [ ] Define employment/gig work components
	- [ ] Implement job matching logic
	- [ ] Implement employer/employee relationship management
	- [ ] Integrate labor system with ECS world
	- [ ] Add unit tests for labor logic

- [ ] Production System
	- [ ] Define production/recipe components
	- [ ] Implement goods transformation logic
	- [ ] Implement skill/tool requirements
	- [ ] Integrate production system with ECS world
	- [ ] Add unit tests for production logic

- [ ] Decision System
	- [ ] Define trait for agent decision-making
	- [ ] Implement utility maximization strategy
	- [ ] Implement pluggable decision strategies
	- [ ] Integrate decision system with ECS world
	- [ ] Add unit tests for decision logic

- [ ] Learning System (Multi-Phase)
	- [ ] Phase 1: First-hand reputation implementation
		- [ ] Define ReputationKnowledge component with first_hand HashMap
		- [ ] Implement ReputationView (alpha, beta, last_interaction_tick, count)
		- [ ] Implement ReputationUpdateSystem consuming TransactionEvent
		- [ ] TrustLevel baseline for unknown agents; optional time-based decay on read
		- [ ] Unit and integration tests for reputation scoring, updates, decay
	- [ ] Phase 2: Information sharing (rumor mill)
		- [ ] Extend ReputationKnowledge with second_hand HashMap
		- [ ] Implement SecondHandView structure (informant, view, received_tick, informant_trust)
		- [ ] Implement InformationSharingSystem with truthfulness probability based on sharer reputation
		- [ ] Aggregation weighted by informant trust and time decay; enforce Dunbar limit and pruning
		- [ ] Unit and integration tests for information flow, aggregation, pruning
	- [ ] Phase 3: Broader knowledge types
		- [ ] Implement PriceMemory (RingBuffer of recent prices, avg_price, trend detection)
		- [ ] Implement InventorySnapshot with confidence decay and bounded storage
		- [ ] Implement DemandProfile for sellers (frequency, typical quantity/price, last_purchase_tick)
		- [ ] Full LearningSystem updates after trades; time/interaction/capacity-based decay; memory bounds (Dunbar)
		- [ ] Unit, integration, and property tests for decay/pruning and invariants

- [ ] Negotiation System
	- [ ] Implement bartering logic
	- [ ] Implement price negotiation algorithm
	- [ ] Integrate negotiation system with ECS world
	- [ ] Add unit tests for negotiation logic

- [ ] Simulation Loop Integration
	- [ ] Connect all systems to ECS world
	- [ ] Implement simulation step/loop function
	- [ ] Expose simulation control via FFI
	- [ ] Add unit tests for simulation loop

- [ ] FFI API Expansion
	- [ ] Expose agent creation functions
	- [ ] Expose simulation control functions
	- [ ] Expose state query functions
	- [ ] Add FFI unit/integration tests

- [ ] Serialization/Deserialization
	- [ ] Implement save/load for simulation state
	- [ ] Add unit tests for serialization logic

- [ ] Graph Structures
	- [ ] Integrate petgraph for trade networks
	- [ ] Integrate petgraph for production chains
	- [ ] Integrate petgraph for employment relationships
	- [ ] Add unit tests for graph logic

- [ ] Testing
	- [ ] Write unit tests for all systems
	- [ ] Add property-based tests for complex logic
	- [ ] Add integration tests for FFI and simulation

- [ ] Examples
	- [x] Add example for agent creation
	- [ ] Add example for market transactions
	- [ ] Add example for labor and production
	- [ ] Add example for simulation loop usage

- [ ] Documentation
	- [ ] Document all public APIs
	- [ ] Add module-level documentation
	- [ ] Add API usage examples to docs

- [ ] Performance Optimization
	- [ ] Profile for low-power devices
	- [ ] Minimize allocations in hot loops
	- [ ] Reuse collections where possible

- [ ] FFI Bindings Validation
	- [ ] Validate cbindgen C header generation
	- [ ] Validate uniffi bindings for all languages
	- [ ] Add FFI usage tests for each language
