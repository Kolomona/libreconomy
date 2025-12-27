// Rendering system for terrain and entities

class RenderSystem {
  constructor(terrainGrid) {
    this.terrainGrid = terrainGrid;
    this.pixelSize = 1; // Size of each terrain pixel when rendered
    this.renderTileSize = 10; // Render terrain in 10x10 tiles for performance
    this.terrainImage = null;  // p5.Image for terrain
    this.imageReady = false;   // Track initialization
  }

  // Initialize terrain image (call once in setup)
  initializeTerrainImage() {
    console.log('Creating terrain p5.Image...');
    const startTime = performance.now();

    // Create p5.Image (RGBA format)
    this.terrainImage = createImage(this.terrainGrid.width, this.terrainGrid.height);
    this.terrainImage.loadPixels();

    // Convert Uint8Array → RGBA pixels
    const pixels = this.terrainImage.pixels;
    const terrainData = this.terrainGrid.data;

    for (let i = 0; i < terrainData.length; i++) {
      const terrainType = terrainData[i];
      const color = this.terrainGrid.getColor(terrainType);

      const pixelIndex = i * 4;
      pixels[pixelIndex + 0] = color.r;
      pixels[pixelIndex + 1] = color.g;
      pixels[pixelIndex + 2] = color.b;
      pixels[pixelIndex + 3] = 255;
    }

    this.terrainImage.updatePixels();
    this.imageReady = true;

    const elapsed = performance.now() - startTime;
    console.log(`✓ Terrain image created in ${elapsed.toFixed(0)}ms`);
  }

  // Render terrain using p5.Image
  renderTerrain(camera) {
    if (!this.imageReady) return;

    smooth();    // Enable p5.js bilinear filtering for smooth scaling
    noStroke();
    image(this.terrainImage, 0, 0);  // Single draw call for entire terrain
  }

  // Update single pixel (for dynamic terrain changes like grass depletion)
  updateTerrainPixel(x, y, newTerrainType) {
    if (!this.imageReady) return;

    this.terrainImage.loadPixels();
    const color = this.terrainGrid.getColor(newTerrainType);
    const pixelIndex = (y * this.terrainGrid.width + x) * 4;

    this.terrainImage.pixels[pixelIndex + 0] = color.r;
    this.terrainImage.pixels[pixelIndex + 1] = color.g;
    this.terrainImage.pixels[pixelIndex + 2] = color.b;
    this.terrainImage.pixels[pixelIndex + 3] = 255;

    this.terrainImage.updatePixels();
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
