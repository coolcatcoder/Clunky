use rand::distributions::{Distribution, Uniform};
use rand::thread_rng;

use crate::collision;
use crate::events;
use crate::transform_biomes;
use std::ops::AddAssign;

pub fn get_biome(biome_noise: (f64, f64)) -> usize {
    // To handle overlapping shapes we should instead go through each biome, if it is a correct biome, add it to a vector, then pick randomly from the vector at the end.
    for biome_index in 0..BIOMES.len() {
        if collision::point_intersects_aabb(BIOMES[biome_index].aabb, biome_noise) {
            //println!("{}",biome_index); // good for debugging
            return biome_index;
        }
    }

    println!("{:?} matches no biomes", biome_noise);
    0
}

pub const BIOME_SCALE: (f64, f64) = (0.05, 0.05);

pub const SPRITE_SIZE: (f32, f32) = (1.0 / 44.0, 1.0);

pub const BIOMES: [Biome; ALL_BIOME_DATA.0.len()] = ALL_BIOME_DATA.0;

pub const RANDOM_PATTERN_MAP_OBJECTS: [RandomPatternMapObject; ALL_BIOME_DATA.1.len()] =
    ALL_BIOME_DATA.1;

pub const SIMPLEX_PATTERN_MAP_OBJECTS: [SimplexPatternMapObject; ALL_BIOME_DATA.2.len()] =
    ALL_BIOME_DATA.2;

pub const SIMPLEX_SMOOTHED_PATTERN_MAP_OBJECTS: [SimplexSmoothedPatternMapObject;
    ALL_BIOME_DATA.3.len()] = ALL_BIOME_DATA.3;

#[derive(Debug, Copy, Clone)]
pub struct RandomPatternMapObject {
    pub detail: u8,
    pub chance: u8,
    pub priority: u8,
    pub behaviour: CollisionBehaviour,
    pub rendering_size: (f32, f32),
    pub collision_size: (f32, f32),
    pub uv: (f32, f32),
    pub depth: f32,
}

#[derive(Debug, Copy, Clone)]
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
    pub depth: f32,
}

#[derive(Debug, Copy, Clone)]
pub struct SimplexSmoothedPatternMapObject {
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
    pub depth: f32,
}

// leaving this structh here, so no one makes the mistake of trying this again. You still gotta match, to get the right array, so this is useless, unless we wanted to store these in their own array, which is a big NO, until we have the easy biomes in testing_biomes.rs sorted. Even then we would still need some sort of index into this array, which sounds slow.
// pub struct CommonMapObject {

// }

#[derive(Debug, Copy, Clone)]
pub enum MapObject {
    None,
    RandomPattern(u8),
    SimplexPattern(u8),
    SimplexSmoothedPattern(u8, u8), // index into S_S_MAP_OBJECTS, index into marching squares
}

#[derive(Debug, Clone, Copy)]
pub struct Biome {
    pub aabb: collision::Aabb,
    pub random_pattern: PatternArray,
    pub simplex_pattern: PatternArray,
    pub simplex_smoothed_pattern: PatternArray,
}

#[derive(Debug, Clone, Copy)]
pub struct Statistics {
    // TODO: is there a better name?
    pub strength: i8,
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
    RunCode(u8), // the u8 should be an index into some array of code
}

pub const MAP_OBJECT_COLLISION_FUNCTIONS: [fn(
    &mut events::UserStorage,
    &mut events::RenderStorage,
    (u32, u32),
    u8,
); 3] = [
    |user_storage: &mut events::UserStorage,
     _render_storage: &mut events::RenderStorage,
     full_position: (u32, u32),
     detail_index: u8| {
        let position_range = Uniform::new(1, 50);
        let mut rng = thread_rng();

        user_storage.map_objects[detail_index as usize][events::full_index_from_full_position(
            (
                full_position.0 + position_range.sample(&mut rng),
                full_position.1 + position_range.sample(&mut rng),
            ),
            user_storage.details[detail_index as usize].scale,
        )] = MapObject::RandomPattern(
            Uniform::new(0, RANDOM_PATTERN_MAP_OBJECTS.len() as u8).sample(&mut rng),
        );

        user_storage.map_objects[detail_index as usize][events::full_index_from_full_position(
            (
                full_position.0 + position_range.sample(&mut rng),
                full_position.1 + position_range.sample(&mut rng),
            ),
            user_storage.details[detail_index as usize].scale,
        )] = MapObject::SimplexPattern(
            Uniform::new(0, SIMPLEX_PATTERN_MAP_OBJECTS.len() as u8).sample(&mut rng),
        );

        user_storage.map_objects[detail_index as usize][events::full_index_from_full_position(
            (
                full_position.0 + position_range.sample(&mut rng),
                full_position.1 + position_range.sample(&mut rng),
            ),
            user_storage.details[detail_index as usize].scale,
        )] = MapObject::SimplexSmoothedPattern(
            Uniform::new(0, SIMPLEX_SMOOTHED_PATTERN_MAP_OBJECTS.len() as u8).sample(&mut rng),
            Uniform::new(0, 16).sample(&mut rng),
        );
    },
    |_user_storage: &mut events::UserStorage,
     render_storage: &mut events::RenderStorage,
     _full_position: (u32, u32),
     _detail_index: u8| {
        render_storage.brightness -= 1.0 * events::FIXED_UPDATE_TIME_STEP;
    },
    |_user_storage: &mut events::UserStorage,
     render_storage: &mut events::RenderStorage,
     _full_position: (u32, u32),
     _detail_index: u8| {
        render_storage.brightness += 1.0 * events::FIXED_UPDATE_TIME_STEP;
    },
];

