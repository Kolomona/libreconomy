// Decision-making system (Phase 5)
// Uses libreconomy stub to decide what entities should do

class DecisionSystem {
  constructor(libreconomyStub, worldQuery) {
    this.libreconomyStub = libreconomyStub;
    this.worldQuery = worldQuery;

    // Store current intents and their urgency for each entity
    this.entityIntents = new Map(); // eid -> { intent, initialUrgency }

    // Interruption threshold: new intent must be this much more urgent
    this.INTERRUPT_THRESHOLD = 20;  // Urgency points

    // Decision result cache - only re-decide when needs change significantly
    this.decisionCache = new Map(); // eid → { decision, needsSnapshot, frame }
    this.DECISION_THRESHOLD = 15;   // Re-decide when any need changes by 15+
    this.MAX_DECISION_AGE = 120;    // Force re-decide after 2 seconds (120 frames @ 60fps)
  }

  update(ecsWorld, frameCounter) {
    const entities = allEntitiesQuery(ecsWorld);

    for (const eid of entities) {
      const currentState = State.current[eid];

      // Get current intent and urgency
      const intentData = this.entityIntents.get(eid);
      const currentIntent = intentData ? intentData.intent : null;
      const currentUrgency = intentData ? intentData.initialUrgency : 0;

      // Check if we should re-decide (uses cache)
      if (this.shouldRedecide(eid, this.decisionCache.get(eid), frameCounter)) {
        // Get new decision from libreconomy
        const newIntent = this.libreconomyStub.decide(eid, ecsWorld, this.worldQuery);

        // Cache the decision
        this.cacheDecision(eid, newIntent, frameCounter);

        // Check if we should interrupt current activity
        if (this.shouldInterrupt(currentIntent, newIntent, currentState, currentUrgency, eid, ecsWorld)) {
          // Apply new intent
          this.applyIntent(eid, newIntent, ecsWorld);

          // Store with urgency
          this.entityIntents.set(eid, {
            intent: newIntent,
            initialUrgency: newIntent.urgency
          });
        }
      } else {
        // Use cached decision, just update current intent targets
        this.updateCurrentIntent(eid, currentIntent);
      }
    }
  }

  // Check if we should re-decide (or use cached decision)
  shouldRedecide(eid, cached, currentFrame) {
    if (!cached) return true;  // No cache
    if (State.current[eid] === EntityState.IDLE) return true;  // Always when idle

    // Check if needs changed significantly
    const hungerDiff = Math.abs(Needs.hunger[eid] - cached.needsSnapshot.hunger);
    const thirstDiff = Math.abs(Needs.thirst[eid] - cached.needsSnapshot.thirst);
    const tirednessDiff = Math.abs(Needs.tiredness[eid] - cached.needsSnapshot.tiredness);

    if (hungerDiff > this.DECISION_THRESHOLD ||
        thirstDiff > this.DECISION_THRESHOLD ||
        tirednessDiff > this.DECISION_THRESHOLD) {
      return true;  // Needs changed significantly
    }

    // Force re-decide if decision is stale
    if (currentFrame - cached.frame > this.MAX_DECISION_AGE) {
      return true;
    }

    return false;  // Use cached decision
  }

  // Cache a decision with needs snapshot
  cacheDecision(eid, decision, frameCounter) {
    this.decisionCache.set(eid, {
      decision: decision,
      needsSnapshot: {
        hunger: Needs.hunger[eid],
        thirst: Needs.thirst[eid],
        tiredness: Needs.tiredness[eid]
      },
      frame: frameCounter
    });
  }

  // Update current intent targets (for hunting and wandering)
  updateCurrentIntent(eid, currentIntent) {
    if (!currentIntent) return;

    // Hunting: update target to follow moving prey
    if (currentIntent.targetEntity !== undefined) {
      const targetX = Position.x[currentIntent.targetEntity];
      const targetY = Position.y[currentIntent.targetEntity];

      // Validate and update target position
      if (!isNaN(targetX) && !isNaN(targetY) && targetX !== undefined && targetY !== undefined) {
        currentIntent.target = { x: targetX, y: targetY };
        Target.x[eid] = targetX;
        Target.y[eid] = targetY;
      }
    } else if (currentIntent.type === IntentType.WANDER) {
      // Wandering: check if reached target, generate new one if so
      if (Target.hasTarget[eid] === 0) {
        // No target - entity reached previous wander target
        // Generate new wander target
        const wanderTarget = this.getWanderTarget(eid);
        Target.x[eid] = wanderTarget.x;
        Target.y[eid] = wanderTarget.y;
        Target.hasTarget[eid] = 1;
        State.current[eid] = EntityState.MOVING;
        currentIntent.target = { x: wanderTarget.x, y: wanderTarget.y };
      }
    }
  }

