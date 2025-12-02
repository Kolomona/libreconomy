# libreconomy: Overall Research and Design Document

Last Updated: December 1, 2025  
Document Purpose: Comprehensive design reference for AI agents implementing libreconomy systems  
Status: Agent System complete; remaining systems in research/design phase

---

## Project Context and Constraints

### Core Requirements
- Cross-platform Rust library for agent-based economy simulation
- Must run efficiently on Raspberry Pi Zero (single-core ARM, minimal RAM)
- FFI exposure via cbindgen for C/C++ and uniffi for Python/Swift/Kotlin/Ruby
- Strict TDD methodology required
- ECS architecture using specs crate
- No panics, no unwrap in library code, use Result types
- Feature-gated parallelism via rayon disabled by default

### Performance Constraints
- Use f32 over f64 unless precision critical
- Minimize allocations in hot loops; reuse buffers
- Bounded memory structures with fixed-size ring buffers and capacity limits
- Lazy computation and pruning strategies
- No global state or singletons
- Graph operations via petgraph for trade networks and production chains

### Economic Model Principles
- Subjective value theory: agents determine value based on internal needs
- No omniscient pricing: agents discover prices through interaction
- Emergent behavior: no centrally-set prices or hardcoded market dynamics
- Pluggable decision-making via trait abstractions
- Support both barter and currency-based transactions
- Information is local, imperfect, and decays over time

---

## Architecture Overview: Entities, Components, Systems

### Entity Types

**Agent Entity (primary):**
- Individual economic participant with needs, resources, knowledge, preferences
- Components: Agent (AgentId), Needs, Inventory, Wallet, Skills, Knowledge, Employment, Preferences, ReputationKnowledge

**Item Entity:**
- Tradable or consumable goods
- Components: ItemType, Quantity, Durability, Owner (AgentId reference)

**Market Entity (future):**
- Trading venue or marketplace
- Components: Listings, Transactions, PriceHistory, Participants

**Production Facility Entity (future):**
- Location where goods are transformed
- Components: Recipes, InputItems, OutputItems, Capacity, Owner

**Transaction Entity (future):**
- Records of completed or pending trades
- Components: Buyer, Seller, Item, Price, Status, Outcome

**Employment/Job Entity (future):**
- Job opportunities or contracts
- Components: Employer, Employee, Wage, Requirements, Status

**Contract Entity (advanced, future):**
- Agreements between agents
- Components: Parties, Terms, Status, Enforcement

**Network Node/Edge (graph layer via petgraph):**
- Relationships: trade networks, production chains, employment hierarchies, reputation graphs

### Core Component Definitions (Implemented)

**Agent:**
- Contains unique AgentId (newtype wrapper around u64)
- Allocated via AgentIdAllocator resource with overflow checking
- Creation functions: create_agent (defaults), create_agent_with_needs, create_agent_with_wallet, create_agent_custom
- Removal: remove_agent cleans up all components and deletes entity

**Needs:**
- thirst f32, hunger f32, range 0.0-100.0, clamped on construction and update
- Drives agent utility calculations
- Decays over time via NeedDecaySystem
- Serializable for save/load via serde

**Inventory:**
- HashMap of item_id String to quantity u32
- Operations: add (saturating), remove (returns actual removed), query
- Panic-free accessors; missing items return 0
- No negative quantities allowed

**Wallet:**
- currency f32 non-negative, clamped on construction
- Operations: deposit (ignores negative amounts), withdraw (returns actual withdrawn, no overdraft allowed)
- All operations saturating and safe

**Skills:**
- HashMap of skill_id String to level u32
- Used for production requirements and wage determination
- Can be extended over time (learning by doing, future feature)

**Knowledge:**
- Complex multi-tiered knowledge structure (detailed in Learning System section)
- Tracks prices, inventory observations, demand patterns, production capabilities, reputation
- Memory-bounded by Dunbar's number (~150 relationships)
- Currently has known_prices HashMap and trade_partners Vec as placeholders

**Employment:**
- job_status Option<String>
- employer Option<String> (AgentId or name reference)
- employees Vec<String> (subordinates)
- Tracks employment relationships for labor systems

**Preferences:**
- utility_function UtilityFunctionType enum (Linear, Exponential, Custom with String parameter)
- risk_tolerance f32 range 0.0-1.0
- Future: honesty f32 to modulate truthfulness in information sharing

### System Architecture

**Implemented Systems:**

*NeedDecaySystem:*
- Increments thirst and hunger linearly over time
- Simple placeholder (tick increases needs by 0.01)
- Can be extended to non-linear decay or event-driven changes

*LearningSystem (stub):*
- Currently provides update method for known_prices in Knowledge component
- Placeholder for full learning implementation detailed below

*NegotiationSystem (stub):*
- Returns true as placeholder
- Will implement full bartering and price negotiation logic

*AgentIdAllocator (resource):*
- Generates unique AgentId values
- Overflow protection via checked_add
- Returns AgentIdError::Overflow if exhausted

*Lifecycle operations:*
- create_agent family of functions
- remove_agent for cleanup and entity deletion

**Planned Core Systems:**

*InventoryManagementSystem:*
- Handles item transfers between agents
- Validates ownership and quantities
- Atomic operations to prevent duplication or loss

*WalletSystem:*
- Processes currency transactions
- Enforces balance constraints
- Tracks transaction history if needed

*MarketMatchingSystem:*
- Matches buyers and sellers based on needs and inventory
- Uses agent knowledge to filter potential partners
- Considers reputation when selecting counterparties

*TradeTransactionSystem:*
- Executes matched trades atomically
- Transfers items and currency in single operation
- Emits TransactionEvent for learning systems
- Handles failure cases gracefully

*PriceDiscoverySystem:*
- Agents update price knowledge after successful trades
- No centralized price-setting; emergent from transactions
- Price trends detected from agent observations

*ProductionSystem:*
- Transforms input items to output items via recipes
- Checks skill requirements and tool availability
- Consumes time and possibly tool durability
- Validates input availability before starting production

*EmploymentSystem:*
- Matches employers (need labor) with workers (offer skills)
- Handles wage negotiations and payments
- Updates Employment components
- Periodic wage payments for ongoing employment

*DecisionSystem (trait-based, pluggable):*
- Agents evaluate available actions (buy, sell, work, produce, rest)
- Initial implementation: utility maximization based on needs satisfaction
- Future implementations: learning algorithms, personality types, heuristics, bounded rationality