#[allow(long_running_const_eval)]
#[allow(unused_assignments)]
const ALL_BIOME_DATA: (
    [Biome; 6],
    [RandomPatternMapObject; 13],
    [SimplexPatternMapObject; 7],
    [SimplexSmoothedPatternMapObject; 2],
) = transform_biomes![
    MANUAL_MAP_OBJECT_STORAGE,
    SPARSE_ROCK,
    MIXED_JUNGLE,
    GRASSLANDS,
    DESERT,
    MOUNTAINS
];

#[allow(dead_code)]
const TEMPLATE_BIOME: EasyBiome<0, 0, 0> = EasyBiome {
    aabb: collision::Aabb {
        size: (0.0, 0.0),
        position: (0.0, 0.0),
    },

    random_pattern: [],

    simplex_pattern: [],

    simplex_smoothed_pattern: [],
};

const MANUAL_MAP_OBJECT_STORAGE: EasyBiome<3, 0, 0> = EasyBiome {
    // this biome should store any map objects that are planned to be used druing CollisionBehaviour::Replace, so reordering of biomes and map objects doesn't change the replacement map objects
    aabb: collision::Aabb {
        size: (0.0, 0.0),
        position: (100.0, 100.0),
    },

    random_pattern: [
        RandomPatternMapObject {
            // debug testing manual placing of structures, should be index 0 in RANDOM_PATTERN_MAP_OBJECTS
            detail: 0,
            chance: 100,
            priority: 255,
            behaviour: CollisionBehaviour::RunCode(0),
            rendering_size: (1.0, 1.0),
            collision_size: (1.0, 1.0),
            uv: (38.0 * SPRITE_SIZE.0, 0.0),
            depth: 0.1,
        },
        RandomPatternMapObject {
            // index 1
            detail: 0,
            chance: 100,
            priority: 255,
            behaviour: CollisionBehaviour::RunCode(1),
            rendering_size: (1.0, 1.0),
            collision_size: (1.0, 1.0),
            uv: (37.0 * SPRITE_SIZE.0, 0.0),
            depth: 0.1,
        },
        RandomPatternMapObject {
            // index 2
            detail: 0,
            chance: 100,
            priority: 255,
            behaviour: CollisionBehaviour::RunCode(2),
            rendering_size: (1.0, 1.0),
            collision_size: (1.0, 1.0),
            uv: (39.0 * SPRITE_SIZE.0, 0.0),
            depth: 0.1,
        },
        // RandomPatternMapObject {
        //     // index 3
        //     detail: 0,
        //     chance: 100,
        //     priority: 255,
        //     behaviour: CollisionBehaviour::RunCode(3),
        //     rendering_size: (1.0, 1.0),
        //     collision_size: (1.0, 1.0),
        //     uv: (41.0 * SPRITE_SIZE.0, 0.0),
        // },
    ],

    simplex_pattern: [],

    simplex_smoothed_pattern: [],
};

const SPARSE_ROCK: EasyBiome<1, 1, 0> = EasyBiome {
    aabb: collision::Aabb {
        size: (1.0 / 3.0 + 0.01, 0.5),
        position: (0.0, 0.5),
    },

    random_pattern: [RandomPatternMapObject {
        // weird stamina rock
        detail: 0,
        chance: 10,
        priority: 2,
        behaviour: CollisionBehaviour::Consume(
            1,
            Statistics {
                strength: 0,
                health: 0,
                stamina: 2,
            },
        ),
        rendering_size: (1.0, 1.0),
        collision_size: (1.0, 1.0),
        uv: (8.0 * SPRITE_SIZE.0, 0.0),
        depth: 0.1,
    }],

    simplex_pattern: [SimplexPatternMapObject {
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
        depth: 0.1,
    }],

    simplex_smoothed_pattern: [],
};