  // Determine if new intent should interrupt current activity
  shouldInterrupt(currentIntent, newIntent, currentState, currentUrgency, entityId, ecsWorld) {
    // Always allow decisions when IDLE (no current activity)
    if (currentState === EntityState.IDLE) {
      return true;
    }

    // No current intent - allow new one
    if (!currentIntent) {
      return true;
    }

    // Force sleep from exhaustion ALWAYS interrupts
    const tiredness = Needs.tiredness[entityId];
    if (tiredness >= 100 && newIntent.type === IntentType.REST) {
      return true;
    }

    // Check if energy is critically low (force REST)
    const energyPercent = (Energy.current[entityId] / Energy.max[entityId]) * 100;
    if (energyPercent < 10 && newIntent.type === IntentType.REST) {
      return true;  // Always allow REST when energy critical
    }

    // SPECIAL: Protect EATING/DRINKING from intent validation interruptions
    // When consuming, targets become stale but activity should continue
    const isConsuming = currentState === EntityState.EATING || currentState === EntityState.DRINKING;

    if (!isConsuming) {
      // Validate current intent - if invalid, interrupt (but NOT during consumption)
      if (!this.libreconomyStub.validateIntent(currentIntent, entityId, ecsWorld, this.worldQuery)) {
        return true;
      }
    }

    // Compare urgency: only interrupt if significantly more urgent
    const urgencyDifference = newIntent.urgency - currentUrgency;

    // SPECIAL: Require higher threshold to interrupt consumption
    const threshold = isConsuming ? 40 : this.INTERRUPT_THRESHOLD;

    // Must exceed threshold to interrupt
    if (urgencyDifference >= threshold) {
      // console.log(`Entity ${entityId}: Interrupting ${currentIntent.type} (urgency=${currentUrgency}) for ${newIntent.type} (urgency=${newIntent.urgency}), diff=${urgencyDifference}`);
      return true;
    }

    // Not urgent enough - continue current activity
    return false;
  }

  // Apply an intent to entity's components
  applyIntent(entityId, intent, ecsWorld) {
    switch (intent.type) {
      case IntentType.SEEK_WATER:
        // Set target position
        if (intent.target && !isNaN(intent.target.x) && !isNaN(intent.target.y)) {
          Target.x[entityId] = intent.target.x;
          Target.y[entityId] = intent.target.y;
          Target.hasTarget[entityId] = 1;
          State.current[entityId] = EntityState.MOVING;
        } else {
          // No target found nearby - wander to explore
          const wanderTarget = this.getWanderTarget(entityId, ecsWorld);
          Target.x[entityId] = wanderTarget.x;
          Target.y[entityId] = wanderTarget.y;
          Target.hasTarget[entityId] = 1;
          State.current[entityId] = EntityState.MOVING;
        }
        break;

      case IntentType.SEEK_FOOD:
        // Set target position
        if (intent.target && !isNaN(intent.target.x) && !isNaN(intent.target.y)) {
          Target.x[entityId] = intent.target.x;
          Target.y[entityId] = intent.target.y;
          Target.hasTarget[entityId] = 1;
          State.current[entityId] = EntityState.MOVING;
        } else {
          // No target found nearby - wander to explore
          // Use larger radius if desperate (high hunger)
          const hunger = Needs.hunger[entityId];
          const wanderTarget = this.getWanderTarget(entityId, ecsWorld, hunger > 70);
          Target.x[entityId] = wanderTarget.x;
          Target.y[entityId] = wanderTarget.y;
          Target.hasTarget[entityId] = 1;
          State.current[entityId] = EntityState.MOVING;
        }
        break;

      case IntentType.REST:
        // Stop moving and rest in place
        Target.hasTarget[entityId] = 0;
        State.current[entityId] = EntityState.SLEEPING;
        Velocity.vx[entityId] = 0;
        Velocity.vy[entityId] = 0;
        break;

      case IntentType.WANDER:
        // Pick a random nearby location
        const wanderTarget = this.getWanderTarget(entityId);
        Target.x[entityId] = wanderTarget.x;
        Target.y[entityId] = wanderTarget.y;
        Target.hasTarget[entityId] = 1;
        State.current[entityId] = EntityState.MOVING;
        break;

      default:
        // Unknown intent, go idle
        Target.hasTarget[entityId] = 0;
        State.current[entityId] = EntityState.IDLE;
        break;
    }
  }

