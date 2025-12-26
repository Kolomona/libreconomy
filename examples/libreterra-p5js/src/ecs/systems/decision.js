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
          intent.target = {
            x: Position.x[intent.targetEntity],
            y: Position.y[intent.targetEntity]
          };
          Target.x[eid] = intent.target.x;
          Target.y[eid] = intent.target.y;
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
        if (intent.target) {
          Target.x[entityId] = intent.target.x;
          Target.y[entityId] = intent.target.y;
          Target.hasTarget[entityId] = 1;
          State.current[entityId] = EntityState.MOVING;
        } else {
          // No target found nearby - wander to explore
          console.warn(`Entity ${entityId} has ${this.libreconomyStub.getIntentName(intent.type)} intent but no target. Wandering.`);
          const wanderTarget = this.libreconomyStub.getWanderTarget(entityId);
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
        const wanderTarget = this.libreconomyStub.getWanderTarget(entityId);
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
}
