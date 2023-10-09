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

pub const SPRITE_SIZE: (f32, f32) = (1.0 / 42.0, 1.0);

pub const BIOMES: [Biome; 5] = [
    Biome {
        // sparse rock
        aabb: Aabb {
            size_x: 1.0 / 3.0 + 0.01,
            size_y: 0.5,
            position_x: 0.0,
            position_y: 0.5,
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
            size_x: 2.0 / 3.0,
            size_y: 0.5,
            position_x: 1.0 / 3.0,
            position_y: 0.5,
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
            size_x: 1.0 / 3.0 + 0.01,
            size_y: 0.501,
            position_x: 0.0,
            position_y: 0.0,
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
            size_x: 1.0 / 3.0 + 0.01,
            size_y: 0.501,
            position_x: 1.0 / 3.0,
            position_y: 0.0,
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
            size_x: 1.0 / 3.0,
            size_y: 0.501,
            position_x: 2.0 / 3.0,
            position_y: 0.0,
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

pub const RANDOM_PATTERN_MAP_OBJECTS: [RandomPatternMapObject; 10] = [
    // start mixed jungle
    RandomPatternMapObject {
        // health fruit medium
        chance: 9,
        priority: 1,
        behaviour: CollisionBehaviour::Consume(0),
        rendering_size: (0.75, 0.75),
        collision_size: (0.6, 0.6),
        uv: (20.0 * SPRITE_SIZE.0, 0.0),
    },
    RandomPatternMapObject {
        // health fruit large
        chance: 1,
        priority: 1,
        behaviour: CollisionBehaviour::Consume(0),
        rendering_size: (1.3, 1.3),
        collision_size: (1.0, 1.0),
        uv: (20.0 * SPRITE_SIZE.0, 0.0),
    },
    // end mixed jungle

    // start sparse rock
    RandomPatternMapObject {
        // weird stamina rock
        chance: 10,
        priority: 2,
        behaviour: CollisionBehaviour::Consume(0),
        rendering_size: (1.0, 1.0),
        collision_size: (1.0, 1.0),
        uv: (8.0 * SPRITE_SIZE.0, 0.0),
    },
    // end sparse rock

    // start grasslands
    RandomPatternMapObject {
        // swishy swishy
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
        chance: 10,
        priority: 2,
        behaviour: CollisionBehaviour::Consume(0),
        rendering_size: (0.75, 0.75),
        collision_size: (0.6, 0.6),
        uv: (21.0 * SPRITE_SIZE.0, 0.0),
    },
    RandomPatternMapObject {
        // spiccaro purple
        chance: 1,
        priority: 2,
        behaviour: CollisionBehaviour::Consume(0),
        rendering_size: (0.75, 0.75),
        collision_size: (0.6, 0.6),
        uv: (22.0 * SPRITE_SIZE.0, 0.0),
    },
    RandomPatternMapObject {
        // spiccaro blue
        chance: 1,
        priority: 2,
        behaviour: CollisionBehaviour::Consume(0),
        rendering_size: (0.75, 0.75),
        collision_size: (0.6, 0.6),
        uv: (23.0 * SPRITE_SIZE.0, 0.0),
    },
    RandomPatternMapObject {
        // spiccaro orange
        chance: 1,
        priority: 2,
        behaviour: CollisionBehaviour::Consume(0),
        rendering_size: (0.75, 0.75),
        collision_size: (0.6, 0.6),
        uv: (24.0 * SPRITE_SIZE.0, 0.0),
    },
    RandomPatternMapObject {
        // dark velvet slicer
        chance: 50,
        priority: 1,
        behaviour: CollisionBehaviour::Consume(0),
        rendering_size: (1.3, 1.3),
        collision_size: (1.0, 1.0),
        uv: (19.0 * SPRITE_SIZE.0, 0.0),
    },
    // end mountain
];

pub const SIMPLEX_PATTERN_MAP_OBJECTS: [SimplexPatternMapObject; 8] = [
    // start mixed jungle
    SimplexPatternMapObject {
        // circus rock
        chance: 100,
        priority: 3,
        behaviour: CollisionBehaviour::Consume(2),
        rendering_size: (1.0, 1.0),
        collision_size: (1.0, 1.0),
        seed: 1,
        acceptable_noise: (0.0, 1.0),
        noise_scale: 0.15,
        uv: (7.0 * SPRITE_SIZE.0, 0.0),
    },
    SimplexPatternMapObject {
        // the weird stamina rock
        chance: 100,
        priority: 2,
        behaviour: CollisionBehaviour::Consume(0),
        rendering_size: (1.0, 1.0),
        collision_size: (1.0, 1.0),
        seed: 1,
        acceptable_noise: (-0.2, 1.0),
        noise_scale: 0.15,
        uv: (8.0 * SPRITE_SIZE.0, 0.0),
    },
    SimplexPatternMapObject {
        // velvet slicer
        chance: 75,
        priority: 1,
        behaviour: CollisionBehaviour::Consume(0),
        rendering_size: (1.3, 1.3),
        collision_size: (1.0, 1.0),
        seed: 2,
        acceptable_noise: (0.2, 0.5),
        noise_scale: 0.1,
        uv: (16.0 * SPRITE_SIZE.0, 0.0),
    },
    SimplexPatternMapObject {
        // Large test
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
        chance: 100,
        priority: 1,
        behaviour: CollisionBehaviour::Consume(2),
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
        chance: 75,
        priority: 2,
        behaviour: CollisionBehaviour::Replace(0, MapObject::RandomPattern(3)),
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
        chance: 45,
        priority: 2,
        behaviour: CollisionBehaviour::Replace(0, MapObject::RandomPattern(4)),
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
        chance: 100,
        priority: 3,
        behaviour: CollisionBehaviour::Consume(2),
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
    pub chance: u8,
    pub priority: u8,
    pub behaviour: CollisionBehaviour,
    pub rendering_size: (f32, f32),
    pub collision_size: (f32, f32),
    pub uv: (f32, f32),
}

#[derive(Debug)]
pub struct SimplexPatternMapObject {
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
    pub chance: u8,
    pub priority: u8,
    pub behaviour: CollisionBehaviour,
    pub size: f32, // Bad name? This is the size of a single square during marching squares.
    pub seed: u8,
    pub acceptable_noise: (f64, f64),
    pub noise_scale: f64,
}

#[derive(Debug, Copy, Clone)]
pub enum MapObject {
    None,
    RandomPattern(u8),
    SimplexPattern(u8),
    SimplexSmoothedPattern(u8),
}

#[derive(Debug)]
pub struct Biome {
    aabb: Aabb,
    pub random_pattern: PatternArray,
    pub simplex_pattern: PatternArray,
    pub simplex_smoothed_pattern: PatternArray,
}

#[derive(Debug)]
pub struct Aabb {
    size_x: f64,
    size_y: f64,

    position_x: f64,
    position_y: f64,
}

impl Aabb {
    fn point_intersects(&self, position: (f64, f64)) -> bool {
        position.0 < self.position_x + self.size_x
            && position.0 > self.position_x
            && position.1 < self.position_y + self.size_y
            && position.1 > self.position_y
    }
}

#[derive(Debug)]
pub struct PatternArray {
    pub starting_index: u8,
    pub length: u8,
}

#[derive(Debug)]
pub enum CollisionBehaviour {
    None,
    Consume(u8),
    Replace(u8, MapObject),
    //Push(u8), Some objects should be pushable, I'm just not sure how yet.
}
