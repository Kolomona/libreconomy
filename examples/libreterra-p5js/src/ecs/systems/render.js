// Rendering system for terrain and entities

class RenderSystem {
  constructor(terrainGrid) {
    this.terrainGrid = terrainGrid;
    this.pixelSize = 1; // Size of each terrain pixel when rendered
    this.renderTileSize = 10; // Render terrain in 10x10 tiles for performance
  }

  // Render visible terrain (with viewport culling)
  renderTerrain(camera) {
    const bounds = camera.getVisibleBounds();

    // Expand bounds slightly to avoid edge artifacts
    const minX = Math.max(0, Math.floor(bounds.minX) - 10);
    const maxX = Math.min(this.terrainGrid.width, Math.ceil(bounds.maxX) + 10);
    const minY = Math.max(0, Math.floor(bounds.minY) - 10);
    const maxY = Math.min(this.terrainGrid.height, Math.ceil(bounds.maxY) + 10);

    // Render terrain as rectangles with no stroke to avoid grid lines
    noStroke();

    // Dynamic tile size based on zoom to prevent moiré patterns
    // When zoomed out, use larger tiles; when zoomed in, use smaller tiles
    let tileSize;
    if (camera.zoom < 0.3) {
      tileSize = 100;  // Very zoomed out
    } else if (camera.zoom < 0.5) {
      tileSize = 50;   // Zoomed out
    } else if (camera.zoom < 1.0) {
      tileSize = 20;   // Medium zoom
    } else {
      tileSize = 10;   // Zoomed in or normal
    }

    // Render in tiles for better performance
    for (let y = minY; y < maxY; y += tileSize) {
      for (let x = minX; x < maxX; x += tileSize) {
        // Sample terrain at tile center
        const terrain = this.terrainGrid.get(x, y);
        const color = this.terrainGrid.getColor(terrain);

        fill(color.r, color.g, color.b);
        rect(x, y, tileSize + 1, tileSize + 1);  // +1 overlap to prevent gaps
      }
    }
  }

  // Render all entities
  renderEntities(ecsWorld, camera, selectedEntity = null) {
    const entities = allEntitiesQuery(ecsWorld);

    for (const eid of entities) {
      const x = Position.x[eid];
      const y = Position.y[eid];

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
