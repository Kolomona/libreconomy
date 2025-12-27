// bitECS component definitions
console.log('✓ components.js loading...');

if (typeof bitecs === 'undefined') {
  console.error('FATAL: bitecs is not defined! Check script loading order.');
  throw new Error('bitecs is not defined');
}

const { defineComponent, Types, defineQuery } = bitecs;
console.log('✓ bitECS destructured successfully');

// Position component (x, y in world coordinates)
const Position = defineComponent({
  x: Types.f32,
  y: Types.f32
});

// Velocity component (for movement)
const Velocity = defineComponent({
  vx: Types.f32,
  vy: Types.f32
});

// Needs component (hunger, thirst, tiredness)
const Needs = defineComponent({
  hunger: Types.f32,
  thirst: Types.f32,
  tiredness: Types.f32
});

// Species component (0=human, 1=rabbit)
const SpeciesComponent = defineComponent({
  type: Types.ui8
});

// Gender component (0=female, 1=male)
const Gender = defineComponent({
  isMale: Types.ui8
});

// Energy component (current and max energy)
const Energy = defineComponent({
  current: Types.f32,
  max: Types.f32
});

// Age component (birth time and lifespan)
const Age = defineComponent({
  birthFrame: Types.ui32,           // Frame when entity was created
  expectedLifespanFrames: Types.ui32, // Expected total lifespan in frames
  energyHistory: Types.f32          // Rolling average of recent energy (for health tracking)
});

// Target component (destination for movement)
const Target = defineComponent({
  x: Types.f32,
  y: Types.f32,
  hasTarget: Types.ui8  // 0=no target, 1=has target
});

// State component (idle, moving, eating, drinking, sleeping)
const State = defineComponent({
  current: Types.ui8  // 0=idle, 1=moving, 2=eating, 3=drinking, 4=sleeping
});

// Define queries for efficient iteration
const humanQuery = defineQuery([Position, SpeciesComponent, Gender, Needs]);
const rabbitQuery = defineQuery([Position, SpeciesComponent, Gender, Needs]);
const allEntitiesQuery = defineQuery([Position]);
const movingEntitiesQuery = defineQuery([Position, Target, Velocity]);

// Helper to check if entity is a specific species
function isHuman(eid) {
  return SpeciesComponent.type[eid] === Species.HUMAN;
}

function isRabbit(eid) {
  return SpeciesComponent.type[eid] === Species.RABBIT;
}
