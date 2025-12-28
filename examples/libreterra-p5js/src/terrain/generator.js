// Terrain generator using Perlin noise

class TerrainGenerator {
  constructor(seed = null) {
    this.seed = seed || Math.floor(Math.random() * 10000);
    console.log(`Terrain generator initialized with seed: ${this.seed}`);
  }

  // Multi-octave noise (Fractal Brownian Motion) to eliminate repetition
  multiOctaveNoise(x, y, octaves = 4, persistence = 0.5, lacunarity = 2.0) {
    // Debug: Log first call
    if (!this._debugLogged) {
      console.log(`✨ multiOctaveNoise called! octaves=${octaves}, persistence=${persistence}, lacunarity=${lacunarity}`);
      this._debugLogged = true;
    }

    let value = 0;
    let amplitude = 1;
    let frequency = 1;
    let maxValue = 0;

    // Offsets in NOISE SPACE (applied after frequency multiplication)
    // Using large prime-like numbers to break repetition alignment
    const octaveOffsets = [
      { x: 0, y: 0 },
      { x: 1234.56789, y: 2345.67891 },
      { x: 3456.78912, y: 4567.89123 },
      { x: 5678.91234, y: 6789.12345 }
    ];

    for (let i = 0; i < octaves; i++) {
      // Apply frequency FIRST, then add offset in noise space
      const offset = octaveOffsets[i % octaveOffsets.length];
      const sampleX = x * frequency + offset.x;
      const sampleY = y * frequency + offset.y;

      value += noise(sampleX, sampleY) * amplitude;
      maxValue += amplitude;
      amplitude *= persistence;  // Each octave has reduced amplitude
      frequency *= lacunarity;   // Each octave has increased frequency
    }

    return value / maxValue;  // Normalize to 0-1 range
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

    // Generate terrain using multi-octave Perlin noise
    const octaves = CONFIG.TERRAIN_GEN.OCTAVES;
    const persistence = CONFIG.TERRAIN_GEN.PERSISTENCE;
    const lacunarity = CONFIG.TERRAIN_GEN.LACUNARITY;

    console.log(`🎲 Using multi-octave noise: scale=${scale}, octaves=${octaves}, persistence=${persistence}, lacunarity=${lacunarity}`);

    let debugLogged = false;

    for (let y = 0; y < terrainGrid.height; y++) {
      for (let x = 0; x < terrainGrid.width; x++) {
        // Sample multi-octave Perlin noise
        const noiseValue = this.multiOctaveNoise(x * scale, y * scale, octaves, persistence, lacunarity);

        // Debug log first pixel
        if (!debugLogged && x === 0 && y === 0) {
          console.log(`🔍 First pixel: multiOctaveNoise(0, 0) = ${noiseValue}`);
          debugLogged = true;
        }

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
    const octaves = CONFIG.TERRAIN_GEN.OCTAVES;
    const persistence = CONFIG.TERRAIN_GEN.PERSISTENCE;
    const lacunarity = CONFIG.TERRAIN_GEN.LACUNARITY;

    const chunks = Math.ceil(terrainGrid.height / chunkSize);

    for (let chunkY = 0; chunkY < chunks; chunkY++) {
      const startY = chunkY * chunkSize;
      const endY = Math.min(startY + chunkSize, terrainGrid.height);

      for (let y = startY; y < endY; y++) {
        for (let x = 0; x < terrainGrid.width; x++) {
          const noiseValue = this.multiOctaveNoise(x * scale, y * scale, octaves, persistence, lacunarity);

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
    const octaves = CONFIG.TERRAIN_GEN.OCTAVES;
    const persistence = CONFIG.TERRAIN_GEN.PERSISTENCE;
    const lacunarity = CONFIG.TERRAIN_GEN.LACUNARITY;

    for (let y = minY; y < maxY; y++) {
      for (let x = minX; x < maxX; x++) {
        const noiseValue = this.multiOctaveNoise(x * scale, y * scale, octaves, persistence, lacunarity);

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
