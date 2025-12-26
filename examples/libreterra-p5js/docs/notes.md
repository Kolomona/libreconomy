# libreterra Design Notes

## Architecture Overview

**libreconomy** = The "brain" (decision-making, AI, learning)
**libreterra** = The "body" (execution, physics, rendering, state management)

These are separate ECS systems that communicate via a WASM bridge.

---

## libreconomy Responsibilities (Decision-Making)

### Pain Response & Decision Priority
- Pain thresholds for each need (hunger, thirst, tiredness)
- Pain weighting system: **thirst > hunger > tiredness**
  - Rationale: Animals can survive longer without food than water
- Decision algorithm: "Which need causes the most pain? Address that first"
- Strategic planning: When to sleep proactively vs. push through tiredness
- Movement urgency: Should I walk, run, or sprint to address this need?

### What libreconomy Returns
High-level intents with urgency levels:
- **Intent type**: `SEEK_WATER`, `SEEK_FOOD`, `REST`, `WANDER`
- **Urgency value**: 0-100 representing how critical this need is
  - Based on pain level of the need
  - Higher urgency = more critical
  - Used by libreterra to determine if interruption is warranted
- **Target location**: Where to go to satisfy this need (if applicable)
- **Reason**: Debug information explaining why this decision was made

---

## libreterra Responsibilities (Execution)

### Time System
- **Time multiplier variable**: 1 real-world second = 30 simulation minutes
- All duration calculations respect this multiplier
- Realistic death timers:
  - 3 days without water before death from dehydration
  - 30 days without food before death from starvation
  - Different species have different tolerances

### Movement Mechanics
- **Speed tiers**: walking, running, sprinting
  - Each tier has different base speeds
  - Energy cost multipliers per tier
  - Tiredness accumulation rates per tier
- Apply speed based on intent urgency from libreconomy
- Terrain-based speed modifications (swimming, rough terrain, etc.)

### Swimming
Per-species attributes:
- `canSwim`: boolean - can this species survive in water?
- `swimSpeedMultiplier`: float
  - Human: 0.25x (slow swimmer)
  - Frog: 2.0x (fast swimmer)
- `swimEnergyCost`: float multiplier
  - Human: 3.0x (swimming is exhausting)
  - Frog: 0.5x (swimming is efficient)
- **Drowning**: If tiredness > 95% while in water → death
- Terrain checking: Detect when entity is in water vs. on land

### Drinking & Eating
- **Satisfaction threshold: 5-10** (small buffer)
  - Entities consume until need is mostly satisfied
  - Prevents complete exhaustion but allows flexibility
  - Too high (20+) causes thrashing: both needs stay elevated, constant switching
  - Too low (0) works but makes entities vulnerable during long consumption
- **Continuous decision evaluation**:
  - libreconomy.decide() called every frame, even while consuming
  - Priority threshold system determines if current activity should be interrupted
  - Only switch activities if new need's urgency exceeds current by threshold
- **Consumption rates** (how much need decreases per frame):
  - Drinking: reduces thirst by 2.0/frame
  - Eating grass: reduces hunger by 1.5/frame (rabbits)
  - Eating rabbits: reduces hunger by 3.0/frame (humans, more efficient)

### Sleeping & Tiredness
Per-species attributes:
- `sleepRestoreRate`: % of tiredness reduced per tick
  - Some species need more sleep than others
- **Voluntary sleep** (entity chose to sleep):
  - Sleep until tiredness <= satisfaction threshold (5-10)
  - Mostly rested, leaves small buffer
- **Exhaustion collapse** (tiredness reached 100):
  - Sleep until tiredness = 40 (penalty for poor management)
  - Entity is forced into SLEEPING state
  - Overrides all other needs (survival mechanism)
- **During sleep**:
  - Hunger and thirst accumulate at 10% of normal rate
  - Tiredness decreases by sleepRestoreRate
- **Sleep interruption**:
  - Can be interrupted by priority threshold system
  - Critical hunger/thirst can wake entity if urgency exceeds sleep urgency by threshold
  - (Future: predator attacks always interrupt sleep)

### Grass Lifecycle
- Grass depletion from rabbit consumption
- **Regeneration**: Dirt → Grass after time period
- Regrowth rates configurable per biome
- Prevents overgrazing from depleting all food sources

### Water Properties
- Water is swimmable (not solid terrain)
- Entities can drink from water shoreline
- Swimming in water has different physics (see Swimming section)

---

## State Management (libreterra)

### Entity States
- `IDLE` - No current activity, ready for new decision
- `MOVING` - Traveling to target location
- `DRINKING` - Actively consuming water
- `EATING` - Actively consuming food
- `SLEEPING` - Resting to reduce tiredness

### State Transition Rules
1. **Every frame** → Evaluate current needs via libreconomy → Determine best intent
2. **Priority check** → Should new intent interrupt current activity? (urgency threshold)
3. **IDLE** → Apply new intent → Transition to MOVING/DRINKING/EATING/SLEEPING
4. **MOVING** → When reaching target → Transition to consumption state (DRINKING/EATING)
5. **DRINKING/EATING** → When need <= satisfaction threshold (5-10) → Transition to IDLE
6. **SLEEPING** → When tiredness <= satisfaction threshold → Transition to IDLE
7. **Any state** → If tiredness >= 100 → Force SLEEPING (exhaustion collapse, overrides everything)

