# Reputation System Research & Design

## Session Date: December 1, 2025

## Goal
Agents should build first-hand reputations of other agents from their own interactions. Each agent has a baseline trust level for unknown counterparts, which is updated based on transaction outcomes (positive/negative/neutral). Reputation is per-agent (no global score) and influences decision-making.

---

## Concepts
- First-hand only: No global reputation; each agent keeps its own view.
- Baseline trust: `TrustLevel` per agent, expressed as [0.0 – 1.0].
- Reputation view: Per-counterparty score [0.0 – 1.0] derived from interaction outcomes.
- Event-driven updates: Transactions emit outcomes that update reputation.
- Optional decay: Reputation can lazily decay over time or by interaction distance.

---

## Data Structures (Rust-friendly)

```rust
// Newtypes for type safety
struct TrustLevel(pub f32);         // 0.0..=1.0
struct ReputationScore(pub f32);    // 0.0..=1.0
struct InteractionCount(pub u32);

// Per-agent view of a specific counterparty
struct ReputationView {
    alpha: f32,                 // positive evidence
    beta: f32,                  // negative evidence
    last_interaction_tick: u64, // for optional decay
    count: InteractionCount,    // number of interactions
}
impl ReputationView {
    fn score(&self) -> ReputationScore {
        ReputationScore(self.alpha / (self.alpha + self.beta))
    }
}

// Agent-local reputation knowledge
struct ReputationKnowledge {
    by_agent: std::collections::HashMap<AgentId, ReputationView>,
    trust: TrustLevel, // baseline for unknown counterparts
}

// Transaction outcome event
enum Outcome {
    Positive(f32), // weight 0.0..=1.0
    Negative(f32),
    Neutral,
}
struct TransactionEvent { from: AgentId, to: AgentId, outcome: Outcome, tick: u64 }
```

Notes:
- Lightweight Beta model: initialize `(alpha, beta)` using TrustLevel as prior.
  - Example prior: `alpha0 = 1.0 + trust*k`, `beta0 = 1.0 + (1.0 - trust)*k`, with small `k` (e.g., 2.0).
- Updates:
  - Positive(w): `alpha += w`
  - Negative(w): `beta += w`
  - Neutral: no change (or very small changes if desired)
- Decay (optional): apply on read via factor based on `tick` difference; keep logic simple and cheap.

---

## Systems
- ReputationUpdateSystem (ECS System):
  - Input: a buffer/resource of `TransactionEvent` records.
  - Operation: for each event, update `ReputationKnowledge` of both participants.
  - Constraints:
    - Use `f32` for performance (Pi Zero friendly).
    - Validate input ranges; avoid panics; no allocations in hot loops (reuse buffers).

- Decision Integration:
  - DecisionMaker consults `ReputationKnowledge`:
    - Unknown partner: use `TrustLevel`.
    - Known partner: use `ReputationView::score()`.
  - Combine with `Preferences.risk_tolerance` to filter partners or adjust prices.

---

## Roadmap Placement
- Learning System:
  - Define `ReputationKnowledge`, `ReputationView`, and the update logic.
  - Consume `TransactionEvent` to learn from outcomes.
- Negotiation/Market Systems:
  - Use reputation to influence partner selection, price adjustments, deposits, or required collateral.
- Graph Structures (later):
  - Optional edges `AgentId -> AgentId` with reputation weights for queries.

---

## TDD Plan (Keep tests simple)
- Unit Tests:
  - TrustLevel prior yields expected initial score.
  - Sequence of outcomes updates `alpha/beta` and computed score deterministically.
  - Unknown partner returns baseline score derived from `TrustLevel`.
  - Decay factor reduces score appropriately on read (if enabled).
- ECS/System Test:
  - Inject a few `TransactionEvent` entries; run `ReputationUpdateSystem`; verify updates in `ReputationKnowledge` storages for both participants.
- FFI (later):
  - `get_reputation(a, b) -> f32` returns expected values; keep FFI thin.

---

## Design Principles Alignment
- Data-driven: stored in components/resources; no hardcoded behavior.
- ECS separation: `ReputationKnowledge` as data, `ReputationUpdateSystem` as logic.
- Performance: uses `f32`, simple updates, optional lazy decay, no panics.
- Extensibility: weights, decay, and priors can be tuned per sim or agent.

```text
Status: Research only (no implementation). Schedule for Learning/Negotiation phases.
```

---

