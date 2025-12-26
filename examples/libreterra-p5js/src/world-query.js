// WorldQuery implementation (Phase 4)
// Provides spatial queries for the libreconomy decision system

class WorldQuery {
  constructor(ecsWorld, terrainGrid, spatialHash) {
    this.ecsWorld = ecsWorld;
    this.terrainGrid = terrainGrid;
    this.spatialHash = spatialHash;
  }

  // Find nearby resources of a specific type (water, grass)
  // Returns array of {x, y, distance} sorted by distance
  getNearbyResources(entityId, resourceType, maxRadius = 500, sampleStep = 10) {
    const posX = Position.x[entityId];
    const posY = Position.y[entityId];

    const resources = [];

    // Determine terrain type to search for
    let targetTerrain;
    if (resourceType === 'water') {
      targetTerrain = TerrainType.WATER;
    } else if (resourceType === 'grass') {
      targetTerrain = TerrainType.GRASS;
    } else {
      return resources;
    }

    // Scan in a spiral pattern outward from entity
    // Sample every sampleStep pixels for performance
    for (let radius = 0; radius < maxRadius; radius += sampleStep) {
      // Sample around the perimeter of current radius
      const samples = Math.max(8, Math.floor(2 * Math.PI * radius / sampleStep));

      for (let i = 0; i < samples; i++) {
        const angle = (i / samples) * Math.PI * 2;
        const x = Math.floor(posX + Math.cos(angle) * radius);
        const y = Math.floor(posY + Math.sin(angle) * radius);

        // Check bounds
        if (x < 0 || x >= this.terrainGrid.width ||
            y < 0 || y >= this.terrainGrid.height) {
          continue;
        }

        const terrain = this.terrainGrid.get(x, y);
        if (terrain === targetTerrain) {
          const dx = x - posX;
          const dy = y - posY;
          const distance = Math.sqrt(dx * dx + dy * dy);

          resources.push({ x, y, distance });

          // Return early if we found something close
          if (resources.length >= 5) {
            break;
          }
        }
      }

      // If we found resources, no need to search further
      if (resources.length > 0) {
        break;
      }
    }

    // Sort by distance
    resources.sort((a, b) => a.distance - b.distance);

    return resources;
  }

  // Find nearby entities matching a species filter
  // Returns array of entity IDs sorted by distance
  getNearbyEntities(entityId, speciesFilter = null, maxCount = 10, maxRadius = 500) {
    const posX = Position.x[entityId];
    const posY = Position.y[entityId];

    // Use spatial hash for efficient lookup
    const nearby = this.spatialHash.queryRadius(posX, posY, maxRadius);

    const results = [];

    for (const otherEid of nearby) {
      // Skip self
      if (otherEid === entityId) continue;

      // Check species filter
      if (speciesFilter !== null) {
        const species = SpeciesComponent.type[otherEid];
        if (species !== speciesFilter) continue;
      }

      // Calculate distance
      const dx = Position.x[otherEid] - posX;
      const dy = Position.y[otherEid] - posY;
      const distance = Math.sqrt(dx * dx + dy * dy);

      results.push({ eid: otherEid, distance });

      // Early exit if we have enough
      if (results.length >= maxCount * 2) {
        break;
      }
    }

    // Sort by distance and limit to maxCount
    results.sort((a, b) => a.distance - b.distance);
    return results.slice(0, maxCount).map(r => r.eid);
  }

  // Check if two entities can interact (within interaction range)
  canInteract(entity1, entity2, interactionRange = 10) {
    const dx = Position.x[entity1] - Position.x[entity2];
    const dy = Position.y[entity1] - Position.y[entity2];
    const distance = Math.sqrt(dx * dx + dy * dy);

    return distance <= interactionRange;
  }

  // Get the terrain type at an entity's position
  getTerrainAt(entityId) {
    const x = Math.floor(Position.x[entityId]);
    const y = Math.floor(Position.y[entityId]);
    return this.terrainGrid.get(x, y);
  }

  // Check if entity is on walkable terrain
  isOnWalkableTerrain(entityId) {
    const x = Math.floor(Position.x[entityId]);
    const y = Math.floor(Position.y[entityId]);
    return this.terrainGrid.isWalkable(x, y);
  }
}

// Simple spatial hash for fast entity queries
class SpatialHash {
  constructor(cellSize = 100) {
    this.cellSize = cellSize;
    this.cells = new Map(); // Key: "x,y", Value: Set of entity IDs
  }

  // Get cell coordinates for a world position
  getCellKey(x, y) {
    const cellX = Math.floor(x / this.cellSize);
    const cellY = Math.floor(y / this.cellSize);
    return `${cellX},${cellY}`;
  }

  // Clear all cells
  clear() {
    this.cells.clear();
  }

  // Add entity to spatial hash
  add(entityId, x, y) {
    const key = this.getCellKey(x, y);

    if (!this.cells.has(key)) {
      this.cells.set(key, new Set());
    }

    this.cells.get(key).add(entityId);
  }

  // Remove entity from spatial hash
  remove(entityId, x, y) {
    const key = this.getCellKey(x, y);

    if (this.cells.has(key)) {
      this.cells.get(key).delete(entityId);

      // Clean up empty cells
      if (this.cells.get(key).size === 0) {
        this.cells.delete(key);
      }
    }
  }

  // Query entities within radius
  queryRadius(x, y, radius) {
    const results = new Set();

    // Determine which cells to check
    const cellRadius = Math.ceil(radius / this.cellSize);
    const centerCellX = Math.floor(x / this.cellSize);
    const centerCellY = Math.floor(y / this.cellSize);

    // Check all cells in range
    for (let dy = -cellRadius; dy <= cellRadius; dy++) {
      for (let dx = -cellRadius; dx <= cellRadius; dx++) {
        const key = `${centerCellX + dx},${centerCellY + dy}`;

        if (this.cells.has(key)) {
          // Add all entities from this cell
          for (const eid of this.cells.get(key)) {
            results.add(eid);
          }
        }
      }
    }

    return Array.from(results);
  }

  // Update the spatial hash with current entity positions
  update(ecsWorld) {
    this.clear();

    const entities = allEntitiesQuery(ecsWorld);
    for (const eid of entities) {
      const x = Position.x[eid];
      const y = Position.y[eid];
      this.add(eid, x, y);
    }
  }
}
