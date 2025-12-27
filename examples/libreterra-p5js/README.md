# libreterra

A p5.js-based demonstration of [libreconomy](../../README.md) integration with WebAssembly.

## Overview

libreterra is a visual simulation of a 10,000x10,000 pixel world populated by humans and rabbits with needs-driven behavior. The simulation demonstrates the clean separation between:
- **libreconomy** (economic/decision logic via WASM)
- **Application code** (spatial world, rendering, movement)

**Now using real libreconomy WASM integration** for decision-making with JavaScript bridge and fallback stubs.

## Features

🌍 **Large Procedural World**
- 10,000x10,000 pixel terrain generated with Perlin noise
- 4 terrain types: Water, Grass, Rocky, Dirt
- Species-specific terrain traversability (rabbits swim poorly, humans climb better)
- Efficient viewport culling for smooth rendering

🤖 **Autonomous Agents**
- Humans (1 initial) - Omnivores that hunt rabbits and drink water
- Rabbits (1 initial) - Herbivores that graze on grass and drink water
- **Full needs system**: Hunger, thirst, tiredness, **energy**
- **Age system** with lifespan curves and max energy decay
- **Energy-aware decision making** with utility-based behavior
- **Desperate exploration** when critically hungry (larger search radius)

🎮 **Interactive Simulation**
- Smooth camera controls (pan, zoom, center, follow entities)
- Real-time stats display (FPS, entity count, selected entity info)
- Pause, speed up, or slow down time
- Bulk spawn/kill entities with hotkeys
- Click entities to view detailed stats (needs, energy, age, terrain, intent)
- Watch entities seek resources, eat, drink, sleep, age, and die

⚡ **Advanced Systems**
- **Energy system** with decay, restoration, and speed penalties
- **Age system** with health-based lifespan and max energy curves
- **Death from multiple causes**: Energy depletion, old age, poor health, critical needs
- **Terrain traversability**: Speed and energy multipliers per species/terrain
- **Species-aware behavior**: Diet types (herbivore, carnivore, omnivore)

🔧 **Clean Architecture**
- ECS architecture using bitECS (0.3.38)
- Spatial hash for O(1) entity queries
- Decision logic via libreconomy WASM with JavaScript bridge
- Full separation between economic and spatial logic

## Quick Start

### Prerequisites

Build the libreconomy WASM module first:
```bash
cd ../../  # Go to libreconomy root
./scripts/build-wasm.sh
```

This generates `pkg/libreconomy.js` and `libreconomy_bg.wasm`.

### Running Locally

1. **Use a local server** (required for WASM):
   ```bash
   # Python 3
   python -m http.server 8000

   # Python 2
   python -m SimpleHTTPServer 8000

   # Node.js (if you have http-server installed)
   npx http-server
   ```

   Then navigate to `http://localhost:8000/examples/libreterra-p5js/`

2. **Browser requirements**: Modern browser with WebAssembly support (Chrome, Firefox, Safari, Edge)

### Controls

**Camera:**
- **Drag**: Pan the camera
- **Scroll**: Zoom in/out
- **Double-click**: Center view on clicked position
- **Click entity**: Select and follow entity (shows detailed info)
- **R**: Reset camera to center

**Simulation:**
- **Space**: Pause/Resume simulation
- **+/-**: Speed up/slow down time
- **H**: Toggle help overlay

**Entity Management:**
- **1-5**: Spawn 1-5 humans at random location
- **Q-Y**: Spawn 1-5 rabbits at random location
- **Shift+K**: Kill all entities (emergency reset)

### What to Watch For

Once running, you'll see:
- **Entities moving** toward resources (water, food) when needs are high
- **Energy depletion** during movement (especially across difficult terrain)
- **Speed penalties** when energy is low (<20%)
- **Desperate exploration** when entities are starving (hunger > 70)
- **Rabbits eating grass**, converting green pixels to brown dirt
- **Humans hunting rabbits** when hungry
- **Entities drinking** from water (blue terrain)
- **Entities sleeping** when tired or low energy (they stop moving)
- **Energy restoration** during sleep (graduated based on nutrition)
- **Entities aging** - max energy decreases after 50% lifespan
- **Death from**:
  - Energy depletion (0 energy → instant death)
  - Old age (100% of expected lifespan)
  - Poor health (prolonged critical energy levels)
  - Critical needs (thirst/hunger at maximum for too long)
- **Terrain avoidance** - rabbits rarely enter water (0.1x speed, 4x energy cost)
- **Selected entity info** - click an entity to see age, terrain type, energy, intent

## Current Implementation Status

