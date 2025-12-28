// Main p5.js sketch for libreterra
console.log('✓ sketch.js loading...');

// Disable friendly errors for performance boost
p5.disableFriendlyErrors = true;

// Check if p5.js is loaded
if (typeof p5 === 'undefined') {
  console.error('FATAL: p5.js is not loaded! Check CDN connection.');
} else {
  console.log('✓ p5.js is available');
  console.log('✓ createCanvas available:', typeof createCanvas !== 'undefined');
  console.log('✓ setup will be called by p5.js when ready');
}

// Global state
let camera;
let terrainGrid;
let terrainGenerator;
let terrainSystem;
let renderSystem;
let ecsWorld;
let spatialHash;
let worldQuery;
let libreconomyStub;
let needsDecaySystem;
let ageSystem;
let decisionSystem;
let movementSystem;
let consumptionSystem;
let deathSystem;
let isPaused = false;
let timeScale = 1.0;
let frameCounter = 0;
let fpsDisplay = 0;
let selectedEntity = null;

// New: Terrain persistence and UI
let terrainStorage;
let loadingOverlay;
let setupComplete = false;

// Selection history for entity cycling
let selectionHistory = [];
let lastHistoryCleanFrame = 0;

// Entity grabbing state
let isGrabbingEntity = false;
let grabbedEntity = null;
let grabbedEntityOriginalState = null;  // Store original state for restoration

// p5.js setup function (async for terrain loading)
async function setup() {
  try {
    window.setupCalled = true;
    console.log('✓ setup() called');

    // Create canvas
    const canvas = createCanvas(windowWidth, windowHeight);
    canvas.parent('canvas-container');

    // Initialize camera
    camera = new CameraSystem();

    // Initialize loading overlay
    loadingOverlay = new LoadingOverlay();
    loadingOverlay.show("Initializing IndexedDB...");

    // Force initial render of loading overlay
    background(20);
    loadingOverlay.render(window);

    // Initialize terrain storage
    terrainStorage = new TerrainStorage();
    await terrainStorage.init();
    console.log('✓ IndexedDB initialized');

    // Try to load cached terrain
    loadingOverlay.show("Checking for cached terrain...");
    background(20);
    loadingOverlay.render(window);

    const cachedTerrain = await terrainStorage.loadTerrain();

    if (cachedTerrain) {
      // Load from cache
      loadingOverlay.show("Loading terrain from cache...");
      background(20);
      loadingOverlay.render(window);

      terrainGrid = new TerrainGrid(cachedTerrain.width, cachedTerrain.height);
      terrainGrid.data = cachedTerrain.data;  // Direct assignment of Uint8Array
      terrainGenerator = new TerrainGenerator(cachedTerrain.seed);
      console.log(`✓ Loaded terrain from cache (seed: ${cachedTerrain.seed})`);
    } else {
      // Generate new terrain
      console.log('No cached terrain found, generating new terrain...');
      terrainGrid = new TerrainGrid(CONFIG.WORLD_WIDTH, CONFIG.WORLD_HEIGHT);
      terrainGenerator = new TerrainGenerator();  // Random seed

      // Generate with progress tracking
      await generateTerrainWithProgress(terrainGrid, terrainGenerator);

      // Save to cache
      loadingOverlay.show("Saving terrain to cache...");
      background(20);
      loadingOverlay.render(window);

      await terrainStorage.saveTerrain(terrainGrid, terrainGenerator.seed);
      console.log(`✓ Generated and cached terrain (seed: ${terrainGenerator.seed})`);
    }

  // Initialize systems
  loadingOverlay.show("Initializing systems...");
  background(20);
  loadingOverlay.render(window);

  terrainSystem = new TerrainSystem(terrainGrid);
  renderSystem = new RenderSystem(terrainGrid);

  // Initialize terrain image
  loadingOverlay.show("Converting terrain to image...");
  background(20);
  loadingOverlay.render(window);

  renderSystem.initializeTerrainImage();
  console.log('✓ Terrain image ready');

  // Build resource cache
  loadingOverlay.show("Building resource cache...");
  background(20);
  loadingOverlay.render(window);

  const resourceCache = new ResourceCache(CONFIG.SPATIAL_GRID.CELL_SIZE);
  resourceCache.buildFromTerrain(terrainGrid);
  terrainGrid.resourceCache = resourceCache;
  console.log('✓ Resource cache ready');

  // Wire up terrain change callbacks
  terrainGrid.onTerrainChange = (x, y, newTerrainType) => {
    renderSystem.updateTerrainPixel(x, y, newTerrainType);
  };

  // Initialize ECS world
  console.log('Initializing ECS world...');
  ecsWorld = createECSWorld();

  // Spawn initial entities
  const spawnedEntities = spawnInitialEntities(ecsWorld, terrainGrid, frameCounter);
  console.log(`Total entities in world: ${allEntitiesQuery(ecsWorld).length}`);

  // Initialize spatial hash and world query (Phase 4)
  spatialHash = new SpatialHash(100); // 100x100 pixel cells
  spatialHash.update(ecsWorld);
  worldQuery = new WorldQuery(ecsWorld, terrainGrid, spatialHash, resourceCache);

  // Initialize decision-making systems (Phase 5)
  libreconomyStub = new LibreconomyWasmBridge();
  libreconomyStub.initialize(WasmWorld, WasmDecisionMaker);
  console.log('✓ WASM bridge initialized');
  needsDecaySystem = new NeedsDecaySystem(terrainGrid);
  ageSystem = new AgeSystem();
  decisionSystem = new DecisionSystem(libreconomyStub, worldQuery);

  // Initialize movement and consumption systems (Phase 6)
  movementSystem = new MovementSystem(terrainGrid);
  consumptionSystem = new ConsumptionSystem(terrainGrid, decisionSystem);

  // Initialize death system
  deathSystem = new DeathSystem(terrainGrid);

  // Set frame rate
  frameRate(CONFIG.SIMULATION.TARGET_FPS);

    // Hide loading overlay
    loadingOverlay.hide();
    setupComplete = true;

    console.log('libreterra initialized successfully!');
    console.log(`World size: ${CONFIG.WORLD_WIDTH}x${CONFIG.WORLD_HEIGHT}`);
    console.log(`Camera: (${Math.round(camera.x)}, ${Math.round(camera.y)}) @ ${camera.zoom}x`);
    console.log(`Press SPACE to center on a random entity`);

    // Start stats logging after initialization
    setInterval(logStats, 25000);
  } catch (error) {
    console.error('FATAL ERROR in setup():', error);
    console.error('Stack trace:', error.stack);
    throw error;
  }
}

