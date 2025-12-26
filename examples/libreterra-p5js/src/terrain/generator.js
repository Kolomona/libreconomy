// Terrain generator using Perlin noise

class TerrainGenerator {
  constructor(seed = null) {
    this.seed = seed || Math.floor(Math.random() * 10000);
    console.log(`Terrain generator initialized with seed: ${this.seed}`);
  }

  // Generate terrain for the entire grid
  generate(terrainGrid) {
    console.log('Generating terrain...');
    const startTime = performance.now();

    // Set random seed for reproducible noise
    noiseSeed(this.seed);

    const scale = CONFIG.TERRAIN_GEN.NOISE_SCALE;
    const waterThreshold = CONFIG.TERRAIN_GEN.WATER_THRESHOLD;
    const grassThreshold = CONFIG.TERRAIN_GEN.GRASS_THRESHOLD;
    const rockyThreshold = CONFIG.TERRAIN_GEN.ROCKY_THRESHOLD;

    // Generate terrain using Perlin noise
    for (let y = 0; y < terrainGrid.height; y++) {
      for (let x = 0; x < terrainGrid.width; x++) {
        // Sample Perlin noise
        const noiseValue = noise(x * scale, y * scale);

        // Map noise value to terrain type
        let terrainType;
        if (noiseValue < waterThreshold) {
          terrainType = TerrainType.WATER;
        } else if (noiseValue < grassThreshold) {
          terrainType = TerrainType.GRASS;
        } else if (noiseValue < rockyThreshold) {
          terrainType = TerrainType.ROCKY;
        } else {
          terrainType = TerrainType.DIRT;
        }

        terrainGrid.set(x, y, terrainType);
      }

      // Progress logging every 10%
      if (y % Math.floor(terrainGrid.height / 10) === 0) {
        const progress = ((y / terrainGrid.height) * 100).toFixed(0);
        console.log(`  Terrain generation: ${progress}%`);
      }
    }

    const endTime = performance.now();
    const duration = ((endTime - startTime) / 1000).toFixed(2);

    console.log(`Terrain generated in ${duration}s`);
    console.log('Terrain distribution:', terrainGrid.getStats());

    return terrainGrid;
  }

  // Generate terrain in chunks (for better performance/progress feedback)
  *generateChunked(terrainGrid, chunkSize = 1000) {
    console.log('Generating terrain in chunks...');

    noiseSeed(this.seed);

    const scale = CONFIG.TERRAIN_GEN.NOISE_SCALE;
    const waterThreshold = CONFIG.TERRAIN_GEN.WATER_THRESHOLD;
    const grassThreshold = CONFIG.TERRAIN_GEN.GRASS_THRESHOLD;
    const rockyThreshold = CONFIG.TERRAIN_GEN.ROCKY_THRESHOLD;

    const chunks = Math.ceil(terrainGrid.height / chunkSize);

    for (let chunkY = 0; chunkY < chunks; chunkY++) {
      const startY = chunkY * chunkSize;
      const endY = Math.min(startY + chunkSize, terrainGrid.height);

      for (let y = startY; y < endY; y++) {
        for (let x = 0; x < terrainGrid.width; x++) {
          const noiseValue = noise(x * scale, y * scale);

          let terrainType;
          if (noiseValue < waterThreshold) {
            terrainType = TerrainType.WATER;
          } else if (noiseValue < grassThreshold) {
            terrainType = TerrainType.GRASS;
          } else if (noiseValue < rockyThreshold) {
            terrainType = TerrainType.ROCKY;
          } else {
            terrainType = TerrainType.DIRT;
          }

          terrainGrid.set(x, y, terrainType);
        }
      }

      const progress = (((chunkY + 1) / chunks) * 100).toFixed(0);
      console.log(`  Chunk ${chunkY + 1}/${chunks} (${progress}%)`);

      yield { chunk: chunkY + 1, total: chunks, progress };
    }

    console.log('Terrain generation complete');
    console.log('Terrain distribution:', terrainGrid.getStats());
  }

  // Generate a specific region (for lazy loading/regeneration)
  generateRegion(terrainGrid, minX, minY, maxX, maxY) {
    noiseSeed(this.seed);

    const scale = CONFIG.TERRAIN_GEN.NOISE_SCALE;
    const waterThreshold = CONFIG.TERRAIN_GEN.WATER_THRESHOLD;
    const grassThreshold = CONFIG.TERRAIN_GEN.GRASS_THRESHOLD;
    const rockyThreshold = CONFIG.TERRAIN_GEN.ROCKY_THRESHOLD;

    for (let y = minY; y < maxY; y++) {
      for (let x = minX; x < maxX; x++) {
        const noiseValue = noise(x * scale, y * scale);

        let terrainType;
        if (noiseValue < waterThreshold) {
          terrainType = TerrainType.WATER;
        } else if (noiseValue < grassThreshold) {
          terrainType = TerrainType.GRASS;
        } else if (noiseValue < rockyThreshold) {
          terrainType = TerrainType.ROCKY;
        } else {
          terrainType = TerrainType.DIRT;
        }

        terrainGrid.set(x, y, terrainType);
      }
    }
  }
}
