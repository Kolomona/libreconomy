// Resource Cache - Spatial grid for O(1) resource lookups
// Replaces O(radius²) spiral search with pre-computed spatial index

class ResourceCache {
  constructor(cellSize = 100) {
    this.cellSize = cellSize;  // 100x100 pixel cells
    this.gridWidth = 0;        // Number of cells wide
    this.gridHeight = 0;       // Number of cells tall

    // Storage: Map<cellKey, {water: Array<{x,y}>, grass: Array<{x,y}>}>
    this.cells = new Map();

    // Statistics
    this.totalWaterTiles = 0;
    this.totalGrassTiles = 0;
  }

  // Convert world coordinates to cell coordinates
  getCellKey(x, y) {
    const cellX = Math.floor(x / this.cellSize);
    const cellY = Math.floor(y / this.cellSize);
    return `${cellX},${cellY}`;
  }

  // Parse cell key back to coordinates
  parseCellKey(key) {
    const [x, y] = key.split(',').map(Number);
    return { cellX: x, cellY: y };
  }

  // Build resource cache from terrain grid
  buildFromTerrain(terrainGrid) {
    const startTime = performance.now();
    console.log('Building resource cache...');

    this.gridWidth = Math.ceil(terrainGrid.width / this.cellSize);
    this.gridHeight = Math.ceil(terrainGrid.height / this.cellSize);
    this.cells.clear();
    this.totalWaterTiles = 0;
    this.totalGrassTiles = 0;

    // OPTIMIZATION: Sample every 20th pixel instead of every pixel
    // For 10,000×10,000: 250K checks instead of 100M (400x faster build)
    // Reduces tile density from 5.8M to ~230K (25x reduction)
    const SAMPLE_STEP = 20;

    for (let y = 0; y < terrainGrid.height; y += SAMPLE_STEP) {
      for (let x = 0; x < terrainGrid.width; x += SAMPLE_STEP) {
        const terrain = terrainGrid.get(x, y);

        if (terrain === TerrainType.WATER || terrain === TerrainType.GRASS) {
          const cellKey = this.getCellKey(x, y);

          if (!this.cells.has(cellKey)) {
            this.cells.set(cellKey, { water: [], grass: [] });
          }

          const cell = this.cells.get(cellKey);

          if (terrain === TerrainType.WATER) {
            cell.water.push({ x, y });
            this.totalWaterTiles++;
          } else {
            cell.grass.push({ x, y });
            this.totalGrassTiles++;
          }
        }
      }

      // Progress logging every 1000 rows
      if (y % 1000 === 0 && y > 0) {
        const progress = ((y / terrainGrid.height) * 100).toFixed(0);
        console.log(`  Resource cache: ${progress}%`);
      }
    }

    const duration = ((performance.now() - startTime) / 1000).toFixed(2);
    console.log(`✓ Resource cache built in ${duration}s`);
    console.log(`  Water tiles: ${this.totalWaterTiles.toLocaleString()}`);
    console.log(`  Grass tiles: ${this.totalGrassTiles.toLocaleString()}`);
    console.log(`  Grid cells: ${this.cells.size} (${this.gridWidth}×${this.gridHeight})`);
  }

  // Find nearest resources of a specific type
  // Returns array of {x, y, distance} sorted by distance
  findNearest(worldX, worldY, resourceType, maxResults = 5, maxRadius = 1000) {
    const targetTiles = resourceType === 'water' ? 'water' : 'grass';
    const results = [];

    // Start with cell containing the entity
    const startCellX = Math.floor(worldX / this.cellSize);
    const startCellY = Math.floor(worldY / this.cellSize);

    // Expand in rings until we find enough resources or hit max radius
    // Limit to reasonable number of rings to prevent excessive checking
    const maxRingRadius = Math.min(10, Math.ceil(maxRadius / this.cellSize));

    for (let ring = 0; ring <= maxRingRadius; ring++) {
      // Check all cells in this ring
      for (let dy = -ring; dy <= ring; dy++) {
        for (let dx = -ring; dx <= ring; dx++) {
          // Only check perimeter cells (skip already-checked interior)
          if (ring > 0 && Math.abs(dx) < ring && Math.abs(dy) < ring) {
            continue;
          }

          const cellX = startCellX + dx;
          const cellY = startCellY + dy;
          const cellKey = `${cellX},${cellY}`;

          if (!this.cells.has(cellKey)) continue;

          const cell = this.cells.get(cellKey);
          const tiles = cell[targetTiles];

          // Check all resource tiles in this cell
          for (const tile of tiles) {
            const dist = Math.sqrt((tile.x - worldX)**2 + (tile.y - worldY)**2);

            if (dist <= maxRadius) {
              results.push({ x: tile.x, y: tile.y, distance: dist });
            }
          }
        }
      }

      // Early exit if we have enough results
      if (results.length >= maxResults) {
        break;
      }

      // If we found any resources in this ring, stop searching (don't check farther rings)
      if (results.length > 0) {
        break;
      }
    }

    // Sort by distance and return top N
    results.sort((a, b) => a.distance - b.distance);
    return results.slice(0, maxResults);
  }

  // Update cache when grass depletes (incremental update)
  removeGrassTile(x, y) {
    const cellKey = this.getCellKey(x, y);
    if (!this.cells.has(cellKey)) return;

    const cell = this.cells.get(cellKey);
    const index = cell.grass.findIndex(tile => tile.x === x && tile.y === y);

    if (index !== -1) {
      cell.grass.splice(index, 1);
      this.totalGrassTiles--;
    }
  }

  // Add grass tile (if grass regrows in future)
  addGrassTile(x, y) {
    const cellKey = this.getCellKey(x, y);
    if (!this.cells.has(cellKey)) {
      this.cells.set(cellKey, { water: [], grass: [] });
    }

    const cell = this.cells.get(cellKey);
    cell.grass.push({ x, y });
    this.totalGrassTiles++;
  }
}
