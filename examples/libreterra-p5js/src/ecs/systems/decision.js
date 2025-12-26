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
  }

  update(ecsWorld) {
    const entities = allEntitiesQuery(ecsWorld);

    for (const eid of entities) {
      const currentState = State.current[eid];

      // Get current intent and urgency
      const intentData = this.entityIntents.get(eid);
      const currentIntent = intentData ? intentData.intent : null;
      const currentUrgency = intentData ? intentData.initialUrgency : 0;

      // ALWAYS get new decision from libreconomy (every frame)
      const newIntent = this.libreconomyStub.decide(eid, ecsWorld, this.worldQuery);

      // Check if we should interrupt current activity
      if (this.shouldInterrupt(currentIntent, newIntent, currentState, currentUrgency, eid, ecsWorld)) {
        // Apply new intent
        this.applyIntent(eid, newIntent, ecsWorld);

        // Store with urgency
        this.entityIntents.set(eid, {
          intent: newIntent,
          initialUrgency: newIntent.urgency
        });
      } else {
        // Continue current activity - update target if hunting or wandering
        if (currentIntent && currentIntent.targetEntity !== undefined) {
          // Hunting: update target to follow moving prey
          const targetX = Position.x[currentIntent.targetEntity];
          const targetY = Position.y[currentIntent.targetEntity];

          // Validate and update target position
          if (!isNaN(targetX) && !isNaN(targetY) && targetX !== undefined && targetY !== undefined) {
            currentIntent.target = { x: targetX, y: targetY };
            Target.x[eid] = targetX;
            Target.y[eid] = targetY;
          }
        } else if (currentIntent && currentIntent.type === IntentType.WANDER) {
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
      case IntentType.SEEK_FOOD:
        // Set target position
        if (intent.target && !isNaN(intent.target.x) && !isNaN(intent.target.y)) {
          Target.x[entityId] = intent.target.x;
          Target.y[entityId] = intent.target.y;
          Target.hasTarget[entityId] = 1;
          State.current[entityId] = EntityState.MOVING;
        } else {
          // No target found nearby or target is invalid - wander to explore
          const wanderTarget = this.getWanderTarget(entityId);
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
  }

  // Calculate a random wander target near an entity
  getWanderTarget(entityId) {
    const x = Position.x[entityId];
    const y = Position.y[entityId];

    // GUARD: Validate position - use world center as safe fallback if invalid
    if (isNaN(x) || isNaN(y) || x === undefined || y === undefined) {
      return {
        x: CONFIG.WORLD_WIDTH / 2,
        y: CONFIG.WORLD_HEIGHT / 2
      };
    }

    const angle = Math.random() * Math.PI * 2;
    const distance = Math.random() * 200 + 50;
    const targetX = Math.max(0, Math.min(CONFIG.WORLD_WIDTH, x + Math.cos(angle) * distance));
    const targetY = Math.max(0, Math.min(CONFIG.WORLD_HEIGHT, y + Math.sin(angle) * distance));

    return { x: targetX, y: targetY };
  }
}
