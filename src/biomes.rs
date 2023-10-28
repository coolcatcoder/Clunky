use std::ops::AddAssign;

// TODO: Add detail level to each block, so the generator can tell whether or not that block belongs to that detail level

pub fn get_biome(biome_noise: (f64, f64)) -> usize {
    // To handle overlapping shapes we should instead go through each biome, if it is a correct biome, add it to a vector, then pick randomly from the vector at the end.
    for biome_index in 0..BIOMES.len() {
        if BIOMES[biome_index].aabb.point_intersects(biome_noise) {
            //println!("{}",biome_index); // good for debugging
            return biome_index;
        }
    }

    println!("{:?} matches no biomes", biome_noise);
    0
}

pub const BIOME_SCALE: (f64, f64) = (0.05, 0.05);

pub const SPRITE_SIZE: (f32, f32) = (1.0 / 44.0, 1.0);

pub const BIOMES: [Biome; 5] = [
    Biome {
        // sparse rock
        aabb: Aabb {
            size: (1.0 / 3.0 + 0.01, 0.5),
            position: (0.0, 0.5),
        },
        random_pattern: PatternArray {
            starting_index: 2,
            length: 1,
        },
        simplex_pattern: PatternArray {
            starting_index: 4,
            length: 1,
        },
        simplex_smoothed_pattern: PatternArray {
            starting_index: 0,
            length: 0,
        },
    },
    Biome {
        // mixed jungle
        aabb: Aabb {
            size: (2.0 / 3.0, 0.5),
            position: (1.0 / 3.0, 0.5),
        },
        random_pattern: PatternArray {
            starting_index: 0,
            length: 2,
        },
        simplex_pattern: PatternArray {
            starting_index: 0,
            length: 4,
        },
        simplex_smoothed_pattern: PatternArray {
            starting_index: 0,
            length: 0,
        },
    },
    Biome {
        // grasslands
        aabb: Aabb {
            size: (1.0 / 3.0 + 0.01, 0.501),
            position: (0.0, 0.0),
        },
        random_pattern: PatternArray {
            starting_index: 3,
            length: 1,
        },
        simplex_pattern: PatternArray {
            starting_index: 5,
            length: 1,
        },
        simplex_smoothed_pattern: PatternArray {
            starting_index: 0,
            length: 0,
        },
    },
    Biome {
        // desert
        aabb: Aabb {
            size: (1.0 / 3.0 + 0.01, 0.501),
            position: (1.0 / 3.0, 0.0),
        },
        random_pattern: PatternArray {
            starting_index: 4,
            length: 1,
        },
        simplex_pattern: PatternArray {
            starting_index: 6,
            length: 1,
        },
        simplex_smoothed_pattern: PatternArray {
            starting_index: 0,
            length: 0,
        },
    },
    Biome {
        // mountains
        aabb: Aabb {
            size: (1.0 / 3.0, 0.501),
            position: (2.0 / 3.0, 0.0),
        },
        random_pattern: PatternArray {
            starting_index: 5,
            length: 5,
        },
        simplex_pattern: PatternArray {
            starting_index: 7,
            length: 1,
        },
        simplex_smoothed_pattern: PatternArray {
            starting_index: 0,
            length: 0,
        },
    },
];

