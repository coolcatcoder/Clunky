use noise::NoiseFn;
use noise::OpenSimplex;
use vulkano::buffer::subbuffer::BufferWriteGuard;
use rand::thread_rng;
use rand::rngs::ThreadRng;
use rand::distributions::{Distribution, Uniform};

use crate::biomes;
use crate::vertex_data;

#[rustfmt::skip]
pub const STARTING_VERTICES: [vertex_data::VertexData; 7] = [
    vertex_data::VertexData {
        position: [-1.0, 1.0],
        uv: [0.0, 1.0],
    },
    vertex_data::VertexData {
        position: [1.0, 1.0],
        uv: [0.0, 1.0],
    },
    vertex_data::VertexData {
        position: [1.0, -1.0],
        uv: [0.0, 1.0],
    },
    vertex_data::VertexData {
        position: [-10.0, 1.5],
        uv: [0.0, 1.0],
    },

    vertex_data::VertexData {
        position: [-1.0, 1.5],
        uv: [0.0, 1.0],
    },
    vertex_data::VertexData {
        position: [-10.0, 1.0],
        uv: [0.0, 1.0],
    },
    vertex_data::VertexData {
        position: [-1.0, 1.0],
        uv: [0.0, 1.0],
    },
];

#[rustfmt::skip]
pub const STARTING_INDICES: [u16; 9] = [
    0,1,2,
    3,4,5,
    4,5,6,
];

pub const STARTING_INDEX_COUNT: u32 = 3;

pub fn start() -> Storage {
    let mut rng = thread_rng();
    let seed_range = Uniform::new(0u32,1000);

    let mut storage = Storage {
        direction: -1.0,
        main_seed: seed_range.sample(&mut rng),
        percent_range: Uniform::new(0u8,100),
        biome_noise_x: OpenSimplex::new(seed_range.sample(&mut rng)),
        biome_noise_y: OpenSimplex::new(seed_range.sample(&mut rng)),
        rng: rng,
    };

    let test_generation = generate_position(&mut storage, 0.0, 0.0);

    match test_generation {
        biomes::MapObject::RandomPattern(i) => {
            println!("{:?}",biomes::RANDOM_PATTERN_MAP_OBJECTS[i as usize]);
        },
        biomes::MapObject::SimplexPattern(i) => {
            println!("{:?}",biomes::SIMPLEX_PATTERN_MAP_OBJECTS[i as usize]);
        },
        biomes::MapObject::SimplexSmoothedPattern(i) => {
            println!("{:?}",biomes::SIMPLEX_SMOOTHED_PATTERN_MAP_OBJECTS[i as usize]);
        },
        biomes::MapObject::None => {}
    }

    storage
}

pub fn update(
    storage: &mut Storage,
    mut vertices: BufferWriteGuard<'_, [vertex_data::VertexData]>,
    mut indices: BufferWriteGuard<'_, [u16]>,
    index_count: &mut u32,
    delta_time: f32,
    average_fps: f32,
) {
    //println!("delta time: {}", delta_time);
    //println!("average fps: {}", average_fps);
    vertices[0].position[1] += storage.direction * delta_time;

    if vertices[0].position[1] < -1.0 {
        storage.direction = 1.0;
    } else if vertices[0].position[1] > 1.0 {
        storage.direction = -1.0;
    }
}

pub fn late_update(storage: &mut Storage, delta_time: f32, average_fps: f32) {}

pub struct Storage {
    // This is for the user's stuff. The event loop should not touch this.
    direction: f32,
    rng: ThreadRng,
    main_seed: u32,
    percent_range: Uniform<u8>,
    biome_noise_x: OpenSimplex,
    biome_noise_y: OpenSimplex,
}

// Block generation:

pub fn generate_position(storage: &mut Storage, position_x: f64, position_y: f64) -> biomes::MapObject {
    let biome = &biomes::BIOMES[biomes::get_biome(
        storage.biome_noise_x.get([position_x, position_y]),
        storage.biome_noise_y.get([position_x, position_y]),
    )];

    let mut map_object = biomes::MapObject::None;
    let mut highest_priority = 0u8;

    for i in biome.random_pattern.starting_index..biome.random_pattern.length {
        let random_pattern_map_object = &biomes::RANDOM_PATTERN_MAP_OBJECTS[i as usize];
        if random_pattern_map_object.priority > highest_priority && storage.percent_range.sample(&mut storage.rng) < random_pattern_map_object.chance {
            map_object = biomes::MapObject::RandomPattern(i);
            highest_priority = random_pattern_map_object.priority
        }
    }

    for i in biome.simplex_pattern.starting_index..biome.simplex_pattern.length {
        let simplex_pattern_map_object = &biomes::SIMPLEX_PATTERN_MAP_OBJECTS[i as usize];
        let simplex_noise = OpenSimplex::new(storage.main_seed + simplex_pattern_map_object.seed as u32).get([position_x,position_y]);
        if simplex_pattern_map_object.priority > highest_priority && storage.percent_range.sample(&mut storage.rng) < simplex_pattern_map_object.chance && simplex_noise > simplex_pattern_map_object.acceptable_noise.0 && simplex_noise < simplex_pattern_map_object.acceptable_noise.1 {
            map_object = biomes::MapObject::SimplexPattern(i);
            highest_priority = simplex_pattern_map_object.priority
        }
    }

    for i in biome.simplex_smoothed_pattern.starting_index..biome.simplex_smoothed_pattern.length {
        let simplex_smoothed_pattern_map_object = &biomes::SIMPLEX_SMOOTHED_PATTERN_MAP_OBJECTS[i as usize];
        let simplex_noise = OpenSimplex::new(storage.main_seed + simplex_smoothed_pattern_map_object.seed as u32).get([position_x,position_y]);
        if simplex_smoothed_pattern_map_object.priority > highest_priority && storage.percent_range.sample(&mut storage.rng) < simplex_smoothed_pattern_map_object.chance && simplex_noise > simplex_smoothed_pattern_map_object.acceptable_noise.0 && simplex_noise < simplex_smoothed_pattern_map_object.acceptable_noise.1 {
            map_object = biomes::MapObject::SimplexSmoothedPattern(i);
            highest_priority = simplex_smoothed_pattern_map_object.priority
        }
    }

    map_object
}