## Phase 2: Information Sharing (Rumor Mill)

### Goal
Agents share reputation information about third parties during interactions. Information reliability depends on the sharer's reputation. This creates emergent social networks and realistic information asymmetry.

### Concepts
- **Second-hand reputation:** Agents exchange information about others they haven't directly interacted with
- **Trust-weighted information:** Receivers weight shared info by their trust in the informant
- **Truthfulness probability:** High-reputation agents share truthful info ~95%+ of the time; low-reputation agents 50-70%
- **Dunbar's number constraint:** ~150 working relationships; beyond that, decay/pruning occurs
- **Information asymmetry:** Agents outside Dunbar limit are only known through rumors

### Truthfulness Mechanics
- **High reputation (0.8-1.0):** 95-99% truthful
- **Medium reputation (0.4-0.8):** 70-90% truthful
- **Low reputation (0.0-0.4):** 50-70% truthful
- **Optional trait:** `Preferences.honesty` can modulate base probability
- **Strategic lying:** Even trusted agents *can* lie (rare, but creates interesting dynamics)

### Data Structures

```rust
// Second-hand reputation view
struct SecondHandView {
    informant: AgentId,        // who told me this
    view: ReputationView,      // what they said (alpha/beta)
    received_tick: u64,        // when I heard it
    informant_trust: f32,      // my trust in the informant at time of sharing
}

// Extended reputation knowledge with rumor tracking
struct ReputationKnowledge {
    // First-hand (authoritative)
    first_hand: HashMap<AgentId, ReputationView>,
    
    // Second-hand (rumors)
    second_hand: HashMap<AgentId, Vec<SecondHandView>>,
    
    // Baseline trust for strangers
    trust: TrustLevel,
    
    // Dunbar limit tracking
    total_relationships: usize,
    last_pruned_tick: u64,
}

// Information sharing event
struct InformationShareEvent {
    sharer: AgentId,
    receiver: AgentId,
    subject: AgentId,      // who the info is about
    shared_view: ReputationView,
    is_truthful: bool,     // determined by sharer's honesty roll
    tick: u64,
}
```

### Aggregation Logic
When querying reputation of an unknown agent:
1. Check first-hand knowledge (if exists, use it—most reliable)
2. If no first-hand data, aggregate second-hand views:
   - Weight each view by `informant_trust`
   - Apply time decay (older rumors matter less)
   - Compute weighted average of `alpha/(alpha+beta)`
3. If no data at all, fall back to baseline `TrustLevel`

Example formula:
```
weighted_score = Σ(view.score() * informant_trust * time_decay) / Σ(informant_trust * time_decay)
```

### Memory Constraints (Dunbar's Number)

**Working memory limits:**
- **First-hand relationships:** ~150 max (Dunbar limit)
- **Second-hand views per agent:** ~5-10 max (most recent/trusted informants)
- **Total agents tracked:** ~150-200 (first-hand + important second-hand)

**Pruning strategy:**
- **Tier 1 (never prune):** Recent interactions, high transaction volume, current employer/employees
- **Tier 2 (prune when capacity reached):** Older relationships, low interaction frequency
- **Tier 3 (aggressive prune):** Second-hand info older than N ticks, informants with low trust

**Decay rates:**
- **First-hand:** Slow decay (you remember your own experiences)
- **Second-hand:** Fast decay (hearsay gets fuzzy quickly)
- **Example:** First-hand half-life = 10,000 ticks; second-hand half-life = 1,000 ticks

### System Interactions

**InformationSharingSystem (ECS System):**
- **Trigger:** During successful interactions (trades, negotiations, employment)
- **Process:**
  1. Sharer rolls for truthfulness based on their reputation
  2. Selects a subset of their first-hand knowledge to share
  3. Emits `InformationShareEvent`
  4. Receiver updates `second_hand` in their `ReputationKnowledge`
- **Constraints:** Only share info about agents within sharer's Dunbar set

**Integration with other systems:**
- **Learning System:** Handles second-hand reputation updates
- **Negotiation System:** Agents share info as part of relationship-building
- **Decision System:** Uses aggregated (first + weighted second-hand) reputation for partner selection

### Emergent Properties

**Positive dynamics:**
- **Social network clustering:** Agents cluster around trusted information hubs
- **Reputation cascades:** Bad actors get collectively blacklisted if trusted agents spread warnings
- **Information brokers:** Well-connected agents with high trust become valuable for introductions

