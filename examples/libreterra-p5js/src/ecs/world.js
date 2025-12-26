// ECS world setup and entity creation

const { createWorld, addEntity, addComponent } = bitecs;

// Create the ECS world
function createECSWorld() {
  const world = createWorld();
  console.log('ECS world created');
  return world;
}

// Create a human entity
function createHuman(world, x, y, isMale) {
  const eid = addEntity(world);

  // Add Position
  addComponent(world, Position, eid);
  Position.x[eid] = x;
  Position.y[eid] = y;

  // Add Velocity
  addComponent(world, Velocity, eid);
  Velocity.vx[eid] = 0;
  Velocity.vy[eid] = 0;

  // Add Needs (start with random values)
  addComponent(world, Needs, eid);
  Needs.hunger[eid] = Math.random() * 50; // 0-50
  Needs.thirst[eid] = Math.random() * 50;
  Needs.tiredness[eid] = Math.random() * 30;

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
function createRabbit(world, x, y, isMale) {
  const eid = addEntity(world);

  // Add Position
  addComponent(world, Position, eid);
  Position.x[eid] = x;
  Position.y[eid] = y;

  // Add Velocity
  addComponent(world, Velocity, eid);
  Velocity.vx[eid] = 0;
  Velocity.vy[eid] = 0;

  // Add Needs (rabbits get hungry/thirsty faster)
  addComponent(world, Needs, eid);
  Needs.hunger[eid] = Math.random() * 50;
  Needs.thirst[eid] = Math.random() * 50;
  Needs.tiredness[eid] = Math.random() * 30;

  // Add Species
  addComponent(world, SpeciesComponent, eid);
  SpeciesComponent.type[eid] = Species.RABBIT;

  // Add Gender
  addComponent(world, Gender, eid);
  Gender.isMale[eid] = isMale ? 1 : 0;

  // Add Energy
  addComponent(world, Energy, eid);
  Energy.current[eid] = 80;
  Energy.max[eid] = 80;

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
function spawnInitialEntities(world, terrainGrid) {
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
      const eid = createHuman(world, x, y, isMale);
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
      const eid = createRabbit(world, x, y, isMale);
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
