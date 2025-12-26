// Movement system (Phase 6)
// Moves entities toward their targets

class MovementSystem {
  constructor(terrainGrid) {
    this.terrainGrid = terrainGrid;

    // Movement speeds (pixels per frame)
    this.speeds = {
      human: {
        walk: 2.0,
        run: 4.0    // Used when hunting
      },
      rabbit: {
        walk: 1.5,
        run: 3.5    // Used when fleeing
      }
    };

    // Distance at which target is considered reached
    this.arrivalThreshold = 5;

    // Obstacle avoidance parameters
    this.avoidanceRadius = 20;
    this.avoidanceStrength = 0.5;
  }

  update(ecsWorld, deltaTime = 1.0) {
    const entities = movingEntitiesQuery(ecsWorld);

    for (const eid of entities) {
      const hasTarget = Target.hasTarget[eid];
      const state = State.current[eid];

      // Validate state/target consistency
      if (hasTarget === 0 && state === EntityState.MOVING) {
        // INCONSISTENT: Entity is MOVING but has no target
        // This indicates a bug - fix it by setting state to IDLE
        State.current[eid] = EntityState.IDLE;
        Velocity.vx[eid] = 0;
        Velocity.vy[eid] = 0;
        continue;
      }

      // Skip if not moving or has no target
      if (hasTarget === 0 || state !== EntityState.MOVING) {
        Velocity.vx[eid] = 0;
        Velocity.vy[eid] = 0;
        continue;
      }

      const species = SpeciesComponent.type[eid];

      // Calculate direction to target
      const dx = Target.x[eid] - Position.x[eid];
      const dy = Target.y[eid] - Position.y[eid];
      const distance = Math.sqrt(dx * dx + dy * dy);

      // Check if arrived at target or distance is zero (CRITICAL: prevents NaN from division by zero)
      if (distance <= this.arrivalThreshold || distance === 0) {
        // Reached target - stop moving
        Velocity.vx[eid] = 0;
        Velocity.vy[eid] = 0;
        Target.hasTarget[eid] = 0;

        // Set state to IDLE - let DecisionSystem decide next action
        // ConsumptionSystem will handle resource interactions if entity is at a resource
        State.current[eid] = EntityState.IDLE;
        continue;
      }

      // Normalize direction (safe now - distance > 0)
      const dirX = dx / distance;
      const dirY = dy / distance;

      // Determine speed based on species and state
      const speedConfig = species === Species.HUMAN ? this.speeds.human : this.speeds.rabbit;
      let speed = speedConfig.walk;

      // Use run speed for hunting (humans) or if low on time (high needs)
      const hunger = Needs.hunger[eid];
      const thirst = Needs.thirst[eid];
      if (hunger > 80 || thirst > 80) {
        speed = speedConfig.run;
      }

      // Apply terrain-based speed modifier
      const terrain = this.terrainGrid.get(
        Math.floor(Position.x[eid]),
        Math.floor(Position.y[eid])
      );
      const terrainAttributes = CONFIG.SPECIES_TERRAIN_ATTRIBUTES[species][terrain];
      speed *= terrainAttributes.speedMultiplier;

      // Apply speed modifier based on delta time
      speed *= deltaTime;

      // Calculate desired velocity
      let desiredVx = dirX * speed;
      let desiredVy = dirY * speed;

      // Simple obstacle avoidance (avoid water)
      const avoidance = this.calculateAvoidance(eid, dirX, dirY);
      desiredVx += avoidance.x * this.avoidanceStrength;
      desiredVy += avoidance.y * this.avoidanceStrength;

      // Set velocity
      Velocity.vx[eid] = desiredVx;
      Velocity.vy[eid] = desiredVy;

      // Update position
      let newX = Position.x[eid] + Velocity.vx[eid];
      let newY = Position.y[eid] + Velocity.vy[eid];

      // GUARD: Validate new position before applying
      if (isNaN(newX) || isNaN(newY)) {
        // Position calculation resulted in NaN, stop movement
        Velocity.vx[eid] = 0;
        Velocity.vy[eid] = 0;
        Target.hasTarget[eid] = 0;
        State.current[eid] = EntityState.IDLE;
        continue;
      }

      // Clamp to world bounds
      newX = Math.max(0, Math.min(CONFIG.WORLD_WIDTH - 1, newX));
      newY = Math.max(0, Math.min(CONFIG.WORLD_HEIGHT - 1, newY));

      // Check if new position is traversable for this species
      const targetTerrain = this.terrainGrid.get(Math.floor(newX), Math.floor(newY));
      const targetAttributes = CONFIG.SPECIES_TERRAIN_ATTRIBUTES[species][targetTerrain];

      if (targetAttributes.speedMultiplier > 0) {
        // Terrain is traversable - allow movement
        Position.x[eid] = newX;
        Position.y[eid] = newY;
      } else {
        // Terrain is impassable for this species, try sliding along it
        const slideX = Position.x[eid] + Velocity.vx[eid];
        const slideY = Position.y[eid];
        const slideTerrainX = this.terrainGrid.get(Math.floor(slideX), Math.floor(slideY));
        const slideAttributesX = CONFIG.SPECIES_TERRAIN_ATTRIBUTES[species][slideTerrainX];

        if (slideAttributesX.speedMultiplier > 0) {
          Position.x[eid] = slideX;
          Position.y[eid] = slideY;
        } else {
          // Try sliding vertically
          const slideX2 = Position.x[eid];
          const slideY2 = Position.y[eid] + Velocity.vy[eid];
          const slideTerrainY = this.terrainGrid.get(Math.floor(slideX2), Math.floor(slideY2));
          const slideAttributesY = CONFIG.SPECIES_TERRAIN_ATTRIBUTES[species][slideTerrainY];

          if (slideAttributesY.speedMultiplier > 0) {
            Position.x[eid] = slideX2;
            Position.y[eid] = slideY2;
          } else {
            // Completely blocked, stop and re-decide
            Velocity.vx[eid] = 0;
            Velocity.vy[eid] = 0;
            Target.hasTarget[eid] = 0;
            State.current[eid] = EntityState.IDLE;
          }
        }
      }
    }
  }