// Helper: Generate terrain with progress tracking
async function generateTerrainWithProgress(terrainGrid, terrainGenerator) {
  return new Promise((resolve) => {
    const chunkSize = 1000;  // Process 1000 rows at a time
    const generator = terrainGenerator.generateChunked(terrainGrid, chunkSize);

    function processChunk() {
      const result = generator.next();

      if (!result.done) {
        const progress = parseFloat(result.value.progress);  // 0-100

        // Update loading overlay
        loadingOverlay.show(`Generating terrain... ${Math.round(progress)}%`, progress);
        background(20);
        loadingOverlay.render(window);

        setTimeout(processChunk, 0);  // Yield to browser
      } else {
        resolve();
      }
    }

    processChunk();
  });
}

// Helper: Show loading overlay
function showLoadingOverlay(message, progress = -1) {
  if (loadingOverlay) {
    loadingOverlay.show(message, progress);
  }
}

// Helper: Hide loading overlay
function hideLoadingOverlay() {
  if (loadingOverlay) {
    loadingOverlay.hide();
  }
}

// p5.js draw function (main game loop)
function draw() {
  const frameStart = performance.now();

  background(20);

  // If setup is not complete, just show loading overlay
  if (!setupComplete) {
    if (loadingOverlay) {
      loadingOverlay.render(window);
    }
    return;
  }

  // Update FPS counter
  if (frameCounter % 30 === 0) {
    fpsDisplay = Math.round(frameRate());
    document.getElementById('fps').textContent = fpsDisplay;
  }
  frameCounter++;

  // Performance profiling - log every 60 frames (1 second at 60fps)
  // const shouldProfile = frameCounter % 60 === 0;
  const shouldProfile = false; // temporary reduce log spam
  let t1, t2, t3, t4, t5, t6, t7, t8, t9;

  if (shouldProfile) t1 = performance.now();

  // Update spatial hash (Phase 4)
  spatialHash.update(ecsWorld);

  if (shouldProfile) t2 = performance.now();

  // Update camera following (before apply)
  camera.update(ecsWorld);

  // Update grabbed entity position (if grabbing)
  if (isGrabbingEntity && grabbedEntity !== null) {
    updateGrabbedEntityPosition();
  }

  // Apply camera transform
  camera.apply();

  if (shouldProfile) t3 = performance.now();

  // Render terrain
  renderSystem.renderTerrain(camera);

  if (shouldProfile) t4 = performance.now();

  // Render entities
  renderSystem.renderEntities(ecsWorld, camera, selectedEntity);

  if (shouldProfile) t5 = performance.now();

  // Draw world bounds
  stroke(255, 255, 0);
  strokeWeight(2 / camera.zoom);
  noFill();
  rect(0, 0, CONFIG.WORLD_WIDTH, CONFIG.WORLD_HEIGHT);

  // Reset camera transform
  camera.reset();

  if (shouldProfile) t6 = performance.now();

  // Update systems (if not paused)
  if (!isPaused) {
    if (shouldProfile) {
      // Detailed system profiling
      const s1 = performance.now();
      needsDecaySystem.update(ecsWorld, timeScale);
      const s2 = performance.now();
      ageSystem.update(ecsWorld, frameCounter);
      const s3 = performance.now();
      decisionSystem.update(ecsWorld, frameCounter);
      const s4 = performance.now();
      movementSystem.update(ecsWorld, timeScale);
      const s5 = performance.now();
      consumptionSystem.update(ecsWorld);
      const s6 = performance.now();
      deathSystem.update(ecsWorld);
      const s7 = performance.now();

      console.log(`  System breakdown:
    Needs: ${(s2-s1).toFixed(1)}ms
    Age: ${(s3-s2).toFixed(1)}ms
    Decision: ${(s4-s3).toFixed(1)}ms
    Movement: ${(s5-s4).toFixed(1)}ms
    Consumption: ${(s6-s5).toFixed(1)}ms
    Death: ${(s7-s6).toFixed(1)}ms`);
    } else {
      updateSystems();
    }
  }

  if (shouldProfile) t7 = performance.now();

  // Update UI
  camera.updateUI();
  updateEntityCountUI();
  updateEntityInfoUI();

  if (shouldProfile) {
    t8 = performance.now();
    const frameEnd = performance.now();
    const total = frameEnd - frameStart;
    console.log(`⏱️ Frame timing (${total.toFixed(1)}ms total):
  Spatial: ${(t2-t1).toFixed(1)}ms
  Camera: ${(t3-t2).toFixed(1)}ms
  Terrain: ${(t4-t3).toFixed(1)}ms
  Entities: ${(t5-t4).toFixed(1)}ms
  Bounds: ${(t6-t5).toFixed(1)}ms
  Systems: ${(t7-t6).toFixed(1)}ms
  UI: ${(t8-t7).toFixed(1)}ms`);
  }

  // Clean selection history periodically (every 300 frames = 5 seconds at 60 FPS)
  if (frameCounter - lastHistoryCleanFrame > 300) {
    cleanSelectionHistory();
    lastHistoryCleanFrame = frameCounter;
  }

  // Render loading overlay (if visible)
  if (loadingOverlay) {
    loadingOverlay.render(window);
  }
}

