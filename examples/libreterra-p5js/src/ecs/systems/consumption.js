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

    // Water restriction: Can't eat while in water (land creatures only)
    const isInWater = (terrain === TerrainType.WATER);
    if (isInWater && intent.type === IntentType.SEEK_FOOD) {
      // Can't eat in water - stay IDLE (entity will need to move out of water)
      return;
    }

    // Check if entity is on water OR adjacent to water for drinking
    if (intent.type === IntentType.SEEK_WATER) {
      let canDrink = terrain === TerrainType.WATER;

      // Check adjacent tiles if not on water
      if (!canDrink) {
        for (let dy = -1; dy <= 1; dy++) {
          for (let dx = -1; dx <= 1; dx++) {
            if (dx === 0 && dy === 0) continue;
            const adjacentTerrain = this.terrainGrid.get(x + dx, y + dy);
            if (adjacentTerrain === TerrainType.WATER) {
              canDrink = true;
              break;
            }
          }
          if (canDrink) break;
        }
      }

      if (canDrink) {
        this.consumeWater(entityId);
      }
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
      // For water, check if water is nearby (not exact position)
      if (intent.type === IntentType.SEEK_WATER) {
        // Check if entity is on water OR adjacent to water
        const entityX = Math.floor(Position.x[entityId]);
        const entityY = Math.floor(Position.y[entityId]);

        // Check current position
        const currentTerrain = this.terrainGrid.get(entityX, entityY);
        let hasWaterNearby = currentTerrain === TerrainType.WATER;

        // If not on water, check adjacent tiles (8 directions)
        if (!hasWaterNearby) {
          for (let dy = -1; dy <= 1; dy++) {
            for (let dx = -1; dx <= 1; dx++) {
              if (dx === 0 && dy === 0) continue;  // Skip center (already checked)

              const checkX = entityX + dx;
              const checkY = entityY + dy;
              const adjacentTerrain = this.terrainGrid.get(checkX, checkY);

              if (adjacentTerrain === TerrainType.WATER) {
                hasWaterNearby = true;
                break;
              }
            }
            if (hasWaterNearby) break;
          }
        }

        if (hasWaterNearby) {
          State.current[entityId] = EntityState.DRINKING;
          Velocity.vx[entityId] = 0;
          Velocity.vy[entityId] = 0;
        }
      }
      // Food consumption (existing logic)
      else if (intent.type === IntentType.SEEK_FOOD) {
        const entityX = Math.floor(Position.x[entityId]);
        const entityY = Math.floor(Position.y[entityId]);
        const terrain = this.terrainGrid.get(entityX, entityY);

        if (species === Species.RABBIT && terrain === TerrainType.GRASS) {
          State.current[entityId] = EntityState.EATING;
          Velocity.vx[entityId] = 0;
          Velocity.vy[entityId] = 0;
        } else if (species === Species.HUMAN && intent.targetEntity !== undefined) {
          // Check if rabbit is still close (use component check instead of O(n) array search)
          const rabbitEid = intent.targetEntity;
          if (Position.x[rabbitEid] !== undefined) {
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

        // Check if rabbit still exists (use component check instead of O(n) array search)
        if (Position.x[rabbitId] !== undefined) {
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

        // Deplete grass very rarely (visual effect only, expensive to update)
        // Reduced from 10% to 0.1% per frame to avoid expensive chunk buffer updates
        const x = Math.floor(Position.x[entityId]);
        const y = Math.floor(Position.y[entityId]);
        const terrain = this.terrainGrid.get(x, y);

        if (terrain === TerrainType.GRASS && Math.random() < 0.001) {
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