### ✅ **All Core Phases Complete!**

**Phase 1-7**: Foundation complete
- ✅ Camera system, terrain, ECS, WorldQuery, needs, movement, consumption

**Phase 8**: Energy & Age Systems ⭐ **NEW**
- ✅ Energy component with current/max tracking
- ✅ Energy decay based on state (IDLE: 0.5x, MOVING: 1.2x, SLEEPING: -2.5x restoration)
- ✅ Energy-based speed penalties (critical <20%: 70% speed, low <50%: scaled)
- ✅ Energy restoration during sleep (minimum 25% even when hungry/thirsty)
- ✅ Age system with birth frame tracking
- ✅ Age-based max energy curves (20% lower childhood, peak at 50%, decline to 0 at death)
- ✅ Health-based lifespan modifiers (healthy: +20%, unhealthy: -20%)
- ✅ Death from energy depletion, old age, and poor health

**Phase 9**: Terrain Traversability ⭐ **NEW**
- ✅ Species-specific terrain attributes (speed/energy multipliers)
- ✅ Humans: 25% speed in water (3x energy), 30% on rocky terrain (3x energy)
- ✅ Rabbits: 10% speed in water (4x energy), 50% on rocky terrain (2x energy)
- ✅ Sliding collision with 8-direction obstacle avoidance
- ✅ Movement system applies terrain costs to speed and energy

**Phase 10**: Energy-Aware Decision Making ⭐ **NEW**
- ✅ Energy synced to WASM decision maker
- ✅ Energy-aware urgency calculations
- ✅ Desperate exploration mode (hunger > 70: 400-1000 pixel wander radius)
- ✅ Normal exploration (200-600 pixel wander radius, up from 100-300)
- ✅ Terrain-cost-aware wander target selection

**Phase 11**: WASM Integration ⭐ **NEW**
- ✅ libreconomy WASM bridge with fallback to JavaScript stubs
- ✅ TypeScript definitions for type safety
- ✅ Decision-making via WasmDecisionMaker
- ✅ WorldQuery implementation for WASM

**Phase 12**: UI & Polish ⭐ **NEW**
- ✅ Entity info panel with terrain type display
- ✅ Age in real-world months display
- ✅ Bulk spawn/kill hotkeys (1-5, Q-Y, Shift+K)
- ✅ Entity selection and following

## Architecture

### Separation of Concerns

**libreconomy WASM** (Economic Logic):
- Need-based decision making (hunger, thirst, tiredness, energy)
- Utility calculations with configurable thresholds
- Species-aware decisions (considers diet type)
- Returns Intents (SeekWater, SeekFood, Rest, Wander)

**libreconomy-wasm-bridge.js** (Integration Layer):
- JavaScript bridge to WASM decision maker
- Fallback to JavaScript stubs if WASM unavailable
- WorldQuery implementation
- Decision validation and intent conversion

**libreterra** (Spatial/Game Logic):
- 10,000x10,000 world representation
- Entity positions and movement with terrain costs
- Energy and age tracking
- Rendering with p5.js
- Camera controls
- Implements WorldQuery trait for libreconomy

See [/docs/ARCHITECTURE.md](../../docs/ARCHITECTURE.md) for detailed integration patterns.

## World Specifications

### Terrain Types

All terrain types have species-specific traversability:

- **Water** (blue pixels)
  - Humans: 25% speed, 3x energy cost (can swim slowly)
  - Rabbits: 10% speed, 4x energy cost (poor swimmers, avoid when possible)
  - Drunk by all entities, doesn't deplete

- **Grass** (green pixels)
  - All species: 100% speed, 1x energy (optimal terrain)
  - Eaten by rabbits, converts to dirt
  - Preferred terrain for wandering

- **Rocky** (gray pixels)
  - Humans: 30% speed, 3x energy cost (can climb slowly)
  - Rabbits: 50% speed, 2x energy cost (better climbers than swimmers)
  - Difficult but traversable terrain

- **Dirt** (brown pixels)
  - All species: 100% speed, 1x energy (optimal terrain)
  - Result of grass consumption

### Entities

- **Male humans** (blue circles) - Omnivores
  - Eat rabbits, drink water
  - Base speed: 2.0 pixels/frame
  - Base max energy: 100
  - Lifespan: ~72 simulation minutes (100 sim years)

- **Female humans** (pink circles) - Omnivores
  - Eat rabbits, drink water
  - Base speed: 2.0 pixels/frame
  - Base max energy: 100
  - Lifespan: ~72 simulation minutes (100 sim years)