*ReputationUpdateSystem:*
- Consumes TransactionEvent to update first-hand reputation
- Applies Beta distribution model updates (alpha += positive weight, beta += negative weight)
- Handles optional decay and Dunbar-limited pruning
- Both transaction participants update their views of each other

*InformationSharingSystem (rumor mill):*
- During successful interactions, agents share knowledge about third parties
- Sharer's reputation determines truthfulness probability
- Receiver weights information by trust in informant
- Updates SecondHandView in ReputationKnowledge component
- Only shares info about agents within sharer's Dunbar set

*LearningSystem (full implementation):*
- Updates price knowledge, inventory observations, demand patterns after trades
- Applies time-based, interaction-based, and capacity-based decay
- Maintains memory bounds via pruning strategies
- Integrates first-hand and second-hand reputation data
- Handles knowledge from both direct experience and shared information

*NegotiationSystem (full implementation):*
- Bartering logic for direct item-item exchanges based on utility calculations
- Price negotiation using price knowledge and needs urgency
- Counter-offers until agreement or breakdown
- Outcome influences reputation updates for both parties

*NetworkSystem:*
- Uses petgraph to maintain and query relationship graphs
- Trade network edges weighted by transaction volume or reputation score
- Production chain edges showing input/output dependencies
- Employment hierarchy graphs for organizational structures
- Efficient graph queries for pathfinding and clustering

*ContractSystem (advanced, future):*
- Creates enforceable agreements between agents
- Tracks fulfillment status and handles breach conditions
- Reputation consequences for contract violations
- Time-limited or conditional contracts

*SimulationLoopSystem:*
- Orchestrates execution order of all systems per tick
- Manages event queues (TransactionEvent, InformationShareEvent, etc.)
- Ensures deterministic execution for reproducibility
- Handles inter-system dependencies and data flow

---

## Knowledge and Information Architecture

### Memory Hierarchy and Constraints

**Dunbar's Number Limit (~150):**
- Each agent maintains meaningful relationships with approximately 150 other agents
- First-hand relationships prioritized; older or less-important pruned when limit reached
- Second-hand (rumor) knowledge allowed for agents beyond Dunbar limit but with lower fidelity and faster decay

**Memory Tiers:**

*Tier 1 (always retained, never pruned):*
- Recent interactions within last N ticks
- High transaction volume partners (frequent traders)
- Current employer and employees
- Critical trading partners for essential goods (water, food)

*Tier 2 (retained with gradual decay):*
- Occasional trading partners
- Historical price data older than N ticks
- Inventory observations from past interactions
- Customer demand patterns for sellers with regular buyers

*Tier 3 (aggressively pruned when capacity reached):*
- One-off interactions with no repeat trades
- Agents completely outside active network
- Very old observations beyond decay threshold
- Low-trust rumors with poor informant reputation

### Knowledge Types

#### 1. Reputation Knowledge

**First-hand reputation (authoritative, highest priority):**

Structure: ReputationView per known agent containing:
- alpha f32: positive evidence accumulator
- beta f32: negative evidence accumulator
- last_interaction_tick u64: for optional time-based decay
- count InteractionCount: total interactions for confidence estimation

Score computation: alpha / (alpha + beta), range 0.0-1.0

Updates via TransactionEvent after each interaction:
- Outcome::Positive(weight): alpha += weight
- Outcome::Negative(weight): beta += weight
- Outcome::Neutral: no change or small symmetric update

Decay strategy (optional): apply on read via exponential decay factor based on tick difference
- Formula: score_aged = score_raw * exp(-decay_rate * delta_ticks)
- Slow decay recommended (half-life approximately 10,000 ticks)

TrustLevel baseline:
- Per-agent default trust for unknown counterparts
- Range 0.0-1.0 representing optimism vs pessimism
- Influences Beta distribution prior for new relationships:
  - alpha0 = 1.0 + trust * k
  - beta0 = 1.0 + (1 - trust) * k
  - k parameter typically 2.0 for weak prior

**Second-hand reputation (rumors, lower priority):**

Structure: SecondHandView containing:
- informant AgentId: who provided this information
- view ReputationView: the reputation data they shared
- received_tick u64: when information was received
- informant_trust f32: receiver's trust in informant at time of sharing (cached for stability)

Maximum storage: 5-10 SecondHandView per subject agent to bound memory

Truthfulness probability based on sharer reputation:
- High reputation (0.8-1.0): 95-99% truthful information
- Medium reputation (0.4-0.8): 70-90% truthful
- Low reputation (0.0-0.4): 50-70% truthful
- Optional honesty trait in Preferences can modulate base probability
- Strategic lying: even trusted agents can occasionally lie (conflict of interest, competitive advantage) but rarely

Aggregation strategy when querying reputation of unknown agent:
1. Check first-hand knowledge; if exists, use directly (most reliable)
2. If no first-hand data, aggregate second-hand views:
   - Weight each view by informant_trust and time_decay
   - Formula: weighted_score = Σ(view.score() * informant_trust * time_decay) / Σ(informant_trust * time_decay)
   - Time decay: exp(-decay_rate * (current_tick - received_tick))
3. If no data at all, fall back to baseline TrustLevel

Decay characteristics:
- First-hand: slow decay, half-life approximately 10,000 ticks (you remember your own experiences)
- Second-hand: fast decay, half-life approximately 1,000 ticks (hearsay gets fuzzy quickly)

Emergent social dynamics:
- Social network clustering: agents form tight groups around trusted information hubs
- Reputation cascades: bad actors collectively blacklisted when trusted agents spread warnings
- Information brokers: well-connected agents with high trust become valuable for introductions and vetting
- Misinformation spread: low-trust agents can poison information pool but influence naturally dampened by trust-weighting
- Strategic manipulation: trusted agents can selectively lie about competitors for advantage but risk reputation damage if discovered
- Echo chambers: isolated clusters may develop distorted local views of outside agents

Defense mechanisms against misinformation:
- Trust-weighting naturally dampens bad actors' influence
- First-hand experience always overrides conflicting rumors
- Time decay ensures stale or false information gradually fades
- Dunbar limit prevents unbounded rumor accumulation

#### 2. Price Knowledge

Structure: PriceMemory per item containing:
- recent_prices: RingBuffer of (AgentId, price f32, tick u64) tuples, fixed size (typically 10)
- avg_price f32: running average for quick estimation
- trend PriceTrend enum: Rising, Falling, Stable detected from recent slope