### Decision Calling Logic (Continuous Evaluation)

**Every Frame:**
```javascript
// 1. Always get current best decision from libreconomy
const newIntent = libreconomy.decide(entityId);

// 2. Check if we should switch activities
if (shouldInterruptActivity(currentIntent, newIntent, currentState)) {
    // New activity is more urgent - interrupt current task
    currentIntent = newIntent;
    applyIntent(newIntent);
} else {
    // Current activity still makes sense - continue
    continueCurrentActivity();
}
```

**Priority Threshold Interruption Logic:**
```javascript
function shouldInterruptActivity(currentIntent, newIntent, currentState) {
    // Always allow new decisions when IDLE
    if (currentState === IDLE) return true;

    // Force sleep always interrupts (exhaustion collapse)
    if (newIntent.type === REST && tiredness >= 100) return true;

    // Compare urgency: only interrupt if new intent is significantly more urgent
    const urgencyDifference = newIntent.urgency - currentIntent.urgency;
    const INTERRUPT_THRESHOLD = 20; // Must be 20+ points more urgent

    return urgencyDifference >= INTERRUPT_THRESHOLD;
}
```

**Why This Works:**
- Entities are always aware of their surroundings and needs
- Prevents thrashing: requires significant urgency difference to switch
- Realistic: like a person who finishes their meal even when starting to feel tired
- Flexible: critical needs (future: predators) can interrupt any activity

---

## Current Issues to Fix

### Thrashing Behavior
**Problem**: Entities drink a little water, then seek food, eat a little, then seek water again. Endless cycle prevents meaningful progress.

**Root Cause**:
1. **Satisfaction threshold = 20** (too high)
   - Entities stop drinking when thirst = 20
   - But thirst = 20 is still "thirsty" → next decision picks drinking again
   - Same with hunger: stops at 20, but hunger = 20 triggers eating again
   - Both needs hover around 15-25, causing constant switching

2. **No interruption threshold** (current system)
   - Decisions made every 30 frames (~1 second)
   - When decision is made, immediately switches to highest priority
   - No concept of "this is urgent enough to interrupt my current task"

**Solution**:
1. **Lower satisfaction threshold: 20 → 5-10**
   - Drink until thirst = 5 (mostly satisfied)
   - Eat until hunger = 5
   - Leaves small buffer for safety, but low enough that other needs become clearly more urgent

2. **Implement continuous evaluation (every frame)**
   - Always call `libreconomy.decide()` to get current best intent
   - But don't immediately apply it

3. **Add priority threshold interruption system**
   - Only switch activities if `newIntent.urgency - currentIntent.urgency >= THRESHOLD`
   - Example: Drinking (thirst pain = 60) won't interrupt eating unless thirst rises to 80+
   - Prevents thrashing while allowing urgent needs to override

4. **Track current intent urgency**
   - Store urgency value when intent is applied
   - Use this as baseline for interruption comparison

**Expected Behavior After Fix**:
- Rabbit starts drinking (thirst = 70, hunger = 40)
- Drinks until thirst = 5
- Evaluates: hunger = 45, thirst = 5 → hunger much higher
- Switches to eating (no thrashing, clear priority difference)
- Eats until hunger = 5
- Evaluates: tiredness = 60, hunger = 5, thirst = 8 → sleep
- Natural cycle without constant back-and-forth

---

## Design Principles

### libreconomy (Brain)
- **What** to do: Which need to address
- **Why**: Pain-based decision making
- **When**: Proactive vs reactive behavior
- Returns: High-level strategic intent

### libreterra (Body)
- **How** to do it: Movement speed, consumption rates
- **Where**: Pathfinding, terrain interaction
- **Until when**: Satisfaction thresholds, state management
- Executes: Low-level mechanical simulation

### Communication Flow (Every Frame)
- libreterra asks libreconomy: "Given my current needs, what should I be doing?"
- libreconomy responds: "You're thirsty (pain=80), SEEK_WATER with urgency=80"
- libreterra checks: "Am I already drinking? Yes, at urgency=80. New urgency=80. Continue drinking."
- libreterra continues: Drinking, thirst decreases 80→78→76...
- Later: libreconomy responds: "Hunger is now critical (pain=65), SEEK_FOOD with urgency=65"
- libreterra checks: "Currently drinking (urgency=60). New urgency=65. Difference=5 < 20 threshold. Keep drinking."
- Even later: "Hunger critical! (pain=85), SEEK_FOOD with urgency=85"
- libreterra checks: "Currently drinking (urgency=40). New urgency=85. Difference=45 > 20 threshold. INTERRUPT! Switch to eating."
- libreterra executes: Stops drinking, moves to food, starts eating

---

## Future Enhancements

### Predation & Hunting
- Predator detection and fleeing (libreconomy decides to flee)
- Hunting behavior (libreconomy decides to hunt when hungry)
- Stealth and awareness systems

### Social Behavior
- Herding/flocking (rabbits group together)
- Territorial behavior
- Mating and reproduction

### Learning & Memory
- Remember good food/water locations
- Learn from dangerous situations
- Adapt behavior based on experience

### Environmental Factors
- Weather effects on energy/tiredness
- Seasonal changes affecting food availability
- Day/night cycle affecting visibility and behavior