**Negative dynamics:**
- **Misinformation spread:** Low-trust agents can poison the information pool (but influence is limited)
- **Echo chambers:** Isolated clusters may develop distorted views
- **Strategic manipulation:** Trusted agents can selectively lie about competitors for advantage

**Defense mechanisms:**
- Trust-weighting naturally dampens bad actors' influence
- First-hand experience always overrides rumors
- Decay ensures old/false information fades

### TDD Strategy (for later implementation)

**Unit tests:**
- Truthfulness probability matches reputation distribution
- Second-hand views are weighted correctly by informant trust
- Aggregation produces expected scores for various scenarios
- Dunbar pruning removes least-important entries
- Time decay reduces second-hand confidence appropriately

**Integration tests:**
- Information sharing during trades updates both parties
- High-trust agent's rumors heavily influence receiver
- Low-trust agent's rumors are discounted
- First-hand experience overrides conflicting rumors
- Pruning maintains memory bounds under load

**Edge cases:**
- Conflicting rumors from multiple sources
- Circular information loops (A→B→C→A)
- Reputation changes after information was shared
- Dunbar limit reached during active information sharing

---

## Phase 3: Broader Knowledge Types (Price, Inventory, Demand)

### Goal
Agents maintain bounded, practical knowledge about prices, inventory availability, and demand patterns. Memory constraints force strategic choices about what to remember, creating realistic information asymmetry.

### Types of Knowledge

#### 1. Price Knowledge
**What agents know:**
- Recent prices for goods/services from specific trading partners
- Average/typical prices for frequently traded items
- Price trends (rising/falling) based on recent observations

**Memory constraints:**
- Max N recent prices per item (e.g., 10 most recent)
- Only track prices for items they care about (based on Needs, production inputs, frequent trades)
- Older prices decay or get pruned
- Aggregate to running averages for commonly traded items

**Use cases:**
- Estimate fair prices when negotiating
- Detect price gouging or suspiciously low offers
- Identify profitable arbitrage opportunities

#### 2. Inventory Knowledge
**What agents know:**
- What items other agents have (or had recently)
- Approximate quantities or categorical abundance (low/medium/high)
- Last observation timestamp