const MIXED_JUNGLE: EasyBiome<2, 3, 2> = EasyBiome {
    aabb: collision::Aabb {
        size: (2.0 / 3.0, 0.5),
        position: (1.0 / 3.0, 0.5),
    },

    random_pattern: [
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
            depth: 0.1,
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
            depth: 0.1,
        },
    ],

    simplex_pattern: [
        SimplexPatternMapObject {
            // circus rock detail 0 blank filler
            detail: 0,
            chance: 100,
            priority: 2,
            behaviour: CollisionBehaviour::None,
            rendering_size: (0.5, 0.5),
            collision_size: (0.0, 0.0),
            seed: 1,
            acceptable_noise: (-0.2, 1.15),
            noise_scale: 0.15,
            uv: (0.0 * SPRITE_SIZE.0, 0.0),
            depth: 0.1,
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
            depth: 0.1,
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
            depth: 0.1,
        },
    ],

    simplex_smoothed_pattern: [
        SimplexSmoothedPatternMapObject {
            // circus rock
            detail: 2,
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
            rendering_size: (1.0 / 3.0, 1.0 / 3.0),
            collision_size: (1.0 / 3.0, 1.0 / 3.0),
            seed: 1,
            acceptable_noise: (0.0, 1.0),
            noise_scale: 0.15,
            uv: (7.0 * SPRITE_SIZE.0, 0.0),
            depth: 0.5,
        },
        SimplexSmoothedPatternMapObject {
            // the weird stamina rock
            detail: 1,
            chance: 100,
            priority: 3,
            behaviour: CollisionBehaviour::Consume(
                1,
                Statistics {
                    strength: 0,
                    health: 0,
                    stamina: 2,
                },
            ),
            rendering_size: (0.5, 0.5),
            collision_size: (0.5, 0.5),
            seed: 1,
            acceptable_noise: (-0.2, 0.08),
            noise_scale: 0.15,
            uv: (8.0 * SPRITE_SIZE.0, 0.0),
            depth: 0.1,
        },
    ],
};

const GRASSLANDS: EasyBiome<1, 1, 0> = EasyBiome {
    aabb: collision::Aabb {
        size: (1.0 / 3.0 + 0.01, 0.501),
        position: (0.0, 0.0),
    },

    random_pattern: [RandomPatternMapObject {
        // swishy swishy
        detail: 0,
        chance: 100,
        priority: 1,
        behaviour: CollisionBehaviour::None,
        rendering_size: (1.0, 1.0),
        collision_size: (0.0, 0.0),
        uv: (14.0 * SPRITE_SIZE.0, 0.0),
        depth: 0.1,
    }],

    simplex_pattern: [SimplexPatternMapObject {
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
        depth: 0.1,
    }],

    simplex_smoothed_pattern: [],
};

const DESERT: EasyBiome<1, 1, 0> = EasyBiome {
    aabb: collision::Aabb {
        size: (1.0 / 3.0 + 0.01, 0.501),
        position: (1.0 / 3.0, 0.0),
    },

    random_pattern: [RandomPatternMapObject {
        // sand
        detail: 0,
        chance: 100,
        priority: 1,
        behaviour: CollisionBehaviour::None,
        rendering_size: (1.0, 1.0),
        collision_size: (0.0, 0.0),
        uv: (11.0 * SPRITE_SIZE.0, 0.0),
        depth: 0.1,
    }],

    simplex_pattern: [SimplexPatternMapObject {
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
        depth: 0.1,
    }],

    simplex_smoothed_pattern: [],
};

const MOUNTAINS: EasyBiome<5, 1, 0> = EasyBiome {
    aabb: collision::Aabb {
        size: (1.0 / 3.0, 0.501),
        position: (2.0 / 3.0, 0.0),
    },

    random_pattern: [
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
            depth: 0.1,
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
            depth: 0.1,
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
            depth: 0.1,
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
            depth: 0.1,
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
            depth: 0.1,
        },
    ],

    simplex_pattern: [SimplexPatternMapObject {
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
        depth: 0.1,
    }],

    simplex_smoothed_pattern: [],
};

struct EasyBiome<const R: usize, const S: usize, const SS: usize> {
    aabb: collision::Aabb,
    pub random_pattern: [RandomPatternMapObject; R],
    pub simplex_pattern: [SimplexPatternMapObject; S],
    pub simplex_smoothed_pattern: [SimplexSmoothedPatternMapObject; SS],
}