// Update entity count in UI (with gender breakdown)
function updateEntityCountUI() {
  const allEntities = allEntitiesQuery(ecsWorld);

  let maleHumans = 0, femaleHumans = 0;
  let maleRabbits = 0, femaleRabbits = 0;

  for (const eid of allEntities) {
    const isMale = Gender.isMale[eid] === 1;
    const species = SpeciesComponent.type[eid];

    if (species === Species.HUMAN) {
      isMale ? maleHumans++ : femaleHumans++;
    } else if (species === Species.RABBIT) {
      isMale ? maleRabbits++ : femaleRabbits++;
    }
  }

  // Calculate totals
  const totalHumans = maleHumans + femaleHumans;
  const totalRabbits = maleRabbits + femaleRabbits;
  const totalEntities = totalHumans + totalRabbits;

  // Update HTML
  document.getElementById('total-count').textContent = totalEntities;
  document.getElementById('humans-count').textContent = `M:${maleHumans} F:${femaleHumans} T:${totalHumans}`;
  document.getElementById('rabbits-count').textContent = `M:${maleRabbits} F:${femaleRabbits} T:${totalRabbits}`;
}

// Update entity info UI
function updateEntityInfoUI() {
  const infoPanel = document.getElementById('entity-info');
  const infoContent = document.getElementById('entity-info-content');

  if (selectedEntity === null) {
    infoPanel.classList.remove('visible');
    return;
  }

  // Check if entity still exists
  const entities = allEntitiesQuery(ecsWorld);
  if (!entities.includes(selectedEntity)) {
    selectedEntity = null;
    camera.stopFollowing();  // Stop following dead entity
    infoPanel.classList.remove('visible');
    return;
  }

  // Show panel
  infoPanel.classList.add('visible');

  // Get entity data
  const species = SpeciesComponent.type[selectedEntity];
  const isMale = Gender.isMale[selectedEntity];
  const state = State.current[selectedEntity];
  const hunger = Needs.hunger[selectedEntity];
  const thirst = Needs.thirst[selectedEntity];
  const tiredness = Needs.tiredness[selectedEntity];
  const x = Position.x[selectedEntity];
  const y = Position.y[selectedEntity];
  const energy = Energy.current[selectedEntity];
  const maxEnergy = Energy.max[selectedEntity];

  // Get terrain type at entity position
  const terrainType = terrainGrid.get(Math.floor(x), Math.floor(y));
  const terrainNames = ['Water', 'Grass', 'Rocky', 'Dirt'];
  const terrainName = terrainNames[terrainType] || 'Unknown';

  // Calculate age in real-world months
  const birthFrame = Age.birthFrame[selectedEntity];
  const currentAge = frameCounter - birthFrame;
  const FRAMES_PER_SIM_MONTH = 360;  // 100 sim years = 432,000 frames at 60 FPS, so 1 month = 360 frames
  const ageInMonths = currentAge / FRAMES_PER_SIM_MONTH;

  // Get species and gender names
  const speciesName = species === Species.HUMAN ? 'Human' : 'Rabbit';
  const genderName = isMale ? 'Male' : 'Female';

  // Get state name
  const stateNames = ['Idle', 'Moving', 'Eating', 'Drinking', 'Sleeping'];
  const stateName = stateNames[state] || 'Unknown';

  // Get intent
  const intent = decisionSystem.getIntent(selectedEntity);
  const intentName = intent ? libreconomyStub.getIntentName(intent.type) : 'None';

  // Build HTML
  let html = `
    <div class="stat-line"><strong>ID:</strong> ${selectedEntity}</div>
    <div class="stat-line"><strong>Type:</strong> ${genderName} ${speciesName}</div>
    <div class="stat-line"><strong>Age:</strong> ${ageInMonths.toFixed(1)} months</div>
    <div class="stat-line"><strong>Position:</strong> (${Math.round(x)}, ${Math.round(y)})</div>
    <div class="stat-line"><strong>Terrain:</strong> ${terrainName}</div>
    <div class="stat-line"><strong>State:</strong> ${stateName}</div>
    <div class="stat-line"><strong>Intent:</strong> ${intentName}</div>
    <div class="stat-line"><strong>Energy:</strong> ${Math.round(energy)}/${Math.round(maxEnergy)}</div>
    <div style="margin-top: 10px; margin-bottom: 5px;"><strong>Needs:</strong></div>
  `;

  // Add need bars
  html += createNeedBar('Hunger', hunger);
  html += createNeedBar('Thirst', thirst);
  html += createNeedBar('Tiredness', tiredness);

  infoContent.innerHTML = html;
}