**Memory constraints:**
- Only remember inventory from recent interactions
- Bounded by Dunbar limit (can't track everyone)
- Confidence decays over time (inventory changes)
- Track "typical inventory" for frequent trading partners

**Use cases:**
- "I need item1, I remember agent2 usually has it"
- Avoid wasting time approaching agents unlikely to have needed items
- Plan production based on known supplier availability

#### 3. Demand Pattern Knowledge (for sellers)
**What agents know:**
- What items specific agents frequently buy from them
- Purchase frequency and typical quantities
- Typical price points for regular customers

**Memory constraints:**
- Only track top N regular customers (e.g., 10-20)
- Aggregate purchase data rather than storing every transaction
- Prune customers who stop buying

**Use cases:**
- "Agent1 often buys item1 from me at price X, so I should keep it in stock"
- Offer better prices to loyal customers
- Predict demand and plan production/procurement

#### 4. Production Knowledge
**What agents know:**
- Who can produce specific items
- What recipes/skills other agents have
- Input/output relationships in production chains

**Memory constraints:**
- Track known producers for items of interest
- Learn through observation or information sharing
- Decay if producers stop making those items

#### 5. Employment/Service Knowledge
**What agents know:**
- Who offers what services
- Wage rates or service fees
- Availability and reliability

### Hierarchical Importance (Memory Tiers)

**Tier 1 (always remembered):**
- Direct relationships within Dunbar limit
- Frequent trading partners
- Current employer/employees
- Recent price observations

**Tier 2 (remembered with decay):**
- Occasional trading partners
- Older price data
- Inventory observations
- Customer demand patterns

**Tier 3 (forgotten quickly):**
- One-off interactions
- Agents outside active network
- Very old prices or observations

### Data Structures (Conceptual)

```rust
// Price memory for an item
struct PriceMemory {
    recent_prices: RingBuffer<(AgentId, f32, tick)>,  // fixed size buffer
    avg_price: f32,                                    // running average
    trend: PriceTrend,                                 // Rising/Falling/Stable
}

enum PriceTrend { Rising, Falling, Stable }

// Inventory observation
struct InventorySnapshot {
    items: HashMap<ItemId, ObservedQuantity>,
    last_seen_tick: u64,
    confidence: f32,  // decays over time
}

enum ObservedQuantity {
    Exact(u32),           // just traded, certain
    Approximate(u32),     // observed but not exact
    Categorical(Level),   // "they have lots"
}

enum Level { None, Low, Medium, High, Abundant }

// Customer demand profile (for sellers)
struct DemandProfile {
    customer: AgentId,
    item: ItemId,
    frequency: f32,              // purchases per tick (smoothed)
    typical_quantity: u32,
    typical_price: f32,
    last_purchase_tick: u64,
    lifetime_purchases: u32,
}

// Extended Knowledge component
struct Knowledge {
    // Price memory (per item, aggregated across agents)
    known_prices: HashMap<ItemId, PriceMemory>,
    
    // Inventory observations (per agent, bounded by Dunbar)
    agent_inventory: HashMap<AgentId, InventorySnapshot>,
    
    // Demand patterns (for sellers tracking customers)
    customer_demand: HashMap<(AgentId, ItemId), DemandProfile>,
    
    // Production capabilities (who can make what)
    known_producers: HashMap<ItemId, Vec<AgentId>>,
    
    // Trade partners (sorted by interaction frequency)
    trade_partners: Vec<AgentId>,
    
    // Memory bounds tracking
    total_tracked_agents: usize,
    last_pruned_tick: u64,
}
```

### Memory Bounds (Example Configuration)

- **Max relationships tracked:** 150 (Dunbar)
- **Max price observations per item:** 10 recent
- **Max inventory snapshots:** 50 agents (active trading subset)
- **Max customer demand profiles:** 20 (regular customers only)
- **Max known producers per item:** 10 agents

### Decay and Forgetting Strategies

**Time-based decay:**
- Prices older than T ticks → pruned
- Inventory observations → confidence decays exponentially
- Demand profiles → if no purchase in N ticks, prune

**Interaction-based:**
- No interaction in N ticks with an agent → decay their data priority
- Active trading partners → refresh all knowledge

**Capacity-based:**
- When hitting memory limits → evict least-recently-used
- Prioritize retention based on interaction frequency and recency

**Confidence decay:**
- Price confidence: recent = 1.0, decays slowly (stable market assumption)
- Inventory confidence: decays quickly (inventory changes fast)
- Demand confidence: decays if customer behavior changes

### System Interactions

**LearningSystem (ECS System):**
- **After trades:** Update price knowledge, inventory observations, demand patterns
- **Periodic tick:** Apply decay, prune old data
- **Query interface:** Provide knowledge for decision-making

**DecisionSystem uses knowledge to:**
- Find trading partners likely to have needed items
- Estimate fair prices based on price memory
- Decide what to stock based on customer demand patterns
- Route around agents with poor inventory history

**MarketSystem integration:**
- Successful trades → update price and inventory knowledge
- Failed trades (no inventory) → update inventory knowledge negatively
- Observations during matching → passive learning

**NegotiationSystem integration:**
- Agents reference price knowledge when making offers
- Counter-offers update price expectations
- Reputation influences how much to trust price signals

### TDD Strategy (for later implementation)

**Unit tests:**
- Price memory bounded correctly (max N entries)
- Decay reduces confidence appropriately
- Inventory snapshots update and age correctly
- Demand profiles track frequent customers only
- Dunbar pruning removes least-important data
- Knowledge queries return expected results

**Integration tests:**
- Trade updates all relevant knowledge types
- Multiple trades with same partner aggregate correctly
- Pruning maintains memory bounds under load
- Knowledge influences decision-making correctly
- Price trends detected from observations

**Property tests:**
- Memory never exceeds configured bounds
- Decay is monotonic (confidence never increases without new data)
- Aggregations are numerically stable

### Emergent Properties

**Information economy:**
- Knowledge is local, imperfect, and costly to maintain
- Agents must actively trade to keep information fresh
- Information asymmetry creates arbitrage opportunities
- Well-connected agents have better market information

**Realistic constraints:**
- Can't know everything about everyone
- Must choose what to remember
- Stale information leads to bad decisions
- Active relationships are more valuable

**Strategic depth:**
- Specialists track deep knowledge in narrow domains
- Generalists track shallow knowledge broadly
- Information sharing becomes valuable (rumor mill synergy)
- Relationships have memory/knowledge value beyond immediate transactions

---

```text
Status: Research and design complete. Implementation scheduled for Learning/Negotiation phases.
Next: Create detailed implementation plan with TDD milestones.
```