// ECS world setup and entity creation

const { createWorld, addEntity, addComponent } = bitecs;

// Create the ECS world
function createECSWorld() {
  const world = createWorld();
  console.log('ECS world created');
  return world;
}

// Calculate expected lifespan in frames for a species
// Time scale: 100 simulation years = 120 minutes real-time
// At 60 FPS: 1 simulation year = 4,320 frames (72 seconds)
function calculateLifespanFrames(species) {
  const FRAMES_PER_SIM_YEAR = 4_320;  // 100 years = 120 minutes at 60 FPS

  if (species === Species.HUMAN) {
    // 70 years base lifespan (84 minutes real-time)
    return 70 * FRAMES_PER_SIM_YEAR;  // 302,400 frames
  } else if (species === Species.RABBIT) {
    // 2 years base lifespan (2.4 minutes real-time)
    return 2 * FRAMES_PER_SIM_YEAR;   // 8,640 frames
  }

  return FRAMES_PER_SIM_YEAR;  // Default: 1 year
}

// Create a human entity
function createHuman(world, x, y, isMale, currentFrame = 0) {
  const eid = addEntity(world);

  // Add Position
  addComponent(world, Position, eid);
  Position.x[eid] = x;
  Position.y[eid] = y;

  // Add Velocity
  addComponent(world, Velocity, eid);
  Velocity.vx[eid] = 0;
  Velocity.vy[eid] = 0;

  // Add Needs (newly spawned entities start with zero needs - healthy and rested)
  addComponent(world, Needs, eid);
  Needs.hunger[eid] = 0;  // No hunger
  Needs.thirst[eid] = 0;  // No thirst
  Needs.tiredness[eid] = 0;  // Not tired

  // Add Species
  addComponent(world, SpeciesComponent, eid);
  SpeciesComponent.type[eid] = Species.HUMAN;

  // Add Gender
  addComponent(world, Gender, eid);
  Gender.isMale[eid] = isMale ? 1 : 0;

  // Add Energy
  addComponent(world, Energy, eid);
  Energy.current[eid] = 100;
  Energy.max[eid] = 100;

  // Add Age
  addComponent(world, Age, eid);
  Age.birthFrame[eid] = currentFrame;
  Age.expectedLifespanFrames[eid] = calculateLifespanFrames(Species.HUMAN);
  Age.energyHistory[eid] = 100;  // Start healthy

  // Add Target
  addComponent(world, Target, eid);
  Target.x[eid] = 0;
  Target.y[eid] = 0;
  Target.hasTarget[eid] = 0;

  // Add State
  addComponent(world, State, eid);
  State.current[eid] = EntityState.IDLE;

  return eid;
}

// Create a rabbit entity
function createRabbit(world, x, y, isMale, currentFrame = 0) {
  const eid = addEntity(world);

  // Add Position
  addComponent(world, Position, eid);
  Position.x[eid] = x;
  Position.y[eid] = y;

  // Add Velocity
  addComponent(world, Velocity, eid);
  Velocity.vx[eid] = 0;
  Velocity.vy[eid] = 0;

  // Add Needs (newly spawned rabbits start with zero needs - healthy and rested)
  addComponent(world, Needs, eid);
  Needs.hunger[eid] = 0;  // No hunger
  Needs.thirst[eid] = 0;  // No thirst
  Needs.tiredness[eid] = 0;  // Not tired

  // Add Species
  addComponent(world, SpeciesComponent, eid);
  SpeciesComponent.type[eid] = Species.RABBIT;

  // Add Gender
  addComponent(world, Gender, eid);
  Gender.isMale[eid] = isMale ? 1 : 0;

  // Add Energy (start at full energy)
  addComponent(world, Energy, eid);
  Energy.current[eid] = 100;
  Energy.max[eid] = 100;

  // Add Age
  addComponent(world, Age, eid);
  Age.birthFrame[eid] = currentFrame;
  Age.expectedLifespanFrames[eid] = calculateLifespanFrames(Species.RABBIT);
  Age.energyHistory[eid] = 100;  // Start healthy

  // Add Target
  addComponent(world, Target, eid);
  Target.x[eid] = 0;
  Target.y[eid] = 0;
  Target.hasTarget[eid] = 0;

  // Add State
  addComponent(world, State, eid);
  State.current[eid] = EntityState.IDLE;

  return eid;
}

// Spawn initial entities randomly on the map
function spawnInitialEntities(world, terrainGrid, currentFrame = 0) {
  console.log('Spawning initial entities...');

  const entities = {
    humans: [],
    rabbits: []
  };

  // Spawn humans
  for (let i = 0; i < CONFIG.SIMULATION.INITIAL_HUMANS; i++) {
    let x, y;
    let attempts = 0;

    // Find a walkable spawn location
    do {
      x = Math.random() * CONFIG.WORLD_WIDTH;
      y = Math.random() * CONFIG.WORLD_HEIGHT;
      attempts++;
    } while (!terrainGrid.isWalkable(Math.floor(x), Math.floor(y)) && attempts < 100);

    if (attempts < 100) {
      const isMale = Math.random() > 0.5;
      const eid = createHuman(world, x, y, isMale, currentFrame);
      entities.humans.push(eid);
    }
  }

  // Spawn rabbits
  for (let i = 0; i < CONFIG.SIMULATION.INITIAL_RABBITS; i++) {
    let x, y;
    let attempts = 0;

    // Find a walkable spawn location (prefer grass areas)
    do {
      x = Math.random() * CONFIG.WORLD_WIDTH;
      y = Math.random() * CONFIG.WORLD_HEIGHT;
      attempts++;
    } while (!terrainGrid.isWalkable(Math.floor(x), Math.floor(y)) && attempts < 100);

    if (attempts < 100) {
      const isMale = Math.random() > 0.5;
      const eid = createRabbit(world, x, y, isMale, currentFrame);
      entities.rabbits.push(eid);
    }
  }

  console.log(`Spawned ${entities.humans.length} humans and ${entities.rabbits.length} rabbits`);

  return entities;
}

// Get total entity count
function getEntityCount(world) {
  return allEntitiesQuery(world).length;
}

// Get entity counts by species
function getEntityCountBySpecies(world) {
  const allEntities = allEntitiesQuery(world);
  let humans = 0;
  let rabbits = 0;

  for (const eid of allEntities) {
    if (isHuman(eid)) {
      humans++;
    } else if (isRabbit(eid)) {
      rabbits++;
    }
  }

  return { humans, rabbits };
}

// Remove an entity from the ECS world
function removeEntityFromWorld(world, entityId) {
  // Clear any decision intent first
  if (typeof decisionSystem !== 'undefined' && decisionSystem) {
    decisionSystem.clearIntent(entityId);
  }

  // Remove from libreconomy WASM bridge if it exists
  if (typeof libreconomyStub !== 'undefined' && libreconomyStub && libreconomyStub.removeEntity) {
    libreconomyStub.removeEntity(entityId);
  }

  // Remove from bitECS world
  bitecs.removeEntity(world, entityId);
}
