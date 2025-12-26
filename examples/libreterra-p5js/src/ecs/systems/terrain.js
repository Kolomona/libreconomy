// Terrain management system

class TerrainSystem {
  constructor(terrainGrid) {
    this.terrainGrid = terrainGrid;
  }

  // Update terrain state (grass depletion, regeneration, etc.)
  update() {
    // Phase 6: Implement grass depletion when rabbits eat
    // For now, terrain is static
  }

  // Find nearest terrain of a specific type
  findNearest(x, y, terrainType, maxRadius = 500) {
    let nearestDist = Infinity;
    let nearest = null;

    // Search in a spiral pattern
    for (let radius = 10; radius < maxRadius; radius += 10) {
      for (let angle = 0; angle < Math.PI * 2; angle += 0.1) {
        const checkX = Math.floor(x + Math.cos(angle) * radius);
        const checkY = Math.floor(y + Math.sin(angle) * radius);

        if (this.terrainGrid.get(checkX, checkY) === terrainType) {
          const dist = Math.sqrt((checkX - x) ** 2 + (checkY - y) ** 2);
          if (dist < nearestDist) {
            nearestDist = dist;
            nearest = { x: checkX, y: checkY };
          }
        }
      }

      // If we found something, return it
      if (nearest) return nearest;
    }

    return nearest;
  }
}
