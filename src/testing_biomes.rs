use crate::biomes;
use crate::transform_biomes;

//const BIOMES: [biomes::Biome; 2] = transform_biomes![TESTING_BIOME];

const TESTING_BIOME: EasyBiome<1> = EasyBiome {
    aabb: biomes::Aabb {
        size: (1.0, 1.0),
        position: (0.0, 0.0),
    },
    random_pattern: [biomes::RandomPatternMapObject {
        detail: 0,
        chance: 100,
        priority: 5,
        behaviour: biomes::CollisionBehaviour::None,
        rendering_size: (1.0, 1.0),
        collision_size: (1.0, 1.0),
        uv: (0.0, 0.0),
    }],
};

struct EasyBiome<const R: usize> {
    aabb: biomes::Aabb,
    pub random_pattern: [biomes::RandomPatternMapObject; R],
}

#[macro_export]
macro_rules! transform_biomes {
    ( ($x:expr ),* ) => {
        {
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
            }; 2];

            let mut random_pattern_map_objects = [biomes::RandomPatternMapObject {
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
            }; 0]

            let mut biome_index = 0usize;

            let mut random_pattern_index = 0u8;

            $(
                biomes[biome_index] = biomes::Biome {
                    aabb: $x.aabb,
                    random_pattern: biomes::PatternArray {
                        starting_index: random_pattern_index,
                        length: $x.random_pattern.len() as u8,
                    },
                    simplex_pattern: biomes::PatternArray {
                        starting_index: 4,
                        length: 1,
                    },
                    simplex_smoothed_pattern: biomes::PatternArray {
                        starting_index: 0,
                        length: 0,
                    },
                };

                biome_index += 1; // I should read this, but I'm too scared.
            )*

            (biomes, random_pattern_map_objects)
        }
    };
}