Memory constraints:
- Only track prices for items of interest determined by:
  - Current needs (thirst/hunger drive interest in water/food)
  - Production inputs required for agent's recipes
  - Frequently traded items (high transaction volume)
- Older prices beyond buffer capacity automatically pruned
- Aggregate to running average for commonly traded items to reduce storage

Use cases:
- Estimate fair prices during trade negotiation
- Detect price gouging (offer significantly above avg_price) or suspiciously low offers (possible fraud)
- Identify arbitrage opportunities between different trading partners
- Adjust willingness to pay based on urgency (high needs increase maximum acceptable price)

Decay strategy:
- Price confidence decays slowly under assumption of relatively stable markets
- Very old prices beyond threshold T ticks pruned entirely from RingBuffer
- Running average updated incrementally to avoid full recomputation

Update triggers:
- Successful trade execution updates both buyer and seller price knowledge
- Observed trades between other agents (passive learning, lower confidence)
- Shared price information from trusted informants (rumor mill, weighted by trust)

#### 3. Inventory Knowledge

Structure: InventorySnapshot per agent containing:
- items HashMap of ItemId to ObservedQuantity
- last_seen_tick u64: timestamp of last observation
- confidence f32: starts at 1.0, decays exponentially over time

ObservedQuantity variants:
- Exact(u32): just completed trade, highly certain of exact quantity
- Approximate(u32): observed indirectly (mentioned in conversation), moderate certainty
- Categorical(Level): vague memory (None, Low, Medium, High, Abundant categories)

Memory constraints:
- Bounded by Dunbar limit subset, typically 50 active traders tracked
- Only recent observations retained; very old pruned
- Confidence decays quickly under assumption that inventory changes rapidly
- Decay formula: confidence = initial_confidence * exp(-decay_rate * delta_ticks)

Use cases:
- Find agents likely to have needed items ("I need item1, agent2 usually has lots")
- Avoid wasting time approaching agents unlikely to have needed items
- Plan production based on known supplier availability
- Opportunistic trades when agent has surplus of item you need

Confidence decay rationale:
- Inventory changes rapidly through consumption, production, and trades
- Old observations become unreliable quickly
- High decay rate (fast confidence loss) reflects this reality

Update triggers:
- Direct trade: exact quantity observed during transaction
- Conversation: approximate quantity mentioned during negotiation
- Observation: categorical level inferred from behavior or statements
- Shared information: weighted by informant trust

#### 4. Demand Pattern Knowledge (for sellers)

Structure: DemandProfile per (customer AgentId, item ItemId) pair containing:
- frequency f32: smoothed purchases per tick (exponential moving average)
- typical_quantity u32: average quantity per purchase
- typical_price f32: average price accepted by customer
- last_purchase_tick u64: most recent transaction timestamp
- lifetime_purchases u32: total transactions for relationship strength estimation

Memory constraints:
- Track only top N regular customers, typically 10-20
- Aggregate purchase data rather than storing every individual transaction
- Prune customers with no recent purchases beyond threshold N ticks
- Sort by frequency or lifetime value to prioritize retention

Use cases:
- Stock management: "Agent1 often buys item1 from me at price X, keep it stocked"
- Loyalty pricing: offer discounts to regular customers with high lifetime_purchases
- Demand prediction: anticipate future purchases based on frequency patterns
- Production planning: manufacture items with reliable demand
- Relationship building: prioritize maintaining inventory for valuable customers

Smoothing strategy:
- Exponential moving average for frequency: freq_new = alpha * (1.0 / delta_ticks) + (1 - alpha) * freq_old
- Prevents single transaction from dominating pattern
- Alpha parameter typically 0.1-0.3 for smooth but responsive tracking

Update triggers:
- Each successful sale to customer updates corresponding DemandProfile
- Frequency recalculated based on time since last purchase
- Quantities and prices updated with running averages

Pruning conditions:
- No purchase in N ticks (typically 1000-5000) triggers removal
- When capacity reached, evict customer with lowest frequency * lifetime_purchases score
- Never prune customers with very recent transactions (grace period)

#### 5. Production Knowledge

Structure: Known producers per item:
- HashMap of ItemId to Vec of AgentId capable of producing that item
- Learned through observation, information sharing, or direct inquiry
- Optional: skill level or reliability score per producer

Memory constraints:
- Track up to 10 producers per item of interest
- Focus on items needed for own consumption or production chains
- Decay if producer stops making that item or becomes unreliable

Use cases:
- Find suppliers for production inputs
- Identify competition in production space for strategic planning
- Build supply chain relationships for reliable sourcing
- Detect monopolies or bottlenecks in production networks

Learning mechanisms:
- Direct observation: agent produces item in your presence
- Shared information: trusted informant tells you who can produce item
- Marketplace listings: producer advertises capability
- Failed attempts: agent claims they cannot produce requested item

Update and decay:
- Add producer when production capability confirmed
- Remove producer after failed sourcing attempts or long inactivity
- Weight producers by reliability (successful deliveries vs failures)

#### 6. Employment and Service Knowledge

Structure: Service providers and wage information:
- HashMap of service type to Vec of (AgentId, wage f32, availability bool)
- Skill levels for workers known through observation or reputation
- Wage rates learned from successful employment relationships

Use cases:
- Find employment opportunities matching agent skills
- Hire workers with required skills for production
- Negotiate wages based on market knowledge of typical rates
- Assess labor supply and demand in local economy

Learning and updates:
- Successful employment relationship confirms wage and availability
- Failed hiring attempts reveal competition or mismatch
- Wage knowledge shared through information networks
- Skill assessments refined through work output observation

---

### Information Sharing Mechanics (Rumor Mill)

**Trigger conditions for information sharing:**
- Successful trade completion between agents
- Negotiation interactions even without trade
- Employment relationships (regular information exchange)
- Social interactions (future: explicit gossip actions)

**Information sharing process:**

1. Sharer evaluates whether to share information:
   - Based on own reputation and optional honesty trait in Preferences
   - Roll for truthfulness: random value compared to probability threshold
   - High reputation sharer more likely to share truthful information

2. Sharer selects subset of first-hand ReputationKnowledge to share:
   - Only agents within sharer's Dunbar set (no propagating third-hand rumors)
   - Prioritize agents with strong reputation views (high interaction count)
   - May strategically withhold or lie about competitors

3. Emit InformationShareEvent with:
   - sharer AgentId
   - receiver AgentId
   - subject AgentId (who the information is about)
   - shared_view ReputationView (alpha/beta values)
   - is_truthful bool (result of truthfulness roll)
   - tick u64 (current simulation time)