// Create need bar HTML
function createNeedBar(label, value) {
  const level = value >= 80 ? 'critical' :
                value >= 60 ? 'high' :
                value >= 30 ? 'moderate' : 'low';

  return `
    <div class="need-bar">
      <span class="need-label">${label}:</span>
      <span class="need-value">${Math.round(value)}</span>
      <div class="need-bar-bg">
        <div class="need-bar-fill need-${level}" style="width: ${value}%"></div>
      </div>
    </div>
  `;
}

// Find entity at screen position
function findEntityAtPosition(screenX, screenY) {
  const worldX = camera.screenToWorldX(screenX);
  const worldY = camera.screenToWorldY(screenY);

  const entities = allEntitiesQuery(ecsWorld);
  let closestEntity = null;
  let closestDistance = Infinity;

  // Maximum selection radius in world coordinates (larger radius = easier to click)
  const maxSelectionRadius = 50 / camera.zoom; // Adjust with zoom level

  for (const eid of entities) {
    const x = Position.x[eid];
    const y = Position.y[eid];

    const dx = x - worldX;
    const dy = y - worldY;
    const distance = Math.sqrt(dx * dx + dy * dy);

    // Always track the closest entity within max radius
    if (distance < maxSelectionRadius && distance < closestDistance) {
      closestDistance = distance;
      closestEntity = eid;
    }
  }

  return closestEntity;
}

// Update grabbed entity position to follow mouse
function updateGrabbedEntityPosition() {
  if (grabbedEntity !== null) {
    const worldX = camera.screenToWorldX(mouseX);
    const worldY = camera.screenToWorldY(mouseY);

    // Clamp to world bounds
    Position.x[grabbedEntity] = Math.max(0, Math.min(CONFIG.WORLD_WIDTH - 1, worldX));
    Position.y[grabbedEntity] = Math.max(0, Math.min(CONFIG.WORLD_HEIGHT - 1, worldY));
  }
}

