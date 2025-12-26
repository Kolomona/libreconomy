// Needs decay system (Phase 5)
// Gradually increases hunger, thirst, and tiredness over time

class NeedsDecaySystem {
  constructor() {
    // Decay rates per frame (at 30 FPS)
    // These values determine how quickly needs increase
    this.decayRates = {
      // Base decay rates (per frame)
      hunger: 0.02,      // Takes ~83 seconds to go from 0 to 50 (hungry threshold)
      thirst: 0.025,     // Slightly faster than hunger
      tiredness: 0.015,  // Slower than hunger/thirst

      // Activity multipliers (applied when entity is moving/active)
      activityMultiplier: {
        hunger: 1.5,     // Moving makes you hungrier faster
        thirst: 2.0,     // Moving makes you thirstier much faster
        tiredness: 2.5   // Moving makes you tired faster
      },

      // Species-specific multipliers
      rabbit: {
        hunger: 1.3,     // Rabbits get hungry faster
        thirst: 1.2,     // Rabbits get thirsty faster
        tiredness: 1.0
      },
      human: {
        hunger: 1.0,
        thirst: 1.0,
        tiredness: 1.0
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

      // Determine if entity is active (moving)
      const isActive = state === EntityState.MOVING;

      // Get species multipliers
      const speciesMultiplier = species === Species.RABBIT
        ? this.decayRates.rabbit
        : this.decayRates.human;

      // Calculate decay amounts
      let hungerDecay = this.decayRates.hunger * speciesMultiplier.hunger * deltaTime;
      let thirstDecay = this.decayRates.thirst * speciesMultiplier.thirst * deltaTime;
      let tirednessDecay = this.decayRates.tiredness * speciesMultiplier.tiredness * deltaTime;

      // Apply activity multipliers if moving
      if (isActive) {
        hungerDecay *= this.decayRates.activityMultiplier.hunger;
        thirstDecay *= this.decayRates.activityMultiplier.thirst;
        tirednessDecay *= this.decayRates.activityMultiplier.tiredness;
      }

      // Apply decay (increase needs)
      Needs.hunger[eid] = Math.min(this.maxNeed, Needs.hunger[eid] + hungerDecay);
      Needs.thirst[eid] = Math.min(this.maxNeed, Needs.thirst[eid] + thirstDecay);
      Needs.tiredness[eid] = Math.min(this.maxNeed, Needs.tiredness[eid] + tirednessDecay);

      // Auto pass out from exhaustion
      if (Needs.tiredness[eid] >= 100) {
        // Force entity to sleep if too tired
        if (State.current[eid] !== EntityState.SLEEPING) {
          const species = SpeciesComponent.type[eid] === Species.HUMAN ? 'Human' : 'Rabbit';
          const gender = Gender.isMale[eid] ? 'Male' : 'Female';
          console.log(`😴 Entity ${eid} (${gender} ${species}) passed out from exhaustion`);

          State.current[eid] = EntityState.SLEEPING;
          Target.hasTarget[eid] = 0;
          Velocity.vx[eid] = 0;
          Velocity.vy[eid] = 0;
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

// Death system - handles entity death from starvation/dehydration
class DeathSystem {
  constructor() {
    this.deathThreshold = 100;  // Die when need reaches 100
    this.timeToDieFrames = 300;  // Must be at 100 for 10 seconds (at 30 FPS)
    this.criticalNeedTime = new Map();  // Track how long entity has been critical
  }

  update(ecsWorld) {
    const entities = allEntitiesQuery(ecsWorld);

    for (const eid of entities) {
      const hunger = Needs.hunger[eid];
      const thirst = Needs.thirst[eid];

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
    console.log(`💀 Entity ${entityId} (${gender} ${species}) died from ${cause}`);

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
