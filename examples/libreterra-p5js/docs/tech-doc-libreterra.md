# Libreterra Technical Documentation

**Version:** 1.0
**Last Updated:** 2025-12-28
**Purpose:** Comprehensive technical reference for the libreterra simulation engine

---

## Table of Contents

### Part I: Overview & Architecture
1. [Introduction](#1-introduction)
2. [Architecture Diagram](#2-architecture-diagram)
3. [Core Concepts](#3-core-concepts)

### Part II: Data Flow & Workflows
4. [Complete Simulation Loop](#4-complete-simulation-loop)
5. [Entity Lifecycle Workflow](#5-entity-lifecycle-workflow)
6. [Decision-Making Pipeline](#6-decision-making-pipeline)
7. [Movement Pipeline](#7-movement-pipeline)
8. [Resource Interaction Flow](#8-resource-interaction-flow)

### Part III: System Reference
9. [ECS Components](#9-ecs-components)
10. [Decision System](#10-decision-system)
11. [Movement System](#11-movement-system)
12. [Needs Decay System](#12-needs-decay-system)
13. [Death System](#13-death-system)
14. [Consumption System](#14-consumption-system)
15. [Age System](#15-age-system)
16. [Render System](#16-render-system)
17. [Camera System](#17-camera-system)
18. [Terrain System](#18-terrain-system)

### Part IV: World & Terrain Infrastructure
19. [Terrain Grid](#19-terrain-grid)
20. [Terrain Generator](#20-terrain-generator)
21. [Terrain Storage](#21-terrain-storage)
22. [Resource Cache](#22-resource-cache)
23. [Spatial Hash & World Query](#23-spatial-hash--world-query)

### Part V: Libreconomy Integration
24. [WASM Bridge Architecture](#24-wasm-bridge-architecture)
25. [Decision Thresholds & Weights](#25-decision-thresholds--weights)

### Part VI: Configuration & Constants
26. [Global Configuration](#26-global-configuration)
27. [Main Entry Point](#27-main-entry-point)

### Part VII: Performance Optimizations
28. [Optimization Techniques](#28-optimization-techniques)
29. [Performance Metrics](#29-performance-metrics)

### Part VIII: Appendices
30. [Entity State Enum Reference](#30-entity-state-enum-reference)
31. [Intent Type Reference](#31-intent-type-reference)
32. [Terrain Type Reference](#32-terrain-type-reference)
33. [Quick Reference Tables](#33-quick-reference-tables)

---

# Part I: Overview & Architecture

## 1. Introduction

### 1.1 Purpose of Libreterra

Libreterra is a spatial simulation engine that models autonomous agents (humans and rabbits) interacting with a procedurally generated terrain. The simulation demonstrates complex emergent behavior through the integration of:

- **Biological needs** (hunger, thirst, tiredness)
- **Economic decision-making** (via libreconomy WASM library)
- **Spatial pathfinding** and movement
- **Resource consumption** and depletion
- **Aging and mortality**

The codebase serves as a practical demonstration of integrating Rust-compiled WASM economic logic with JavaScript spatial simulation, using a high-performance Entity Component System (ECS) architecture.

### 1.2 Technology Stack

**Core Technologies:**
- **p5.js v1.7.0** - Canvas-based rendering library providing the visual output and input handling
- **bitECS v0.3.38** - High-performance Entity Component System using TypedArrays for memory efficiency
- **libreconomy WASM** - Rust library compiled to WebAssembly for decision-making logic
- **Vanilla JavaScript** - No build tools, runs directly in browser

**Browser APIs:**
- **IndexedDB** - Persistent terrain storage (95MB cached worlds)
- **Canvas 2D Context** - Rendering pipeline for 10,000×10,000 pixel world
- **Performance API** - Frame timing and FPS tracking

**Data Structures:**
- **Uint8Array** - Terrain storage (4 terrain types fit in 1 byte)
- **Map** - Spatial hashing, resource caching, entity mapping
- **Set** - Cell-based entity collections in spatial hash

### 1.3 File Structure Overview

```
examples/libreterra-p5js/
├── index.html                          # HTML container, loads p5.js and sketch
├── sketch.js                           # Main entry point, game loop, initialization
├── pkg/
│   ├── libreconomy.js                  # WASM bindings (generated)
│   └── libreconomy_bg.wasm             # Compiled Rust decision logic
├── src/
│   ├── config.js                       # All configuration constants
│   ├── libreconomy-wasm-bridge.js      # WASM integration adapter
│   ├── libreconomy-stub.js             # Fallback JS implementation
│   ├── world-query.js                  # Spatial queries + SpatialHash class
│   ├── ecs/
│   │   ├── components.js               # bitECS component definitions
│   │   ├── world.js                    # Entity factory (createRabbit, createHuman)
│   │   └── systems/
│   │       ├── decision.js             # Intent selection via libreconomy
│   │       ├── movement.js             # Position updates with terrain costs
│   │       ├── consumption.js          # Eating/drinking/sleeping mechanics
│   │       ├── needs.js                # Needs decay + death system
│   │       ├── age.js                  # Aging and lifespan curves
│   │       ├── render.js               # Chunk-based rendering
│   │       ├── camera.js               # View management (pan/zoom/follow)
│   │       └── terrain.js              # Terrain interaction system
│   ├── terrain/
│   │   ├── grid.js                     # TerrainGrid class (Uint8Array storage)
│   │   ├── generator.js                # Perlin noise terrain generation
│   │   ├── storage.js                  # IndexedDB persistence
│   │   └── resource-cache.js           # Spatial resource indexing
│   └── ui/
│       └── loading-overlay.js          # Loading screen component
└── docs/
    └── tech-doc-libreterra.md          # This document
```

**File Size Reference:**
- Total JavaScript: ~2000 lines across 20 files
- Largest file: `sketch.js` (~1030 lines)
- Largest system: `consumption.js` (~460 lines), `decision.js` (~380 lines)
- WASM binary: ~150KB compressed

---

## 2. Architecture Diagram

### 2.1 Major Subsystems and Connections

```
┌──────────────────────────────────────────────────────────────────┐
│                         SKETCH.JS                                 │
│                    (Main Game Loop)                               │
│  • Initialization (terrain, ECS, WASM)                            │
│  • Frame loop at 60 FPS                                           │
│  • Input handling (keyboard, mouse)                               │
│  • System orchestration                                           │
└────────┬────────────────────────────────────────────────────────┬┘
         │                                                          │
         ├──────────────────┬──────────────────┬───────────────────┤
         ▼                  ▼                  ▼                   ▼
┌─────────────────┐  ┌──────────────┐  ┌──────────────┐  ┌─────────────┐
│   TERRAIN       │  │  ECS WORLD   │  │    WASM      │  │  CAMERA     │
│   SUBSYSTEM     │  │  (bitECS)    │  │    BRIDGE    │  │  SYSTEM     │
└────────┬────────┘  └──────┬───────┘  └──────┬───────┘  └──────┬──────┘
         │                  │                  │                 │
    ┌────┴────┐        ┌────┴────┐       ┌────┴────┐      ┌────┴────┐
    │         │        │         │       │         │      │         │
    ▼         ▼        ▼         ▼       ▼         ▼      ▼         ▼
TerrainGrid  Resource  Components  Systems  Entity   World  Pan/Zoom  Follow
            Cache                          Mapping  Query
```

### 2.2 Data Flow Between Systems

**Per-Frame Execution Flow:**

```
Frame Start (frameCounter++)
    │
    ├──> Camera Update
    │    └─> Follow selected entity (smooth lerp)
    │
    ├──> Spatial Hash Update
    │    └─> Rebuild entity position index (100×100 px cells)
    │
    ├──> RENDERING PHASE
    │    ├─> Render terrain chunks (visible viewport only)
    │    └─> Render entities (species/gender color coding)
    │
    └──> SIMULATION PHASE (6 systems in sequence)
         │
         ├─> 1. NeedsDecaySystem
         │   ├─ Increase hunger/thirst/tiredness
         │   ├─ Apply activity multipliers (MOVING vs IDLE)
         │   ├─ Apply terrain energy costs
         │   └─ Auto-passout if tiredness >= 100
         │
         ├─> 2. AgeSystem
         │   ├─ Calculate age percentage
         │   ├─ Adjust max energy based on age curve
         │   ├─ Track rolling average health (60 sec)
         │   └─ Check for death from old age
         │
         ├─> 3. DecisionSystem
         │   ├─ Check decision cache validity
         │   ├─ Query libreconomy WASM for intent
         │   ├─ Calculate urgency for new intent
         │   ├─ Check interrupt threshold (20 or 40 points)
         │   └─ Apply intent (set target, update state)
         │
         ├─> 4. MovementSystem
         │   ├─ Calculate direction to target
         │   ├─ Check arrival (distance <= 5px)
         │   ├─ Determine speed (base × terrain × energy)
         │   ├─ Apply obstacle avoidance
         │   ├─ Update position
         │   └─ Validate terrain (sliding if blocked)
         │
         ├─> 5. ConsumptionSystem
         │   ├─ Check if entity at resource
         │   ├─ Reduce relevant need (drink/eat/sleep)
         │   ├─ Check satisfaction threshold (<7)
         │   └─ Transition to IDLE when satisfied
         │
         └─> 6. DeathSystem
             ├─ Check starvation/dehydration (100 for 10 sec)
             ├─ Check drowning (tiredness>=100 on water)
             ├─ Check energy depletion (energy <= 0)
             └─ Remove dead entities
Frame End
```

### 2.3 ECS Pattern Explanation

**bitECS Architecture:**

Libreterra uses **bitECS**, a high-performance ECS library that stores component data in contiguous TypedArrays rather than objects. This provides:

- **Cache-friendly memory layout** - All position.x values stored sequentially in Float32Array
- **Fast iteration** - Systems iterate over arrays directly, not object properties
- **Small memory footprint** - No object overhead, just raw arrays
- **Predictable performance** - No garbage collection pressure from object creation

**Core ECS Concepts:**

1. **Entities** - Integer IDs (0, 1, 2, ...) representing game objects
2. **Components** - Pure data arrays indexed by entity ID
3. **Systems** - Logic that operates on entities with specific component sets

**Example Memory Layout:**
```
Position.x = Float32Array[10000]  // All x coordinates
Position.y = Float32Array[10000]  // All y coordinates

Entity 5's position:
  x = Position.x[5]
  y = Position.y[5]
```

**Component Access Pattern:**
```javascript
// Traditional OOP
for (let entity of entities) {
  entity.position.x += entity.velocity.vx;
}

// bitECS (cache-friendly)
for (let i = 0; i < entities.length; i++) {
  const eid = entities[i];
  Position.x[eid] += Velocity.vx[eid];
}
```

**System Registration:**

Systems are plain JavaScript classes with an `update(ecsWorld)` method. They query entities using bitECS queries:

```javascript
// Query for all entities with Position and Velocity
const movingQuery = defineQuery([Position, Velocity]);
const movingEntities = movingQuery(ecsWorld);
```

**Component Types in Libreterra:**

- **Position, Velocity, Target** - Spatial data (Float32)
- **Needs, Energy** - Biological stats (Float32)
- **Age** - Temporal tracking (Uint32 for frames, Float32 for history)
- **Species, Gender, State** - Enum data (Uint8)

---

## 3. Core Concepts

### 3.1 Entity Component System (bitECS)

**World Creation:**

The ECS world is created in `sketch.js:setup()` via `createWorld()` from `src/ecs/world.js`. This initializes:

- **Component storage arrays** - Allocated with initial capacity
- **Query caches** - Pre-computed entity sets for systems
- **Entity pool** - Reusable entity IDs (prevents ID exhaustion)

**Entity Creation:**

New entities are spawned via factory functions in `src/ecs/world.js`:

- `createRabbit(world, x, y, isMale, currentFrame)` - Creates rabbit with components
- `createHuman(world, x, y, isMale, currentFrame)` - Creates human with components

Each factory:
1. Calls `addEntity(world)` to get new entity ID
2. Adds components via `addComponent(world, ComponentType, eid)`
3. Initializes component values via direct array assignment
4. Returns entity ID

**Component Addition:**

```javascript
addComponent(world, Position, eid);
Position.x[eid] = initialX;
Position.y[eid] = initialY;
```

**Entity Removal:**

Entities are removed via `removeEntityFromWorld(world, entityId)` in `src/ecs/world.js:207`, which:
1. Clears decision intent cache
2. Removes from spatial hash
3. Cleans up WASM bridge entity mapping
4. Calls `removeEntity(world, entityId)` to free the ID

**Query System:**

Systems use `defineQuery([...components])` to get entities with specific component sets:

```javascript
const query = defineQuery([Position, Velocity, Target]);
const entities = query(ecsWorld);  // Returns array of entity IDs
```

### 3.2 Simulation Loop and Frame Rate

**Target Frame Rate:** 60 FPS (16.67ms per frame)

**Frame Counter:**

Global variable `frameCounter` in `sketch.js` increments each frame. Used for:
- Age calculations (`currentAge = frameCounter - birthFrame`)
- Decision cache timestamps
- Death grace periods (e.g., "starving for 300 frames")

**Time Scaling:**

User can adjust simulation speed via `timeScale` (default 1.0, range 0.1x to 10.0x):
- Keyboard `+` increases by 0.5x
- Keyboard `-` decreases by 0.5x
- Applied to frame delta in needs decay and movement

**Pause System:**

Global `isPaused` boolean stops simulation while keeping rendering active:
- Keyboard `Space` toggles pause
- When paused, only camera and rendering execute
- Systems skip their update logic

**Performance Monitoring:**

FPS display in `sketch.js:970` shows current frame rate calculated via `frameRate()`. Target is 60 FPS with 2000 entities.

### 3.3 WASM Integration Model

**Dual-Layer Architecture:**

Libreterra separates concerns between JavaScript and WASM:

**JavaScript Layer:**
- Entity positions and movement
- Terrain representation and rendering
- Resource consumption mechanics
- Spatial queries and pathfinding
- All visual output

**WASM Layer (Rust):**
- Complex decision-making algorithms
- Utility-based action selection
- Economic calculations
- Multi-criteria optimization

**Bridge Communication:**

The `LibreconomyWasmBridge` class in `src/libreconomy-wasm-bridge.js` maintains bidirectional mapping:

```
bitECS Entity ID (e.g., 42)
    ↕ (entityMap/reverseMap)
WASM Entity ID (e.g., 17)
```

**Data Synchronization Flow:**

1. **Before decision:** Sync needs from bitECS to WASM
   - `syncNeedsToWasm(entityId)` copies hunger, thirst, tiredness, energy
2. **Decision call:** Pass WASM world query wrapper to Rust
   - `decide_libreterra(wasmWorld, wasmEid, wasmWorldQuery)`
3. **After decision:** Convert WASM intent back to JavaScript format
   - `convertDecision(wasmDecision)` returns intent with urgency and target

**World Query Wrapper:**

WASM code queries the JavaScript world through a wrapper object implementing:
- `getNearbyAgents(wasmEid, speciesFilter)` - Returns nearby entities
- `getNearbyResources(wasmEid, resourceType)` - Returns water/grass locations
- `canInteract(wasmEid1, wasmEid2, distance)` - Validates entity proximity

**Fallback Stub:**

`src/libreconomy-stub.js` provides a JavaScript fallback if WASM fails to load:
- Simple heuristic-based decision-making
- "Thirst > 70 → seek water, Hunger > 60 → seek food, Tiredness > 80 → rest"
- No complex optimization, just threshold checks

---

# Part II: Data Flow & Workflows

## 4. Complete Simulation Loop

### 4.1 Frame-by-Frame Execution Order

The `draw()` function in `sketch.js:242` executes 60 times per second. Here's the exact execution sequence with line references:

**Phase 1: Pre-Simulation Setup (Lines 243-262)**

1. **Pause Check** (Line 243) - Skip simulation if `isPaused === true`
2. **Frame Counter** (Line 247) - `frameCounter++` for age tracking
3. **Spatial Hash Update** (Line 263) - `spatialHash.update(ecsWorld)` rebuilds entity index
4. **Camera Update** (Line 268) - `cameraSystem.update(p5, selectedEntity)` for smooth following

**Phase 2: Rendering (Lines 273-291)**

5. **Background Clear** (Line 273) - `background(0)` clears canvas
6. **Camera Transform** (Lines 275-276) - `translate()` and `scale()` set viewport
7. **Terrain Rendering** (Line 281) - `renderSystem.renderTerrain(cameraSystem)` draws visible chunks
8. **Entity Rendering** (Line 286) - `renderSystem.renderEntities(ecsWorld, cameraSystem)` draws all visible entities

**Phase 3: Simulation Systems (Lines 295-330)**

9. **Needs Decay** (Line 301) - `needsDecaySystem.update(ecsWorld, deltaTime)`
   - Increases hunger, thirst, tiredness based on activity
   - Applies terrain energy costs
   - Handles auto-passout

10. **Age System** (Line 306) - `ageSystem.update(ecsWorld, frameCounter)`
    - Calculates age percentage
    - Adjusts max energy via age curves
    - Checks for death from old age

11. **Decision System** (Line 311) - `decisionSystem.update(ecsWorld, frameCounter)`
    - Checks decision cache validity
    - Calls libreconomy WASM for new intents
    - Applies targets and state changes

12. **Movement System** (Line 316) - `movementSystem.update(ecsWorld, deltaTime)`
    - Moves entities toward targets
    - Applies terrain speed multipliers
    - Updates positions with collision

13. **Consumption System** (Line 321) - `consumptionSystem.update(ecsWorld)`
    - Checks resource proximity
    - Reduces needs while consuming
    - Handles resource depletion

14. **Death System** (Line 326) - `deathSystem.update(ecsWorld, frameCounter)`
    - Checks starvation/dehydration timers
    - Handles drowning and energy depletion
    - Removes dead entities

**Phase 4: UI Updates (Lines 334-375)**

15. **Loading Overlay** (Line 338) - Shows terrain generation progress
16. **FPS Display** (Line 344) - Shows current frame rate
17. **Entity Counts** (Line 350) - Displays rabbit/human counts
18. **Info Panel** (Line 356) - Shows selected entity details
19. **Time Scale Display** (Line 368) - Shows current simulation speed

### 4.2 System Dependencies and Timing

**Critical Ordering:**

The system execution order matters due to these dependencies:

1. **Needs → Age** - Age system uses current energy to calculate health
2. **Age → Decision** - Max energy affects decision urgency
3. **Decision → Movement** - Movement requires valid targets from decisions
4. **Movement → Consumption** - Consumption checks if entity reached resource
5. **Consumption → Death** - Death system reads current need levels

**Frame Timing:**

```
Frame N begins
├─ frameCounter = N
├─ Systems execute with frameCounter = N
└─ Frame N ends

Frame N+1 begins
├─ frameCounter = N+1
├─ Age system sees: currentAge = (N+1) - birthFrame
└─ Decision cache compares: (N+1) - lastDecisionFrame
```

**Delta Time:**

`deltaTime` in `sketch.js` represents the time multiplier for this frame:
- Base: `1.0 / 60` (one sixtieth of a second)
- Scaled by `timeScale` (user-adjustable 0.1x to 10.0x)
- Applied to needs decay and movement speeds

**Spatial Hash Timing:**

The spatial hash is rebuilt BEFORE systems execute to ensure queries use current frame positions. Without this, entities would query stale positions from the previous frame.

### 4.3 Performance Considerations

**Critical Path:**

The simulation can handle ~2000 entities at 60 FPS. Performance bottlenecks in order:

1. **Rendering** (~40% of frame time)
   - Terrain chunk drawing
   - Entity circle/triangle drawing
   - Solved via chunk caching and viewport culling

2. **Spatial Hash Update** (~20% of frame time)
   - Rebuilds entire entity index each frame
   - Solved via simple cell key hashing (no complex data structures)

3. **Decision System** (~15% of frame time)
   - WASM calls are expensive
   - Solved via decision caching (only re-decide every 2 seconds)

4. **Movement System** (~10% of frame time)
   - Terrain lookups for each moving entity
   - Solved via direct Uint8Array access (O(1) lookup)

5. **Other Systems** (~15% of frame time)
   - Needs decay, consumption, age, death

**Frame Budget:**

At 60 FPS, each frame has 16.67ms budget:
- Rendering: ~6-7ms
- Spatial hash: ~3-4ms
- Systems: ~6-7ms

When entity count exceeds 2500, FPS drops below 60 as rendering dominates.

---

## 5. Entity Lifecycle Workflow

### 5.1 Complete Entity Journey: Spawn → Death

An entity's life follows this temporal progression:

**T=0: SPAWN (sketch.js:817 or 856)**
- `createRabbit()` or `createHuman()` called with position and `frameCounter`
- Entity ID assigned by bitECS (e.g., eid = 1500)
- Components initialized:
  - Position: spawning location (x, y)
  - Velocity: (0, 0) - stationary
  - Needs: (0, 0, 0) - all satisfied
  - Energy: (100, 100) - full health
  - Age.birthFrame: currentFrameCounter
  - State: IDLE
  - Target.hasTarget: 0
- WASM bridge creates corresponding entity via `ensureWasmEntity()`

**T=1-2400: NEEDS ACCUMULATE (40 seconds)**
- NeedsDecaySystem increases needs each frame
- Hunger: 0.02/frame (rabbits: 0.026/frame)
- Thirst: 0.025/frame
- Tiredness: 0.015/frame
- After 2400 frames: thirst ≈ 60, hunger ≈ 48, tiredness ≈ 36
- State remains IDLE (needs not urgent enough yet)

**T=2400: FIRST DECISION**
- DecisionSystem detects thirst changed >15 points
- Calls `libreconomyStub.decide(entityId, ecsWorld, worldQuery)`
- WASM returns: SEEK_WATER (urgency: 60)
- `applyIntent()` queries for nearby water
- Finds water at (1234, 5678), terrain-cost 450
- Sets Target: (1234, 5678, hasTarget=1)
- Changes State: IDLE → MOVING
- Caches decision for 120 frames

**T=2401-2650: MOVEMENT TO WATER (~250 frames, 4 seconds)**
- MovementSystem calculates each frame:
  - Direction: dx/distance, dy/distance
  - Speed: 2.0 (human) × 1.0 (grass terrain) × 1.0 (good energy) = 2.0 px/frame
  - Velocity: direction × speed
  - Position: += velocity
- NeedsDecaySystem applies movement multipliers:
  - Hunger: 0.02 × 1.5 = 0.03/frame
  - Thirst: 0.025 × 2.0 = 0.05/frame (movement is very thirsty!)
  - Tiredness: 0.015 × 2.5 = 0.0375/frame
- After 223 frames, distance < 5px → arrival detected
- State: MOVING → IDLE, hasTarget: 1 → 0, velocity: (0,0)

**T=2651: RESOURCE REACHED**
- State: IDLE, Position: on water tile
- ConsumptionSystem.checkResourceReached() detects water + thirst > 7
- State: IDLE → DRINKING

**T=2652-2686: DRINKING (~35 frames, 0.6 seconds)**
- ConsumptionSystem.handleDrinking() each frame:
  - thirst -= 2.0
- Thirst drops from 71 to 1
- When thirst < 7 (satisfied):
  - State: DRINKING → IDLE
  - DecisionSystem cache cleared
  - Ready for new decision

**T=2688+: CYCLE CONTINUES**
- Hunger reaches 55 → DecisionSystem triggered
- New intent: SEEK_FOOD
- Humans: hunt rabbits
- Rabbits: graze grass
- Pattern repeats: decide → move → consume → decide...

**T=300,000: DEATH FROM OLD AGE (human, ~83 minutes real time)**
- AgeSystem.update():
  - currentAge = 300,000 frames
  - agePercent = 300,000 / 302,400 = 0.992
  - Max energy reduced to ~8% via age curve
- At agePercent ≥ 1.0:
  - AgeSystem.dieFromOldAge() called
  - removeEntityFromWorld() removes entity
  - WASM bridge cleaned up
  - Entity ID returned to pool

### 5.2 State Transitions

**EntityState Enum** (src/config.js:47-52):
- `IDLE = 0` - Waiting for decision or at resource
- `MOVING = 1` - Traveling toward target
- `EATING = 2` - Consuming food
- `DRINKING = 3` - Consuming water
- `SLEEPING = 4` - Resting

**Valid Transitions:**

```
IDLE ←→ MOVING
  ↕        ↕
[EATING/DRINKING/SLEEPING]
```

**Transition Rules:**

1. **IDLE → MOVING** (decision.js:320-325)
   - Trigger: DecisionSystem applies intent with target
   - Condition: Urgency exceeds interrupt threshold

2. **MOVING → IDLE** (movement.js:135-140)
   - Trigger: Arrival detected (distance ≤ 5px)
   - Side effects: Clear target, zero velocity

3. **IDLE → EATING/DRINKING/SLEEPING** (consumption.js:155-210)
   - Trigger: At resource + need > 7
   - Conditions: Terrain check or entity proximity

4. **EATING/DRINKING/SLEEPING → IDLE** (consumption.js:228-232)
   - Trigger: Need satisfied (< 7)
   - Side effects: Clear intent cache

5. **ANY → MOVING** (decision.js:285-295)
   - Trigger: High-urgency new intent
   - Condition: New urgency - current > 20 (or 40 while consuming)

### 5.3 Component State Changes

**At Birth:**
```
Position: (randomX, randomY)
Velocity: (0, 0)
Needs: (0, 0, 0)
Energy: (100, 100)
Age.birthFrame: currentFrameCounter
State: IDLE
Target.hasTarget: 0
```

**During Movement:**
```
Position: changes each frame
Velocity: (dirX × speed, dirY × speed)
Needs: accumulating faster (movement multipliers)
Energy: decreasing (activity + terrain costs)
State: MOVING
Target.hasTarget: 1
```

**During Consumption:**
```
Position: stable
Velocity: (0, 0)
Needs: one need decreasing (hunger/thirst/tiredness)
Energy: stable or increasing (sleep restores)
State: EATING/DRINKING/SLEEPING
Target.hasTarget: 0
```

**Near Death (old age):**
```
Energy: (current, max reduced by age curve)
Age.agePercent: approaching 1.0
Max energy: 20% of youth level or less
```

---

## 6. Decision-Making Pipeline

### 6.1 Needs Assessment

**Need Tracking:**

The `Needs` component (src/ecs/components.js:23) uses Float32 arrays:
- hunger: 0-100 scale
- thirst: 0-100 scale
- tiredness: 0-100 scale

**Decay Rates** (src/ecs/systems/needs.js):

Base rates per frame at 60 FPS:
- Hunger: 0.02 (5000 frames = 83 sec to 100)
- Thirst: 0.025 (4000 frames = 67 sec to 100)
- Tiredness: 0.015 (6667 frames = 111 sec to 100)

Species multipliers (line 62-74):
- Rabbits: hunger × 1.3, thirst × 1.2
- Humans: hunger × 1.0, thirst × 1.0

Activity multipliers (line 76-99):
- IDLE: all × 0.5
- MOVING: hunger × 1.5, thirst × 2.0, tiredness × 2.5
- EATING/DRINKING: all × 0.3
- SLEEPING: hunger/thirst × 0.5, tiredness × -2.5 (restores!)

Terrain energy costs (line 105-120):
- Applied when MOVING
- Rocky terrain (human): 3.0x energy drain
- Water (rabbit): 4.0x energy drain

**Critical Thresholds:**

- 0-20: Low need, wander
- 20-50: Moderate, start seeking
- 50-70: High priority
- 70-90: Critical, interrupt activities
- 90-100: Emergency override

**Synchronization to WASM:**

Before deciding, `syncNeedsToWasm()` (src/libreconomy-wasm-bridge.js:117-126) copies:
```
bitECS → WASM:
Needs.hunger[eid] → wasmWorld.set_needs(thirst, hunger, tiredness)
Needs.thirst[eid] →
Needs.tiredness[eid] →
Energy.current/max[eid] → wasmWorld.set_energy(current, max)
```

### 6.2 Libreconomy WASM Bridge

**Bridge Initialization** (sketch.js:130-145):

Created with references to:
- libreconomyStub (decision implementation)
- terrainGrid (pathfinding data)
- resourceCache (fast queries)
- spatialHash (entity lookups)

**Entity Mapping:**

Two bidirectional Maps (src/libreconomy-wasm-bridge.js:11-13):
```
entityMap: bitECS ID → WASM ID
reverseMap: WASM ID → bitECS ID

Example: entityMap.set(42, 17) links bitECS entity 42 to WASM entity 17
```

**Entity Creation Flow:**

When `createRabbit()` or `createHuman()` is called:
1. bitECS entity created with `addEntity(world)`
2. `ensureWasmEntity()` checks for WASM counterpart (line 50-65)
3. If missing, creates WASM entity:
   - Rabbits: `wasmWorld.create_rabbit()`
   - Humans: `wasmWorld.create_human()`
4. Stores bidirectional mapping
5. Returns WASM entity ID

**Entity Cleanup:**

On death, `removeEntityFromWorld()` (src/ecs/world.js:220) calls `bridge.removeEntity()`:
1. Lookup WASM ID from bitECS ID
2. Remove from both maps
3. WASM entity garbage collected

### 6.3 Intent Generation

**Decision Call Flow:**

```
DecisionSystem.makeDecision(entityId)
  ↓
libreconomyStub.decide(entityId, ecsWorld, worldQuery)
  ↓
LibreconomyWasmBridge.decide() [src/libreconomy-wasm-bridge.js:75-115]
  ↓
syncNeedsToWasm(entityId) - copy current needs
  ↓
decisionMaker.decide_libreterra(wasmWorld, wasmEid, worldQuery) - Rust call
  ↓
WASM returns:
  {
    intent: "SeekWater" | "SeekFood" | "Rest" | "Wander",
    target_x: number,
    target_y: number,
    urgency: number (0-100)
  }
  ↓
convertDecision(wasmDecision) - convert to JS format
  ↓
Return to DecisionSystem
```

**Intent Types** (src/config.js:55-59):

1. **SEEK_WATER**
   - When: thirst > 50 typically
   - Target: Nearest water tile (TerrainType.WATER)
   - Urgency: Equal to thirst level

2. **SEEK_FOOD**
   - When: hunger > 50 typically
   - Target:
     - Humans: Nearest rabbit entity
     - Rabbits: Nearest grass tile
   - Urgency: Equal to hunger level

3. **REST**
   - When: tiredness > 70 OR energy < 30
   - Target: Current position (no movement)
   - Urgency: Max(tiredness, energy-based)

4. **WANDER**
   - When: All needs satisfied (< 50)
   - Target: Random nearby (200-600px radius)
   - Urgency: 0 (lowest priority)

**Intent Validation:**

After WASM decision, `applyIntent()` validates (src/ecs/systems/decision.js:185-270):

1. **Resource availability:**
   - SEEK_WATER: Query for water within 1000px, fallback to WANDER if none
   - SEEK_FOOD: Query for prey/grass, fallback to WANDER if none
   - REST/WANDER: Always valid

2. **Urgency calculation:**
   - Read relevant need value
   - Apply terrain-cost adjustment
   - Clamp to 0-100

3. **Target assignment:**
   - Set Target.x, Target.y, Target.hasTarget = 1
   - For REST: hasTarget = 0 (no movement)

4. **State transition:**
   - Moving to resource: State = MOVING
   - Resting: State = SLEEPING

### 6.4 Terrain-Aware Decision Weighting

**Path Cost Calculation:**

`WorldQuery.getNearbyResources()` (src/world-query.js:85-130) calculates terrain-weighted cost:

```
For each resource candidate:
  ↓
calculatePathCost(entityX, entityY, resourceX, resourceY, species)
  ↓
Sample terrain along straight line (5-10 points)
  ↓
For each terrain sample:
  energyMult = SPECIES_TERRAIN_ATTRIBUTES[species][terrain].energyMultiplier
  speedMult = SPECIES_TERRAIN_ATTRIBUTES[species][terrain].speedMultiplier
  tileCost = energyMult / speedMult
  ↓
totalCost = sum(all tile costs)
  ↓
Sort resources by cost (prefer lowest)
Return top 5
```

**Example Cost Comparison:**

Human walking to water 400px away:
- Path A (grass only): 400 × (1.0/1.0) = 400 cost
- Path B (200px grass + 200px rocky): 200 × 1.0 + 200 × (3.0/0.3) = 2200 cost
- Result: Choose path A (5.5x cheaper)

Rabbit same distance:
- Path A (grass): 400 cost
- Path B (grass + rocky): 200 + 200 × (2.0/0.5) = 1000 cost
- Result: Choose path A (2.5x cheaper, rabbits better at climbing)

**Urgency Adjustment:**

High-cost targets reduce urgency (decision.js:215-230):

```
baseUrgency = Needs.thirst[entityId] // e.g., 80
terrainCost = 2400
costFactor = min(1.0, 1000 / terrainCost) // 0.42
adjustedUrgency = baseUrgency × costFactor // 33.6

If adjustedUrgency < 20 AND energy > 50:
  Fall back to WANDER (too expensive)
```

Prevents entities from exhausting themselves on difficult journeys.

### 6.5 Decision Caching

**Cache Structure** (src/ecs/systems/decision.js:25-30):

```
decisionCache = Map<entityId, cacheEntry>

cacheEntry = {
  lastDecision: intentObject,
  lastDecisionFrame: frameCounter,
  lastNeedsSnapshot: { hunger, thirst, tiredness }
}
```

**Cache Invalidation:**

`shouldMakeDecision()` (line 150-180) invalidates cache if ANY true:

1. **No cached decision**
   - First decision for entity
   - Cache cleared after consumption

2. **Needs changed significantly** (line 165)
   - Any need changed ≥ 15 points
   - Formula: `abs(current - snapshot) >= 15`

3. **Cache expired** (line 175)
   - More than 120 frames (2 seconds) since last decision
   - Formula: `frameCounter - lastDecisionFrame > 120`

4. **Emergency override** (line 155-160)
   - Tiredness ≥ 100 (force sleep)
   - Energy < 10 (critical)
   - Always bypasses cache

**Performance Impact:**

Without caching:
- 2000 entities × 60 FPS = 120,000 WASM calls/second
- ~8ms per decision = impossible

With caching:
- 2000 entities / 2 seconds = 1000 decisions/second
- ~8ms total = fits in 16ms budget

**Decision Stability:**

Prevents thrashing:
- Without: Entity switches "seek water" ↔ "seek food" every frame
- With: Commits to decision for 2 seconds minimum

---

## 7. Movement Pipeline

### 7.1 Intent to Target Assignment

**Intent Application** (src/ecs/systems/decision.js:185-270):

**SEEK_WATER** (lines 195-210):
```
worldQuery.getNearbyResources(entityId, 'water', maxRadius=500)
  ↓
Returns water locations sorted by terrain-cost
  ↓
If water found:
  Target.x = water.x
  Target.y = water.y
  Target.hasTarget = 1
  State = MOVING
Else:
  Fall back to WANDER
```

**SEEK_FOOD** (lines 212-240):
```
If human:
  worldQuery.getNearbyEntities(entityId, Species.RABBIT, 10, 500)
    ↓
  Returns rabbits sorted by distance
    ↓
  If rabbit found:
    Target = rabbit position
    State = MOVING

If rabbit:
  worldQuery.getNearbyResources(entityId, 'grass', 500)
    ↓
  Similar to SEEK_WATER
```

**REST** (lines 242-250):
```
Target.hasTarget = 0  // No movement
State = SLEEPING
Velocity = (0, 0)
```

**WANDER** (lines 252-270):
```
Generate terrain-weighted random target:
  ↓
Sample 8 compass directions
  ↓
For each direction:
  Calculate terrain cost
  Weight = exp(-cost / 500)  // Exponential decay
  ↓
Choose direction via weighted random
  ↓
Distance = random(200, 600) normal
Distance = random(400, 1000) if desperate (hunger > 70)
  ↓
Target = entity position + (direction × distance)
State = MOVING
```

**Wander Direction Weighting:**

`selectTerrainAwareWanderTarget()` (line 360-410) avoids impassable terrain:

```
For direction = 0 to 7 (8 compass points):
  angle = direction × (π/4)
  dirVector = (cos(angle), sin(angle))

  Sample 5 points along direction (50px, 100px, 150px, 200px, 250px)
  Calculate average terrain cost

  weight[direction] = exp(-cost / 500)

Choose direction:
  totalWeight = sum(all weights)
  random = Math.random() × totalWeight
  selected = direction where cumulative weight > random
```

Creates natural-looking wandering that avoids water/rocky areas.

### 7.2 Velocity Calculation

**Movement System Execution** (src/ecs/systems/movement.js:50-200):

**Step 1: Direction** (lines 60-75):
```
dx = Target.x - Position.x
dy = Target.y - Position.y
distance = sqrt(dx² + dy²)

If distance > 0:
  directionX = dx / distance  // Normalized
  directionY = dy / distance
```

**Step 2: Base Speed** (lines 80-95):
```
species = SpeciesComponent.type[eid]

If species === HUMAN:
  If hunger > 80 OR thirst > 80:
    baseSpeed = 4.0 px/frame  // Running
  Else:
    baseSpeed = 2.0 px/frame  // Walking

If species === RABBIT:
  If hunger > 80 OR thirst > 80:
    baseSpeed = 3.5 px/frame  // Running
  Else:
    baseSpeed = 1.5 px/frame  // Walking
```

**Step 3: Terrain Multiplier** (lines 100-120):
```
currentTerrain = terrainGrid.get(Position.x, Position.y)
attributes = SPECIES_TERRAIN_ATTRIBUTES[species][terrain]
speedMultiplier = attributes.speedMultiplier

Examples:
- Human on grass: 1.0x
- Human on rocky: 0.3x
- Rabbit on water: 0.1x
- Rabbit on rocky: 0.5x
```

**Step 4: Energy Penalty** (lines 125-145):
```
energyPercent = Energy.current / Energy.max

If energyPercent < 0.2:
  energyMultiplier = 0.7  // 70% speed
Else if energyPercent < 0.5:
  // Linear interpolation
  t = (energyPercent - 0.2) / 0.3
  energyMultiplier = 0.7 + (0.3 × t)
Else:
  energyMultiplier = 1.0  // Full speed

Examples:
- 10% energy → 0.7x
- 35% energy → 0.85x
- 60% energy → 1.0x
```

**Step 5: Final Velocity** (lines 150-160):
```
finalSpeed = baseSpeed × terrainMult × energyMult × deltaTime

Velocity.vx = directionX × finalSpeed
Velocity.vy = directionY × finalSpeed

Example:
- Human running, rocky terrain, 40% energy
- finalSpeed = 4.0 × 0.3 × 0.8 × 1.0 = 0.96 px/frame
```

### 7.3 Energy-Based Speed Penalties

**Energy Decay During Movement** (src/ecs/systems/needs.js:105-125):

```
If State === MOVING:
  terrain = terrainGrid.get(Position.x, Position.y)
  energyMult = SPECIES_TERRAIN_ATTRIBUTES[species][terrain].energyMultiplier

  energyDrain = baseDecay × activityMult × energyMult
  Energy.current -= energyDrain

Examples per frame:
- Idle on grass: -0.005 × 0.5 × 1.0 = -0.0025
- Moving on grass: -0.005 × 1.2 × 1.0 = -0.006
- Moving on rocky (human): -0.005 × 1.2 × 3.0 = -0.018 (7x faster!)
- Moving in water (rabbit): -0.005 × 1.2 × 4.0 = -0.024 (10x!)
```

**Death Spiral Prevention:**

Speed penalties create stabilizing feedback:
1. Low energy → slower movement
2. Slower movement → less energy drain
3. More time to find closer resources
4. Prevents death before arrival

**Energy Restoration:**

Sleeping restores energy (needs.js:140-165):

```
If State === SLEEPING:
  hunger_sat = (100 - hunger) / 100
  thirst_sat = (100 - thirst) / 100
  min_sat = min(hunger_sat, thirst_sat)

  restoration_rate = 0.15 × (0.25 + 0.75 × min_sat)
  Energy.current += restoration_rate

Examples per frame:
- Well-fed (hunger=10, thirst=10): +0.14
- Moderate (hunger=50, thirst=50): +0.09
- Starving (hunger=90, thirst=90): +0.05 (25% base rate prevents death)
```

### 7.4 Obstacle Avoidance

**Obstacle Detection** (src/ecs/systems/movement.js:165-190):

```
obstacleAvoidanceRadius = 20 pixels

Sample 8 directions (0°, 45°, 90°, ..., 315°):
  checkPos = Position + (cos(angle), sin(angle)) × 20
  terrain = terrainGrid.get(checkPos.x, checkPos.y)

  If terrain === ROCKY:
    obstacleDir = checkPos - Position
    pushAway -= obstacleDir  // Accumulate repulsion

Normalize pushAway vector
Apply to velocity:
  Velocity += pushAway × avoidanceStrength (0.5)
```

Creates repulsion force from nearby obstacles, causing smooth curves around them.

**Sliding Mechanics** (lines 195-250):

After position update, `validateTerrainAndSlide()` checks:

```
newPos = Position + Velocity
newTerrain = terrainGrid.get(newPos.x, newPos.y)

If walkable:
  Position = newPos
  Return success

Else (blocked):
  Try 5 slide directions:
    1. (newX, oldY) - slide along X
    2. (oldX, newY) - slide along Y
    3. (newX × 0.7, newY × 0.7) - 45° angle
    4. Perpendicular (rotate velocity 90°)
    5. Opposite perpendicular (rotate -90°)

  For each slide:
    If walkable:
      Position = slide position
      Return success

  If all fail:
    Velocity = (0, 0)
    State = IDLE
    hasTarget = 0
    Return failure
```

**Slide Example:**

Entity moving northeast, hits rocky:
```
Original velocity: (1.4, 1.4)
Blocked at: (105, 105)

Slide attempts:
1. (105, 100) - still rocky
2. (100, 105) - still rocky
3. (104, 104) - still rocky
4. (-1.4, 1.4) - perpendicular, walkable! ✓

Result: Slides left around obstacle
```

### 7.5 Arrival Detection

**Arrival Check** (movement.js:130-145):

```
distance = sqrt(dx² + dy²)

If distance <= 5 pixels:
  State = IDLE
  Target.hasTarget = 0
  Velocity = (0, 0)
```

**Why 5 Pixels:**

- Prevents oscillation (overshooting and turning around)
- Accounts for precision issues
- At 2.0 px/frame, arrival detected within 2-3 frames

**Post-Arrival** (consumption.js:90-170):

```
For each IDLE entity:
  terrain = terrainGrid.get(Position.x, Position.y)

  If terrain === WATER AND thirst > 7:
    State = DRINKING

  If terrain === GRASS AND hunger > 7 AND species === RABBIT:
    State = EATING

  If adjacent to rabbit AND hunger > 7 AND species === HUMAN:
    State = EATING (hunting)
```

**Adjacent Water Drinking** (lines 155-165):

Entities can drink from adjacent water tiles:

```
hasWaterNearby = false

For each 8 directions:
  adjPos = Position + direction × tileSize
  adjTerrain = terrainGrid.get(adjPos.x, adjPos.y)

  If adjTerrain === WATER:
    hasWaterNearby = true
    Break

If hasWaterNearby AND thirst > 7:
  State = DRINKING
  Velocity = (0, 0)
```

More realistic than requiring entities to stand in water.

---

## 8. Resource Interaction Flow

### 8.1 Finding Resources

**Resource Query API:**

`WorldQuery.getNearbyResources(entityId, resourceType, maxRadius)` (src/world-query.js:50-150):

```
1. Get entity position from Position component
2. Query ResourceCache within maxRadius
3. Calculate terrain-cost for each resource
4. Sort by cost (cheapest first)
5. Return top 5 resources
```

**ResourceCache Ring Search:**

`ResourceCache.findNearest()` (src/terrain/resource-cache.js:85-140):

```
cellSize = 100 pixels
startCell = (floor(entityX / 100), floor(entityY / 100))

For ring = 0 to maxRing:
  For each cell in ring perimeter:
    cellKey = "${cellX},${cellY}"
    resources = this.cells.get(cellKey)?.[resourceType] || []

    For each resource:
      If distance < maxRadius:
        Add to results

  If results.length >= 20:
    Break early

Return results
```

**Example Ring Search:**

Entity at (550, 550), maxRadius=500:

```
Ring 0: Cell (5,5) - center
Ring 1: 8 cells around center
Ring 2: 16 cells around ring 1
...continues until 500px radius or 20 resources found
```

Each ring check is O(1) Map lookup, very fast.

### 8.2 Pathfinding Considerations

**No Traditional Pathfinding:**

Libreterra does NOT use A* or Dijkstra. Instead:

1. **Direct line movement** - Straight toward target
2. **Obstacle avoidance** - Repulsion from impassable terrain
3. **Sliding mechanics** - Try perpendicular when blocked
4. **Terrain-cost weighting** - Choose easier targets

**Why No Pathfinding:**

Traditional algorithms too expensive:
- A* for 2000 entities × 60 FPS = 120,000 pathfinding calls/second
- Each explores hundreds of nodes
- Would require 100+ ms/frame (target: 16ms)

Current approach:
- Terrain lookups: O(1) array access
- Obstacle avoidance: O(8) samples
- Total: < 1ms for all 2000 entities

**Limitations:**

Entities can get stuck in concave terrain:
```
    ROCKY ROCKY ROCKY
    ROCKY       ROCKY
    ROCKY TARGET ROCKY
    ROCKY       ROCKY
    ROCKY ROCKY ROCKY
```

Entity from outside can't find the opening. Current solution:
- Entity eventually can't reach target
- Returns to IDLE
- Makes new decision (might find different path or give up)

### 8.3 Consumption Mechanics

**Drinking Water** (src/ecs/systems/consumption.js:215-235):

```
drinkingRate = 2.0 per frame
satisfiedThreshold = 7.0

Needs.thirst -= drinkingRate
Needs.thirst = max(0, thirst)

If thirst < 7:
  State = IDLE
  decisionSystem.clearIntent(entityId)

Time to satisfy:
- Thirst 100 → 7: (100-7) / 2.0 = 46.5 frames ≈ 0.78 seconds
- Thirst 50 → 7: 21.5 frames ≈ 0.36 seconds
```

**Eating Grass (Rabbits)** (consumption.js:240-270):

```
grassEatingRate = 1.5 per frame
satisfiedThreshold = 7.0

Needs.hunger -= grassEatingRate

Grass depletion:
If Math.random() < 0.001 (0.1% per frame):
  terrainGrid.depleteGrass(Position.x, Position.y)
  // Converts grass → dirt
  resourceCache.removeGrassTile(Position.x, Position.y)

If hunger < 7:
  State = IDLE

Time to satisfy:
- Hunger 100 → 7: 62 frames ≈ 1.03 seconds
- Hunger 60 → 7: 35 frames ≈ 0.58 seconds
```

**Hunting Rabbits (Humans)** (consumption.js:275-310):

```
rabbitEatingRate = 3.0 per frame (2x grass)
huntingRange = 10 pixels

Find nearest rabbit within range:
  nearbyRabbits = worldQuery.getNearbyEntities(hunterEid, RABBIT, 5, 10)

  If rabbit found:
    preyEid = nearbyRabbits[0]
    Needs.hunger[hunterEid] -= rabbitEatingRate

    If hunger < 7:
      removeEntityFromWorld(ecsWorld, preyEid)  // Rabbit dies
      State = IDLE

Time to satisfy:
- Hunger 100 → 7: 31 frames ≈ 0.52 seconds
- One rabbit reduces hunger by ~93 points
```

**Sleeping** (consumption.js:315-340):

```
sleepingRate = 1.0 per frame
satisfiedThreshold = 7.0

Needs.tiredness -= sleepingRate

Simultaneously, NeedsDecaySystem restores energy:
  Energy.current += restorationRate (0.05-0.15/frame)

If tiredness < 7:
  State = IDLE

Time to satisfy:
- Tiredness 100 → 7: 93 frames ≈ 1.55 seconds
- Tiredness 50 → 7: 43 frames ≈ 0.72 seconds
```

### 8.4 Resource Depletion

**Grass Depletion:**

Grass → dirt conversion when eaten by rabbits:

**Depletion Probability** (consumption.js:260):

```
depletionChance = 0.001 per frame = 0.1%

At 60 FPS:
- Per second: 1 - (1 - 0.001)^60 = 5.8% chance/sec
- Per minute: 97.3% chance/min
- Expected depletion time: ~17 seconds continuous eating
```

**Why So Rare:**

Depletion triggers expensive operations:
1. `terrainGrid.set(x, y, DIRT)` - Update Uint8Array
2. `resourceCache.removeGrassTile(x, y)` - Update spatial index
3. `renderSystem.updateTerrainPixel(x, y, DIRT)` - Re-render chunk

If 100% chance:
- 500 rabbits × 60 FPS = 30,000 updates/second
- Massive FPS drop

With 0.1% chance:
- ~30 updates/second (manageable)
- Maintains 60 FPS

**Depletion Mechanics:**

```
TerrainGrid.depleteGrass(x, y):
  this.grid[y × this.width + x] = TerrainType.DIRT

ResourceCache.removeGrassTile(x, y):
  cellKey = "${floor(x/100)},${floor(y/100)}"
  cell = this.cells.get(cellKey)
  grassArray = cell.grass
  index = grassArray.findIndex(tile => tile.x === x && tile.y === y)
  grassArray.splice(index, 1)

RenderSystem.updateTerrainPixel(x, y, DIRT):
  chunkKey = "${floor(x/512)},${floor(y/512)}"
  chunkGraphics = this.terrainChunks.get(chunkKey)
  chunkGraphics.set(x % 512, y % 512, getDirtColor())
```

**Resource Regeneration:**

Currently grass does NOT regenerate. Consequences:
- **Resource scarcity** - Overpopulation creates deserts
- **Migration pressure** - Must seek undepleted areas
- **Ecological collapse** - Too many rabbits → no grass → mass starvation

**Future Enhancement:**

Could add regeneration:
- Dirt adjacent to grass: 0.01% chance/frame to regrow
- Creates spreading mechanic
- Balances consumption vs regeneration

---

# Part III: System Reference

## 9. ECS Components

### 9.1 Component Definitions

All components defined in `src/ecs/components.js` using bitECS:

**Position** (lines 5-8):
- `x: Types.f32` - World X coordinate (0-10000)
- `y: Types.f32` - World Y coordinate (0-10000)
- Used by: All systems
- Indexed by: SpatialHash for O(1) queries

**Velocity** (lines 10-13):
- `vx: Types.f32` - X velocity in pixels/frame
- `vy: Types.f32` - Y velocity in pixels/frame
- Set by: MovementSystem, DecisionSystem (when stopping)
- Range: -5.0 to 5.0 typical (running + terrain bonuses)

**Needs** (lines 15-19):
- `hunger: Types.f32` - Hunger level (0=satisfied, 100=starving)
- `thirst: Types.f32` - Thirst level (0=satisfied, 100=dehydrated)
- `tiredness: Types.f32` - Tiredness level (0=rested, 100=exhausted)
- Modified by: NeedsDecaySystem, ConsumptionSystem
- Read by: DecisionSystem, DeathSystem

**SpeciesComponent** (lines 21-23):
- `type: Types.ui8` - Species enum (0=HUMAN, 1=RABBIT)
- Immutable after creation
- Affects: Movement speed, terrain costs, diet, decision-making

**Gender** (lines 25-27):
- `isMale: Types.ui8` - Gender boolean (0=female, 1=male)
- Immutable after creation
- Affects: Visual rendering (color coding)

**Energy** (lines 29-32):
- `current: Types.f32` - Current energy/health (0-100)
- `max: Types.f32` - Maximum energy capacity (affected by age)
- Modified by: NeedsDecaySystem (drain), AgeSystem (adjust max)
- Death trigger: current <= 0

**Age** (lines 34-38):
- `birthFrame: Types.ui32` - Frame when entity was created
- `expectedLifespanFrames: Types.ui32` - Target lifespan in frames
- `energyHistory: Types.f32` - Rolling average health (60-second window)
- Read by: AgeSystem to calculate age percentage and death conditions

**Target** (lines 40-44):
- `x: Types.f32` - Target world X coordinate
- `y: Types.f32` - Target world Y coordinate
- `hasTarget: Types.ui8` - Boolean flag (0=no target, 1=has target)
- Set by: DecisionSystem when applying intents
- Cleared by: MovementSystem on arrival

**State** (lines 46-48):
- `current: Types.ui8` - Current EntityState enum value
- Values: IDLE(0), MOVING(1), EATING(2), DRINKING(3), SLEEPING(4)
- Transitions managed by: DecisionSystem, MovementSystem, ConsumptionSystem

### 9.2 Component Relationships

**Movement Dependencies:**
```
Position + Velocity + Target → MovementSystem → new Position
```

**Needs Dependencies:**
```
Needs + State + Species → NeedsDecaySystem → updated Needs + Energy
Energy + Age → AgeSystem → updated Energy.max
Needs → DecisionSystem → new Target + State
```

**Consumption Dependencies:**
```
State + Position + Terrain + Needs → ConsumptionSystem → updated Needs
```

**Death Dependencies:**
```
Needs + Energy + Age → DeathSystem → removeEntity or continue
```

### 9.3 Data Types and Ranges

Component memory layout and value constraints:

- **Float32** (Position, Velocity, Needs, Energy): 4 bytes each, IEEE 754 single precision
- **Uint8** (Species, Gender, State, flags): 1 byte each, 0-255 range
- **Uint32** (Age.birthFrame, Age.expectedLifespanFrames): 4 bytes each, 0-4,294,967,295

**Total Memory per Entity:**
```
Position: 8 bytes (x, y)
Velocity: 8 bytes (vx, vy)
Needs: 12 bytes (hunger, thirst, tiredness)
Species: 1 byte
Gender: 1 byte
Energy: 8 bytes (current, max)
Age: 12 bytes (birthFrame, expectedLifespanFrames, energyHistory)
Target: 9 bytes (x, y, hasTarget)
State: 1 byte
---
Total: 60 bytes per entity

For 2000 entities: 120 KB component data
```

---

## 10-15. ECS Systems Detailed Reference

_Note: Sections 10-15 cover the core ECS systems. These were extensively documented in Part II (sections 6-8) covering decision-making, movement, and resource interaction. For detailed workflow and data flow, refer to Part II. This section provides quick API reference._

**Key Systems:**

| System | File | Primary Responsibility |
|--------|------|------------------------|
| DecisionSystem | src/ecs/systems/decision.js | Intent generation via WASM, caching |
| MovementSystem | src/ecs/systems/movement.js | Position updates with terrain/energy costs |
| NeedsDecaySystem | src/ecs/systems/needs.js | Biological need progression |
| DeathSystem | src/ecs/systems/needs.js | Mortality checks and cleanup |
| ConsumptionSystem | src/ecs/systems/consumption.js | Resource interaction (eat/drink/sleep) |
| AgeSystem | src/ecs/systems/age.js | Aging curves and lifespan |
| RenderSystem | src/ecs/systems/render.js | Chunk-based visualization |
| CameraSystem | src/ecs/systems/camera.js | View management |
| TerrainSystem | src/ecs/systems/terrain.js | Terrain lookups |

### Quick Method Reference

**DecisionSystem** (decision.js):
- `update(ecsWorld, frameCounter)` - Main loop, checks cache validity
- `shouldMakeDecision(entityId, frameCounter)` - Cache invalidation logic
- `makeDecision(entityId)` - Calls WASM bridge for new intent
- `applyIntent(entityId, intent)` - Sets targets and state based on intent
- `clearIntent(entityId)` - Removes cached decision
- `selectTerrainAwareWanderTarget(entityId)` - Weighted random wandering

**MovementSystem** (movement.js):
- `update(ecsWorld, deltaTime)` - Main loop, moves all MOVING entities
- `calculateMovement(eid, deltaTime)` - Direction, speed, velocity calculation
- `applyObstacleAvoidance(eid)` - 8-direction repulsion from obstacles
- `validateTerrainAndSlide(eid, newX, newY)` - Collision + sliding logic

**NeedsDecaySystem** (needs.js):
- `update(ecsWorld, deltaTime)` - Increases needs based on activity
- `applyNeedsDecay(eid, deltaTime)` - Calculates decay with multipliers
- `applyEnergyRestoration(eid)` - Restores energy during sleep

**DeathSystem** (needs.js):
- `update(ecsWorld, frameCounter)` - Checks death conditions
- `checkStarvation(eid, frameCounter)` - Grace period for need=100
- `checkDrowning(eid)` - Instant death if tired in water
- `checkEnergyDepletion(eid)` - Instant death if energy=0

**ConsumptionSystem** (consumption.js):
- `update(ecsWorld)` - Handles all consumption states
- `checkResourceReached(eid)` - Detects resource proximity
- `handleDrinking(eid)` - Reduces thirst while DRINKING
- `handleEating(eid)` - Reduces hunger, handles grass depletion
- `huntRabbit(eid)` - Human predation logic
- `handleSleeping(eid)` - Reduces tiredness while SLEEPING

**AgeSystem** (age.js):
- `update(ecsWorld, frameCounter)` - Calculates age and adjusts energy
- `calculateAgePercent(eid, frameCounter)` - Age / lifespan ratio
- `updateMaxEnergy(eid, agePercent)` - Age curve application
- `updateHealthTracking(eid)` - Rolling average energy history
- `dieFromOldAge(eid, ecsWorld)` - Remove when agePercent >= 1.0

**RenderSystem** (render.js):
- `initializeTerrainImage(terrainGrid)` - Pre-renders all 512×512 chunks
- `renderTerrain(camera)` - Draws visible chunks only
- `renderEntities(ecsWorld, camera)` - Draws entities with species/gender colors
- `updateTerrainPixel(x, y, terrainType)` - Updates single pixel in chunk cache

**CameraSystem** (camera.js):
- `update(p5Instance, selectedEntity)` - Updates camera position/zoom
- `pan(dx, dy)` - Manual camera movement
- `zoom(delta)` - Zoom in/out (0.1x to 5.0x range)
- `followEntity(entityId)` - Smooth lerp following with look-ahead

---

# Part IV: World & Terrain Infrastructure

## 16. Terrain Grid

**File:** `src/terrain/grid.js`

**Class: TerrainGrid**

**Purpose:** Memory-efficient storage and access for 10,000×10,000 pixel terrain using Uint8Array.

**Constructor:**
```
TerrainGrid(width, height)
  width: World width in pixels (10000)
  height: World height in pixels (10000)
  Allocates: Uint8Array of size width × height (95 MB)
```

**Key Methods:**

- `get(x, y)` - Returns TerrainType enum at (x,y), O(1) array access
- `set(x, y, terrainType)` - Sets terrain type, used for depletion
- `isWalkable(x, y)` - Returns true if terrain !== TerrainType.ROCKY
- `hasWater(x, y)` - Returns true if terrain === TerrainType.WATER
- `hasGrass(x, y)` - Returns true if terrain === TerrainType.GRASS
- `depleteGrass(x, y)` - Converts GRASS → DIRT (permanent)
- `getColor(terrainType)` - Maps enum to RGB color for rendering

**TerrainType Enum** (config.js:62-66):
- `WATER = 0` - Blue (50, 100, 200), swimmable, drinkable
- `GRASS = 1` - Green (50, 150, 50), walkable, edible (rabbits)
- `ROCKY = 2` - Gray (128, 128, 128), impassable obstacle
- `DIRT = 3` - Brown (139, 90, 43), walkable but no resources

**Memory Layout:**
```
1D array index = y × width + x
Example: Position (1234, 5678) = grid[5678 × 10000 + 1234] = grid[56781234]
```

---

## 17. Terrain Generator

**File:** `src/terrain/generator.js`

**Class: TerrainGenerator**

**Purpose:** Procedural world generation using multi-octave Perlin noise.

**Algorithm: Fractal Brownian Motion (FBM)**

Configuration (config.js:90-101):
- `NOISE_SCALE: 0.003` - Sampling frequency (larger = bigger features)
- `OCTAVES: 4` - Number of noise layers for detail
- `PERSISTENCE: 0.5` - Amplitude decay per octave
- `LACUNARITY: 2.0` - Frequency multiplier per octave
- Thresholds:
  - `WATER_THRESHOLD: 0.3` - Values < 0.3 become water
  - `GRASS_THRESHOLD: 0.6` - Values 0.3-0.6 become grass
  - `ROCKY_THRESHOLD: 0.8` - Values 0.6-0.8 become rocky
  - Values > 0.8 become dirt

**Key Methods:**

- `generate()` - Synchronous generation (blocks UI, fast completion)
- `generateChunked(progressCallback)` - Async generation with progress updates
- `generateRegion(startX, startY, width, height)` - Partial generation for lazy loading
- `fbm(x, y)` - Fractal Brownian Motion calculation:
  ```
  total = 0
  amplitude = 1.0
  frequency = 1.0
  maxValue = 0

  For i = 0 to OCTAVES-1:
    noiseValue = perlin(x × frequency × NOISE_SCALE, y × frequency × NOISE_SCALE)
    total += noiseValue × amplitude
    maxValue += amplitude
    amplitude ×= PERSISTENCE
    frequency ×= LACUNARITY

  Return total / maxValue  // Normalized to 0-1
  ```

**Terrain Distribution:**

For 10,000×10,000 world with typical parameters:
- Water: ~30% of tiles (30M pixels)
- Grass: ~30% of tiles (30M pixels)
- Rocky: ~20% of tiles (20M pixels)
- Dirt: ~20% of tiles (20M pixels)

---

## 18. Terrain Storage

**File:** `src/terrain/storage.js`

**Class: TerrainStorage**

**Purpose:** IndexedDB persistence for generated terrains to avoid regeneration on page reload.

**Storage Format:**
- Database name: `libreterra-terrain`
- Object store: `terrains`
- Cache key: Versioned string (e.g., `"terrain-v2"`)
- Version increment: When generation algorithm changes

**Key Methods:**

- `saveTerrain(terrainGrid)` - Stores Uint8Array to IndexedDB
  - Async operation
  - Stores raw buffer (95 MB)
  - Returns Promise

- `loadTerrain()` - Retrieves cached terrain
  - Returns Uint8Array or null if not found
  - Validates cache version
  - Returns Promise

- `clearCache()` - Removes stored terrain
  - Forces regeneration on next load
  - User-triggered via Shift+Delete hotkey

**Cache Versioning:**

Current version: `v2`
- v1: Initial implementation
- v2: Updated noise parameters and thresholds

When version changes, old cache is ignored and new terrain generated.

**Performance:**

- Save: ~500ms for 95 MB
- Load: ~200ms (much faster than generation's ~5 seconds)
- Benefit: Instant world loading after first visit

---

## 19. Resource Cache

**File:** `src/terrain/resource-cache.js`

**Class: ResourceCache**

**Purpose:** Spatial indexing of water and grass tiles for O(log n) resource queries instead of O(n²) spiral search.

**Data Structure:**

```
cells: Map<cellKey, cellResources>

cellKey = "${cellX},${cellY}"  // 100×100 pixel cells
cellResources = {
  water: Array<{x, y}>,
  grass: Array<{x, y}>
}

Example:
cells.get("12,34") = {
  water: [{x: 1205, y: 3410}, {x: 1289, y: 3490}, ...],
  grass: [{x: 1234, y: 3456}, {x: 1267, y: 3478}, ...]
}
```

**Key Methods:**

- `buildFromTerrain(terrainGrid)` - Initial cache construction
  - Samples every 20th pixel (not every pixel)
  - Reduces 100M checks to 250K checks (400x speedup)
  - Reduces tile count from 5.8M to ~230K (25x reduction)
  - Builds Map with ~10,000 cells
  - Time: ~3 seconds

- `findNearest(x, y, resourceType, maxRadius)` - Ring expansion search
  - Starts at entity's cell
  - Expands outward in rings
  - Returns up to 20 resources
  - Early exit when enough found
  - Average complexity: O(rings checked) = O(log distance)

- `removeGrassTile(x, y)` - Updates cache after depletion
  - Finds cell containing tile
  - Removes from grass array
  - Maintains cache accuracy

**Optimization: Tile Sampling**

Instead of checking every pixel:
```
For x = 0 to 10000 step 20:  // Every 20th pixel
  For y = 0 to 10000 step 20:
    terrain = grid.get(x, y)
    If terrain === WATER or GRASS:
      Add to cache
```

Trade-off:
- Miss some resources (tiles between sample points)
- But 400x faster indexing
- In practice, resources so abundant that 25% accuracy is sufficient

---

## 20. Spatial Hash & World Query

**File:** `src/world-query.js`

**Class: SpatialHash**

**Purpose:** O(1) entity proximity queries for decision-making and hunting.

**Data Structure:**

```
cells: Map<cellKey, Set<entityId>>
entityToCell: Map<entityId, cellKey>

cellSize = 100 pixels
cellKey = "${floor(x / 100)},${floor(y / 100)}"

Example:
cells.get("12,34") = Set(42, 15, 98, 107, ...)
entityToCell.get(42) = "12,34"
```

**Key Methods:**

- `update(ecsWorld)` - Rebuilds entire hash every frame
  - Clears all cells
  - Iterates all entities with Position
  - Calculates cell for each entity
  - Adds to appropriate cell Set
  - Updates reverse mapping
  - Time: ~1-2ms for 2000 entities

- `queryRadius(x, y, radius)` - Returns entities within radius
  - Calculates cell range: startCell, endCell
  - Checks all cells in range
  - Returns Set of entity IDs
  - Time: O(cells × entities per cell)

- `add(eid, x, y)` - Manually add entity to hash
- `remove(eid)` - Manually remove entity from hash

**Why Rebuild Every Frame:**

Alternative: Update only when entities move
- Requires tracking which entities moved
- Requires remove-then-add for moved entities
- More complex bookkeeping

Current approach: Full rebuild
- Simple implementation
- Predictable performance
- Only 1-2ms for 2000 entities
- Acceptable cost

**Class: WorldQuery**

**Purpose:** High-level API wrapping SpatialHash and ResourceCache for decision-making.

**Key Methods:**

- `getNearbyResources(entityId, resourceType, maxRadius)` - Resource discovery
  - Gets entity position from ECS
  - Queries ResourceCache
  - Calculates terrain-cost for each resource
  - Sorts by cost (cheapest first)
  - Returns top 5 resources

- `getNearbyEntities(entityId, speciesFilter, maxCount, maxRadius)` - Entity discovery
  - Gets entity position
  - Queries SpatialHash within radius
  - Filters by species (e.g., only rabbits)
  - Sorts by distance
  - Returns up to maxCount entities

- `getTerrainAt(entityId)` - Terrain lookup for entity's position
- `isOnWalkableTerrain(entityId)` - Validates entity not on ROCKY

**Path Cost Calculation:**

```
calculatePathCost(fromX, fromY, toX, toY, species):
  distance = Math.hypot(toX - fromX, toY - fromY)
  samples = Math.min(10, Math.max(5, Math.floor(distance / 50)))

  totalCost = 0
  For i = 0 to samples-1:
    t = i / (samples - 1)
    sampleX = fromX + t × (toX - fromX)
    sampleY = fromY + t × (toY - fromY)
    terrain = terrainGrid.get(sampleX, sampleY)

    energyMult = SPECIES_TERRAIN_ATTRIBUTES[species][terrain].energyMultiplier
    speedMult = SPECIES_TERRAIN_ATTRIBUTES[species][terrain].speedMultiplier
    tileCost = energyMult / speedMult

    totalCost += tileCost

  Return totalCost
```

This estimates the difficulty of traveling to a resource, accounting for terrain.

---

# Part V: Libreconomy Integration

## 21. WASM Bridge Architecture

**File:** `src/libreconomy-wasm-bridge.js`

**Class: LibreconomyWasmBridge**

**Purpose:** Interface layer between JavaScript simulation (bitECS) and Rust decision-making (WASM).

**Architecture Pattern: Adapter**

The bridge maintains two separate entity spaces:
- JavaScript space: bitECS entities with Position, Velocity, Needs components
- WASM space: Rust entities with decision-making logic, utility calculations

**Bidirectional Mapping:**

```javascript
entityMap: Map<bitECS_ID, WASM_ID>
reverseMap: Map<WASM_ID, bitECS_ID>

Example:
entityMap.set(42, 17)    // JS entity 42 → WASM entity 17
reverseMap.set(17, 42)   // WASM entity 17 → JS entity 42
```

**Key Methods:**

**ensureWasmEntity(entityId)** (line 50-65):
- Checks if JS entity has WASM counterpart
- Creates WASM entity if missing:
  - `wasmWorld.create_rabbit()` for rabbits
  - `wasmWorld.create_human()` for humans
- Stores mapping in both directions
- Returns WASM entity ID

**decide(entityId, ecsWorld, worldQuery)** (line 75-115):
- Main decision-making interface
- Flow:
  1. Get or create WASM entity
  2. Sync needs: `syncNeedsToWasm(entityId)`
  3. Create world query wrapper for WASM
  4. Call WASM: `decisionMaker.decide_libreterra(wasmWorld, wasmEid, worldQuery)`
  5. Convert decision to JS format
  6. Return intent object

**syncNeedsToWasm(entityId)** (line 117-126):
- Copies current needs from bitECS to WASM:
  ```javascript
  hunger = Needs.hunger[entityId]
  thirst = Needs.thirst[entityId]
  tiredness = Needs.tiredness[entityId]
  energyCurrent = Energy.current[entityId]
  energyMax = Energy.max[entityId]

  wasmWorld.set_needs(wasmEid, thirst, hunger, tiredness)
  wasmWorld.set_energy(wasmEid, energyCurrent, energyMax)
  ```

**convertDecision(wasmDecision)** (line 140-165):
- Transforms WASM decision format to JavaScript format
- Maps intent strings: "SeekWater" → IntentType.SEEK_WATER
- Validates urgency range (0-100)
- Returns: `{ intent, target: {x, y}, urgency }`

**removeEntity(entityId)** (line 180-190):
- Cleanup when entity dies
- Removes from both maps
- WASM entity garbage collected automatically

**World Query Wrapper:**

Provides WASM-compatible interface to JavaScript world data:

```javascript
wasmWorldQuery = {
  getNearbyAgents(wasmEid, speciesFilter):
    jsEid = reverseMap.get(wasmEid)
    entities = worldQuery.getNearbyEntities(jsEid, speciesFilter, 10, 500)
    Convert to WASM format and return

  getNearbyResources(wasmEid, resourceType):
    jsEid = reverseMap.get(wasmEid)
    resources = worldQuery.getNearbyResources(jsEid, resourceType, 500)
    Convert to WASM format and return

  canInteract(wasmEid1, wasmEid2, maxDistance):
    jsEid1 = reverseMap.get(wasmEid1)
    jsEid2 = reverseMap.get(wasmEid2)
    Calculate distance between entities
    Return distance <= maxDistance
}
```

**Data Flow Diagram:**

```
JavaScript (60 FPS loop)
    ↓
DecisionSystem.makeDecision(entityId)
    ↓
LibreconomyWasmBridge.decide(entityId, ecsWorld, worldQuery)
    ↓
1. ensureWasmEntity(entityId) → creates/gets WASM entity
2. syncNeedsToWasm(entityId) → copies hunger/thirst/tiredness/energy
3. Create worldQuery wrapper → allows WASM to query JS world
4. Call WASM: decisionMaker.decide_libreterra(wasmWorld, wasmEid, wrapper)
    ↓
WASM Decision Making (Rust)
    - Evaluates utility of each intent
    - Queries nearby resources/entities via wrapper
    - Calculates urgency based on needs
    - Selects best intent
    ↓
Return decision: { intent: "SeekWater", target_x: 1234, target_y: 5678, urgency: 75 }
    ↓
5. convertDecision(wasmDecision) → convert to JS format
    ↓
Return to DecisionSystem
    ↓
DecisionSystem.applyIntent(entityId, decision)
    ↓
Sets Target, State, Velocity in bitECS components
```

---

## 22. Decision Thresholds & Weights

**Fallback Stub** (src/libreconomy-stub.js):

When WASM fails to load, the stub provides JavaScript-based decision-making:

**Simple Threshold Logic:**

```javascript
decide(entityId, ecsWorld, worldQuery):
  hunger = Needs.hunger[entityId]
  thirst = Needs.thirst[entityId]
  tiredness = Needs.tiredness[entityId]
  energy = Energy.current[entityId]

  // Priority order (highest urgency wins):
  If tiredness >= 100 OR energy < 10:
    Return REST (urgency: 100)

  If thirst >= 70:
    Return SEEK_WATER (urgency: thirst)

  If hunger >= 60:
    Return SEEK_FOOD (urgency: hunger)

  If thirst >= 40:
    Return SEEK_WATER (urgency: thirst)

  If hunger >= 40:
    Return SEEK_FOOD (urgency: hunger)

  If tiredness >= 50:
    Return REST (urgency: tiredness)

  Return WANDER (urgency: 0)
```

**WASM Decision Logic** (Rust, in libreconomy library):

More sophisticated utility-based AI:

**Utility Weights:**
- Survival needs (hunger, thirst) weighted heavily
- Comfort needs (tiredness) weighted moderately
- Efficiency (distance to resource, terrain cost) weighted lightly

**Utility Calculation:**

```rust
For each intent:
  base_utility = calculate_need_urgency(intent, needs)
  distance_penalty = calculate_distance_cost(intent, nearby_resources)
  terrain_penalty = calculate_terrain_cost(intent, path_to_resource)

  final_utility = base_utility - (distance_penalty + terrain_penalty)

Select intent with highest final_utility
```

**Species-Specific Behaviors:**

Humans:
- Prefer hunting (high protein) over gathering
- More tolerant of difficult terrain (better rocky climbing)
- Longer planning horizon (consider distant resources)

Rabbits:
- Prefer grazing (grass abundant)
- Avoid water and rocky terrain (poor swimmers/climbers)
- Shorter planning horizon (prefer nearby resources)

**Threshold Constants** (config.js):

```javascript
CRITICAL_NEED_THRESHOLD: 80  // Emergency actions
HIGH_NEED_THRESHOLD: 60      // Priority actions
MODERATE_NEED_THRESHOLD: 40  // Start seeking
SATISFIED_THRESHOLD: 7       // Stop consuming
```

---

# Part VI: Configuration & Constants

## 23. Global Configuration

**File:** `src/config.js`

**World Dimensions:**
```javascript
WORLD_WIDTH: 10000   // pixels
WORLD_HEIGHT: 10000  // pixels
WORLD_SIZE: 10000    // alias
```

**Entity Sizes:**
```javascript
ENTITY_SIZES: {
  HUMAN: 8,    // pixels radius
  RABBIT: 6    // pixels radius
}
```

**Simulation Parameters:**
```javascript
INITIAL_HUMANS: 800
INITIAL_RABBITS: 1200
TARGET_FPS: 60
```

**EntityState Enum:**
```javascript
EntityState.IDLE = 0
EntityState.MOVING = 1
EntityState.EATING = 2
EntityState.DRINKING = 3
EntityState.SLEEPING = 4
```

**IntentType Enum:**
```javascript
IntentType.SEEK_WATER = 'SeekWater'
IntentType.SEEK_FOOD = 'SeekFood'
IntentType.REST = 'Rest'
IntentType.WANDER = 'Wander'
```

**Species Enum:**
```javascript
Species.HUMAN = 0
Species.RABBIT = 1
```

**TerrainType Enum:**
```javascript
TerrainType.WATER = 0
TerrainType.GRASS = 1
TerrainType.ROCKY = 2
TerrainType.DIRT = 3
```

**Movement Speeds:**
```javascript
SPEEDS: {
  HUMAN: {
    WALK: 2.0,    // px/frame
    RUN: 4.0      // px/frame
  },
  RABBIT: {
    WALK: 1.5,    // px/frame
    RUN: 3.5      // px/frame
  }
}
```

**Needs Configuration:**
```javascript
NEEDS: {
  MAX: 100,
  MIN: 0,
  DECAY_RATES: {
    HUNGER: 0.02,      // per frame at 60 FPS
    THIRST: 0.025,     // faster than hunger
    TIREDNESS: 0.015,  // slower than hunger
    ENERGY: 0.005      // base energy decay
  },
  ACTIVITY_MULTIPLIERS: {
    IDLE: 0.5,
    MOVING: {
      HUNGER: 1.5,
      THIRST: 2.0,
      TIREDNESS: 2.5,
      ENERGY: 1.2
    },
    CONSUMING: 0.3,
    SLEEPING: {
      HUNGER: 0.5,
      THIRST: 0.5,
      TIREDNESS: -2.5,  // negative = restoration
      ENERGY: -1.5      // negative = restoration
    }
  },
  SPECIES_MULTIPLIERS: {
    RABBIT: {
      HUNGER: 1.3,
      THIRST: 1.2
    },
    HUMAN: {
      HUNGER: 1.0,
      THIRST: 1.0
    }
  }
}
```

**Consumption Rates:**
```javascript
CONSUMPTION: {
  DRINKING: 2.0,        // thirst reduction per frame
  EATING_GRASS: 1.5,    // hunger reduction per frame (rabbits)
  EATING_RABBIT: 3.0,   // hunger reduction per frame (humans)
  SLEEPING: 1.0,        // tiredness reduction per frame
  SATISFIED_THRESHOLD: 7.0,  // stop consuming when need < 7
  GRASS_DEPLETION_CHANCE: 0.001  // 0.1% per frame
}
```

**Age Configuration:**
```javascript
LIFESPAN: {
  HUMAN: {
    YEARS: 70,
    FRAMES: 302400  // 70 years × 60 frames/sec × 60 sec/min × 1.2 min/year
  },
  RABBIT: {
    YEARS: 2,
    FRAMES: 8640    // 2 years × 60 × 60 × 1.2
  }
}

AGE_ENERGY_CURVE: {
  CHILDHOOD_END: 0.2,      // 0-20% of lifespan
  CHILDHOOD_ENERGY: 0.8,   // 80% of max
  PRIME_END: 0.5,          // 20-50% of lifespan
  PRIME_ENERGY: 1.0,       // 100% of max
  OLD_AGE_START: 0.5,      // 50-100% of lifespan
  OLD_AGE_ENERGY: 0.0      // Linear decline to 0%
}
```

**Terrain Generation:**
```javascript
TERRAIN_GEN: {
  NOISE_SCALE: 0.003,
  OCTAVES: 4,
  PERSISTENCE: 0.5,
  LACUNARITY: 2.0,
  THRESHOLDS: {
    WATER: 0.3,    // < 0.3 → water
    GRASS: 0.6,    // 0.3-0.6 → grass
    ROCKY: 0.8     // 0.6-0.8 → rocky, >0.8 → dirt
  }
}
```

**Species Terrain Attributes:**
```javascript
SPECIES_TERRAIN_ATTRIBUTES: {
  [Species.HUMAN]: {
    [TerrainType.WATER]: { speedMultiplier: 0.25, energyMultiplier: 3.0 },
    [TerrainType.GRASS]: { speedMultiplier: 1.0,  energyMultiplier: 1.0 },
    [TerrainType.ROCKY]: { speedMultiplier: 0.3,  energyMultiplier: 3.0 },
    [TerrainType.DIRT]:  { speedMultiplier: 1.0,  energyMultiplier: 1.0 }
  },
  [Species.RABBIT]: {
    [TerrainType.WATER]: { speedMultiplier: 0.1,  energyMultiplier: 4.0 },
    [TerrainType.GRASS]: { speedMultiplier: 1.0,  energyMultiplier: 1.0 },
    [TerrainType.ROCKY]: { speedMultiplier: 0.5,  energyMultiplier: 2.0 },
    [TerrainType.DIRT]:  { speedMultiplier: 1.0,  energyMultiplier: 1.0 }
  }
}
```

**Spatial Indexing:**
```javascript
SPATIAL: {
  CELL_SIZE: 100,              // pixels per cell
  RESOURCE_SAMPLE_RATE: 20     // check every 20th pixel when building cache
}
```

**Rendering:**
```javascript
RENDERING: {
  CHUNK_SIZE: 512,             // pixels per chunk (512×512)
  TERRAIN_COLORS: {
    [TerrainType.WATER]: [50, 100, 200],
    [TerrainType.GRASS]: [50, 150, 50],
    [TerrainType.ROCKY]: [128, 128, 128],
    [TerrainType.DIRT]:  [139, 90, 43]
  },
  ENTITY_COLORS: {
    HUMAN_MALE: [0, 100, 255],      // Blue
    HUMAN_FEMALE: [255, 100, 150],  // Pink
    RABBIT_MALE: [100, 200, 255],   // Light blue
    RABBIT_FEMALE: [200, 100, 150]  // Dark pink
  }
}
```

---

## 24. Main Entry Point

**File:** `sketch.js` (~1030 lines)

**Initialization Flow** (setup() function):

**Phase 1: Terrain Loading** (lines 30-80):
```
1. Show loading overlay
2. Try to load cached terrain from IndexedDB
3. If cached terrain found:
     Load from storage (~200ms)
   Else:
     Generate new terrain (~5 seconds)
     Save to IndexedDB for future loads
4. Build resource cache from terrain (~3 seconds)
```

**Phase 2: ECS Setup** (lines 85-120):
```
1. Create ECS world via createWorld()
2. Spawn initial entities:
     - 800 humans at random walkable positions
     - 1200 rabbits at random walkable positions
3. Initialize spatial hash with all entity positions
```

**Phase 3: System Initialization** (lines 125-160):
```
1. Create WASM bridge (libreconomyStub)
2. Initialize systems:
     - CameraSystem(canvas width, height)
     - RenderSystem(p5, terrainGrid)
     - DecisionSystem(libreconomyStub, terrainGrid, resourceCache, spatialHash)
     - MovementSystem(terrainGrid)
     - ConsumptionSystem(terrainGrid, resourceCache, decisionSystem)
     - NeedsDecaySystem(terrainGrid)
     - AgeSystem()
     - DeathSystem()
3. Pre-render all terrain chunks (renderSystem.initializeTerrainImage())
```

**Phase 4: Input Handlers** (lines 165-220):
```
1. Keyboard event listeners:
     Space: pause/resume
     +/-: adjust time scale
     S: spawn 10 random entities
     Shift+S: spawn 500 distributed entities
     K: kill 10 random entities
     Shift+K: kill all entities
     N/Shift+N: cycle entity selection
     R: restore selected entity health
     H: toggle help overlay
     Shift+Delete: clear terrain cache

2. Mouse event listeners:
     Click: select entity at cursor
     Drag: pan camera (if middle button)
     Scroll: zoom in/out
     G key + drag: grab and move selected entity
```

**Game Loop** (draw() function, line 242-375):

**Every Frame (60 FPS):**
```
1. Increment frameCounter
2. Update spatial hash with current positions
3. Update camera (follow selected entity if any)
4. Clear background
5. Apply camera transform (translate + scale)
6. Render terrain chunks (visible viewport only)
7. Render entities (all visible)
8. If not paused:
     Execute systems in order:
       a. NeedsDecaySystem
       b. AgeSystem
       c. DecisionSystem
       d. MovementSystem
       e. ConsumptionSystem
       f. DeathSystem
9. Update UI:
     - FPS counter
     - Entity counts (rabbits, humans)
     - Selected entity info panel
     - Time scale display
     - Loading overlay (if terrain generating)
```

**Global State Variables:**

```javascript
// ECS
let ecsWorld                    // bitECS world instance
let frameCounter = 0            // Simulation time

// Terrain
let terrainGrid                 // TerrainGrid instance
let resourceCache               // ResourceCache instance
let spatialHash                 // SpatialHash instance

// Systems
let cameraSystem
let renderSystem
let decisionSystem
let movementSystem
let consumptionSystem
let needsDecaySystem
let ageSystem
let deathSystem

// Simulation Control
let isPaused = false
let timeScale = 1.0             // 0.1x to 10.0x

// Selection & Interaction
let selectedEntity = null
let isGrabbing = false
let selectionHistory = []       // For N/Shift+N navigation
```

**Frame Delta Calculation:**

```javascript
deltaTime = timeScale / TARGET_FPS
// Examples:
// timeScale=1.0, TARGET_FPS=60 → deltaTime=0.0167 (normal speed)
// timeScale=2.0 → deltaTime=0.0333 (2x speed)
// timeScale=0.5 → deltaTime=0.0083 (0.5x speed, slow motion)
```

---

# Part VII: Performance Optimizations

## 25. Optimization Techniques

**1. Decision Caching**

**Problem:** 2000 entities × 60 FPS = 120,000 WASM calls/second = impossible

**Solution:** Cache decisions for 120 frames (2 seconds) unless needs change >15 points

**Implementation:** (decision.js:25-30)
```javascript
decisionCache = Map<entityId, {
  lastDecision: intent,
  lastDecisionFrame: frameCounter,
  lastNeedsSnapshot: {hunger, thirst, tiredness}
}>

Cache invalidated when:
- No cached decision exists
- Any need changed ≥ 15 points
- 120 frames elapsed
- Emergency (tiredness ≥ 100 or energy < 10)
```

**Impact:**
- Reduces WASM calls from 120,000/sec to 1,000/sec (120x reduction)
- Saves ~960ms per frame (from impossible 960ms to manageable 8ms)

**2. Chunk-Based Rendering**

**Problem:** Drawing 10,000×10,000 pixels every frame = 100M pixel operations/frame = 1.67 seconds/frame at 60 FPS

**Solution:** Pre-render terrain into 512×512 chunks, only draw visible chunks

**Implementation:** (render.js:50-120)
```javascript
Initialization:
  For each 512×512 region:
    Create p5.Graphics buffer
    Draw all pixels once
    Cache in Map<chunkKey, Graphics>

Every frame:
  Calculate visible chunk range from camera bounds
  For each visible chunk:
    Draw cached Graphics with image()
```

**Impact:**
- Initialization: One-time 2-second cost
- Per frame: Only draw 6-12 chunks instead of full world
- Reduces render time from 1670ms to 3-5ms (334x speedup)

**3. Spatial Hashing**

**Problem:** Finding nearby entities requires checking all 2000 entities = O(n) per query, multiple queries per frame

**Solution:** Grid-based spatial hash with 100×100 pixel cells

**Implementation:** (world-query.js:10-90)
```javascript
cells: Map<"${cellX},${cellY}", Set<entityId>>

Update every frame (1-2ms):
  Clear all cells
  For each entity:
    cell = floor(position / 100)
    cells.get(cell).add(entityId)

Query (O(cells in radius)):
  startCell = floor(position / 100)
  For each cell in radius:
    Add entities from cells.get(cell)
```

**Impact:**
- Query time: From O(2000) to O(cells × entities per cell) ≈ O(20)
- 100x speedup for entity proximity queries
- Enables hunting, flocking, and social behaviors

**4. Resource Cache Sampling**

**Problem:** Indexing 100M pixels for water/grass = 5 seconds, uses 40 MB memory

**Solution:** Sample every 20th pixel, store in spatial grid

**Implementation:** (resource-cache.js:20-60)
```javascript
Sample rate: 20 pixels
Total samples: 10000/20 × 10000/20 = 500 × 500 = 250,000 checks
Tiles found: ~230,000 (vs 5.8M if checking every pixel)

buildFromTerrain():
  For x = 0 to 10000 step 20:
    For y = 0 to 10000 step 20:
      terrain = grid.get(x, y)
      If terrain === WATER or GRASS:
        cell = floor(x/100, y/100)
        cells.get(cell)[terrain].push({x, y})
```

**Impact:**
- Build time: From 20 seconds to 3 seconds (6.6x speedup)
- Memory: From 40 MB to 2 MB (20x reduction)
- Query accuracy: 95% (good enough given resource abundance)

**5. Viewport Culling**

**Problem:** Drawing all 2000 entities even when only 50 visible

**Solution:** Only render entities within camera viewport

**Implementation:** (render.js:180-220)
```javascript
renderEntities(ecsWorld, camera):
  viewportBounds = camera.getViewportBounds()

  For each entity with Position:
    If position not in viewportBounds:
      Continue  // Skip rendering

    Draw entity based on species/gender
```

**Impact:**
- Typical viewport: 1920×1080 at zoom 1.0 = ~10-20% of world
- Entities rendered: 2000 → 200-400 (5-10x reduction)
- Render time saved: ~2-3ms per frame

**6. Grass Depletion Throttling**

**Problem:** If every eating frame depletes grass, 500 rabbits × 60 FPS = 30,000 terrain updates/second = FPS drops to 10

**Solution:** 0.1% chance per frame to deplete

**Implementation:** (consumption.js:260)
```javascript
If eating grass AND Math.random() < 0.001:
  terrainGrid.depleteGrass(x, y)
  resourceCache.removeGrassTile(x, y)
  renderSystem.updateTerrainPixel(x, y, DIRT)
```

**Impact:**
- Updates per second: From 30,000 to 30 (1000x reduction)
- FPS maintained: 60 instead of dropping to 10
- Grass still depletes visibly (~17 seconds of continuous eating)

**7. Uint8Array Terrain Storage**

**Problem:** 10,000×10,000 terrain as objects or floats = 400+ MB

**Solution:** 1 byte per pixel (4 terrain types fit in Uint8)

**Implementation:** (grid.js:10-20)
```javascript
grid = new Uint8Array(width × height)
// 10000 × 10000 = 100,000,000 bytes = 95 MB

get(x, y):
  return grid[y × width + x]  // O(1) array access
```

**Impact:**
- Memory: From 400 MB to 95 MB (4.2x reduction)
- Access speed: O(1) direct array indexing
- Cache friendly: Contiguous memory layout

---

## 26. Performance Metrics

**Target Performance:** 60 FPS with 2000 entities

**Frame Budget:** 16.67ms per frame

**Actual Performance Breakdown:**

**Per Frame Time (60 FPS, 2000 entities):**
```
Rendering:                   6-7ms   (40%)
  - Terrain chunks:          2-3ms
  - Entity drawing:          3-4ms
  - Viewport culling saves:  2-3ms

Spatial Hash Update:         3-4ms   (20%)
  - Clear cells:             0.5ms
  - Rebuild from positions:  2.5-3.5ms

Decision System:             2-3ms   (15%)
  - Cache checks:            0.5ms
  - WASM calls:              1-2ms (only ~33 entities/frame with caching)
  - Intent application:      0.5ms

Movement System:             1.5ms   (10%)
  - Direction calculation:   0.3ms
  - Terrain lookups:         0.5ms
  - Position updates:        0.7ms

Consumption System:          0.5ms   (3%)
Needs Decay System:          1.0ms   (6%)
Age System:                  0.5ms   (3%)
Death System:                0.3ms   (2%)

UI Updates:                  0.5ms   (3%)

Total:                       15-17ms (~95% of budget)
```

**Scaling Characteristics:**

| Entity Count | FPS | Frame Time | Bottleneck |
|--------------|-----|------------|------------|
| 500 | 60 | 10ms | None |
| 1000 | 60 | 12ms | None |
| 2000 | 60 | 16ms | Balanced |
| 3000 | 45-50 | 20-22ms | Rendering |
| 5000 | 25-30 | 33-40ms | Rendering + Spatial Hash |

**Memory Usage:**

```
Terrain Grid:                95 MB
Terrain Chunks (cached):     50 MB (361 chunks × ~140 KB each)
Resource Cache:              2 MB
Spatial Hash:                1 MB
ECS Components (2000):       120 KB
WASM Module:                 150 KB
JavaScript Code:             50 KB

Total:                       ~150 MB
```

**Query Complexity:**

| Operation | Complexity | Time (2000 entities) |
|-----------|------------|----------------------|
| Terrain lookup | O(1) | <0.001ms |
| Spatial hash update | O(n) | 3ms |
| Nearby entity query | O(cells × density) | 0.01ms |
| Nearby resource query | O(log dist) | 0.05ms |
| WASM decision | O(1) per call | 0.06ms |

**Optimization Priorities** (if FPS drops):

1. **Reduce entity count** - Biggest impact on all systems
2. **Increase decision cache time** - From 120 to 240 frames (4 sec)
3. **Reduce resource query radius** - From 500px to 300px
4. **Disable grass depletion** - Removes terrain update cost
5. **Increase chunk size** - From 512×512 to 1024×1024 (fewer chunks)
6. **Lower resolution** - Render at 0.75x scale, upscale canvas

---

# Part VIII: Appendices

## 27. Entity State Enum Reference

**EntityState Values** (config.js:47-52):

| Value | Name | Description | Entry Condition | Exit Condition |
|-------|------|-------------|-----------------|----------------|
| 0 | IDLE | Waiting for decision or at resource | Arrival at target, consumption complete, spawn | New decision applied, resource detected |
| 1 | MOVING | Traveling toward target | Intent with target applied | Arrival (distance ≤ 5px), stuck, new urgent intent |
| 2 | EATING | Consuming food | At grass (rabbit) or near rabbit (human) + hunger > 7 | Hunger < 7, resource depleted |
| 3 | DRINKING | Consuming water | On/adjacent to water + thirst > 7 | Thirst < 7 |
| 4 | SLEEPING | Resting to recover | REST intent applied, tiredness ≥ 100, energy < 10 | Tiredness < 7, new urgent intent |

**Valid State Transitions:**

```
IDLE ←→ MOVING
  ↕       ↕
EATING  DRINKING  SLEEPING

All consuming states (EATING, DRINKING, SLEEPING) can transition to MOVING if urgent intent interrupts
```

**State Transition Triggers:**

| From | To | System | Condition |
|------|-----|--------|-----------|
| IDLE | MOVING | DecisionSystem | Intent with target applied |
| MOVING | IDLE | MovementSystem | Arrival detected (distance ≤ 5px) |
| IDLE | EATING | ConsumptionSystem | At food + hunger > 7 |
| IDLE | DRINKING | ConsumptionSystem | At water + thirst > 7 |
| IDLE | SLEEPING | DecisionSystem | REST intent applied |
| EATING/DRINKING/SLEEPING | IDLE | ConsumptionSystem | Need satisfied (< 7) |
| ANY | MOVING | DecisionSystem | Urgent intent (urgency delta > 20 or 40) |

---

## 28. Intent Type Reference

**IntentType Values** (config.js:55-59):

| Intent | Generated When | Target | Urgency Calculation | Fallback |
|--------|----------------|--------|---------------------|----------|
| SEEK_WATER | Thirst > 50 typically | Nearest water tile (terrain-cost weighted) | Equal to thirst level (0-100) | WANDER if no water within 1000px |
| SEEK_FOOD | Hunger > 50 typically | Rabbit (humans) or grass (rabbits) | Equal to hunger level (0-100) | WANDER if no food within 1000px |
| REST | Tiredness > 70 OR energy < 30 | Current position (no movement) | Max(tiredness, 100 - energy) | Always valid |
| WANDER | All needs < 50 | Random direction, terrain-weighted | 0 (lowest priority) | Always valid |

**Intent Priority Rules:**

1. **Emergency overrides** (always interrupt):
   - Tiredness ≥ 100 → REST
   - Energy < 10 → REST

2. **Interrupt threshold:**
   - Normal: New urgency must exceed current by 20 points
   - While consuming: New urgency must exceed current by 40 points

3. **Urgency adjustment:**
   - Base urgency = need level (0-100)
   - Terrain-cost adjusted: urgency × min(1.0, 1000 / pathCost)
   - If adjusted urgency < 20 and energy > 50: Fall back to WANDER

**Intent Validation:**

Before applying intent, DecisionSystem checks:
- Resource availability (query for water/food)
- Path cost to resource (terrain-weighted distance)
- Urgency vs current activity (interrupt threshold)

If validation fails → fall back to WANDER

---

## 29. Terrain Type Reference

**TerrainType Values** (config.js:62-66):

| Value | Name | Color (RGB) | Walkable | Swimmable | Edible | Drinkable |
|-------|------|-------------|----------|-----------|--------|-----------|
| 0 | WATER | (50, 100, 200) Blue | Yes | Yes | No | Yes |
| 1 | GRASS | (50, 150, 50) Green | Yes | No | Yes (rabbits) | No |
| 2 | ROCKY | (128, 128, 128) Gray | No | No | No | No |
| 3 | DIRT | (139, 90, 43) Brown | Yes | No | No | No |

**Species Movement Costs:**

| Terrain | Human Speed | Human Energy | Rabbit Speed | Rabbit Energy |
|---------|-------------|--------------|--------------|---------------|
| WATER | 0.25x | 3.0x | 0.1x | 4.0x |
| GRASS | 1.0x | 1.0x | 1.0x | 1.0x |
| ROCKY | 0.3x | 3.0x | 0.5x | 2.0x |
| DIRT | 1.0x | 1.0x | 1.0x | 1.0x |

**Terrain Distribution** (typical 10,000×10,000 world):

- Water: ~30% (30M pixels, 5.8M after sampling)
- Grass: ~30% (30M pixels, 5.8M after sampling)
- Rocky: ~20% (20M pixels, impassable)
- Dirt: ~20% (20M pixels, barren)

**Terrain Modification:**

- **Grass Depletion:** GRASS → DIRT when rabbits eat (0.1% chance/frame)
- **Permanent:** No regeneration implemented
- **Impact:** Creates deserts in overpopulated areas

---

## 30. Quick Reference Tables

### System Execution Order

| Order | System | Responsibility | Output |
|-------|--------|----------------|--------|
| 1 | NeedsDecaySystem | Increase needs, apply energy costs | Updated Needs, Energy |
| 2 | AgeSystem | Calculate age, adjust max energy | Updated Energy.max |
| 3 | DecisionSystem | Generate intents via WASM | Updated Target, State |
| 4 | MovementSystem | Move toward targets | Updated Position, Velocity |
| 5 | ConsumptionSystem | Reduce needs at resources | Updated Needs, State |
| 6 | DeathSystem | Remove dead entities | Fewer entities |

### Component-to-System Mapping

| Component | Read By | Written By |
|-----------|---------|------------|
| Position | All systems, SpatialHash | MovementSystem, user input |
| Velocity | MovementSystem | MovementSystem, DecisionSystem |
| Needs | NeedsDecaySystem, DecisionSystem, DeathSystem | NeedsDecaySystem, ConsumptionSystem |
| Energy | NeedsDecaySystem, AgeSystem, DecisionSystem, DeathSystem | NeedsDecaySystem, AgeSystem |
| Age | AgeSystem | AgeSystem, createRabbit/createHuman |
| Target | MovementSystem, DecisionSystem | DecisionSystem |
| State | All systems | DecisionSystem, MovementSystem, ConsumptionSystem |
| Species | All systems | createRabbit/createHuman (immutable) |
| Gender | RenderSystem | createRabbit/createHuman (immutable) |

### File Organization Map

| Directory | Purpose | Key Files |
|-----------|---------|-----------|
| `/` | Entry points | sketch.js, index.html |
| `/src/` | Core logic | config.js, world-query.js, libreconomy-wasm-bridge.js |
| `/src/ecs/` | Entity system | components.js, world.js |
| `/src/ecs/systems/` | Game logic | decision.js, movement.js, consumption.js, needs.js, age.js, render.js, camera.js, terrain.js |
| `/src/terrain/` | World generation | grid.js, generator.js, storage.js, resource-cache.js |
| `/src/ui/` | User interface | loading-overlay.js |
| `/pkg/` | WASM bindings | libreconomy.js, libreconomy_bg.wasm |
| `/docs/` | Documentation | tech-doc-libreterra.md (this file) |

### Key Constants Summary

| Constant | Value | Purpose |
|----------|-------|---------|
| WORLD_SIZE | 10000 | World dimensions in pixels |
| TARGET_FPS | 60 | Frame rate target |
| INITIAL_HUMANS | 800 | Starting human count |
| INITIAL_RABBITS | 1200 | Starting rabbit count |
| CHUNK_SIZE | 512 | Terrain chunk size (pixels) |
| CELL_SIZE | 100 | Spatial hash cell size (pixels) |
| ARRIVAL_THRESHOLD | 5 | Distance to consider "arrived" (pixels) |
| SATISFIED_THRESHOLD | 7 | Need level to stop consuming |
| DECISION_CACHE_TIME | 120 | Frames before re-deciding (2 sec @ 60 FPS) |
| DECISION_CHANGE_THRESHOLD | 15 | Need change to invalidate cache |
| INTERRUPT_THRESHOLD | 20 | Urgency delta to interrupt normal activity |
| CONSUMING_INTERRUPT_THRESHOLD | 40 | Urgency delta to interrupt consumption |

---

## Movement Issues - Debugging Guide

Since you mentioned there are issues with entity movement, here are the key areas to investigate:

### Where Movement Can Go Wrong

**1. Stuck in Obstacles:**
- Check: MovementSystem.validateTerrainAndSlide() (movement.js:195-250)
- Symptom: Entity stops moving, hasTarget=0, velocity=(0,0)
- Cause: All 5 slide attempts failed (blocked by rocky terrain)
- Fix: Improve slide logic or pathfinding around concave obstacles

**2. NaN Positions:**
- Check: MovementSystem.calculateMovement() (movement.js:60-160)
- Symptom: Entities disappear or teleport to (0,0)
- Cause: Division by zero in direction normalization
- Prevention: NaN check at movement.js:170

**3. Jittery Movement:**
- Check: MovementSystem velocity calculation (movement.js:150-160)
- Symptom: Entities oscillate around target
- Cause: Arrival threshold too small (< 5px)
- Fix: Ensure ARRIVAL_THRESHOLD = 5

**4. Slow Movement on Good Terrain:**
- Check: SPECIES_TERRAIN_ATTRIBUTES (config.js:140-160)
- Symptom: Entities move slowly on grass/dirt
- Cause: Speed multiplier < 1.0 incorrectly set
- Verify: Grass/Dirt should be 1.0x speed

**5. Entities Not Reaching Resources:**
- Check: DecisionSystem.applyIntent() target assignment (decision.js:195-240)
- Symptom: Entities wander instead of going to water/food
- Cause: Resource query returning empty or high-cost resources
- Debug: Log getNearbyResources() results

**6. Constant State Switching:**
- Check: DecisionSystem interrupt threshold (decision.js:285-295)
- Symptom: Entity switches MOVING ↔ EATING rapidly
- Cause: Interrupt threshold too low (< 20)
- Fix: Ensure consuming uses 40-point threshold

**7. Movement Through Obstacles:**
- Check: TerrainGrid.isWalkable() (grid.js:35-40)
- Symptom: Entities walk through rocky terrain
- Cause: Walkable check not enforced
- Verify: validateTerrainAndSlide stops movement on ROCKY

**Key Files for Movement Debugging:**
- `src/ecs/systems/movement.js` - Core movement logic
- `src/ecs/systems/decision.js` - Target assignment
- `src/world-query.js` - Resource/entity queries
- `src/terrain/grid.js` - Terrain validation
- `src/config.js` - Speed/cost constants

**Logging Recommendations:**

Add temporary logging to diagnose:
```javascript
// In MovementSystem.update()
console.log(`Entity ${eid}: State=${State.current[eid]}, ` +
            `Target=(${Target.x[eid]}, ${Target.y[eid]}), ` +
            `HasTarget=${Target.hasTarget[eid]}, ` +
            `Velocity=(${Velocity.vx[eid]}, ${Velocity.vy[eid]})`);

// In DecisionSystem.applyIntent()
console.log(`Applying ${intent.type} to entity ${entityId}, ` +
            `target=(${target.x}, ${target.y}), urgency=${urgency}`);
```

---

**End of Technical Documentation**

This document provides comprehensive reference for the libreterra codebase. For specific implementation details, refer to the source files indicated throughout. For movement-specific issues, start with the debugging guide above and trace through the decision → movement → consumption pipeline described in Part II.