// Drop grabbed entity
function dropGrabbedEntity() {
  if (grabbedEntity !== null) {
    console.log(`Dropped entity ${grabbedEntity} at (${Position.x[grabbedEntity].toFixed(0)}, ${Position.y[grabbedEntity].toFixed(0)})`);

    // Entity stays at drop location
    // Don't restore needs/energy (those were frozen during grab)

    isGrabbingEntity = false;
    grabbedEntity = null;
    grabbedEntityOriginalState = null;
  }
}

// Clean dead entities from selection history
function cleanSelectionHistory() {
  const allEntities = allEntitiesQuery(ecsWorld);
  const aliveSet = new Set(allEntities);

  // Filter out dead entities
  const before = selectionHistory.length;
  selectionHistory = selectionHistory.filter(eid => aliveSet.has(eid));

  const removed = before - selectionHistory.length;
  if (removed > 0) {
    console.log(`Removed ${removed} dead entities from selection history`);
  }

  // If current selection is dead, clear it
  if (selectedEntity !== null && !aliveSet.has(selectedEntity)) {
    selectedEntity = null;
    camera.stopFollowing();
  }
}

// Update all ECS systems
function updateSystems() {
  // Phase 5: Update needs decay system (includes pass out logic)
  needsDecaySystem.update(ecsWorld, timeScale);

  // Update age system (adjusts max energy based on age)
  ageSystem.update(ecsWorld, frameCounter);

  // Phase 5: Update decision system
  decisionSystem.update(ecsWorld, frameCounter);

  // Phase 6: Update movement system
  movementSystem.update(ecsWorld, timeScale);

  // Phase 6: Update consumption system
  consumptionSystem.update(ecsWorld);

  // Freeze grabbed entity's state (restore original values)
  if (isGrabbingEntity && grabbedEntity !== null && grabbedEntityOriginalState !== null) {
    Needs.hunger[grabbedEntity] = grabbedEntityOriginalState.hunger;
    Needs.thirst[grabbedEntity] = grabbedEntityOriginalState.thirst;
    Needs.tiredness[grabbedEntity] = grabbedEntityOriginalState.tiredness;
    Energy.current[grabbedEntity] = grabbedEntityOriginalState.energy;
  }

  // Check for deaths (after all other systems)
  deathSystem.update(ecsWorld);
}

// p5.js mouse pressed handler
function mousePressed() {
  // Middle-click: Pan camera (don't select entities)
  if (mouseButton === CENTER) {
    camera.handleMousePressed();  // Start pan
    return false;
  }

  // Left-click: Select entity or drop grabbed entity
  if (mouseButton === LEFT) {
    // Check if in grabbing mode
    if (isGrabbingEntity) {
      // Drop entity at current location
      dropGrabbedEntity();
      return false;
    }

    // Normal entity selection
    const clickedEntity = findEntityAtPosition(mouseX, mouseY);

    if (clickedEntity !== null) {
      // Select entity
      selectedEntity = clickedEntity;

      // Start camera following
      camera.startFollowing(clickedEntity);

      // Add to selection history if not already present
      if (!selectionHistory.includes(clickedEntity)) {
        selectionHistory.push(clickedEntity);
        console.log(`Selected entity ${clickedEntity} (added to history, ${selectionHistory.length} total)`);
      } else {
        console.log(`Selected entity ${clickedEntity} (already in history)`);
      }
    } else {
      // Deselect if clicking empty space
      if (selectedEntity !== null) {
        selectedEntity = null;

        // Stop camera following
        camera.stopFollowing();

        console.log('Entity deselected');
      }
    }
    return false;
  }
}

// p5.js mouse dragged handler
function mouseDragged() {
  // Only pan on middle-click drag
  if (mouseButton === CENTER) {
    camera.handleMouseDragged();
  }

  // Handle entity grabbing drag
  if (isGrabbingEntity && grabbedEntity !== null) {
    updateGrabbedEntityPosition();
  }

  return false;
}

// p5.js mouse released handler
function mouseReleased() {
  camera.handleMouseReleased();
}

// p5.js mouse wheel handler
function mouseWheel(event) {
  return camera.handleMouseWheel(event);
}

// p5.js double-click handler
function doubleClicked() {
  // Center camera on clicked world position
  const worldX = camera.screenToWorldX(mouseX);
  const worldY = camera.screenToWorldY(mouseY);
  camera.centerOn(worldX, worldY);
  return false; // prevent default
}

