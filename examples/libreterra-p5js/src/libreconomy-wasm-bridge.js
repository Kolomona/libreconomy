// WASM libreconomy bridge - Adapter between WASM and libreterra
// Maintains API compatibility with libreconomy-stub.js

class LibreconomyWasmBridge {
  constructor() {
    // WASM instances (will be set after WASM loads)
    this.decisionMaker = null;
    this.wasmWorld = null;

    // Map bitECS entity IDs to WASM entity IDs
    this.entityMap = new Map(); // bitECS eid -> WASM eid
    this.reverseMap = new Map(); // WASM eid -> bitECS eid

    // Decision thresholds (matching stub)
    this.thresholds = {
      criticalThirst: 80,
      highThirst: 60,
      criticalHunger: 70,
      highHunger: 50,
      criticalTiredness: 85,
      highTiredness: 70
    };

    // Utility weights (matching stub)
    this.weights = {
      survival: 2.0,
      comfort: 1.0,
      efficiency: 0.5
    };

    console.log('✓ LibreconomyWasmBridge constructed (waiting for WASM initialization)');
  }

  // Initialize with WASM instances
  initialize(WasmWorld, WasmDecisionMaker) {
    try {
      if (!WasmWorld || !WasmDecisionMaker) {
        throw new Error('WASM classes not provided to initialize()');
      }
      this.wasmWorld = new WasmWorld();
      this.decisionMaker = new WasmDecisionMaker();
      console.log('✓ LibreconomyWasmBridge initialized with WASM');
    } catch (error) {
      console.error('Failed to initialize WASM bridge:', error);
      throw error;
    }
  }

  // Ensure WASM entity exists for bitECS entity
  ensureWasmEntity(entityId) {
    if (this.entityMap.has(entityId)) {
      return this.entityMap.get(entityId);
    }

    // Determine species from bitECS
    const species = SpeciesComponent.type[entityId];
    const wasmEid = species === Species.RABBIT
      ? this.wasmWorld.create_rabbit()
      : this.wasmWorld.create_human();

    this.entityMap.set(entityId, wasmEid);
    this.reverseMap.set(wasmEid, entityId);

    return wasmEid;
  }

  // Sync bitECS needs to WASM world
  syncNeedsToWasm(entityId) {
    const wasmEid = this.ensureWasmEntity(entityId);
    const hunger = Needs.hunger[entityId];
    const thirst = Needs.thirst[entityId];
    const tiredness = Needs.tiredness[entityId];

    this.wasmWorld.set_needs(wasmEid, thirst, hunger, tiredness);
  }

