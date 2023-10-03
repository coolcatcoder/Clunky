use noise::NoiseFn;
use noise::OpenSimplex;
use rand::distributions::{Distribution, Uniform};
use rand::rngs::ThreadRng;
use rand::thread_rng;
use std::ops::{Add, Mul, Div, Rem};
use vulkano::buffer::subbuffer::BufferWriteGuard;

use crate::biomes;
use crate::vertex_data;

const FULL_GRID_WIDTH: u32 = 256;
const FULL_GRID_WIDTH_SQUARED: u32 = 65536; // 256**2

const CHUNK_WIDTH: u16 = 128;
const CHUNK_WIDTH_SQUARED: u16 = CHUNK_WIDTH * CHUNK_WIDTH;
const CHUNK_WIDTH_LOG2: u16 = (u16::BITS - CHUNK_WIDTH.leading_zeros()) as u16;

const CHUNK_GRID_WIDTH: u32 = FULL_GRID_WIDTH / CHUNK_WIDTH as u32;

#[rustfmt::skip]
pub const STARTING_VERTICES: [vertex_data::VertexData; 7] = [
    vertex_data::VertexData {
        position: [-1.0, 1.0],
        uv: [0.0, 0.5],
    },
    vertex_data::VertexData {
        position: [1.0, 1.0],
        uv: [1.0, 1.0],
    },
    vertex_data::VertexData {
        position: [1.0, -1.0],
        uv: [1.0, 0.0],
    },
    
    vertex_data::VertexData {
        position: [-4.0, 1.5],
        uv: [-4.0, 1.0],
    },
    vertex_data::VertexData {
        position: [4.0, 1.5],
        uv: [4.0, 1.0],
    },
    vertex_data::VertexData {
        position: [-4.0, 1.0],
        uv: [-4.0, 0.0],
    },
    vertex_data::VertexData {
        position: [4.0, 1.0],
        uv: [4.0, 0.0],
    },
];

#[rustfmt::skip]
pub const STARTING_INDICES: [u16; 9] = [
    0,1,2,
    3,4,5,
    4,5,6,
];

pub const STARTING_INDEX_COUNT: u32 = 9;

pub fn start(camera: &mut Camera) -> Storage {
    camera.scale = 0.1;

    let mut rng = thread_rng();
    let seed_range = Uniform::new(0u32, 1000);

    let mut storage = Storage {
        direction: -1.0,
        main_seed: seed_range.sample(&mut rng),
        percent_range: Uniform::new(0u8, 100),
        biome_noise_x: OpenSimplex::new(seed_range.sample(&mut rng)),
        biome_noise_y: OpenSimplex::new(seed_range.sample(&mut rng)),
        rng: rng,
        map_objects: [biomes::MapObject::None; FULL_GRID_WIDTH_SQUARED as usize],
    };

    let test_position = (2u32, 1u32);

    let test_generation = generate_position(&mut storage, test_position);

    let test_index = full_index_from_full_position(test_position);

    storage.map_objects[test_index] = test_generation;

    match test_generation {
        biomes::MapObject::RandomPattern(i) => {
            println!("{:?}", biomes::RANDOM_PATTERN_MAP_OBJECTS[i as usize]);
        }
        biomes::MapObject::SimplexPattern(i) => {
            println!("{:?}", biomes::SIMPLEX_PATTERN_MAP_OBJECTS[i as usize]);
        }
        biomes::MapObject::SimplexSmoothedPattern(i) => {
            println!(
                "{:?}",
                biomes::SIMPLEX_SMOOTHED_PATTERN_MAP_OBJECTS[i as usize]
            );
        }
        biomes::MapObject::None => {}
    }

    println!("{}", test_index);

    storage
}

