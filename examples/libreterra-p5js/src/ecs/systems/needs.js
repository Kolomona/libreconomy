// Needs decay system (Phase 5)
// Gradually increases hunger, thirst, and tiredness over time

class NeedsDecaySystem {
  constructor(terrainGrid) {
    this.terrainGrid = terrainGrid;

    // Decay rates per frame (at 30 FPS)
    // These values determine how quickly needs increase
    this.decayRates = {
      // Base decay rates (per frame)
      hunger: 0.02,      // Takes ~83 seconds to go from 0 to 50 (hungry threshold)
      thirst: 0.025,     // Slightly faster than hunger
      tiredness: 0.015,  // Slower than hunger/thirst
      energy: 0.005,     // Energy decay rate (reduced from 0.01 to prevent death spiral)

      // Activity multipliers (applied when entity is moving/active)
      activityMultiplier: {
        hunger: 1.5,     // Moving makes you hungrier faster
        thirst: 2.0,     // Moving makes you thirstier much faster
        tiredness: 2.5   // Moving makes you tired faster
      },

      // State-specific energy multipliers
      energyStateMultiplier: {
        [EntityState.IDLE]: 0.5,      // Idle conserves energy
        [EntityState.MOVING]: 1.2,    // Moving drains energy (reduced from 2.0 to 1.2)
        [EntityState.EATING]: 0.3,    // Eating restores slightly
        [EntityState.DRINKING]: 0.3,  // Drinking restores slightly
        [EntityState.SLEEPING]: -2.5  // Sleeping RESTORES energy (increased from -1.0 to -2.5)
      },

      // Species-specific multipliers
      rabbit: {
        hunger: 1.3,     // Rabbits get hungry faster
        thirst: 1.2,     // Rabbits get thirsty faster
        tiredness: 1.0,
        energy: 1.0
      },
      human: {
        hunger: 1.0,
        thirst: 1.0,
        tiredness: 1.0,
        energy: 1.0
      }
    };

    // Maximum values for needs (prevent overflow)
    this.maxNeed = 100;
  }

  update(ecsWorld, deltaTime = 1.0) {
    const entities = allEntitiesQuery(ecsWorld);

    for (const eid of entities) {
      const species = SpeciesComponent.type[eid];
      const state = State.current[eid];

      // Get species multipliers
      const speciesMultiplier = species === Species.RABBIT
        ? this.decayRates.rabbit
        : this.decayRates.human;

      // Calculate BASE decay amounts (before state modifiers)
      let hungerDecay = this.decayRates.hunger * speciesMultiplier.hunger * deltaTime;
      let thirstDecay = this.decayRates.thirst * speciesMultiplier.thirst * deltaTime;
      let tirednessDecay = this.decayRates.tiredness * speciesMultiplier.tiredness * deltaTime;
      let energyDecay = this.decayRates.energy * speciesMultiplier.energy * deltaTime;

      // Apply state-specific modifiers
      switch (state) {
        case EntityState.MOVING:
          // Moving: apply activity multipliers (increased decay)
          hungerDecay *= this.decayRates.activityMultiplier.hunger;
          thirstDecay *= this.decayRates.activityMultiplier.thirst;
          tirednessDecay *= this.decayRates.activityMultiplier.tiredness;

          // Apply state multiplier for energy
          energyDecay *= this.decayRates.energyStateMultiplier[EntityState.MOVING];

          // Apply terrain-based energy cost when moving
          const terrain = this.terrainGrid.get(
            Math.floor(Position.x[eid]),
            Math.floor(Position.y[eid])
          );
          const terrainAttributes = CONFIG.SPECIES_TERRAIN_ATTRIBUTES[species][terrain];
          hungerDecay *= terrainAttributes.energyMultiplier;
          thirstDecay *= terrainAttributes.energyMultiplier;
          tirednessDecay *= terrainAttributes.energyMultiplier;
          energyDecay *= terrainAttributes.energyMultiplier;
          break;

        case EntityState.DRINKING:
          // Drinking: thirst doesn't increase, other needs normal
          thirstDecay = 0;
          // Apply state multiplier for energy
          energyDecay *= this.decayRates.energyStateMultiplier[EntityState.DRINKING];
          break;

        case EntityState.EATING:
          // Eating: hunger doesn't increase, other needs normal
          hungerDecay = 0;
          // Apply state multiplier for energy
          energyDecay *= this.decayRates.energyStateMultiplier[EntityState.EATING];
          break;

        case EntityState.SLEEPING:
          // Sleeping: tiredness doesn't increase, hunger/thirst at 10% rate
          tirednessDecay = 0;
          hungerDecay *= 0.1;
          thirstDecay *= 0.1;

          // Energy RESTORATION during sleep (graduated formula)
          energyDecay *= this.decayRates.energyStateMultiplier[EntityState.SLEEPING];

          // Graduated restoration based on hunger/thirst satisfaction
          const hungerSatisfaction = 1 - (Needs.hunger[eid] / 100);
          const thirstSatisfaction = 1 - (Needs.thirst[eid] / 100);

          // Calculate average satisfaction (allows partial restoration)
          const avgSatisfaction = (hungerSatisfaction + thirstSatisfaction) / 2;

          // ALWAYS restore at least 25% of base rate, even if starving
          // This prevents complete inability to recover energy
          const minRestoration = 0.25;
          const restorationMultiplier = Math.max(minRestoration, avgSatisfaction);

          // Reduce restoration if hungry/thirsty (but never below 25%)
          energyDecay *= restorationMultiplier;
          break;

        case EntityState.IDLE:
          // Idle: normal base decay (no changes needed)
          // Apply state multiplier for energy
          energyDecay *= this.decayRates.energyStateMultiplier[EntityState.IDLE];
          break;
      }

      // Apply decay (increase needs)
      Needs.hunger[eid] = Math.min(this.maxNeed, Needs.hunger[eid] + hungerDecay);
      Needs.thirst[eid] = Math.min(this.maxNeed, Needs.thirst[eid] + thirstDecay);
      Needs.tiredness[eid] = Math.min(this.maxNeed, Needs.tiredness[eid] + tirednessDecay);

      // Apply energy change (can decrease OR increase with max cap from Age system)
      // Note: energyDecay can be negative (restoration) during sleeping
      const maxEnergy = Energy.max[eid];  // Age-adjusted max (will be set by AgeSystem)
      Energy.current[eid] = Math.max(0, Math.min(maxEnergy, Energy.current[eid] - energyDecay));

      // Auto pass out from exhaustion
      if (Needs.tiredness[eid] >= 100) {
        // Check if in water
        const x = Math.floor(Position.x[eid]);
        const y = Math.floor(Position.y[eid]);
        const terrain = this.terrainGrid.get(x, y);

        if (terrain === TerrainType.WATER) {
          // DROWN instead of falling asleep
          // DeathSystem will handle drowning death
          // Don't force sleep - entity will drown
        } else {
          // Force entity to sleep if too tired (only on land)
          if (State.current[eid] !== EntityState.SLEEPING) {
            const species = SpeciesComponent.type[eid] === Species.HUMAN ? 'Human' : 'Rabbit';
            const gender = Gender.isMale[eid] ? 'Male' : 'Female';
            // DO NOT uncomment the console log below
            // console.log(`😴 Entity ${eid} (${gender} ${species}) passed out from exhaustion`);

            State.current[eid] = EntityState.SLEEPING;
            Target.hasTarget[eid] = 0;
            Velocity.vx[eid] = 0;
            Velocity.vy[eid] = 0;
          }
        }
      }
    }
  }

