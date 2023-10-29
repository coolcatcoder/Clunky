use crate::biomes;
use crate::ignore;
use crate::transform_biomes;

#[allow(long_running_const_eval)]
const ALL_BIOME_DATA: (
    [biomes::Biome; 1],
    [biomes::RandomPatternMapObject; 1],
    [biomes::SimplexPatternMapObject; 1],
    [biomes::SimplexSmoothedPatternMapObject; 0],
) = transform_biomes![SPARSE_ROCK];

const SPARSE_ROCK: EasyBiome<1, 1, 0> = EasyBiome {
    aabb: biomes::Aabb {
        size: (1.0, 1.0),
        position: (0.0, 0.0),
    },
    random_pattern: [biomes::RandomPatternMapObject {
        // weird stamina rock
        detail: 0,
        chance: 10,
        priority: 2,
        behaviour: biomes::CollisionBehaviour::Consume(
            0,
            biomes::Statistics {
                strength: 0,
                health: 0,
                stamina: 2,
            },
        ),
        rendering_size: (1.0, 1.0),
        collision_size: (1.0, 1.0),
        uv: (8.0 * biomes::SPRITE_SIZE.0, 0.0),
    }],

    simplex_pattern: [biomes::SimplexPatternMapObject {
        // circus rock
        detail: 0,
        chance: 100,
        priority: 1,
        behaviour: biomes::CollisionBehaviour::Consume(
            2,
            biomes::Statistics {
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
        uv: (7.0 * biomes::SPRITE_SIZE.0, 0.0),
    }],

    simplex_smoothed_pattern: [],
};

struct EasyBiome<const R: usize, const S: usize, const SS: usize> {
    aabb: biomes::Aabb,
    pub random_pattern: [biomes::RandomPatternMapObject; R],
    pub simplex_pattern: [biomes::SimplexPatternMapObject; S],
    pub simplex_smoothed_pattern: [biomes::SimplexSmoothedPatternMapObject; SS],
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

            let mut biomes = [biomes::Biome {
                aabb: biomes::Aabb {
                    position: (0.0,0.0),
                    size: (0.0,0.0),
                },
                random_pattern: biomes::PatternArray {
                    starting_index: 0,
                    length: 0,
                },
                simplex_pattern: biomes::PatternArray {
                    starting_index: 0,
                    length: 0,
                },
                simplex_smoothed_pattern: biomes::PatternArray {
                    starting_index: 0,
                    length: 0,
                },
            }; BIOME_AMOUNT];

            let mut random_pattern_map_objects = [biomes::RandomPatternMapObject {
                detail: 0,
                chance: 0,
                priority: 0,
                behaviour: biomes::CollisionBehaviour::None,
                rendering_size: (0.0, 0.0),
                collision_size: (0.0, 0.0),
                uv: (0.0, 0.0),
            }; RANDOM_AMOUNT];

            let mut simplex_pattern_map_objects = [biomes::SimplexPatternMapObject {
                detail: 0,
                chance: 0,
                priority: 0,
                behaviour: biomes::CollisionBehaviour::None,
                rendering_size: (0.0, 0.0),
                collision_size: (0.0, 0.0),
                seed: 0,
                acceptable_noise: (0.0, 0.0),
                noise_scale: 0.0,
                uv: (0.0, 0.0),
            }; SIMPLEX_AMOUNT];

            let mut simplex_smoothed_pattern_map_objects = [biomes::SimplexSmoothedPatternMapObject {
                detail: 0,
                chance: 0,
                priority: 0,
                behaviour: biomes::CollisionBehaviour::None,
                size: 0.0,
                seed: 0,
                acceptable_noise: (0.0, 0.0),
                noise_scale: 0.0,
            }; SIMPLEX_SMOOTHED_AMOUNT];

            let mut biome_index = 0;

            let mut random_pattern_index = 0;
            let mut simplex_pattern_index = 0;
            let mut simplex_smoothed_pattern_index = 0;

            $(
                biomes[biome_index] = biomes::Biome {
                    aabb: $x.aabb,
                    random_pattern: biomes::PatternArray {
                        starting_index: random_pattern_index as u8,
                        length: $x.random_pattern.len() as u8,
                    },
                    simplex_pattern: biomes::PatternArray {
                        starting_index: 0,
                        length: 0,
                    },
                    simplex_smoothed_pattern: biomes::PatternArray {
                        starting_index: 0,
                        length: 0,
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

#[macro_export]
macro_rules! ignore {
    ($($t:tt)*) => {};
}
