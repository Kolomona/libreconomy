// Resource consumption system (Phase 6)
// Handles eating, drinking, and sleeping

class ConsumptionSystem {
  constructor(terrainGrid, decisionSystem) {
    this.terrainGrid = terrainGrid;
    this.decisionSystem = decisionSystem;

    // Consumption rates (how much need is reduced per frame while consuming)
    this.consumptionRates = {
      drinking: 2.0,    // Reduces thirst by 2 per frame
      eating: 1.5,      // Reduces hunger by 1.5 per frame
      sleeping: 1.0     // Reduces tiredness by 1 per frame
    };

    // Interaction range (distance to resource to consume it)
    this.interactionRange = 10;

    // Minimum need level before stopping consumption
    this.satisfiedThreshold = 20;
  }

  update(ecsWorld) {
    const entities = allEntitiesQuery(ecsWorld);

    for (const eid of entities) {
      const state = State.current[eid];
      const species = SpeciesComponent.type[eid];

      // Handle different states
      switch (state) {
        case EntityState.IDLE:
          this.handleIdleConsumption(eid, species);
          break;

        case EntityState.SLEEPING:
          this.handleSleeping(eid);
          break;

        case EntityState.MOVING:
          // Check if entity reached a resource target
          this.checkResourceReached(eid, species);
          break;
      }
    }
  }

  // Handle consumption when entity is idle at a resource
  handleIdleConsumption(entityId, species) {
    const intent = this.decisionSystem.getIntent(entityId);
    if (!intent) return;

    // Safety: If entity has a target but is Idle, transition to Moving
    // This prevents entities from getting stuck in Idle with active intents
    if (Target.hasTarget[entityId] === 1 &&
        (intent.type === IntentType.SEEK_WATER ||
         intent.type === IntentType.SEEK_FOOD ||
         intent.type === IntentType.WANDER)) {
      State.current[entityId] = EntityState.MOVING;
      return;
    }

    const x = Math.floor(Position.x[entityId]);
    const y = Math.floor(Position.y[entityId]);
    const terrain = this.terrainGrid.get(x, y);

    // Check if entity is on the terrain type it was seeking
    if (intent.type === IntentType.SEEK_WATER && terrain === TerrainType.WATER) {
      this.consumeWater(entityId);
    } else if (intent.type === IntentType.SEEK_FOOD) {
      if (species === Species.RABBIT && terrain === TerrainType.GRASS) {
        this.consumeGrass(entityId, x, y);
      } else if (species === Species.HUMAN && intent.targetEntity !== undefined) {
        this.consumeRabbit(entityId, intent.targetEntity, ecsWorld);
      }
    }
  }

  // Check if moving entity has reached its resource target
  checkResourceReached(entityId, species) {
    const intent = this.decisionSystem.getIntent(entityId);
    if (!intent || !intent.target) return;

    // Calculate distance to target
    const dx = intent.target.x - Position.x[entityId];
    const dy = intent.target.y - Position.y[entityId];
    const distance = Math.sqrt(dx * dx + dy * dy);

    if (distance <= this.interactionRange) {
      // Reached target, start consuming
      const x = Math.floor(Position.x[entityId]);
      const y = Math.floor(Position.y[entityId]);
      const terrain = this.terrainGrid.get(x, y);

      if (intent.type === IntentType.SEEK_WATER && terrain === TerrainType.WATER) {
        State.current[entityId] = EntityState.DRINKING;
        Velocity.vx[entityId] = 0;
        Velocity.vy[entityId] = 0;
      } else if (intent.type === IntentType.SEEK_FOOD) {
        if (species === Species.RABBIT && terrain === TerrainType.GRASS) {
          State.current[entityId] = EntityState.EATING;
          Velocity.vx[entityId] = 0;
          Velocity.vy[entityId] = 0;
        } else if (species === Species.HUMAN && intent.targetEntity !== undefined) {
          // Check if rabbit is still close
          const rabbitEid = intent.targetEntity;
          const entities = allEntitiesQuery(ecsWorld);
          if (entities.includes(rabbitEid)) {
            State.current[entityId] = EntityState.EATING;
            Velocity.vx[entityId] = 0;
            Velocity.vy[entityId] = 0;
          }
        }
      }
    }
  }