  // Get the current need level as a category (for UI/debugging)
  getNeedLevel(needValue) {
    if (needValue >= 80) return 'critical';
    if (needValue >= 60) return 'high';
    if (needValue >= 30) return 'moderate';
    return 'low';
  }

  // Get entity's most urgent need
  getMostUrgentNeed(entityId) {
    const hunger = Needs.hunger[entityId];
    const thirst = Needs.thirst[entityId];
    const tiredness = Needs.tiredness[entityId];

    const needs = [
      { type: 'hunger', value: hunger },
      { type: 'thirst', value: thirst },
      { type: 'tiredness', value: tiredness }
    ];

    needs.sort((a, b) => b.value - a.value);
    return needs[0];
  }
}

// Death system - handles entity death from starvation/dehydration/drowning
class DeathSystem {
  constructor(terrainGrid) {
    this.terrainGrid = terrainGrid;
    this.deathThreshold = 100;  // Die when need reaches 100
    this.timeToDieFrames = 300;  // Must be at 100 for 10 seconds (at 30 FPS)
    this.criticalNeedTime = new Map();  // Track how long entity has been critical
  }

  update(ecsWorld) {
    const entities = allEntitiesQuery(ecsWorld);

    for (const eid of entities) {
      const hunger = Needs.hunger[eid];
      const thirst = Needs.thirst[eid];
      const tiredness = Needs.tiredness[eid];

      // Check for drowning (instant death, no grace period)
      const x = Math.floor(Position.x[eid]);
      const y = Math.floor(Position.y[eid]);
      const terrain = this.terrainGrid.get(x, y);

      if (terrain === TerrainType.WATER && tiredness >= this.deathThreshold) {
        // Instant drowning (too tired to swim)
        this.killEntity(eid, 'drowning', ecsWorld);
        this.criticalNeedTime.delete(eid);
        continue;
      }

      // Check for energy depletion (instant death, similar to drowning)
      if (Energy.current[eid] <= 0) {
        this.killEntity(eid, 'energy_depletion', ecsWorld);
        this.criticalNeedTime.delete(eid);
        continue;
      }

      // Check if hunger or thirst is lethal
      const isStarving = hunger >= this.deathThreshold;
      const isDehydrated = thirst >= this.deathThreshold;

      if (isStarving || isDehydrated) {
        // Track critical time
        if (!this.criticalNeedTime.has(eid)) {
          this.criticalNeedTime.set(eid, 0);
        }
        this.criticalNeedTime.set(eid, this.criticalNeedTime.get(eid) + 1);

        // Die after being critical for too long
        if (this.criticalNeedTime.get(eid) >= this.timeToDieFrames) {
          this.killEntity(eid, isStarving ? 'starvation' : 'dehydration', ecsWorld);
          this.criticalNeedTime.delete(eid);
        }
      } else {
        // Recovered from critical state
        this.criticalNeedTime.delete(eid);
      }
    }
  }

  killEntity(entityId, cause, ecsWorld) {
    const species = SpeciesComponent.type[entityId] === Species.HUMAN ? 'Human' : 'Rabbit';
    const gender = Gender.isMale[entityId] ? 'Male' : 'Female';
    // console.log(`💀 Entity ${entityId} (${gender} ${species}) died from ${cause}`);

    // Properly remove entity from ECS world
    removeEntityFromWorld(ecsWorld, entityId);
  }

  // Get time until death for entity (for debugging/UI)
  getTimeUntilDeath(entityId) {
    if (!this.criticalNeedTime.has(entityId)) {
      return null;
    }
    const framesRemaining = this.timeToDieFrames - this.criticalNeedTime.get(entityId);
    return Math.max(0, framesRemaining / 30); // Convert to seconds
  }
}
