// Age and lifespan management system
// Manages age-based max energy curve, aging, and death from old age

class AgeSystem {
  constructor() {
    // Energy curve parameters
    this.CHILDHOOD_DURATION = 0.20;  // First 20% of life
    this.ADULTHOOD_PEAK = 0.50;      // Peak at 50% of life
    this.ENERGY_CHILDHOOD_PENALTY = 0.20;  // 20% lower energy during childhood

    // Lifespan modifiers
    this.MAX_LIFESPAN_BONUS = 0.20;  // Maximum 20% lifespan extension
    this.ENERGY_HISTORY_FRAMES = 3600;  // Track last 60 seconds (at 60 FPS)
  }

  update(ecsWorld, frameCounter) {
    const entities = allEntitiesQuery(ecsWorld);

    for (const eid of entities) {
      const species = SpeciesComponent.type[eid];
      const birthFrame = Age.birthFrame[eid];
      const expectedLifespan = Age.expectedLifespanFrames[eid];

      // Calculate current age
      const currentAge = frameCounter - birthFrame;
      const agePercent = currentAge / expectedLifespan;

      // Calculate base max energy for species
      const baseMaxEnergy = species === Species.HUMAN ? 100 : 80;

      // Calculate age-adjusted max energy using curve
      const ageAdjustedMax = this.calculateMaxEnergyCurve(agePercent, baseMaxEnergy);
      Energy.max[eid] = ageAdjustedMax;

      // Update rolling average energy (for health tracking)
      const currentEnergy = Energy.current[eid];
      const oldAverage = Age.energyHistory[eid];
      const alpha = 1 / this.ENERGY_HISTORY_FRAMES;  // Exponential moving average
      Age.energyHistory[eid] = oldAverage * (1 - alpha) + currentEnergy * alpha;

      // Calculate health-based lifespan modifier
      const healthScore = Age.energyHistory[eid] / baseMaxEnergy;  // 0-1 scale

      // Health affects lifespan:
      // - Healthy (energy > 70%): Slow aging (up to 20% lifespan bonus)
      // - Unhealthy (energy < 30%): Fast aging (lifespan decreases faster)
      let lifespanModifier = 1.0;
      if (healthScore > 0.70) {
        // Good health: lifespan bonus (up to +20%)
        const bonus = ((healthScore - 0.70) / 0.30) * this.MAX_LIFESPAN_BONUS;
        lifespanModifier = 1.0 + bonus;
      } else if (healthScore < 0.30) {
        // Poor health: lifespan penalty (can reach 0, causing early death)
        lifespanModifier = healthScore / 0.30;  // Linear scale from 0% to 100%
      }

      // Adjust expected lifespan (but can't exceed max bonus)
      const baseLifespan = calculateLifespanFrames(species);
      const minLifespan = baseLifespan * 0.80;  // Can decrease to 80% of base
      const maxLifespan = baseLifespan * 1.20;  // Can increase to 120% of base
      Age.expectedLifespanFrames[eid] = Math.max(
        minLifespan,
        Math.min(maxLifespan, baseLifespan * lifespanModifier)
      );

      // Check for premature death from poor health
      if (Age.energyHistory[eid] < 5 && agePercent > 0.20) {
        // Prolonged critical health → death
        this.dieFromPoorHealth(eid, ecsWorld);
        continue;
      }

      // Check for death from old age
      if (agePercent >= 1.0) {
        this.dieFromOldAge(eid, ecsWorld);
      }
    }
  }

  // Calculate max energy based on age percentage
  // Curve: 20% lower at 0%, peak at 50%, zero at 100%
  calculateMaxEnergyCurve(agePercent, baseMax) {
    if (agePercent <= this.CHILDHOOD_DURATION) {
      // Childhood: linearly increase from 80% to 100% of base
      const progress = agePercent / this.CHILDHOOD_DURATION;
      const energyPercent = 0.80 + (progress * 0.20);
      return baseMax * energyPercent;
    } else if (agePercent <= this.ADULTHOOD_PEAK) {
      // Early adulthood: at peak (100% of base)
      return baseMax;
    } else {
      // Late adulthood to death: linearly decrease from 100% to 0%
      const declinePhase = (agePercent - this.ADULTHOOD_PEAK) / (1.0 - this.ADULTHOOD_PEAK);
      const energyPercent = 1.0 - declinePhase;
      return Math.max(0, baseMax * energyPercent);  // Ensure non-negative
    }
  }

  // Handle death from old age
  dieFromOldAge(entityId, ecsWorld) {
    removeEntityFromWorld(ecsWorld, entityId);
    // console.log(`Entity ${entityId} died from old age`);
  }

  // Handle death from poor health
  dieFromPoorHealth(entityId, ecsWorld) {
    removeEntityFromWorld(ecsWorld, entityId);
    // console.log(`Entity ${entityId} died from poor health`);
  }

  // Get entity's current age in simulation years
  getAgeInYears(entityId, frameCounter) {
    const birthFrame = Age.birthFrame[entityId];
    const currentAge = frameCounter - birthFrame;
    const FRAMES_PER_SIM_YEAR = 4_320;  // 100 years = 120 minutes at 60 FPS
    return currentAge / FRAMES_PER_SIM_YEAR;
  }

  // Get percentage of expected lifespan (for debugging/UI)
  getAgePercent(entityId, frameCounter) {
    const birthFrame = Age.birthFrame[entityId];
    const expectedLifespan = Age.expectedLifespanFrames[entityId];
    const currentAge = frameCounter - birthFrame;
    return currentAge / expectedLifespan;
  }
}