  // Check if entity has reached its target
  hasReachedTarget(entityId, threshold = 10) {
    if (Target.hasTarget[entityId] === 0) {
      return false;
    }

    const dx = Target.x[entityId] - Position.x[entityId];
    const dy = Target.y[entityId] - Position.y[entityId];
    const distance = Math.sqrt(dx * dx + dy * dy);

    return distance <= threshold;
  }

  // Get current intent for debugging
  getIntent(entityId) {
    const intentData = this.entityIntents.get(entityId);
    return intentData ? intentData.intent : null;
  }

  // Clear intent (when entity dies, etc.)
  clearIntent(entityId) {
    this.entityIntents.delete(entityId);
    this.decisionCache.delete(entityId);
  }

  // Calculate terrain-cost-aware wander target (Phase 3.1)
  getWanderTarget(entityId, ecsWorld, desperate = false) {
    const x = Position.x[entityId];
    const y = Position.y[entityId];

    // GUARD: Validate position - use world center as safe fallback if invalid
    if (isNaN(x) || isNaN(y) || x === undefined || y === undefined) {
      return {
        x: CONFIG.WORLD_WIDTH / 2,
        y: CONFIG.WORLD_HEIGHT / 2
      };
    }

    const species = SpeciesComponent.type[entityId];

    // Sample 8 directions and weight by terrain cost
    const sampleDirections = 8;
    const candidates = [];

    for (let i = 0; i < sampleDirections; i++) {
      const angle = (i / sampleDirections) * Math.PI * 2;

      // If desperate (high hunger), explore even farther
      // Normal: 200-600 pixels (increased from 100-300)
      // Desperate: 400-1000 pixels
      const distance = desperate
        ? 400 + Math.random() * 600
        : 200 + Math.random() * 400;
      const targetX = Math.max(0, Math.min(CONFIG.WORLD_WIDTH - 1, x + Math.cos(angle) * distance));
      const targetY = Math.max(0, Math.min(CONFIG.WORLD_HEIGHT - 1, y + Math.sin(angle) * distance));

      // Calculate terrain cost along path
      const cost = this.samplePathCost(x, y, targetX, targetY, species);

      // Weight inversely by cost (lower cost = higher weight)
      // Use exponential to strongly prefer low-cost paths while still allowing high-cost occasionally
      const weight = Math.exp(-cost / 10);  // Exponential decay

      candidates.push({ x: targetX, y: targetY, weight });
    }

    // Weighted random selection (prefer low-cost, but don't prohibit high-cost)
    const totalWeight = candidates.reduce((sum, c) => sum + c.weight, 0);
    let random = Math.random() * totalWeight;

    for (const candidate of candidates) {
      random -= candidate.weight;
      if (random <= 0) {
        return { x: candidate.x, y: candidate.y };
      }
    }

    // Fallback (should rarely happen)
    return { x: x + 100, y: y };
  }

  // Sample terrain cost along a straight-line path (Phase 3.1 helper)
  samplePathCost(x1, y1, x2, y2, species) {
    // Sample 5 points along the line
    let totalCost = 0;
    for (let t = 0; t <= 1; t += 0.2) {
      const sampleX = Math.floor(x1 + (x2 - x1) * t);
      const sampleY = Math.floor(y1 + (y2 - y1) * t);
      const terrain = terrainGrid.get(sampleX, sampleY);
      const attributes = CONFIG.SPECIES_TERRAIN_ATTRIBUTES[species][terrain];

      // Cost = (time cost) * (energy cost)
      // Time cost: inverse of speed (slower = higher cost)
      // Energy cost: direct multiplier
      const timeCost = 1 / (attributes.speedMultiplier + 0.01);  // Avoid division by zero
      const cost = timeCost * attributes.energyMultiplier;
      totalCost += cost;
    }
    return totalCost;
  }
}