pub const RANDOM_PATTERN_MAP_OBJECTS: [RandomPatternMapObject; 11] = [
    // start mixed jungle
    RandomPatternMapObject {
        // health fruit medium
        detail: 0,
        chance: 9,
        priority: 1,
        behaviour: CollisionBehaviour::Consume(
            0,
            Statistics {
                strength: 0,
                health: 5,
                stamina: 2,
            },
        ),
        rendering_size: (0.75, 0.75),
        collision_size: (0.6, 0.6),
        uv: (20.0 * SPRITE_SIZE.0, 0.0),
    },
    RandomPatternMapObject {
        // health fruit large
        detail: 0,
        chance: 1,
        priority: 1,
        behaviour: CollisionBehaviour::Consume(
            0,
            Statistics {
                strength: 0,
                health: 10,
                stamina: 5,
            },
        ),
        rendering_size: (1.3, 1.3),
        collision_size: (1.0, 1.0),
        uv: (20.0 * SPRITE_SIZE.0, 0.0),
    },
    // end mixed jungle

    // start sparse rock
    RandomPatternMapObject {
        // weird stamina rock
        detail: 0,
        chance: 10,
        priority: 2,
        behaviour: CollisionBehaviour::Consume(
            0,
            Statistics {
                strength: 0,
                health: 0,
                stamina: 2,
            },
        ),
        rendering_size: (1.0, 1.0),
        collision_size: (1.0, 1.0),
        uv: (8.0 * SPRITE_SIZE.0, 0.0),
    },
    // end sparse rock

    // start grasslands
    RandomPatternMapObject {
        // swishy swishy
        detail: 0,
        chance: 100,
        priority: 1,
        behaviour: CollisionBehaviour::None,
        rendering_size: (1.0, 1.0),
        collision_size: (0.0, 0.0),
        uv: (14.0 * SPRITE_SIZE.0, 0.0),
    },
    // end grasslands

    // start desert
    RandomPatternMapObject {
        // sand
        detail: 0,
        chance: 100,
        priority: 1,
        behaviour: CollisionBehaviour::None,
        rendering_size: (1.0, 1.0),
        collision_size: (0.0, 0.0),
        uv: (11.0 * SPRITE_SIZE.0, 0.0),
    },
    // end desert

    // start mountain
    RandomPatternMapObject {
        // spiccaro mixed
        detail: 0,
        chance: 10,
        priority: 2,
        behaviour: CollisionBehaviour::Consume(
            0,
            Statistics {
                strength: 1,
                health: -10,
                stamina: -10,
            },
        ),
        rendering_size: (0.75, 0.75),
        collision_size: (0.6, 0.6),
        uv: (21.0 * SPRITE_SIZE.0, 0.0),
    },
    RandomPatternMapObject {
        // spiccaro purple
        detail: 0,
        chance: 1,
        priority: 2,
        behaviour: CollisionBehaviour::Consume(
            0,
            Statistics {
                strength: 2,
                health: -20,
                stamina: 0,
            },
        ),
        rendering_size: (0.75, 0.75),
        collision_size: (0.6, 0.6),
        uv: (22.0 * SPRITE_SIZE.0, 0.0),
    },
    RandomPatternMapObject {
        // spiccaro blue
        detail: 0,
        chance: 1,
        priority: 2,
        behaviour: CollisionBehaviour::Consume(
            0,
            Statistics {
                strength: 0,
                health: 0,
                stamina: -20,
            },
        ),
        rendering_size: (0.75, 0.75),
        collision_size: (0.6, 0.6),
        uv: (23.0 * SPRITE_SIZE.0, 0.0),
    },
    RandomPatternMapObject {
        // spiccaro orange
        detail: 0,
        chance: 1,
        priority: 2,
        behaviour: CollisionBehaviour::Consume(
            0,
            Statistics {
                strength: 0,
                health: 10,
                stamina: 10,
            },
        ),
        rendering_size: (0.75, 0.75),
        collision_size: (0.6, 0.6),
        uv: (24.0 * SPRITE_SIZE.0, 0.0),
    },
    RandomPatternMapObject {
        // dark velvet slicer
        detail: 0,
        chance: 50,
        priority: 1,
        behaviour: CollisionBehaviour::Consume(
            0,
            Statistics {
                strength: 0,
                health: -10,
                stamina: -10,
            },
        ),
        rendering_size: (1.3, 1.3),
        collision_size: (1.0, 1.0),
        uv: (19.0 * SPRITE_SIZE.0, 0.0),
    },
    // end mountain
    RandomPatternMapObject {
        // swishy swishy. This is debug, remove asap.
        detail: 1,
        chance: 100,
        priority: 1,
        behaviour: CollisionBehaviour::None,
        rendering_size: (0.5, 0.5),
        collision_size: (0.0, 0.0),
        uv: (14.0 * SPRITE_SIZE.0, 0.0),
    },
];

