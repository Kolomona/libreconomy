// Rendering system for terrain and entities

class RenderSystem {
  constructor(terrainGrid) {
    this.terrainGrid = terrainGrid;
    this.pixelSize = 1; // Size of each terrain pixel when rendered
    this.renderTileSize = 10; // Render terrain in 10x10 tiles for performance

    // Chunk system for industry-standard terrain rendering
    this.CHUNK_SIZE = 512;  // 512×512 pixels per chunk
    this.chunksX = Math.ceil(terrainGrid.width / this.CHUNK_SIZE);
    this.chunksY = Math.ceil(terrainGrid.height / this.CHUNK_SIZE);

    // Cache for pre-rendered chunks
    this.chunkCache = new Map();  // key: "x,y" -> p5.Graphics buffer

    // Initialization state
    this.chunksInitialized = false;
    this.initializingChunks = false;
  }

  // Pre-render a single chunk to offscreen buffer
  renderChunk(chunkX, chunkY) {
    // Create offscreen graphics buffer for this chunk
    const chunkBuffer = createGraphics(this.CHUNK_SIZE, this.CHUNK_SIZE);

    // Calculate world pixel coordinates for this chunk
    const worldX = chunkX * this.CHUNK_SIZE;
    const worldY = chunkY * this.CHUNK_SIZE;

    // Render terrain pixels to chunk buffer
    chunkBuffer.loadPixels();

    for (let y = 0; y < this.CHUNK_SIZE; y++) {
      for (let x = 0; x < this.CHUNK_SIZE; x++) {
        const terrainX = worldX + x;
        const terrainY = worldY + y;

        // Skip if outside terrain bounds
        if (terrainX >= this.terrainGrid.width || terrainY >= this.terrainGrid.height) {
          continue;
        }

        const terrainType = this.terrainGrid.get(terrainX, terrainY);
        const color = this.terrainGrid.getColor(terrainType);

        const pixelIndex = (y * this.CHUNK_SIZE + x) * 4;
        chunkBuffer.pixels[pixelIndex + 0] = color.r;
        chunkBuffer.pixels[pixelIndex + 1] = color.g;
        chunkBuffer.pixels[pixelIndex + 2] = color.b;
        chunkBuffer.pixels[pixelIndex + 3] = 255;
      }
    }

    chunkBuffer.updatePixels();

    // Store in cache
    const chunkKey = `${chunkX},${chunkY}`;
    this.chunkCache.set(chunkKey, chunkBuffer);
  }

  // Initialize terrain chunks (call once in setup)
  initializeTerrainImage() {
    console.log('Initializing chunk-based terrain rendering...');
    console.log(`Creating ${this.chunksX}×${this.chunksY} chunks (${this.chunksX * this.chunksY} total)`);

    const startTime = performance.now();
    const totalChunks = this.chunksX * this.chunksY;

    // Pre-render all chunks SYNCHRONOUSLY (will freeze UI, but ensures completion)
    for (let chunkY = 0; chunkY < this.chunksY; chunkY++) {
      for (let chunkX = 0; chunkX < this.chunksX; chunkX++) {
        this.renderChunk(chunkX, chunkY);
      }
    }

    this.chunksInitialized = true;
    const elapsed = performance.now() - startTime;
    console.log(`✓ All chunks rendered in ${elapsed.toFixed(0)}ms`);
  }

  // Render terrain using chunk-based rendering
  renderTerrain(camera) {
    if (!this.chunksInitialized) return;

    // Calculate visible chunk range
    const bounds = camera.getVisibleBounds();

    const minChunkX = Math.max(0, Math.floor(bounds.minX / this.CHUNK_SIZE));
    const maxChunkX = Math.min(this.chunksX - 1, Math.floor(bounds.maxX / this.CHUNK_SIZE));
    const minChunkY = Math.max(0, Math.floor(bounds.minY / this.CHUNK_SIZE));
    const maxChunkY = Math.min(this.chunksY - 1, Math.floor(bounds.maxY / this.CHUNK_SIZE));

    const visibleChunks = (maxChunkY - minChunkY + 1) * (maxChunkX - minChunkX + 1);

    // Debug FPS issue - log once per second
    if (!this._lastDebugTime || Date.now() - this._lastDebugTime > 1000) {
      console.log(`📊 Rendering ${visibleChunks} chunks at zoom ${camera.zoom.toFixed(2)}x | FPS: ${Math.round(frameRate())}`);
      this._lastDebugTime = Date.now();
    }

    // Render visible chunks
    smooth();  // Enable smooth scaling
    noStroke();

    for (let cy = minChunkY; cy <= maxChunkY; cy++) {
      for (let cx = minChunkX; cx <= maxChunkX; cx++) {
        const chunkKey = `${cx},${cy}`;
        const chunkBuffer = this.chunkCache.get(chunkKey);

        if (chunkBuffer) {
          const worldX = cx * this.CHUNK_SIZE;
          const worldY = cy * this.CHUNK_SIZE;

          // Draw pre-rendered chunk
          image(chunkBuffer, worldX, worldY);
        }
      }
    }
  }