- **Male rabbits** (light-blue triangles) - Herbivores
  - Eat grass, drink water
  - Base speed: 3.0 pixels/frame
  - Base max energy: 80
  - Lifespan: ~36 simulation minutes (50 sim years)

- **Female rabbits** (dark-pink triangles) - Herbivores
  - Eat grass, drink water
  - Base speed: 3.0 pixels/frame
  - Base max energy: 80
  - Lifespan: ~36 simulation minutes (50 sim years)

### Needs & Energy System

All entities have four needs:

- **Hunger**: Increases over time (0.02/frame base), satisfied by eating
  - Critical threshold: 80+
  - Species modifiers: Rabbits 1.3x faster

- **Thirst**: Increases faster (0.025/frame base), satisfied by drinking
  - Critical threshold: 60+
  - Activity multiplier: 2.0x when moving

- **Tiredness**: Increases with activity (0.015/frame base), satisfied by sleeping
  - Critical threshold: 70+
  - Activity multiplier: 2.5x when moving

- **Energy**: Fitness/health tracking (0.005/frame base)
  - Decays faster when moving (1.2x multiplier)
  - Restores during sleep (-2.5x multiplier)
  - Restoration scaled by hunger/thirst (minimum 25% even when starving)
  - Affects movement speed (<20%: 70% speed, <50%: scaled penalty)
  - Death occurs at 0 energy

### Age System

- **Birth**: Entities spawn with birthFrame = current frame
- **Childhood** (0-20% lifespan): 80-100% of base max energy
- **Adulthood** (20-50% lifespan): 100% of base max energy (peak)
- **Decline** (50-100% lifespan): 100% → 0% of base max energy
- **Health modifiers**:
  - Good health (avg energy >70%): Up to +20% lifespan
  - Poor health (avg energy <30%): Down to -20% lifespan
- **Death causes**:
  - Energy depletion (instant)
  - Old age (100% of expected lifespan)
  - Poor health (prolonged critical energy, age >20%)

## File Structure

```
libreterra-p5js/
├── index.html              # Main HTML file ✅
├── sketch.js               # p5.js main sketch (setup/draw loop) ✅
├── README.md              # This documentation ✅
└── src/
    ├── config.js           # Constants and configuration ✅
    ├── ecs/
    │   ├── components.js   # bitECS component definitions ✅
    │   ├── world.js        # ECS world & entity factories ✅
    │   └── systems/
    │       ├── camera.js   # Camera pan/zoom/center ✅
    │       ├── render.js   # Rendering with viewport culling ✅
    │       ├── needs.js    # Needs decay & energy system ✅
    │       ├── age.js      # Age & lifespan management ✅ NEW
    │       ├── decision.js # Decision-making system ✅
    │       ├── movement.js # Movement with terrain costs ✅
    │       └── consumption.js # Eating/drinking/sleeping ✅
    ├── terrain/
    │   ├── generator.js    # Perlin noise terrain generation ✅
    │   └── grid.js         # Terrain grid (Uint8Array) ✅
    ├── libreconomy-wasm-bridge.js # WASM bridge + fallback stubs ✅
    └── world-query.js      # WorldQuery + SpatialHash ✅
```

## Development Notes

### Technologies Used
- **p5.js** (v1.7.0) - Creative coding library for canvas rendering
- **bitECS** (v0.3.38) - High-performance ECS library
- **libreconomy WASM** - Decision-making via WebAssembly
- **Vanilla JavaScript** - Minimal build requirements

### Performance Targets
- **30+ FPS** with 50+ entities ✅
- **Smooth camera** controls at all zoom levels ✅
- **Efficient rendering** using viewport culling ✅
- **O(1) spatial queries** using spatial hash ✅

### Performance Achieved
- Stable 60 FPS with 40 entities
- 10,000×10,000 world with smooth panning/zooming
- Efficient terrain-cost calculations
- Real-time decision-making via WASM

## libreconomy Integration Status

### ✅ Fully Integrated
- Decision-making via WASM `WasmDecisionMaker`
- WorldQuery implementation for proximity queries
- Need satisfaction tracking
- Species-aware behavior (diet types)
- Utility-based decision making
- Energy component synchronization

### 🔧 Using Bridge/Stubs
- Market systems (planned)
- Trading protocols (planned)
- Employment systems (planned)

The simulation currently uses real libreconomy WASM for core decision-making, with fallback JavaScript stubs for systems still under development in the main library.

## Contributing

This is a demonstration application for libreconomy integration patterns. Contributions welcome!

See the main [libreconomy repository](../..) for contribution guidelines.

## License

Same as libreconomy (see repository root).
