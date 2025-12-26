// Configuration and constants for libreterra

const CONFIG = {
  // World dimensions
  WORLD_WIDTH: 10000,
  WORLD_HEIGHT: 10000,

  // Camera settings
  CAMERA: {
    INITIAL_ZOOM: 1.0,
    MIN_ZOOM: 0.1,
    MAX_ZOOM: 5.0,
    ZOOM_SPEED: 0.1,
    PAN_SPEED: 1.0,
  },

  // Terrain colors
  TERRAIN: {
    WATER: { r: 50, g: 100, b: 200 },      // Blue
    GRASS: { r: 50, g: 150, b: 50 },       // Green
    ROCKY: { r: 128, g: 128, b: 128 },     // Gray
    DIRT: { r: 139, g: 90, b: 43 },        // Brown
  },

  // Entity colors
  ENTITY_COLORS: {
    MALE_HUMAN: { r: 100, g: 100, b: 255 },      // Blue
    FEMALE_HUMAN: { r: 255, g: 182, b: 193 },    // Pink
    MALE_RABBIT: { r: 173, g: 216, b: 230 },     // Light blue
    FEMALE_RABBIT: { r: 199, g: 21, b: 133 },    // Dark pink
  },

  // Entity sizes
  ENTITY_SIZES: {
    HUMAN_RADIUS: 8,
    RABBIT_RADIUS: 6,
  },

  // Simulation settings
  SIMULATION: {
    INITIAL_HUMANS: 100,   // Increased from 10 (10x more)
    INITIAL_RABBITS: 300,  // Increased from 30 (10x more)
    TARGET_FPS: 60,
    TIME_SCALE: 1.0,  // Can be modified for speed control
    PAUSED: false,
  },

  // Needs settings
  NEEDS: {
    MAX: 100.0,
    MIN: 0.0,
    DECAY_RATE: {
      HUNGER: 0.05,      // Per tick
      THIRST: 0.08,      // Per tick (thirst increases faster)
      TIREDNESS: 0.03,   // Per tick
    },
    CRITICAL_THRESHOLD: 80.0,  // When needs become urgent
  },

  // Movement settings
  MOVEMENT: {
    HUMAN_SPEED: 2.0,
    RABBIT_SPEED: 3.0,
    WANDER_DISTANCE: 50,
  },

  // Consumption settings
  CONSUMPTION: {
    WATER_SATISFACTION: -30.0,      // Reduces thirst
    FOOD_SATISFACTION: -40.0,       // Reduces hunger
    SLEEP_SATISFACTION: -50.0,      // Reduces tiredness
    CONSUMPTION_RADIUS: 10,         // How close to be to consume
  },

  // Spatial grid settings (for fast lookups)
  SPATIAL_GRID: {
    CELL_SIZE: 100,  // 100x100 pixel cells
  },

  // Terrain generation
  TERRAIN_GEN: {
    NOISE_SCALE: 0.01,
    WATER_THRESHOLD: 0.3,
    GRASS_THRESHOLD: 0.6,
    ROCKY_THRESHOLD: 0.8,
  },
};

// Terrain type enum
const TerrainType = {
  WATER: 0,
  GRASS: 1,
  ROCKY: 2,
  DIRT: 3,
};

// Species enum
const Species = {
  HUMAN: 0,
  RABBIT: 1,
};

// State enum
const EntityState = {
  IDLE: 0,
  MOVING: 1,
  EATING: 2,
  DRINKING: 3,
  SLEEPING: 4,
};

// Intent types (from libreconomy stub)
const IntentType = {
  SEEK_WATER: 'SeekWater',
  SEEK_FOOD: 'SeekFood',
  REST: 'Rest',
  WANDER: 'Wander',
};
