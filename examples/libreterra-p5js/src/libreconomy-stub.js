// libreconomy stub - Mock decision-making API (Phase 4)
// Mimics the libreconomy library's decision-making interface

class LibreconomyStub {
  constructor() {
    // Decision thresholds (configurable)
    this.thresholds = {
      criticalThirst: 80,
      highThirst: 60,
      criticalHunger: 70,
      highHunger: 50,
      criticalTiredness: 85,
      highTiredness: 70
    };

    // Utility weights for decision making
    this.weights = {
      survival: 2.0,    // Multiplier for critical needs
      comfort: 1.0,     // Multiplier for high needs
      efficiency: 0.5   // Preference for nearby resources
    };
  }

  // Main decision-making function
  // Returns an Intent object: { type: IntentType, target?: {x, y}, targetEntity?: eid }
  decide(entityId, ecsWorld, worldQuery) {
    const hunger = Needs.hunger[entityId];
    const thirst = Needs.thirst[entityId];
    const tiredness = Needs.tiredness[entityId];
    const species = SpeciesComponent.type[entityId];

    // Calculate utilities for different actions
    const utilities = [];

    // 1. SEEK WATER (if thirsty)
    if (thirst > this.thresholds.highThirst) {
      const waterSources = worldQuery.getNearbyResources(entityId, 'water', 1000);
      if (waterSources.length > 0) {
        const closest = waterSources[0];
        const urgency = thirst / 100;
        const distanceFactor = Math.max(0, 1 - closest.distance / 1000);
        const utility = urgency * this.weights.survival + distanceFactor * this.weights.efficiency;

        utilities.push({
          type: IntentType.SEEK_WATER,
          target: { x: closest.x, y: closest.y },
          utility: utility,
          reason: `Thirst: ${thirst.toFixed(0)}`
        });
      } else {
        // No water found nearby - create high-priority wander intent
        const urgency = thirst / 100;
        utilities.push({
          type: IntentType.SEEK_WATER,  // Keep as SEEK_WATER (applyIntent will convert to wander)
          utility: urgency * this.weights.survival,
          reason: `Thirst: ${thirst.toFixed(0)} (searching for water)`
        });
      }
    }

    // 2. SEEK FOOD (species-dependent)
    if (hunger > this.thresholds.highHunger) {
      if (species === Species.RABBIT) {
        // Rabbits eat grass
        const grassSources = worldQuery.getNearbyResources(entityId, 'grass', 1000);
        if (grassSources.length > 0) {
          const closest = grassSources[0];
          const urgency = hunger / 100;
          const distanceFactor = Math.max(0, 1 - closest.distance / 1000);
          const utility = urgency * this.weights.survival + distanceFactor * this.weights.efficiency;

          utilities.push({
            type: IntentType.SEEK_FOOD,
            target: { x: closest.x, y: closest.y },
            utility: utility,
            reason: `Hunger: ${hunger.toFixed(0)} (grass)`
          });
        } else {
          // No grass found - wander to find it
          const urgency = hunger / 100;
          utilities.push({
            type: IntentType.SEEK_FOOD,
            utility: urgency * this.weights.survival,
            reason: `Hunger: ${hunger.toFixed(0)} (searching for grass)`
          });
        }
      } else if (species === Species.HUMAN) {
        // Humans hunt rabbits
        const nearbyRabbits = worldQuery.getNearbyEntities(entityId, Species.RABBIT, 5, 1000);
        if (nearbyRabbits.length > 0) {
          const targetRabbit = nearbyRabbits[0];
          const urgency = hunger / 100;
          const dx = Position.x[targetRabbit] - Position.x[entityId];
          const dy = Position.y[targetRabbit] - Position.y[entityId];
          const distance = Math.sqrt(dx * dx + dy * dy);
          const distanceFactor = Math.max(0, 1 - distance / 1000);
          const utility = urgency * this.weights.survival + distanceFactor * this.weights.efficiency;

          utilities.push({
            type: IntentType.SEEK_FOOD,
            target: { x: Position.x[targetRabbit], y: Position.y[targetRabbit] },
            targetEntity: targetRabbit,
            utility: utility,
            reason: `Hunger: ${hunger.toFixed(0)} (hunt rabbit)`
          });
        } else {
          // No rabbits found - wander to find them
          const urgency = hunger / 100;
          utilities.push({
            type: IntentType.SEEK_FOOD,
            utility: urgency * this.weights.survival,
            reason: `Hunger: ${hunger.toFixed(0)} (hunting for rabbits)`
          });
        }
      }
    }

    // 3. REST (if tired)
    if (tiredness > this.thresholds.highTiredness) {
      const urgency = tiredness / 100;
      const utility = urgency * this.weights.comfort;

      utilities.push({
        type: IntentType.REST,
        utility: utility,
        reason: `Tiredness: ${tiredness.toFixed(0)}`
      });
    }

    // 4. WANDER (default/exploration)
    // Lower utility so it's only chosen when no urgent needs
    utilities.push({
      type: IntentType.WANDER,
      utility: 0.1,
      reason: 'Idle exploration'
    });

    // Choose action with highest utility
    utilities.sort((a, b) => b.utility - a.utility);
    const decision = utilities[0];

    // Log decision for debugging (only occasionally to avoid spam)
    if (Math.random() < 0.01) {
      console.log(`Entity ${entityId} (${species === Species.HUMAN ? 'Human' : 'Rabbit'}) decided: ${this.getIntentName(decision.type)} - ${decision.reason}`);
    }

    return decision;
  }

  // Helper to get readable intent name
  getIntentName(intentType) {
    switch (intentType) {
      case IntentType.SEEK_WATER: return 'SeekWater';
      case IntentType.SEEK_FOOD: return 'SeekFood';
      case IntentType.REST: return 'Rest';
      case IntentType.WANDER: return 'Wander';
      default: return 'Unknown';
    }
  }

  // Check if an action is still valid (target still exists, etc.)
  validateIntent(intent, entityId, ecsWorld, worldQuery) {
    if (intent.type === IntentType.SEEK_FOOD && intent.targetEntity !== undefined) {
      // Check if target rabbit still exists
      const entities = allEntitiesQuery(ecsWorld);
      if (!entities.includes(intent.targetEntity)) {
        return false; // Rabbit was eaten or died
      }

      // Update target position (rabbit may have moved)
      intent.target = {
        x: Position.x[intent.targetEntity],
        y: Position.y[intent.targetEntity]
      };
    }

    if (intent.target) {
      // Check if target is in bounds
      const x = Math.floor(intent.target.x);
      const y = Math.floor(intent.target.y);

      if (x < 0 || x >= CONFIG.WORLD_WIDTH || y < 0 || y >= CONFIG.WORLD_HEIGHT) {
        return false;
      }
    }

    return true;
  }

  // Get a random wander target near the entity
  getWanderTarget(entityId) {
    const x = Position.x[entityId];
    const y = Position.y[entityId];

    // Wander within 200 pixels
    const angle = Math.random() * Math.PI * 2;
    const distance = Math.random() * 200 + 50;

    const targetX = Math.max(0, Math.min(CONFIG.WORLD_WIDTH, x + Math.cos(angle) * distance));
    const targetY = Math.max(0, Math.min(CONFIG.WORLD_HEIGHT, y + Math.sin(angle) * distance));

    return { x: targetX, y: targetY };
  }
}
