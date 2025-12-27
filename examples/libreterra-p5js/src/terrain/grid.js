// Terrain grid data structure for 10,000 x 10,000 world

class TerrainGrid {
  constructor(width, height) {
    this.width = width;
    this.height = height;

    // Use Uint8Array for memory efficiency (4 terrain types fit in 1 byte)
    // 10,000 x 10,000 = 100,000,000 bytes = ~95 MB
    this.data = new Uint8Array(width * height);

    // Callback for terrain changes (used by RenderSystem to update terrain image)
    this.onTerrainChange = null;

    console.log(`Terrain grid initialized: ${width}x${height} (${(this.data.length / 1024 / 1024).toFixed(1)} MB)`);
  }

  // Get terrain type at coordinates
  get(x, y) {
    if (x < 0 || x >= this.width || y < 0 || y >= this.height) {
      return TerrainType.WATER; // Out of bounds is water
    }

    const index = y * this.width + x;
    return this.data[index];
  }

  // Set terrain type at coordinates
  set(x, y, terrainType) {
    if (x < 0 || x >= this.width || y < 0 || y >= this.height) {
      return; // Ignore out of bounds
    }

    const index = y * this.width + x;
    this.data[index] = terrainType;
  }

  // Get terrain color for rendering
  getColor(terrainType) {
    switch (terrainType) {
      case TerrainType.WATER:
        return CONFIG.TERRAIN.WATER;
      case TerrainType.GRASS:
        return CONFIG.TERRAIN.GRASS;
      case TerrainType.ROCKY:
        return CONFIG.TERRAIN.ROCKY;
      case TerrainType.DIRT:
        return CONFIG.TERRAIN.DIRT;
      default:
        return { r: 0, g: 0, b: 0 };
    }
  }

  // Check if terrain is walkable
  isWalkable(x, y) {
    const terrain = this.get(x, y);
    return terrain !== TerrainType.ROCKY; // Everything except rocky is walkable
  }

  // Check if terrain provides water
  hasWater(x, y) {
    return this.get(x, y) === TerrainType.WATER;
  }

  // Check if terrain has grass (food for rabbits)
  hasGrass(x, y) {
    return this.get(x, y) === TerrainType.GRASS;
  }

  // Convert grass to dirt (when eaten by rabbits)
  depleteGrass(x, y) {
    if (this.get(x, y) === TerrainType.GRASS) {
      this.set(x, y, TerrainType.DIRT);

      // Notify render system of terrain change
      if (this.onTerrainChange) {
        this.onTerrainChange(x, y, TerrainType.DIRT);
      }

      return true;
    }
    return false;
  }

  // Get terrain statistics (for debugging)
  getStats() {
    const stats = {
      water: 0,
      grass: 0,
      rocky: 0,
      dirt: 0,
    };

    for (let i = 0; i < this.data.length; i++) {
      switch (this.data[i]) {
        case TerrainType.WATER:
          stats.water++;
          break;
        case TerrainType.GRASS:
          stats.grass++;
          break;
        case TerrainType.ROCKY:
          stats.rocky++;
          break;
        case TerrainType.DIRT:
          stats.dirt++;
          break;
      }
    }

    // Convert to percentages
    const total = this.data.length;
    return {
      water: ((stats.water / total) * 100).toFixed(1) + '%',
      grass: ((stats.grass / total) * 100).toFixed(1) + '%',
      rocky: ((stats.rocky / total) * 100).toFixed(1) + '%',
      dirt: ((stats.dirt / total) * 100).toFixed(1) + '%',
    };
  }
}
