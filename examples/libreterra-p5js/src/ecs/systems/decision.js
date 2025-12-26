// Decision-making system (Phase 5)
// Uses libreconomy stub to decide what entities should do

class DecisionSystem {
  constructor(libreconomyStub, worldQuery) {
    this.libreconomyStub = libreconomyStub;
    this.worldQuery = worldQuery;

    // How often to make decisions (frames)
    this.decisionInterval = 30; // Make decisions once per second at 30 FPS
    this.frameCounter = 0;

    // Store current intents for each entity
    this.entityIntents = new Map(); // eid -> Intent
  }

  update(ecsWorld) {
    this.frameCounter++;

    // Only make new decisions at intervals
    const shouldDecide = this.frameCounter % this.decisionInterval === 0;

    const entities = allEntitiesQuery(ecsWorld);

    for (const eid of entities) {
      const currentState = State.current[eid];

      // Get or create intent for this entity
      let intent = this.entityIntents.get(eid);

      // Decide on a new action if:
      // 1. It's time to make decisions (interval)
      // 2. Entity has no intent
      // 3. Entity is idle (previous action completed)
      // 4. Entity's intent is no longer valid
      const needsNewDecision =
        shouldDecide ||
        !intent ||
        currentState === EntityState.IDLE ||
        !this.libreconomyStub.validateIntent(intent, eid, ecsWorld, this.worldQuery);

      if (needsNewDecision) {
        // Get decision from libreconomy stub
        intent = this.libreconomyStub.decide(eid, ecsWorld, this.worldQuery);
        this.entityIntents.set(eid, intent);

        // Apply the intent to entity components
        this.applyIntent(eid, intent, ecsWorld);
      }

      // Update intent if target entity has moved (for hunting)
      if (intent && intent.targetEntity !== undefined) {
        if (!this.libreconomyStub.validateIntent(intent, eid, ecsWorld, this.worldQuery)) {
          // Target lost, make new decision
          intent = this.libreconomyStub.decide(eid, ecsWorld, this.worldQuery);
          this.entityIntents.set(eid, intent);
          this.applyIntent(eid, intent, ecsWorld);
        } else {
          // Update target position
          const targetX = Position.x[intent.targetEntity];
          const targetY = Position.y[intent.targetEntity];

          // Validate target entity position
          if (!isNaN(targetX) && !isNaN(targetY) && targetX !== undefined && targetY !== undefined) {
            intent.target = { x: targetX, y: targetY };
            Target.x[eid] = targetX;
            Target.y[eid] = targetY;
          } else {
            // Target entity has invalid position, make new decision
            intent = this.libreconomyStub.decide(eid, ecsWorld, this.worldQuery);
            this.entityIntents.set(eid, intent);
            this.applyIntent(eid, intent, ecsWorld);
          }
        }
      }
    }
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
    return this.entityIntents.get(entityId);
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