4. Receiver processes information:
   - Retrieves own trust in sharer from first-hand ReputationKnowledge
   - If sharer unknown, uses TrustLevel baseline
   - Creates SecondHandView with informant=sharer, cached informant_trust
   - Stores in second_hand HashMap, bounded to max 5-10 views per subject
   - If capacity exceeded, evicts oldest or least-trusted SecondHandView

**Constraints and limitations:**
- Only share information about agents within sharer's Dunbar set
- Receivers store maximum 5-10 SecondHandView per subject agent
- Information decays faster than first-hand knowledge
- Cannot propagate third-hand information (no "A told me that B told them about C")

**Emergent dynamics from rumor mill:**
- Well-connected, trusted agents become valuable information sources and brokers
- Bad actors' influence naturally limited by trust-weighting in aggregation
- Isolated clusters may develop distorted local reputation views
- Strategic lying for competitive advantage possible but risky (reputation damage if discovered)
- Information cascades: widely-trusted source can rapidly shift collective opinion

**Defense mechanisms:**
- First-hand experience always overrides rumors in decision-making
- Trust-weighting in aggregation dampens unreliable sources
- Time decay ensures stale or false information eventually fades
- Dunbar limit prevents unbounded rumor accumulation
- Multiple conflicting rumors average out (wisdom of crowds effect)

---

### Decay and Forgetting Strategies

**Time-based decay mechanisms:**

*Reputation knowledge:*
- First-hand: slow exponential decay, half-life approximately 10,000 ticks
- Second-hand: fast exponential decay, half-life approximately 1,000 ticks
- Rationale: personal experiences fade slowly, hearsay fades quickly

*Price knowledge:*
- Slow linear or exponential decay for confidence
- Hard cutoff: prices older than T ticks (e.g., 5000) pruned entirely
- Running average remains for items with no recent data
- Rationale: markets relatively stable over short periods, but old data becomes unreliable

*Inventory observations:*
- Fast exponential decay for confidence
- Formula: confidence = initial * exp(-0.001 * delta_ticks) for rapid decay
- Observations older than threshold T ticks (e.g., 1000) pruned entirely
- Rationale: inventory changes rapidly through consumption and trading

*Demand profiles:*
- Pruned if no purchase in N ticks (typically 2000-5000)
- Frequency naturally decays if purchases become infrequent
- No explicit confidence decay; absence of purchases indicates pattern breakdown

**Interaction-based decay:**
- Active trading partners: all knowledge types refreshed on interaction
- No interaction in N ticks with agent: decay priority increases (faster pruning)
- Tier demotion: agents without recent interactions move from Tier 1 to Tier 2 to Tier 3
- Grace period: recently added agents not immediately pruned even without interactions

**Capacity-based pruning:**
- Triggered when memory limits approached or exceeded
- Eviction strategy: least-recently-used (LRU) within each tier
- Priority scoring: interaction_frequency * recency * importance_weight
- Never prune current employer, employees, or very recent interactions (Tier 1 protection)
- Prune aggressively from Tier 3, cautiously from Tier 2, rarely from Tier 1

**Confidence decay functions:**
- Exponential: confidence = initial * exp(-decay_rate * delta_ticks)
  - Appropriate for inventory, second-hand reputation
  - Captures rapid unreliability growth
- Linear: confidence = max(0, initial - decay_rate * delta_ticks)
  - Appropriate for price knowledge with stable markets
  - Simple computation
- Hybrid: initial exponential then linear after threshold
  - Balance realism and performance

---

## Decision-Making Architecture

### Trait-Based Pluggable Design

**DecisionMaker trait definition:**
- Primary method: decide(&self, agent: &Agent, world: &World, options: &[Action]) -> Action
- Allows swapping decision-making algorithms without changing simulation logic
- Initial implementation: UtilityMaximizationDecisionMaker
- Future implementations:
  - LearningDecisionMaker: reinforcement learning or evolutionary strategies
  - HeuristicDecisionMaker: fast approximate rules (satisficing)
  - PersonalityBasedDecisionMaker: different types (risk-averse, greedy, social, etc.)
  - BoundedRationalityDecisionMaker: imperfect optimization with cognitive limits

**Action types available to agents:**
- Trade: buy or sell specific item to/from specific partner at negotiated price
- Produce: execute production recipe consuming inputs to create outputs
- Work: accept employment offer from specific employer at wage rate
- Negotiate: make offer or counter-offer during bartering or price negotiation
- Rest: do nothing, conserve resources, wait for better opportunities
- Socialize (future): build relationships, share information explicitly
- Explore (future): seek new trading partners or opportunities

### Utility Maximization (Initial Implementation)

**Utility function per agent from Preferences component:**
- Linear: U = Σ(weight_i * needs_satisfaction_i)
  - Simple weighted sum of need satisfaction levels
  - weight_i derived from urgency (lower need value = higher weight)
- Exponential: U = Σ(exp(weight_i * needs_satisfaction_i))
  - Non-linear, emphasizes critical needs more
  - Captures risk aversion (sharply higher utility for satisfying urgent needs)
- Custom: pluggable formula via Preferences.utility_function Custom(formula_string)
  - Allows simulation designer to specify arbitrary utility functions
  - Formula parsed and evaluated (future: compiled for performance)

**Utility calculation procedure:**
1. For each available action, project resulting state:
   - New needs levels after consumption or time passage
   - New inventory contents after trade or production
   - New wallet balance after purchase or wage receipt
2. Compute utility of projected state using agent's utility function
3. Subtract costs:
   - Currency cost (price paid in trade)
   - Time cost (opportunity cost of action duration)
   - Effort cost (energy or resource expenditure)
   - Risk cost (uncertainty penalty based on risk_tolerance)
4. Select action with maximum net utility: argmax(utility - costs)
5. If all actions have negative net utility, choose Rest

**Influences on decision-making:**

*Current needs:*
- Low thirst drives high willingness to pay for water
- Low hunger drives high willingness to pay for food or accept food-producing work
- Satiated needs reduce urgency for related items

*Price knowledge:*
- Estimate fair price from PriceMemory avg_price
- Detect price gouging (offer >> avg_price) and adjust willingness
- Identify good deals (offer << avg_price) and prioritize

*Reputation:*
- Query ReputationKnowledge for trust score of potential partners
- Filter out agents below risk_tolerance threshold
- Prefer agents with good reputation for equal-utility trades
- Accept slightly worse deals from high-reputation partners (trust premium)