  // Calculate avoidance vector for nearby obstacles (water)
  calculateAvoidance(entityId, dirX, dirY) {
    const x = Position.x[entityId];
    const y = Position.y[entityId];

    let avoidX = 0;
    let avoidY = 0;

    // Sample points around entity in movement direction
    const samples = 8;
    for (let i = 0; i < samples; i++) {
      const angle = (i / samples) * Math.PI * 2;
      const checkX = Math.floor(x + Math.cos(angle) * this.avoidanceRadius);
      const checkY = Math.floor(y + Math.sin(angle) * this.avoidanceRadius);

      // Check if this point is walkable
      if (!this.terrainGrid.isWalkable(checkX, checkY)) {
        // Push away from this obstacle
        const awayX = x - checkX;
        const awayY = y - checkY;
        const dist = Math.sqrt(awayX * awayX + awayY * awayY);

        if (dist > 0) {
          avoidX += (awayX / dist) * (this.avoidanceRadius - dist) / this.avoidanceRadius;
          avoidY += (awayY / dist) * (this.avoidanceRadius - dist) / this.avoidanceRadius;
        }
      }
    }

    return { x: avoidX, y: avoidY };
  }

  // Get entity's current speed
  getCurrentSpeed(entityId) {
    const vx = Velocity.vx[entityId];
    const vy = Velocity.vy[entityId];
    return Math.sqrt(vx * vx + vy * vy);
  }

  // Stop entity movement
  stop(entityId) {
    Velocity.vx[entityId] = 0;
    Velocity.vy[entityId] = 0;
    Target.hasTarget[entityId] = 0;
    State.current[entityId] = EntityState.IDLE;
  }
}