pub const SIMPLEX_PATTERN_MAP_OBJECTS: [SimplexPatternMapObject; 8] = [
    // start mixed jungle
    SimplexPatternMapObject {
        // circus rock
        detail: 0,
        chance: 100,
        priority: 3,
        behaviour: CollisionBehaviour::Consume(
            2,
            Statistics {
                // replace with rock eating collision behaviour
                strength: 0,
                health: 0,
                stamina: 2,
            },
        ),
        rendering_size: (1.0, 1.0),
        collision_size: (1.0, 1.0),
        seed: 1,
        acceptable_noise: (0.0, 1.0),
        noise_scale: 0.15,
        uv: (7.0 * SPRITE_SIZE.0, 0.0),
    },
    SimplexPatternMapObject {
        // the weird stamina rock
        detail: 0,
        chance: 100,
        priority: 2,
        behaviour: CollisionBehaviour::Consume(
            0,
            Statistics {
                strength: 0,
                health: 0,
                stamina: 2,
            },
        ),
        rendering_size: (1.0, 1.0),
        collision_size: (1.0, 1.0),
        seed: 1,
        acceptable_noise: (-0.2, 1.0),
        noise_scale: 0.15,
        uv: (8.0 * SPRITE_SIZE.0, 0.0),
    },
    SimplexPatternMapObject {
        // velvet slicer
        detail: 0,
        chance: 75,
        priority: 1,
        behaviour: CollisionBehaviour::Consume(
            0,
            Statistics {
                strength: 0,
                health: -3,
                stamina: -5,
            },
        ),
        rendering_size: (1.3, 1.3),
        collision_size: (1.0, 1.0),
        seed: 2,
        acceptable_noise: (0.2, 0.5),
        noise_scale: 0.1,
        uv: (16.0 * SPRITE_SIZE.0, 0.0),
    },
    SimplexPatternMapObject {
        // Large test
        detail: 0,
        chance: 100,
        priority: 4,
        behaviour: CollisionBehaviour::None,
        rendering_size: (1.0, 1.0),
        collision_size: (1.0, 1.0),
        seed: 3,
        acceptable_noise: (-1.0, -0.6),
        noise_scale: 0.05,
        uv: (9.0 * SPRITE_SIZE.0, 0.0),
    },
    // end mixed jungle

    // start sparse rock
    SimplexPatternMapObject {
        // circus rock
        detail: 0,
        chance: 100,
        priority: 1,
        behaviour: CollisionBehaviour::Consume(
            2,
            Statistics {
                // rock eater asap
                strength: 0,
                health: 0,
                stamina: 2,
            },
        ),
        rendering_size: (1.0, 1.0),
        collision_size: (1.0, 1.0),
        seed: 1,
        acceptable_noise: (0.2, 0.8),
        noise_scale: 0.15,
        uv: (7.0 * SPRITE_SIZE.0, 0.0),
    },
    // end sparse rock

    // start grasslands
    SimplexPatternMapObject {
        // flowers
        detail: 0,
        chance: 75,
        priority: 2,
        behaviour: CollisionBehaviour::Replace(
            0,
            Statistics {
                strength: 0,
                health: 1,
                stamina: 2,
            },
            MapObject::RandomPattern(3),
        ),
        rendering_size: (1.0, 1.0),
        collision_size: (1.0, 1.0),
        seed: 15,
        acceptable_noise: (0.2, 0.5),
        noise_scale: 0.1,
        uv: (15.0 * SPRITE_SIZE.0, 0.0),
    },
    // end grasslands

    // start desert
    SimplexPatternMapObject {
        // ???
        detail: 0,
        chance: 45,
        priority: 2,
        behaviour: CollisionBehaviour::Replace(
            0,
            Statistics {
                strength: 0,
                health: -5,
                stamina: -5,
            },
            MapObject::RandomPattern(4),
        ),
        rendering_size: (1.0, 1.0),
        collision_size: (1.0, 1.0),
        seed: 15,
        acceptable_noise: (0.2, 0.5),
        noise_scale: 0.17,
        uv: (12.0 * SPRITE_SIZE.0, 0.0),
    },
    // end desert

    // start mountains
    SimplexPatternMapObject {
        // circus rock
        detail: 0,
        chance: 100,
        priority: 3,
        behaviour: CollisionBehaviour::Consume(
            2,
            Statistics {
                // rock eater asap
                strength: 0,
                health: 0,
                stamina: 2,
            },
        ),
        rendering_size: (1.0, 1.0),
        collision_size: (1.0, 1.0),
        seed: 1,
        acceptable_noise: (0.0, 1.0),
        noise_scale: 0.5,
        uv: (7.0 * SPRITE_SIZE.0, 0.0),
    },
    // end mountains
];