  // Consume water (reduce thirst)
  consumeWater(entityId) {
    const thirst = Needs.thirst[entityId];

    if (thirst > this.satisfiedThreshold) {
      Needs.thirst[entityId] = Math.max(0, thirst - this.consumptionRates.drinking);
      State.current[entityId] = EntityState.DRINKING;
    } else {
      // Finished drinking
      State.current[entityId] = EntityState.IDLE;
      this.decisionSystem.clearIntent(entityId);
    }
  }

  // Consume grass (reduce hunger, convert grass to dirt)
  consumeGrass(entityId, x, y) {
    const hunger = Needs.hunger[entityId];

    if (hunger > this.satisfiedThreshold) {
      Needs.hunger[entityId] = Math.max(0, hunger - this.consumptionRates.eating);
      State.current[entityId] = EntityState.EATING;

      // Deplete grass (convert to dirt)
      // Only deplete occasionally to make grass last longer
      if (Math.random() < 0.1) {
        this.terrainGrid.depleteGrass(x, y);
      }
    } else {
      // Finished eating
      State.current[entityId] = EntityState.IDLE;
      this.decisionSystem.clearIntent(entityId);
    }
  }

  // Consume rabbit (reduce hunger, remove rabbit entity)
  consumeRabbit(humanId, rabbitId, ecsWorld) {
    const hunger = Needs.hunger[humanId];

    // Check if rabbit still exists
    const entities = allEntitiesQuery(ecsWorld);
    if (!entities.includes(rabbitId)) {
      // Rabbit is gone
      State.current[humanId] = EntityState.IDLE;
      this.decisionSystem.clearIntent(humanId);
      return;
    }

    // Check if rabbit is close enough
    const dx = Position.x[rabbitId] - Position.x[humanId];
    const dy = Position.y[rabbitId] - Position.y[humanId];
    const distance = Math.sqrt(dx * dx + dy * dy);

    if (distance > this.interactionRange) {
      // Rabbit escaped, back to moving
      State.current[humanId] = EntityState.MOVING;
      return;
    }

    if (hunger > this.satisfiedThreshold) {
      // Eating rabbit
      Needs.hunger[humanId] = Math.max(0, hunger - this.consumptionRates.eating * 2); // Rabbits are filling
      State.current[humanId] = EntityState.EATING;

      // Remove rabbit entity (TODO: proper entity removal in Phase 7)
      // For now, just move it far away
      Position.x[rabbitId] = -1000;
      Position.y[rabbitId] = -1000;
      this.decisionSystem.clearIntent(rabbitId);
    } else {
      // Finished eating
      State.current[humanId] = EntityState.IDLE;
      this.decisionSystem.clearIntent(humanId);

      // Remove eaten rabbit
      Position.x[rabbitId] = -1000;
      Position.y[rabbitId] = -1000;
      this.decisionSystem.clearIntent(rabbitId);
    }
  }

  // Handle sleeping (reduce tiredness)
  handleSleeping(entityId) {
    const tiredness = Needs.tiredness[entityId];

    if (tiredness > this.satisfiedThreshold) {
      Needs.tiredness[entityId] = Math.max(0, tiredness - this.consumptionRates.sleeping);
    } else {
      // Finished sleeping
      State.current[entityId] = EntityState.IDLE;
      this.decisionSystem.clearIntent(entityId);
    }
  }

  // Check if entity is currently consuming
  isConsuming(entityId) {
    const state = State.current[entityId];
    return state === EntityState.EATING ||
           state === EntityState.DRINKING ||
           state === EntityState.SLEEPING;
  }
}