#[macro_export]
macro_rules! transform_biomes {
    ($($x:expr),*) => {
        {
            const ALL_AMOUNTS: (usize,usize,usize,usize) = {
                let mut biome_amount = 0;
                let mut random_amount = 0;
                let mut simplex_amount = 0;
                let mut simplex_smoothed_amount = 0;
                $(
                    biome_amount += 1;
                    random_amount += $x.random_pattern.len();
                    simplex_amount += $x.simplex_pattern.len();
                    simplex_smoothed_amount += $x.simplex_smoothed_pattern.len();
                )*
                (biome_amount,random_amount,simplex_amount,simplex_smoothed_amount)
            };

            const BIOME_AMOUNT: usize = ALL_AMOUNTS.0;
            const RANDOM_AMOUNT: usize = ALL_AMOUNTS.1;
            const SIMPLEX_AMOUNT: usize = ALL_AMOUNTS.2;
            const SIMPLEX_SMOOTHED_AMOUNT: usize = ALL_AMOUNTS.3;

            let mut biomes = [Biome {
                aabb: collision::Aabb {
                    position: (0.0,0.0),
                    size: (0.0,0.0),
                },
                random_pattern: PatternArray {
                    starting_index: 0,
                    length: 0,
                },
                simplex_pattern: PatternArray {
                    starting_index: 0,
                    length: 0,
                },
                simplex_smoothed_pattern: PatternArray {
                    starting_index: 0,
                    length: 0,
                },
            }; BIOME_AMOUNT];

            let mut random_pattern_map_objects = [RandomPatternMapObject {
                detail: 0,
                chance: 0,
                priority: 0,
                behaviour: CollisionBehaviour::None,
                rendering_size: (0.0, 0.0),
                collision_size: (0.0, 0.0),
                uv: (0.0, 0.0),
                depth: 0.1,
            }; RANDOM_AMOUNT];

            let mut simplex_pattern_map_objects = [SimplexPatternMapObject {
                detail: 0,
                chance: 0,
                priority: 0,
                behaviour: CollisionBehaviour::None,
                rendering_size: (0.0, 0.0),
                collision_size: (0.0, 0.0),
                seed: 0,
                acceptable_noise: (0.0, 0.0),
                noise_scale: 0.0,
                uv: (0.0, 0.0),
                depth: 0.1,
            }; SIMPLEX_AMOUNT];

            let mut simplex_smoothed_pattern_map_objects = [SimplexSmoothedPatternMapObject {
                detail: 0,
                chance: 0,
                priority: 0,
                behaviour: CollisionBehaviour::None,
                rendering_size: (0.0, 0.0),
                collision_size: (0.0, 0.0),
                seed: 0,
                acceptable_noise: (0.0, 0.0),
                noise_scale: 0.0,
                uv: (0.0,0.0),
                depth: 0.1,
            }; SIMPLEX_SMOOTHED_AMOUNT];

            let mut biome_index = 0;

            let mut random_pattern_index = 0;
            let mut simplex_pattern_index = 0;
            let mut simplex_smoothed_pattern_index = 0;

            $(
                biomes[biome_index] = Biome {
                    aabb: $x.aabb,
                    random_pattern: PatternArray {
                        starting_index: random_pattern_index as u8,
                        length: $x.random_pattern.len() as u8,
                    },
                    simplex_pattern: PatternArray {
                        starting_index: simplex_pattern_index as u8,
                        length: $x.simplex_pattern.len() as u8,
                    },
                    simplex_smoothed_pattern: PatternArray {
                        starting_index: simplex_smoothed_pattern_index as u8,
                        length: $x.simplex_smoothed_pattern.len() as u8,
                    },
                };

                let mut i = random_pattern_index;
                while i < random_pattern_index + $x.random_pattern.len() {
                    random_pattern_map_objects[i] = $x.random_pattern[i-random_pattern_index];
                    i += 1;
                }

                let mut i = simplex_pattern_index;
                while i < simplex_pattern_index + $x.simplex_pattern.len() {
                    simplex_pattern_map_objects[i] = $x.simplex_pattern[i-simplex_pattern_index];
                    i += 1;
                }

                let mut i = simplex_smoothed_pattern_index;
                while i < simplex_smoothed_pattern_index + $x.simplex_smoothed_pattern.len() {
                    simplex_smoothed_pattern_map_objects[i] = $x.simplex_smoothed_pattern[i-simplex_smoothed_pattern_index];
                    i += 1;
                }

                biome_index += 1;
                random_pattern_index += $x.random_pattern.len();
                simplex_pattern_index += $x.simplex_pattern.len();
                simplex_smoothed_pattern_index += $x.simplex_smoothed_pattern.len();
            )*

            (biomes, random_pattern_map_objects, simplex_pattern_map_objects, simplex_smoothed_pattern_map_objects)
        }
    };
}