  // Main decision-making function (matches libreconomy-stub.js API)
  decide(entityId, ecsWorld, worldQuery) {
    if (!this.wasmWorld || !this.decisionMaker) {
      console.error('WASM not initialized!');
      return {
        type: IntentType.WANDER,
        utility: 0.1,
        reason: 'WASM not initialized'
      };
    }

    // Sync entity state to WASM
    this.syncNeedsToWasm(entityId);
    const wasmEid = this.entityMap.get(entityId);

    // Create WASM-compatible world query wrapper
    const wasmWorldQuery = {
      getNearbyAgents: (agentId, maxCount) => {
        // Convert WASM agent ID back to bitECS ID
        const bitecsId = this.reverseMap.get(agentId);
        if (bitecsId === undefined) return [];

        const nearbyBitecs = worldQuery.getNearbyEntities(
          bitecsId, null, maxCount, 1000
        );

        // Convert back to WASM IDs
        return nearbyBitecs
          .map(eid => this.entityMap.get(eid))
          .filter(id => id !== undefined);
      },

      getNearbyResources: (agentId, resourceType, maxRadius) => {
        const bitecsId = this.reverseMap.get(agentId);
        if (bitecsId === undefined) return [];

        return worldQuery.getNearbyResources(
          bitecsId, resourceType, maxRadius
        );
      },

      canInteract: (agent1Id, agent2Id) => {
        const bitecs1 = this.reverseMap.get(agent1Id);
        const bitecs2 = this.reverseMap.get(agent2Id);
        if (bitecs1 === undefined || bitecs2 === undefined) return false;

        return worldQuery.canInteract(bitecs1, bitecs2, 50);
      }
    };

    // Get decision from WASM
    const decision = this.decisionMaker.decide_libreterra(
      this.wasmWorld,
      wasmEid,
      wasmWorldQuery
    );

    // Convert to libreterra intent format
    const intentType = this.convertIntentType(decision.intent_type);

    // Find target using world query
    let target = null;
    let targetEntity = null;

    if (decision.intent_type === 'SEEK_WATER') {
      const waterSources = worldQuery.getNearbyResources(entityId, 'water', 1000);
      if (waterSources.length > 0) {
        target = { x: waterSources[0].x, y: waterSources[0].y };
      }
    } else if (decision.intent_type === 'SEEK_FOOD') {
      // Food seeking - species-dependent
      const species = SpeciesComponent.type[entityId];
      if (species === Species.RABBIT) {
        const grassSources = worldQuery.getNearbyResources(entityId, 'grass', 1000);
        if (grassSources.length > 0) {
          target = { x: grassSources[0].x, y: grassSources[0].y };
        }
      } else if (species === Species.HUMAN) {
        const nearbyRabbits = worldQuery.getNearbyEntities(entityId, Species.RABBIT, 5, 1000);
        if (nearbyRabbits.length > 0) {
          targetEntity = nearbyRabbits[0];
          target = {
            x: Position.x[targetEntity],
            y: Position.y[targetEntity]
          };
        }
      }
    }

    return {
      type: intentType,
      target: target,
      targetEntity: targetEntity,
      utility: decision.utility,
      reason: decision.reason
    };
  }

  // Convert WASM intent type strings to libreterra IntentType enum
  convertIntentType(intentTypeStr) {
    switch (intentTypeStr) {
      case 'SEEK_WATER':
        return IntentType.SEEK_WATER;
      case 'SEEK_FOOD':
        return IntentType.SEEK_FOOD;
      case 'REST':
        return IntentType.REST;
      case 'WANDER':
      default:
        return IntentType.WANDER;
    }
  }

  // Validation (same as stub)
  validateIntent(intent, entityId, ecsWorld, worldQuery) {
    // Check if target entity still exists
    if (intent.type === IntentType.SEEK_FOOD && intent.targetEntity !== undefined) {
      const entities = allEntitiesQuery(ecsWorld);
      if (!entities.includes(intent.targetEntity)) {
        return false;
      }
      // Update target position in case prey moved
      intent.target = {
        x: Position.x[intent.targetEntity],
        y: Position.y[intent.targetEntity]
      };
    }

    // Check if target is within world bounds
    if (intent.target) {
      const x = Math.floor(intent.target.x);
      const y = Math.floor(intent.target.y);
      if (x < 0 || x >= CONFIG.WORLD_WIDTH || y < 0 || y >= CONFIG.WORLD_HEIGHT) {
        return false;
      }
    }

    return true;
  }

  // Helper to get readable intent name (same as stub)
  getIntentName(intentType) {
    switch (intentType) {
      case IntentType.SEEK_WATER: return 'SeekWater';
      case IntentType.SEEK_FOOD: return 'SeekFood';
      case IntentType.REST: return 'Rest';
      case IntentType.WANDER: return 'Wander';
      default: return 'Unknown';
    }
  }

  // Clean up when entity is removed
  removeEntity(entityId) {
    const wasmEid = this.entityMap.get(entityId);
    if (wasmEid !== undefined) {
      this.wasmWorld.remove_agent(wasmEid);
      this.reverseMap.delete(wasmEid);
      this.entityMap.delete(entityId);
    }
  }
}