*Inventory observations:*
- Prioritize trading partners likely to have needed items (InventorySnapshot confidence)
- Avoid approaching agents unlikely to have needed items
- Factor in travel or search costs when low confidence

*Risk tolerance (from Preferences):*
- Low risk_tolerance: strongly prefer known, trusted partners and fair prices
- High risk_tolerance: willing to trade with unknown partners or accept uncertain deals for higher potential reward
- Affects utility calculation: uncertain_utility = expected_utility - risk_tolerance * variance

### Integration with Knowledge Systems

**Partner selection process:**
1. Identify action goal (e.g., buy water to satisfy thirst)
2. Query InventorySnapshot for agents likely to have water
3. For each candidate, query ReputationKnowledge for trust score
4. Filter out candidates with trust score below risk_tolerance threshold
5. Rank remaining candidates by:
   - Inventory confidence (likelihood of having item)
   - Reputation score (trustworthiness)
   - Price knowledge (expected fair price)
   - Relationship strength (interaction frequency, Dunbar tier)
6. Select top candidate or broadcast request if no good candidates

**Price estimation for negotiation:**
1. Query PriceMemory for item
2. Use avg_price as initial estimate
3. Adjust based on urgency:
   - High needs: willing to pay up to avg_price * (1 + urgency_factor)
   - Low needs: only pay up to avg_price * (1 - patience_factor)
4. Adjust based on reputation:
   - Low-trust partner: demand discount (avg_price * trust_score)
   - High-trust partner: accept slight premium for reliability
5. Adjust based on inventory:
   - Abundant item: offer lower price (supply assumption)
   - Scarce item: accept higher price (demand assumption)

**Inventory targeting:**
1. Query InventorySnapshot for agents with needed item
2. Sort by confidence * trust_score (reliable and likely to have it)
3. If no high-confidence matches, broadcast request or explore network
4. If still no success, adjust needs (substitute item) or wait

**Demand prediction for sellers:**
1. Query DemandProfile for regular customers
2. Identify items with high frequency * typical_quantity
3. Allocate production or procurement to stock those items
4. Prioritize customers with high lifetime_purchases (loyalty value)
5. Adjust pricing based on loyalty (discounts for regulars)

---

## Economic Mechanisms

### Trading System

**Dual trading modes coexist:**
- Barter: direct item-item exchange, negotiated via NegotiationSystem, no currency involved
- Currency: item-currency exchange, priced via price knowledge and negotiation, currency transferred via Wallet

**Trade execution atomicity and safety:**

1. MarketMatchingSystem identifies compatible buyer-seller pairs:
   - Buyer needs item, seller has item
   - Both agents evaluate potential trade utility
   - Reputation filtering applied

2. NegotiationSystem handles price or barter terms agreement:
   - Iterative offers and counter-offers
   - Reputation influences willingness to compromise
   - Outcome: accept, reject, or breakdown

3. TradeTransactionSystem executes atomic trade:
   - Validates inventory availability (seller has item)
   - Validates wallet balance (buyer has currency)
   - Transfers item from seller inventory to buyer inventory
   - Transfers currency from buyer wallet to seller wallet
   - All operations succeed together or all fail (atomicity)

4. Emit TransactionEvent with outcome:
   - Positive outcome: trade completed as agreed, both parties satisfied
   - Negative outcome: trade completed but issues (quality, quantity mismatch)
   - Neutral outcome: trade completed, neither particularly satisfied nor dissatisfied