// p5.js key pressed handler
function keyPressed() {
  // Space: Center and zoom on random entity
  if (key === ' ') {
    const entities = allEntitiesQuery(ecsWorld);
    if (entities.length > 0) {
      const randomEntity = entities[Math.floor(Math.random() * entities.length)];
      const x = Position.x[randomEntity];
      const y = Position.y[randomEntity];
      camera.centerOn(x, y);
      camera.zoom = 1.0; // Zoom in to see the entity
      selectedEntity = randomEntity;

      // Start camera following
      camera.startFollowing(randomEntity);

      console.log(`Centered on entity ${randomEntity} at (${Math.round(x)}, ${Math.round(y)})`);
    } else {
      console.log('No entities to center on');
    }
  }

  // P: Pause/Resume (toggle p5 loop)
  if (key === 'p' || key === 'P') {
    isPaused = !isPaused;
    if (isPaused) {
      noLoop();
      console.log('Simulation paused (noLoop)');
    } else {
      loop();
      console.log('Simulation resumed (loop)');
    }
  }

  // +: Speed up
  if (key === '+' || key === '=') {
    timeScale = Math.min(timeScale * 1.5, 10.0);
    console.log(`Time scale: ${timeScale.toFixed(1)}x`);
  }

  // -: Slow down
  if (key === '-' || key === '_') {
    timeScale = Math.max(timeScale / 1.5, 0.1);
    console.log(`Time scale: ${timeScale.toFixed(1)}x`);
  }

  // Shift+Numpad Minus: Set speed to minimum (0.1x)
  if (keyCode === 109 && keyIsDown(SHIFT)) {  // Numpad minus
    timeScale = 0.1;
    console.log(`Time scale: ${timeScale.toFixed(1)}x (MINIMUM)`);
  }

  // Shift+Numpad Plus: Set speed to maximum (10.0x)
  if (keyCode === 107 && keyIsDown(SHIFT)) {  // Numpad plus
    timeScale = 10.0;
    console.log(`Time scale: ${timeScale.toFixed(1)}x (MAXIMUM)`);
  }

  // Shift+Ctrl+Numpad Plus: Set speed to normal (1.0x)
  if (keyCode === 107 && keyIsDown(SHIFT) && keyIsDown(CONTROL)) {  // Numpad plus with Shift+Ctrl
    timeScale = 1.0;
    console.log(`Time scale: ${timeScale.toFixed(1)}x (NORMAL)`);
  }

  // R: Restore selected entity's needs and energy
  if (key === 'r' || key === 'R') {
    if (selectedEntity !== null) {
      // Restore selected entity's needs and energy
      Needs.hunger[selectedEntity] = 0;
      Needs.thirst[selectedEntity] = 0;
      Needs.tiredness[selectedEntity] = 0;
      Energy.current[selectedEntity] = Energy.max[selectedEntity];
      console.log(`Restored entity ${selectedEntity} - needs reset and energy replenished`);
    } else {
      console.log('No entity selected - press Space to select a random entity');
    }
  }

  // H: Toggle help
  if (key === 'h' || key === 'H') {
    const controls = document.getElementById('controls');
    controls.style.display = controls.style.display === 'none' ? 'block' : 'none';
  }

  // Numpad period: Set zoom to 1.0x and center on selected entity
  if (keyCode === 110) {  // Numpad period (.)
    camera.zoom = 1.0;
    if (selectedEntity !== null) {
      const x = Position.x[selectedEntity];
      const y = Position.y[selectedEntity];
      camera.centerOn(x, y);
      console.log(`Camera zoom set to 1.0x and centered on entity ${selectedEntity}`);
    } else {
      console.log('Camera zoom set to 1.0x');
    }
  }

  // G: Grab selected entity
  if (key === 'g' || key === 'G') {
    if (selectedEntity !== null && !isGrabbingEntity) {
      // Start grabbing
      isGrabbingEntity = true;
      grabbedEntity = selectedEntity;

      // Save original state
      grabbedEntityOriginalState = {
        hunger: Needs.hunger[grabbedEntity],
        thirst: Needs.thirst[grabbedEntity],
        tiredness: Needs.tiredness[grabbedEntity],
        energy: Energy.current[grabbedEntity],
        state: State.current[grabbedEntity]
      };

      // Position entity under mouse cursor
      const worldX = camera.screenToWorldX(mouseX);
      const worldY = camera.screenToWorldY(mouseY);
      Position.x[grabbedEntity] = worldX;
      Position.y[grabbedEntity] = worldY;

      // Stop entity movement
      Velocity.vx[grabbedEntity] = 0;
      Velocity.vy[grabbedEntity] = 0;
      Target.hasTarget[grabbedEntity] = 0;
      State.current[grabbedEntity] = EntityState.IDLE;

      console.log(`Grabbing entity ${grabbedEntity} - Left-click to drop`);
    }
  }

  // S: Spawn 10 random entities
  if ((key === 's' || key === 'S') && !event.shiftKey) {
    for (let i = 0; i < 10; i++) {
      // Random position (all terrain is walkable)
      const x = Math.random() * CONFIG.WORLD_WIDTH;
      const y = Math.random() * CONFIG.WORLD_HEIGHT;

      // Random species (70% rabbits, 30% humans)
      const isRabbit = Math.random() > 0.3;
      const isMale = Math.random() > 0.5;

      if (isRabbit) {
        createRabbit(ecsWorld, x, y, isMale, frameCounter);
      } else {
        createHuman(ecsWorld, x, y, isMale, frameCounter);
      }
    }
    console.log(`✨ Spawned 10 random entities`);
  }

  // Shift+S: Spawn 500 random entities (with spatial distribution)
  if ((key === 's' || key === 'S') && event.shiftKey) {
    console.log('Mass spawning 500 entities with spatial distribution...');

    const SPAWN_COUNT = 500;
    const GRID_DIVISIONS = 10;  // 10×10 grid
    const CELL_WIDTH = CONFIG.WORLD_WIDTH / GRID_DIVISIONS;
    const CELL_HEIGHT = CONFIG.WORLD_HEIGHT / GRID_DIVISIONS;
    const SPAWNS_PER_CELL = Math.ceil(SPAWN_COUNT / (GRID_DIVISIONS * GRID_DIVISIONS));

    let spawned = 0;

    // Iterate through grid cells
    for (let gridY = 0; gridY < GRID_DIVISIONS; gridY++) {
      for (let gridX = 0; gridX < GRID_DIVISIONS; gridX++) {
        // Calculate cell bounds
        const cellMinX = gridX * CELL_WIDTH;
        const cellMaxX = (gridX + 1) * CELL_WIDTH;
        const cellMinY = gridY * CELL_HEIGHT;
        const cellMaxY = (gridY + 1) * CELL_HEIGHT;

        // Spawn SPAWNS_PER_CELL entities in this cell
        for (let i = 0; i < SPAWNS_PER_CELL && spawned < SPAWN_COUNT; i++) {
          // Random position in this cell (all terrain is walkable)
          const x = cellMinX + Math.random() * (cellMaxX - cellMinX);
          const y = cellMinY + Math.random() * (cellMaxY - cellMinY);

          const isMale = Math.random() < 0.5;
          const isRabbit = Math.random() < 0.75;  // 75% rabbits, 25% humans

          if (isRabbit) {
            createRabbit(ecsWorld, x, y, isMale, frameCounter);
          } else {
            createHuman(ecsWorld, x, y, isMale, frameCounter);
          }

          spawned++;
        }
      }
    }

    console.log(`Spawned ${spawned} entities (target: ${SPAWN_COUNT})`);

    // Log species breakdown for debugging
    const entities = allEntitiesQuery(ecsWorld);
    let rabbits = 0, humans = 0;
    for (const eid of entities) {
      if (SpeciesComponent.type[eid] === Species.RABBIT) rabbits++;
      if (SpeciesComponent.type[eid] === Species.HUMAN) humans++;
    }
    console.log(`  Breakdown: ${rabbits} rabbits, ${humans} humans`);

    updateEntityCountUI();
  }

  // K: Kill 10 random entities
  if ((key === 'k' || key === 'K') && !event.shiftKey) {
    const entities = allEntitiesQuery(ecsWorld);
    const toKill = Math.min(10, entities.length);

    for (let i = 0; i < toKill; i++) {
      const randomIndex = Math.floor(Math.random() * entities.length);
      const eid = entities[randomIndex];

      // Properly remove entity
      removeEntityFromWorld(ecsWorld, eid);

      // Remove from array to avoid duplicate kills
      entities.splice(randomIndex, 1);
    }
    console.log(`☠️ Killed ${toKill} random entities`);
  }

  // Shift+K: Kill ALL entities
  if ((key === 'k' || key === 'K') && event.shiftKey) {
    const entities = allEntitiesQuery(ecsWorld);
    const totalKilled = entities.length;

    // Remove all entities (iterate backwards to avoid index issues)
    for (let i = entities.length - 1; i >= 0; i--) {
      removeEntityFromWorld(ecsWorld, entities[i]);
    }

    // Clear camera following if we killed the followed entity
    camera.stopFollowing();
    selectedEntity = null;

    console.log(`💀💀💀 MASS EXTINCTION: Killed all ${totalKilled} entities (Shift+K)`);
  }

  // N: Cycle to next entity in selection history
  if ((key === 'n' || key === 'N') && !event.shiftKey) {
    cleanSelectionHistory();  // Ensure history is up-to-date

    if (selectionHistory.length === 0) {
      console.log('No entities in selection history. Click entities to add them.');
      return;
    }

    // Find current index in history
    let currentIndex = selectionHistory.indexOf(selectedEntity);

    // If not found or no selection, start at beginning
    if (currentIndex === -1) {
      currentIndex = -1;  // Will wrap to 0
    }

    // Next index (with wraparound)
    const nextIndex = (currentIndex + 1) % selectionHistory.length;
    selectedEntity = selectionHistory[nextIndex];

    // Center camera on selected entity
    const x = Position.x[selectedEntity];
    const y = Position.y[selectedEntity];
    camera.centerOn(x, y);
    camera.startFollowing(selectedEntity);

    console.log(`Cycled to entity ${selectedEntity} (${nextIndex + 1}/${selectionHistory.length})`);
  }

  // Shift+N: Cycle to previous entity in selection history
  if ((key === 'n' || key === 'N') && event.shiftKey) {
    cleanSelectionHistory();  // Ensure history is up-to-date

    if (selectionHistory.length === 0) {
      console.log('No entities in selection history. Click entities to add them.');
      return;
    }

    // Find current index in history
    let currentIndex = selectionHistory.indexOf(selectedEntity);

    // If not found or no selection, start at end
    if (currentIndex === -1) {
      currentIndex = 0;  // Will wrap to end
    }

    // Previous index (with wraparound)
    const prevIndex = (currentIndex - 1 + selectionHistory.length) % selectionHistory.length;
    selectedEntity = selectionHistory[prevIndex];

    // Center camera on selected entity
    const x = Position.x[selectedEntity];
    const y = Position.y[selectedEntity];
    camera.centerOn(x, y);
    camera.startFollowing(selectedEntity);

    console.log(`Cycled to entity ${selectedEntity} (${prevIndex + 1}/${selectionHistory.length})`);
  }

  // Shift+Delete: Clear terrain storage
  if (keyCode === DELETE && event.shiftKey) {
    if (!terrainStorage) {
      console.warn('Terrain storage not initialized');
      return;
    }

    terrainStorage.clearTerrain()
      .then(() => {
        console.log('✓ Terrain storage cleared. Refresh page to regenerate terrain.');
        alert('Terrain storage cleared!\n\nRefresh the page (F5) to regenerate a new random terrain.');
      })
      .catch(err => {
        console.error('Failed to clear terrain storage:', err);
        alert('Error clearing terrain storage. Check console.');
      });
  }
}