pub fn update(
    storage: &mut Storage,
    mut vertices: BufferWriteGuard<'_, [vertex_data::VertexData]>,
    mut indices: BufferWriteGuard<'_, [u16]>,
    index_count: &mut u32,
    scale: f32,
    delta_time: f32,
    average_fps: f32,
    camera: &mut Camera,
) {
    //println!("delta time: {}", delta_time);
    //println!("average fps: {}", average_fps);
    vertices[0].position[1] += storage.direction * delta_time;
    vertices[0].uv[1] = (vertices[0].position[1] + 1.0) / 2.0;

    if vertices[0].position[1] < -1.0 {
        storage.direction = 1.0;
    } else if vertices[0].position[1] > 1.0 {
        storage.direction = -1.0;
    }

    //println!("{}",scale);

    let bottom_left_world_position_of_screen = ((camera.position.0-1.0).floor() as i32, (-1.0/scale).floor() as i32);

    for i in 0..10 {
        let i_position = position_from_index(i, screen_width);
        let full_position = (i_position.0 + bottom_left_world_position_of_screen.0, i_position.1 + bottom_left_world_position_of_screen.1);
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
    map_objects: [biomes::MapObject; FULL_GRID_WIDTH_SQUARED as usize],
}

// Block generation:

pub fn generate_position(storage: &mut Storage, position: (u32, u32)) -> biomes::MapObject {
    let position_as_float_array = [position.0 as f64, position.1 as f64];

    let biome = &biomes::BIOMES[biomes::get_biome(
        storage.biome_noise_x.get(position_as_float_array),
        storage.biome_noise_y.get(position_as_float_array),
    )];

    let mut map_object = biomes::MapObject::None;
    let mut highest_priority = 0u8;

    for i in biome.random_pattern.starting_index..biome.random_pattern.length {
        let random_pattern_map_object = &biomes::RANDOM_PATTERN_MAP_OBJECTS[i as usize];
        if random_pattern_map_object.priority > highest_priority
            && storage.percent_range.sample(&mut storage.rng) < random_pattern_map_object.chance
        {
            map_object = biomes::MapObject::RandomPattern(i);
            highest_priority = random_pattern_map_object.priority
        }
    }

    for i in biome.simplex_pattern.starting_index..biome.simplex_pattern.length {
        let simplex_pattern_map_object = &biomes::SIMPLEX_PATTERN_MAP_OBJECTS[i as usize];
        let simplex_noise =
            OpenSimplex::new(storage.main_seed + simplex_pattern_map_object.seed as u32)
                .get(position_as_float_array);
        if simplex_pattern_map_object.priority > highest_priority
            && storage.percent_range.sample(&mut storage.rng) < simplex_pattern_map_object.chance
            && simplex_noise > simplex_pattern_map_object.acceptable_noise.0
            && simplex_noise < simplex_pattern_map_object.acceptable_noise.1
        {
            map_object = biomes::MapObject::SimplexPattern(i);
            highest_priority = simplex_pattern_map_object.priority
        }
    }

    for i in biome.simplex_smoothed_pattern.starting_index..biome.simplex_smoothed_pattern.length {
        let simplex_smoothed_pattern_map_object =
            &biomes::SIMPLEX_SMOOTHED_PATTERN_MAP_OBJECTS[i as usize];
        let simplex_noise =
            OpenSimplex::new(storage.main_seed + simplex_smoothed_pattern_map_object.seed as u32)
                .get(position_as_float_array);
        if simplex_smoothed_pattern_map_object.priority > highest_priority
            && storage.percent_range.sample(&mut storage.rng)
                < simplex_smoothed_pattern_map_object.chance
            && simplex_noise > simplex_smoothed_pattern_map_object.acceptable_noise.0
            && simplex_noise < simplex_smoothed_pattern_map_object.acceptable_noise.1
        {
            map_object = biomes::MapObject::SimplexSmoothedPattern(i);
            highest_priority = simplex_smoothed_pattern_map_object.priority
        }
    }

    map_object
}

fn full_index_from_full_position(full_position: (u32, u32)) -> usize {
    let chunk_position = (
        full_position.0 >> CHUNK_WIDTH_LOG2,
        full_position.1 >> CHUNK_WIDTH_LOG2,
    );
    let chunk_index = index_from_position(chunk_position, CHUNK_GRID_WIDTH);
    let full_index_start = chunk_index * CHUNK_WIDTH_SQUARED as u32;

    let local_position = (
        full_position.0 % CHUNK_WIDTH as u32,
        full_position.1 % CHUNK_WIDTH as u32,
    );
    let local_index = index_from_position(local_position, CHUNK_WIDTH as u32);

    (full_index_start + local_index) as usize
}

fn index_from_position<T>(position: (T, T), width: T) -> T
where
    T: Mul<T, Output = T> + Add<T, Output = T>,
{
    position.1 * width + position.0
}

fn position_from_index<T>(index: T, width: T) -> (T,T)
where
    T: Rem<T, Output = T> + Div<T, Output = T>,
{
    (index % width, index / width)
}

pub struct Camera {
    pub scale: f32,
    pub position: (f32,f32),
}