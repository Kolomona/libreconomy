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
    this.satisfiedThreshold = 7;  // Mostly satisfied, small buffer
  }

  update(ecsWorld) {
    const entities = allEntitiesQuery(ecsWorld);

    for (const eid of entities) {
      const state = State.current[eid];
      const species = SpeciesComponent.type[eid];

      // Handle different states
      switch (state) {
        case EntityState.IDLE:
          this.handleIdleConsumption(eid, species, ecsWorld);
          break;

        case EntityState.DRINKING:
          // Handle continuous drinking
          this.handleDrinking(eid);
          break;

        case EntityState.EATING:
          // Handle continuous eating
          this.handleEating(eid, species, ecsWorld);
          break;

        case EntityState.SLEEPING:
          this.handleSleeping(eid);
          break;

        case EntityState.MOVING:
          // Check if entity reached a resource target
          this.checkResourceReached(eid, species, ecsWorld);
          break;
      }
    }
  }

  // Handle consumption when entity is idle at a resource
  handleIdleConsumption(entityId, species, ecsWorld) {
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
  checkResourceReached(entityId, species, ecsWorld) {
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
    // Transition to DRINKING state
    // (handleDrinking will take over on next frame)
    State.current[entityId] = EntityState.DRINKING;
  }

  // Consume grass (reduce hunger, convert grass to dirt)
  consumeGrass(entityId, x, y) {
    // Transition to EATING state
    // (handleEating will take over on next frame)
    State.current[entityId] = EntityState.EATING;
  }

  // Consume rabbit (reduce hunger, remove rabbit entity)
  consumeRabbit(humanId, rabbitId, ecsWorld) {
    // Transition to EATING state
    // (handleEating will manage the consumption and rabbit removal)
    State.current[humanId] = EntityState.EATING;
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

  // Handle continuous drinking (entity is at water)
  handleDrinking(entityId) {
    const thirst = Needs.thirst[entityId];

    if (thirst > this.satisfiedThreshold) {
      // Continue drinking
      Needs.thirst[entityId] = Math.max(0, thirst - this.consumptionRates.drinking);
    } else {
      // Finished drinking
      State.current[entityId] = EntityState.IDLE;
      this.decisionSystem.clearIntent(entityId);
    }
  }

  // Handle continuous eating (entity is at food)
  handleEating(entityId, species, ecsWorld) {
    const hunger = Needs.hunger[entityId];

    if (hunger > this.satisfiedThreshold) {
      // Continue eating
      const intent = this.decisionSystem.getIntent(entityId);

      // Check if hunting rabbit (humans only)
      if (species === Species.HUMAN && intent && intent.targetEntity !== undefined) {
        const rabbitId = intent.targetEntity;
        const entities = allEntitiesQuery(ecsWorld);

        if (entities.includes(rabbitId)) {
          // Check if rabbit is still close
          const dx = Position.x[rabbitId] - Position.x[entityId];
          const dy = Position.y[rabbitId] - Position.y[entityId];
          const distance = Math.sqrt(dx * dx + dy * dy);

          if (distance <= this.interactionRange) {
            // Eating rabbit (more filling)
            Needs.hunger[entityId] = Math.max(0, hunger - this.consumptionRates.eating * 2);
          } else {
            // Rabbit escaped, back to moving
            State.current[entityId] = EntityState.MOVING;
            return;
          }
        } else {
          // Rabbit is gone
          State.current[entityId] = EntityState.IDLE;
          this.decisionSystem.clearIntent(entityId);
          return;
        }
      } else if (species === Species.RABBIT) {
        // Eating grass
        Needs.hunger[entityId] = Math.max(0, hunger - this.consumptionRates.eating);

        // Deplete grass occasionally
        const x = Math.floor(Position.x[entityId]);
        const y = Math.floor(Position.y[entityId]);
        const terrain = this.terrainGrid.get(x, y);

        if (terrain === TerrainType.GRASS && Math.random() < 0.1) {
          this.terrainGrid.depleteGrass(x, y);
        }
      }

      State.current[entityId] = EntityState.EATING;
    } else {
      // Finished eating - remove rabbit if applicable
      const intent = this.decisionSystem.getIntent(entityId);
      if (species === Species.HUMAN && intent && intent.targetEntity !== undefined) {
        removeEntityFromWorld(ecsWorld, intent.targetEntity);
      }

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