// p5.js window resized handler
function windowResized() {
  resizeCanvas(windowWidth, windowHeight);
  camera.clampToBounds();
}

// Utility function to log simulation stats
function logStats() {
  if (!camera || !ecsWorld) return; // Wait for initialization

  console.log('--- Simulation Stats ---');
  console.log(`FPS: ${fpsDisplay}`);
  console.log(`Camera: (${Math.round(camera.x)}, ${Math.round(camera.y)}) @ ${camera.zoom.toFixed(2)}x`);
  console.log(`Time Scale: ${timeScale.toFixed(1)}x`);
  console.log(`Paused: ${isPaused}`);

  const counts = getEntityCountBySpecies(ecsWorld);
  console.log(`Entities: ${counts.humans} humans, ${counts.rabbits} rabbits (${counts.humans + counts.rabbits} total)`);
}

// Diagnostic: Check if setup() is ever called
window.setupCalled = false;

setTimeout(() => {
  if (!window.setupCalled) {
    console.error('DIAGNOSTIC: setup() was never called by p5.js after 2 seconds!');
    console.error('This usually means p5.js did not initialize properly.');
    console.error('Checking p5 state:', {
      p5Defined: typeof p5 !== 'undefined',
      createCanvasDefined: typeof createCanvas !== 'undefined',
      setupDefined: typeof setup === 'function',
      windowWidth: typeof windowWidth,
      windowHeight: typeof windowHeight
    });
  } else {
    console.log('✓ setup() was called successfully');
  }
}, 2000);
