# libreterra

A p5.js-based test application for demonstrating and testing [libreconomy](../../README.md) integration.

## Overview

libreterra is a visual simulation of a 10,000x10,000 pixel world populated by humans and rabbits with needs-driven behavior. The simulation demonstrates the clean separation between:
- **libreconomy** (economic/decision logic)
- **Application code** (spatial world, rendering, movement)

Currently using JavaScript stubs for libreconomy API to enable rapid development and testing.

## Features

🌍 **Large Procedural World**
- 10,000x10,000 pixel terrain generated with Perlin noise
- 4 terrain types: Water, Grass, Rocky, Dirt
- Efficient viewport culling for smooth rendering

🤖 **Autonomous Agents**
- Humans (10 initial) that hunt rabbits
- Rabbits (30 initial) that graze on grass
- Needs-driven behavior (hunger, thirst, tiredness)
- Utility-based decision making via libreconomy stub

🎮 **Interactive Simulation**
- Smooth camera controls (pan, zoom, center)
- Real-time stats display (FPS, entity count)
- Pause, speed up, or slow down time
- Watch entities seek resources, eat, drink, and sleep

🔧 **Clean Architecture**
- ECS architecture using bitECS
- Spatial hash for O(1) entity queries
- Decision logic cleanly separated from spatial logic
- Ready for WASM libreconomy integration

## Quick Start

### Running Locally

1. **Open in browser**: Simply open `index.html` in a modern web browser
   - No build step required!
   - All dependencies loaded from CDN

2. **Use a local server** (recommended for development):
   ```bash
   # Python 3
   python -m http.server 8000

   # Python 2
   python -m SimpleHTTPServer 8000

   # Node.js (if you have http-server installed)
   npx http-server
   ```

   Then navigate to `http://localhost:8000`

### Controls

- **Drag**: Pan the camera
- **Scroll**: Zoom in/out
- **Double-click**: Center view on clicked position
- **Space**: Pause/Resume simulation
- **+/-**: Speed up/slow down time
- **R**: Reset camera to center
- **H**: Toggle help overlay

### What to Watch For

Once running, you'll see:
- **Entities moving** toward resources (water, food) when needs are high
- **Rabbits eating grass**, converting green pixels to brown dirt
- **Humans hunting rabbits** when hungry
- **Entities drinking** from water (blue terrain)
- **Entities sleeping** when tired (they stop moving)
- **Grass depletion** over time as rabbits graze
- **Decision logging** in browser console (occasional messages about entity decisions)

## Current Implementation Status

### ✅ **All Phases Complete!**

**Phase 1**: Project setup & camera system
- ✅ Project structure, HTML, configuration
- ✅ Camera pan, zoom, center controls
- ✅ UI overlay with stats

**Phase 2**: Terrain system
- ✅ Perlin noise terrain generation
- ✅ 10,000x10,000 world with 4 terrain types
- ✅ Viewport culling for performance

**Phase 3**: ECS components & entities
- ✅ bitECS component definitions
- ✅ Human and rabbit entity factories
- ✅ Entity rendering with gender-specific colors

**Phase 4**: libreconomy stub & WorldQuery
- ✅ JavaScript decision-making stub
- ✅ Spatial hash for fast entity queries
- ✅ WorldQuery implementation (getNearbyResources, getNearbyEntities)

**Phase 5**: Needs & decision systems
- ✅ Needs decay system (hunger, thirst, tiredness)
- ✅ Decision system using libreconomy stub
- ✅ Utility-based action selection

**Phase 6**: Movement & consumption systems
- ✅ Movement with obstacle avoidance
- ✅ Eating, drinking, sleeping behaviors
- ✅ Grass depletion, rabbit hunting

**Phase 7**: Polish & documentation
- ✅ Complete documentation
- ✅ Fully functional simulation

## Architecture

### Separation of Concerns

**libreconomy-stub.js** (Economic Logic):
- Need-based decision making
- Utility calculations
- Returns Intents (SeekWater, SeekFood, Rest, Wander)

**libreterra** (Spatial/Game Logic):
- 10,000x10,000 world representation
- Entity positions and movement
- Rendering with p5.js
- Camera controls
- Implements WorldQuery trait

See [/docs/ARCHITECTURE.md](../../docs/ARCHITECTURE.md) for detailed integration patterns.

## World Specifications

### Terrain Types
- **Water** (blue pixels) - Doesn't deplete, drunk by all entities
- **Grass** (green pixels) - Eaten by rabbits, converts to dirt
- **Rocky** (gray pixels) - Impassable terrain
- **Dirt** (brown pixels) - Result of grass consumption

### Entities
- **Male humans** (blue circles) - Eat rabbits, drink water
- **Female humans** (pink circles) - Eat rabbits, drink water
- **Male rabbits** (light-blue triangles) - Eat grass, drink water
- **Female rabbits** (dark-pink triangles) - Eat grass, drink water

### Needs System
All entities have three needs:
- **Hunger**: Increases over time, satisfied by eating
- **Thirst**: Increases faster, satisfied by drinking
- **Tiredness**: Increases with activity, satisfied by sleeping

## File Structure

```
libreterra-p5js/
├── index.html              # Main HTML file ✅
├── sketch.js               # p5.js main sketch (setup/draw loop) ✅
├── src/
│   ├── config.js           # Constants and configuration ✅
│   ├── ecs/
│   │   ├── components.js   # bitECS component definitions ✅
│   │   ├── world.js        # ECS world & entity factories ✅
│   │   └── systems/
│   │       ├── camera.js   # Camera pan/zoom/center ✅
│   │       ├── terrain.js  # Terrain management ✅
│   │       ├── needs.js    # Needs decay system ✅
│   │       ├── decision.js # Decision-making system ✅
│   │       ├── movement.js # Movement with obstacle avoidance ✅
│   │       ├── consumption.js # Eating/drinking/sleeping ✅
│   │       └── render.js   # Rendering with viewport culling ✅
│   ├── terrain/
│   │   ├── generator.js    # Perlin noise terrain generation ✅
│   │   └── grid.js         # Terrain grid (Uint8Array) ✅
│   ├── libreconomy-stub.js # Mock libreconomy decision API ✅
│   └── world-query.js      # WorldQuery + SpatialHash ✅
└── README.md              # Documentation ✅
```

## Development Notes

### Technologies Used
- **p5.js** (v1.7.0) - Creative coding library for canvas rendering
- **bitECS** (v0.3.38) - High-performance ECS library
- **Vanilla JavaScript** - No build step required

### Performance Targets
- **30+ FPS** with 50+ entities
- **Smooth camera** controls at all zoom levels
- **Efficient rendering** using viewport culling

### Future: Real libreconomy Integration

Once libreconomy's Rust library is ready, we can replace the JavaScript stubs with real WASM bindings:

1. Compile libreconomy to WebAssembly using `wasm-pack`
2. Replace `src/libreconomy-stub.js` with WASM module
3. Update `src/world-query.js` to match final API
4. No changes needed to rendering or spatial logic!

## Contributing

This is a test application for libreconomy development. Contributions welcome!

See the main [libreconomy repository](../..) for contribution guidelines.

## License

Same as libreconomy (see repository root).