5. Both parties update knowledge:
   - Price knowledge updated with actual trade price
   - Inventory observation updated (buyer saw seller's inventory)
   - Reputation updated based on outcome
   - Demand profile updated if seller tracking buyer

**Price discovery mechanism:**
- No centralized price authority or omniscient market prices
- Agents negotiate based on individual price knowledge and needs
- Successful trades update PriceMemory for both participants
- Price trends emerge from aggregate agent behavior across many trades
- Information asymmetry: agents may have different price expectations
- Arbitrage opportunities when price knowledge diverges

**Reputation influence on trading:**

*Low-reputation sellers:*
- May need to offer lower prices to attract buyers
- Buyers demand deposits or collateral for trust protection
- Matched less frequently by MarketMatchingSystem

*High-reputation sellers:*
- Can command premium prices for reliability
- Buyers accept deferred payment or advance payment
- Matched more frequently, building customer base

*Reputation updates:*
- Successful trade: small positive outcome
- Quality issues or delays: negative outcome
- Fraud or non-delivery: large negative outcome

### Labor and Employment System

**Employment types and structures:**

*Regular employment:*
- Ongoing employer-employee relationship
- Periodic wage payments per tick
- Updates Employment component (employer, job_status fields)
- Stable income for worker, reliable labor for employer

*Gig work:*
- Short-term contract for specific task
- Fixed payment on completion
- No ongoing relationship in Employment component
- Flexible for both parties, less commitment

*One-off jobs:*
- Single task, immediate payment
- No formal employment relationship
- Reputation still affected by performance

**Matching process via EmploymentSystem:**

1. Employers post job offers:
   - Required skills and minimum levels
   - Wage offer (currency per tick or lump sum)
   - Duration or task description

2. Workers evaluate offers:
   - Check skill compatibility
   - Compare wage to price knowledge (opportunity cost)
   - Evaluate employer reputation for reliability
   - Calculate utility of accepting vs other actions

3. EmploymentSystem matches compatible pairs:
   - Worker skills meet job requirements
   - Wage acceptable to worker based on decision-making
   - Employer trusts worker based on reputation

4. Update Employment components:
   - Worker: employer set, job_status updated
   - Employer: employees list updated

**Wage negotiation:**
- Initial offer based on skill level and market knowledge
- Worker can counter-offer based on own needs and alternatives
- Reputation influences:
  - High-reputation workers command higher wages
  - High-reputation employers attract workers at lower wages (trust and stability)
- Outcome: agreement at negotiated wage or breakdown

**Wage payment mechanics:**
- Periodic payments: WalletSystem transfers currency per tick from employer to employee
- On-completion payments: lump sum transfer when task verified complete
- Failure to pay:
  - Large negative reputation outcome for employer
  - Employee terminates relationship
  - Other workers avoid employer (information spreads via rumor mill)

**Skills influence:**
- Higher skill levels command higher wages (supply and demand)
- Production systems may require minimum skill levels for recipes
- Skills can improve over time (learning by doing, future feature)
- Skill specialization creates employment niches

### Production System

**Recipe-based transformation logic:**
- Input items: specific quantities consumed from inventory
- Skill requirements: minimum skill levels in relevant skills
- Tool requirements: specific items present (possibly consumed or degraded)
- Time cost: production takes multiple ticks
- Output items: specific quantities added to inventory

**Production planning by agents:**

Decision-making considers:
- Demand patterns: DemandProfile shows reliable customers for outputs
- Price knowledge: profitable outputs (output_price > input_cost)
- Input availability: InventorySnapshot of suppliers or own inventory
- Skill matching: agent has required skills at sufficient levels
- Opportunity cost: compare production utility to trade or work alternatives

**Production execution:**

1. Agent selects recipe based on decision-making
2. ProductionSystem validates:
   - All input items available in agent inventory
   - Agent skills meet requirements
   - Tools available (if required)
3. Consume inputs from inventory atomically
4. Degrade tools if applicable (reduce durability)
5. Wait for production time (ticks)
6. Add outputs to inventory
7. Update knowledge:
   - Input prices implicitly learned (cost basis)
   - Output target price based on input costs plus markup
   - Skills may improve slightly (learning by doing)

**Production chains and supply networks:**
- Graph structure via petgraph:
  - Nodes: agents with production capabilities
  - Edges: input-output relationships (recipe dependencies)
- Example chain: raw_materials -> intermediate_goods -> finished_products
- Agents specialize in different stages for efficiency
- Creates interdependencies and trade relationships
- Bottlenecks and monopolies can emerge naturally

**Supply chain planning:**
- Agents identify required inputs for production
- Query known_producers for suppliers
- Establish regular trading relationships for reliability
- Build inventory buffers for input stability
- Negotiate long-term supply agreements (future: Contract system)

### Negotiation System

**Bartering mechanism (item-item exchange):**

1. Agents propose direct exchanges: "I give you item1, you give me item2"
2. Both agents calculate utility of exchange:
   - Utility gain from receiving item vs utility loss from giving item
   - Net utility compared to current state
3. If both net utilities positive, accept
4. If one negative, counter-offer:
   - Adjust quantities: "I give 2 item1 for 1 item2"
   - Propose different items: "I give item3 instead"
5. Iterative offers and counter-offers until:
   - Agreement reached: execute barter trade
   - Breakdown: no mutually acceptable terms
6. Reputation updates based on outcome and behavior

**Price negotiation (item-currency exchange):**

1. Buyer offers initial price based on:
   - PriceMemory avg_price for item
   - Needs urgency (willingness to pay more)
   - Seller reputation (trust premium or discount)

2. Seller evaluates offer:
   - Compare to own price expectations
   - Consider inventory level (surplus vs scarcity)
   - Factor in buyer reputation and relationship

3. Seller responds:
   - Accept: execute trade at offered price
   - Counter-offer: propose different price
   - Reject: end negotiation

4. Iterative bargaining until agreement or breakdown
5. Both parties update price knowledge with final price
6. Reputation updated based on fairness and completion

**Reputation influence on negotiation:**

*High-reputation agents:*
- Their offers taken more seriously
- Greater willingness from partners to compromise
- Can request deferred payment or prepayment

*Low-reputation agents:*
- Offers viewed skeptically
- Partners demand favorable terms or collateral
- Must accept disadvantageous deals or face exclusion

*Negotiation behavior affects reputation:*
- Fair offers and honest dealing: small positive updates
- Extreme lowballing or reneging: negative updates

**Negotiation strategies (future enhancements):**
- Anchoring: first offer influences bargaining range
- Concession patterns: gradual vs abrupt price adjustments
- Deadline pressure: urgent needs force faster concessions
- Good cop/bad cop: multiple agents coordinate in negotiation
- Relationship building: accept worse deals to build trust

---

## Implementation Priorities and Roadmap Alignment

### Completed: Agent System

**Implemented components:**
- Agent entity with AgentId allocation via AgentIdAllocator
- Core components: Needs, Inventory, Wallet, Skills, Knowledge, Employment, Preferences
- Agent creation functions: create_agent (defaults), create_agent_with_needs, create_agent_with_wallet, create_agent_custom
- Agent removal: remove_agent with proper component cleanup and entity deletion
- ECS integration: registration with World, storage access patterns
- FFI exports: agent creation, removal, counting via cbindgen

**Comprehensive testing:**
- Unit tests in src/agent/creation.rs for creation and removal
- Unit tests in src/agent/components.rs for component behaviors
- Integration tests in tests/agent_core_phase1.rs for ECS registration and queries
- Integration tests in tests/agent_components_phase2.rs for component operations
- Integration tests in tests/agent_creation_phase3.rs for creation APIs
- Integration tests in tests/agent_additional_components_phase6.rs for Skills, Knowledge, Employment, Preferences
- FFI completeness tests in tests/ffi_completeness.rs
- Doc tests for all public APIs
- Example showcase in examples/basic_simulation.rs

### Next Priority: Learning System (Multi-Phase)

**Phase 1: First-hand reputation implementation**

Tasks:
- Implement ReputationKnowledge component with first_hand HashMap
- Implement ReputationView structure with alpha, beta, last_interaction_tick, count
- Implement ReputationUpdateSystem consuming TransactionEvent resource
- Beta distribution model: alpha += positive, beta += negative
- TrustLevel baseline for unknown agents
- Optional time-based decay on read
- Unit tests for reputation scoring, updates, decay
- Integration tests with mock trades

**Phase 2: Information sharing (rumor mill) implementation**

Tasks:
- Extend ReputationKnowledge with second_hand HashMap
- Implement SecondHandView structure
- Implement InformationSharingSystem triggered on successful interactions
- Truthfulness probability rolls based on sharer reputation
- Aggregation logic for querying unknown agents (weighted by trust)
- Dunbar limit enforcement and pruning strategies
- Unit tests for truthfulness distribution, aggregation, pruning
- Integration tests for information flow between agents

**Phase 3: Broader knowledge types implementation**

Tasks:
- Implement PriceMemory with RingBuffer, avg_price, trend detection
- Implement InventorySnapshot with confidence decay
- Implement DemandProfile for sellers tracking customers
- Extend Knowledge component with all knowledge types
- Implement full LearningSystem updating all knowledge after trades
- Time-based, interaction-based, capacity-based decay
- Memory bounds enforcement (Dunbar limit, per-type limits)
- Pruning algorithms (LRU, priority scoring)
- Unit tests for each knowledge type
- Integration tests for decay and pruning under load
- Property tests for memory bounds invariants

### Subsequent Priorities (from roadmap)

**Market System:**
- MarketMatchingSystem for buyer-seller pairing
- TradeTransactionSystem for atomic trade execution
- PriceDiscoverySystem for emergent pricing
- Integration with Learning and Reputation systems
 
#### Market Evolution & Optional Activation

The market layer is intentionally emergent and modular. Prices remain decentralized at the agent level; the "market" only lowers search and settlement costs when organic trade density demands it.

Phased progression (each phase feature-gated and metrics-triggered):
1. Stage 0 – Direct bilateral search & negotiation (baseline): agents scan local neighbors (spatial or network) and negotiate directly. No shared structures.
2. Stage 1 – Intent Registry: lightweight `OfferIndex` (bounded ring buffers per item) storing ephemeral bids/asks with expiry timestamps; simple matching scans index before neighbor search.
3. Stage 2 – Venue Activation: introduce `Venue` component when thresholds exceeded (trade density, avg search attempts). Venue groups related offers; still no central price. Graph edges (agent ↔ venue) tracked for network analysis.
4. Stage 3 – Order Book & Escrow: bounded per-item `OrderBook` plus optional `Escrow/SettlementSystem` for higher-risk trades (trust below threshold). Generates non-binding price indicators (median/EMA) feeding `PriceMemory`.
5. Stage 4 – Advanced Features: fees, latency modeling, specialized venue rules (minimum reputation, capacity caps), analytics hooks.

Activation Metrics (sample):
- `avg_search_attempts_per_trade`
- `failed_match_rate`
- `distinct_traders_per_item`
- `offer_churn_ratio` (posted vs fulfilled)
- `reputation_dispute_frequency`

Gating & Config:
`MarketConfig { intents_enabled, venues_enabled, order_book_enabled, escrow_enabled }` toggled dynamically when metrics cross configurable thresholds; disable if overhead > benefit (e.g., on constrained devices).

Performance Safeguards:
- Fixed-size ring buffers & bounded vectors (no unbounded growth)
- Time-based expiry + LRU pruning for stale offers
- Reuse allocation pools; avoid heap allocs in hot loops
- `f32` for metrics; integer ticks for timestamps
- Feature flags (`#[cfg(feature = "market_venues")]`) to strip advanced layers for embedded targets

Integration Notes:
- Matching leverages `ReputationKnowledge` & `PriceMemory` to prioritize trustworthy, fair-priced offers.
- Each successful trade emits `TransactionEvent` consumed by Reputation & Learning systems.
- Offer/venue relationships modeled as petgraph subgraphs (optional) enabling queries (e.g., centrality of a venue).
- No central price enforcement; `PriceDiscoverySystem` computes indicative stats only (median, EMA, volatility) that agents may optionally ingest.

Roadmap Impact: Market System tasks should be restructured into phased evolution (Stages 0–4 + gating & metrics) rather than a monolithic implementation.

**Labor System:**
- EmploymentSystem for job matching
- Wage negotiation logic
- Periodic wage payments
- Skills and employment component interactions

**Production System:**
- Recipe definitions and storage
- ProductionSystem for item transformations
- Skill requirement validation
- Production planning decision-making integration

**Decision System:**
- DecisionMaker trait definition
- UtilityMaximizationDecisionMaker initial implementation
- Action evaluation and selection
- Integration with all knowledge types

**Negotiation System:**
- Bartering logic for item-item exchanges
- Price negotiation with offers and counter-offers
- Reputation influence on negotiation outcomes
- Integration with Trade and Reputation systems

**Graph Structures:**
- petgraph integration for trade networks
- Production chain graphs
- Employment hierarchy graphs
- Efficient querying and pathfinding

**Simulation Loop:**
- System execution orchestration
- Event queue management (TransactionEvent, InformationShareEvent)
- Deterministic execution for reproducibility
- Performance profiling and optimization

**FFI Expansion:**
- Expose all systems to C API via cbindgen
- uniffi bindings for Python, Swift, Kotlin, Ruby
- FFI completeness testing for all new systems
- Cross-language integration examples

**Serialization:**
- Save entire simulation state via serde
- Load and restore state with validation
- Backward compatibility for version upgrades
- Incremental save/load for large simulations

**Testing:**
- Property-based tests for complex invariants
- Integration tests for multi-system interactions
- Benchmarks for performance-critical paths
- Fuzz testing for robustness

**Examples:**
- Market trading example
- Production chain example
- Employment and labor example
- Full simulation with all systems

**Documentation:**
- Complete API documentation for all public items
- Usage guides for each system
- FFI integration guides per language
- Tutorial examples and walkthroughs

---

## Testing Strategy

### TDD Mandate (Non-Negotiable)

All code development must follow strict test-driven development:

1. Write failing test defining expected behavior
2. Implement minimal code to make test pass
3. Refactor for clarity, performance, maintainability
4. Add property tests for complex logic and invariants
5. Add integration tests for system interactions
6. All tests must pass before any commit

### Test Categories and Requirements

**Unit tests (in src/ modules):**
- Component behavior: clamping, saturation, validation, boundary conditions
- System logic in isolation: pure functions, transformations, calculations
- Edge cases: zero values, negative inputs, overflow, underflow, NaN, infinity
- Decay and pruning algorithms: correctness, monotonicity, bounds
- Aggregation and weighting formulas: numerical stability, expected values
- Error handling: Result types return correct errors, no panics

**Integration tests (in tests/ directory):**
- System interactions: trades update knowledge and reputation correctly
- ECS world queries and joins: correct components retrieved
- Multi-agent scenarios: multiple agents interacting, emergent behavior
- Event propagation: events correctly consumed by multiple systems
- State consistency: no inconsistencies after complex operations
- Performance under load: stress testing with many agents

**Property tests (via proptest or quickcheck):**
- Memory bounds never exceeded: Dunbar limit, buffer sizes, storage capacity
- Decay is monotonic: confidence never increases without new data
- Aggregations numerically stable: no overflow, underflow, or NaN
- Reputation scores always in valid range: 0.0-1.0 inclusive
- Currency balances non-negative: wallet operations never produce negative
- Inventory quantities non-negative: no item duplication or loss

**FFI tests (in tests/ and separate language repos):**
- C API exports match Rust public API signatures
- Generated header completeness via ffi_completeness test
- Binding generation successful for all target languages (uniffi)
- Cross-language integration: Python import, Swift import, basic calls work
- FFI behavior matches Rust behavior: same inputs produce same outputs

**Benchmarks (in benches/ directory):**
- Critical paths profiled: trade execution, knowledge updates, reputation aggregation
- Memory usage under load: agent count scaling, knowledge accumulation
- Pruning performance: Dunbar limit enforcement, decay application
- System execution order: simulation tick overhead
- Comparison with baseline: regressions detected

### Test Simplicity Principle

Maintain simple, focused tests:
- One concept or behavior per test
- Descriptive test names: test_reputation_score_updates_on_positive_outcome
- Minimal setup and teardown: only create necessary entities
- Use pretty_assertions for clear failure messages and diffs
- Avoid complex mocking: use real ECS world when possible, simple stubs otherwise
- Clear arrange-act-assert structure in every test

### Coverage Expectations

- All public API functions have unit tests
- All public API functions have doc tests (compile-checked examples)
- All systems have integration tests
- All complex algorithms have property tests
- All FFI exports have FFI tests
- Critical paths have benchmarks

---

## Performance Optimization Guidelines

### Measurement Before Optimization (Critical)

Never optimize without measuring:
- Profile on target platform: Raspberry Pi Zero or equivalent emulator
- Identify hot loops: perf, flamegraph, or criterion for Rust benchmarks
- Measure baseline: establish performance metrics before changes
- Benchmark after optimization: verify improvement, check for regressions
- Document performance characteristics: expected tick rate, memory usage

### Optimization Strategies by Category

**Memory optimization:**
- Reuse buffers and collections: pre-allocate and clear instead of allocating
- Use fixed-size ring buffers: bounded history with constant memory
- Prefer Vec over HashMap: cache-friendly for small collections (< 10 items)
- Prune aggressively: enforce memory bounds strictly (Dunbar limit)
- Lazy computation: compute on read, cache only if frequently accessed
- Avoid clone: use references (&) or swap when possible
- Compact representations: use u32 instead of u64 where range sufficient

**CPU optimization:**
- Minimize branching in hot loops: branchless algorithms where possible
- Use f32 over f64: faster on many platforms, sufficient precision for economics
- Batch operations: process multiple events together to amortize overhead
- Avoid expensive operations in ECS joins: move complex logic to separate passes
- Cache-friendly data access: iterate contiguous data (Vec) over scattered (HashMap)
- Inline hot functions: use inline attribute for small frequently-called functions
- SIMD opportunities (future): parallel operations on Vec with explicit vectorization

**Algorithmic improvements:**
- Beta distribution model: simple arithmetic (alpha += w, score = a/(a+b))
- Exponential decay approximation: table lookup for exp(-λ*t) if frequent
- LRU eviction: timestamp comparison without full sorting
- Aggregation with running totals: avoid recomputing sums
- Early termination: break loops when result determined
- Incremental updates: update running averages instead of recomputing

**Feature gating for optional performance:**
- Parallel system execution behind "parallel" feature flag using rayon
- Disabled by default for single-core Pi Zero
- Enabled for multi-core desktop/server deployments
- Thread pool management: reuse threads, avoid spawn overhead

---

## Open Questions and Future Research Areas

### Agent Behavior Evolution

Topics for exploration:
- Learning by doing: skills improve with repeated use, diminishing returns curve
- Personality traits: risk-averse vs risk-seeking, honest vs deceptive, social vs isolated
- Social preferences: altruism, reciprocity, spite, fairness concerns
- Bounded rationality: satisficing (good enough) instead of optimizing, computational limits
- Habits and routines: repeated actions become default, inertia
- Adaptation: agents adjust strategies based on success/failure

### Market Dynamics

Advanced market phenomena:
- Spatial markets: agents in different locations, transportation costs and time
- Market failures: monopolies, monopsonies, information asymmetries, externalities
- Bubbles and crashes: speculative behavior, herd dynamics, panic selling
- Regulation and intervention: taxes, subsidies, price controls, rationing
- Market segmentation: luxury vs necessity goods, different price elasticities
- Auction mechanisms: different trading protocols (continuous double auction, sealed bid)

### Advanced Production

Production system enhancements:
- Capital goods: tools and machines that enhance production efficiency or enable new recipes
- Durability and maintenance: items degrade with use, require repair or replacement
- Innovation: new recipes discovered through experimentation or research
- Economies of scale: production efficiency increases with volume
- Joint production: single recipe produces multiple outputs
- Byproducts and waste: production generates unwanted items requiring disposal

### Social Structures

Organizational and institutional features:
- Firms and organizations: agents form collectives for production or trade
- Hierarchies: leadership roles, delegation, command structures
- Social norms: conventions that emerge from repeated interactions (fairness, honesty)
- Institutions: rules and enforcement mechanisms (property rights, contracts)
- Cultural transmission: knowledge and norms spread through social networks
- Group identity: in-group favoritism, out-group discrimination

### Long-term Dynamics

Macro-level simulation features:
- Population dynamics: agent birth/death, replacement, generations
- Technological progress: new goods, better production methods, innovation diffusion
- Environmental constraints: resource depletion, pollution, carrying capacity
- Path dependence: early random events have lasting effects on system trajectory
- Economic growth: increasing production and consumption over time
- Inequality and distribution: wealth concentration vs dispersion

### Methodological Improvements

Simulation and analysis tools:
- Sensitivity analysis: vary parameters, measure outcome robustness
- Calibration: fit simulation to real-world data or stylized facts
- Validation: compare emergent phenomena to economic theory predictions
- Visualization: real-time graphs, network diagrams, agent traces
- Experiment design: controlled interventions to test hypotheses
- Statistical analysis: aggregate statistics, distributions, correlations

---

## Document Maintenance

**Update triggers:**
- System implementation completion: mark as implemented, document actual behavior
- Design changes: update affected sections with rationale
- Research findings: add new insights or references
- Performance discoveries: document benchmarks and optimization results
- Community feedback: incorporate suggestions and corrections

**Version history:**
- December 1, 2025: Initial comprehensive consolidation from multiple research documents
- Agent System marked complete with full test coverage
- Learning System phases 1-3 detailed design complete
- All future systems in research/planning phase

**Document status:** Living document, updated as implementation proceeds and research evolves. Primary reference for AI agents implementing libreconomy systems. Human-friendly formatting not prioritized; focus on precision, completeness, and internal consistency.