  // Update single pixel (for dynamic terrain changes like grass depletion)
  updateTerrainPixel(x, y, newTerrainType) {
    if (!this.chunksInitialized) return;

    // Calculate which chunk contains this pixel
    const chunkX = Math.floor(x / this.CHUNK_SIZE);
    const chunkY = Math.floor(y / this.CHUNK_SIZE);
    const chunkKey = `${chunkX},${chunkY}`;

    const chunkBuffer = this.chunkCache.get(chunkKey);
    if (!chunkBuffer) return;

    // Calculate pixel position within chunk
    const localX = x % this.CHUNK_SIZE;
    const localY = y % this.CHUNK_SIZE;

    // Use set() instead of loadPixels/updatePixels for single pixel updates
    // This is MUCH faster as it doesn't copy the entire 512×512 buffer
    const color = this.terrainGrid.getColor(newTerrainType);
    chunkBuffer.set(localX, localY, [color.r, color.g, color.b, 255]);
    chunkBuffer.updatePixels();  // Still needed to flush the change
  }

  // Render all entities
  renderEntities(ecsWorld, camera, selectedEntity = null) {
    const entities = allEntitiesQuery(ecsWorld);

    for (const eid of entities) {
      // Round entity coordinates to prevent sub-pixel shimmering
      const x = Math.round(Position.x[eid]);
      const y = Math.round(Position.y[eid]);

      // Skip if outside viewport
      if (!this.isInViewport(x, y, camera)) continue;

      const species = SpeciesComponent.type[eid];
      const isMale = Gender.isMale[eid];

      // Draw based on species and gender
      if (species === Species.HUMAN) {
        this.drawHuman(x, y, isMale === 1);
      } else if (species === Species.RABBIT) {
        this.drawRabbit(x, y, isMale === 1);
      }

      // Draw selection indicator
      if (eid === selectedEntity) {
        this.drawSelectionIndicator(x, y, species, camera);
      }
    }
  }

  // Draw selection indicator around entity
  drawSelectionIndicator(x, y, species, camera) {
    const radius = species === Species.HUMAN ?
      CONFIG.ENTITY_SIZES.HUMAN_RADIUS + 3 :
      CONFIG.ENTITY_SIZES.RABBIT_RADIUS + 3;

    stroke(255, 255, 0);
    strokeWeight(2 / camera.zoom);
    noFill();
    circle(x, y, radius * 2 + 4);

    // Pulse effect
    const pulseSize = radius * 2 + 4 + Math.sin(frameCount * 0.1) * 2;
    stroke(255, 255, 0, 100);
    strokeWeight(1 / camera.zoom);
    circle(x, y, pulseSize);
  }

  // Check if point is in viewport
  isInViewport(x, y, camera) {
    const bounds = camera.getVisibleBounds();
    return x >= bounds.minX && x <= bounds.maxX &&
           y >= bounds.minY && y <= bounds.maxY;
  }

  // Draw a human entity
  drawHuman(x, y, isMale) {
    const color = isMale ? CONFIG.ENTITY_COLORS.MALE_HUMAN : CONFIG.ENTITY_COLORS.FEMALE_HUMAN;
    fill(color.r, color.g, color.b);
    stroke(255);
    strokeWeight(0.5);
    circle(x, y, CONFIG.ENTITY_SIZES.HUMAN_RADIUS * 2);
  }

  // Draw a rabbit entity
  drawRabbit(x, y, isMale) {
    const color = isMale ? CONFIG.ENTITY_COLORS.MALE_RABBIT : CONFIG.ENTITY_COLORS.FEMALE_RABBIT;
    fill(color.r, color.g, color.b);
    stroke(255);
    strokeWeight(0.5);

    // Draw triangle
    const size = CONFIG.ENTITY_SIZES.RABBIT_RADIUS;
    triangle(
      x, y - size,           // Top
      x - size, y + size,    // Bottom left
      x + size, y + size     // Bottom right
    );
  }

  // Draw debug info for an entity
  drawEntityDebug(eid, x, y, ecsWorld) {
    // TODO Phase 3: Show entity stats when selected
    // This could show needs, state, target, etc.
  }

  // Render UI elements (entity selection, etc.)
  renderUI(selectedEntity, ecsWorld) {
    // TODO Phase 7: Implement UI rendering
    // This will show selected entity details
  }
}