pub const SIMPLEX_SMOOTHED_PATTERN_MAP_OBJECTS: [SimplexSmoothedPatternMapObject; 0] = [];

#[derive(Debug)]
pub struct RandomPatternMapObject {
    pub detail: u8,
    pub chance: u8,
    pub priority: u8,
    pub behaviour: CollisionBehaviour,
    pub rendering_size: (f32, f32),
    pub collision_size: (f32, f32),
    pub uv: (f32, f32),
}

#[derive(Debug)]
pub struct SimplexPatternMapObject {
    pub detail: u8,
    pub chance: u8,
    pub priority: u8,
    pub behaviour: CollisionBehaviour,
    pub rendering_size: (f32, f32),
    pub collision_size: (f32, f32),
    pub seed: u8,
    pub acceptable_noise: (f64, f64),
    pub noise_scale: f64,
    pub uv: (f32, f32),
}

#[derive(Debug)]
pub struct SimplexSmoothedPatternMapObject {
    pub detail: u8,
    pub chance: u8,
    pub priority: u8,
    pub behaviour: CollisionBehaviour,
    pub size: f32, // Bad name? This is the size of a single square during marching squares.
    pub seed: u8,
    pub acceptable_noise: (f64, f64),
    pub noise_scale: f64,
}

// leaving this structh here, so no one makes the mistake of trying this again. You still gotta match, to get the right array, so this is useless, unless we wanted to store these in their own array, which is a big NO, until we have the easy biomes in testing_biomes.rs sorted. Even then we would still need some sort of index into this array, which sounds slow.
// pub struct CommonMapObject {

// }

#[derive(Debug, Copy, Clone)]
pub enum MapObject {
    None,
    RandomPattern(u8),
    SimplexPattern(u8),
    SimplexSmoothedPattern(u8),
}

#[derive(Debug, Clone, Copy)]
pub struct Biome {
    pub aabb: Aabb,
    pub random_pattern: PatternArray,
    pub simplex_pattern: PatternArray,
    pub simplex_smoothed_pattern: PatternArray,
}

#[derive(Debug, Clone, Copy)]
pub struct Aabb {
    pub size: (f64, f64),

    pub position: (f64, f64),
}

impl Aabb {
    fn point_intersects(&self, position: (f64, f64)) -> bool {
        position.0 < self.position.0 + self.size.0
            && position.0 > self.position.0
            && position.1 < self.position.1 + self.size.1
            && position.1 > self.position.1
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Statistics {
    // TODO: is there a better name?
    pub strength: u8,
    pub health: i32, // Although player health will never be below zero, map objects could wish to lower the health, therefore requiring negative health.
    pub stamina: i32,
}

impl AddAssign for Statistics {
    fn add_assign(&mut self, other: Self) {
        *self = Statistics {
            strength: self.strength + other.strength,
            health: self.health + other.health,
            stamina: self.stamina + other.stamina,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PatternArray {
    pub starting_index: u8,
    pub length: u8,
}

#[derive(Debug, Clone, Copy)]
pub enum CollisionBehaviour {
    None,
    Consume(u8, Statistics),
    Replace(u8, Statistics, MapObject),
    //Push(u8), Some objects should be pushable. This is not possible until we get a proper physics system working.
}
