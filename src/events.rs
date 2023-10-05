use noise::NoiseFn;
use noise::OpenSimplex;
use rand::distributions::{Distribution, Uniform};
use rand::rngs::ThreadRng;
use rand::thread_rng;
use std::ops::{Add, Div, Mul, Rem};
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::mpsc::TryRecvError;
use std::thread;
use vulkano::buffer::subbuffer::BufferWriteGuard;
use winit::event::ElementState;
use winit::event::KeyboardInput;
use winit::event::VirtualKeyCode;

use crate::biomes;
use crate::vertex_data;

const SPRITE_SIZE: (f32, f32) = (1.0, 1.0);

const FULL_GRID_WIDTH: u32 = 256;
const FULL_GRID_WIDTH_SQUARED: u32 = FULL_GRID_WIDTH * FULL_GRID_WIDTH; // 256**2

const CHUNK_WIDTH: u16 = 128;
pub const CHUNK_WIDTH_SQUARED: u16 = CHUNK_WIDTH * CHUNK_WIDTH;
//const CHUNK_WIDTH_LOG2: u16 = (u16::BITS - CHUNK_WIDTH.leading_zeros()) as u16;

const CHUNK_GRID_WIDTH: u32 = FULL_GRID_WIDTH / CHUNK_WIDTH as u32;

pub const STARTING_INDEX_COUNT: u32 = 0;

pub fn start(camera: &mut Camera) -> Storage {
    camera.scale = 0.05;
    camera.position = (0.0, 0.0);

    let mut rng = thread_rng();
    let seed_range = Uniform::new(0u32, 1000);

    let (chunk_sender, chunk_receiver) = mpsc::channel();

    let storage = Storage {
        wasd_held: (false, false, false, false),
        main_seed: seed_range.sample(&mut rng),
        percent_range: Uniform::new(0u8, 100),
        biome_noise: (
            OpenSimplex::new(seed_range.sample(&mut rng)),
            OpenSimplex::new(seed_range.sample(&mut rng)),
        ),
        map_objects: [biomes::MapObject::None; FULL_GRID_WIDTH_SQUARED as usize],
        chunk_sender,
        chunk_receiver,
    };

    generate_chunk(&storage, (0, 0));
    //generate_chunk(&storage, (1,0));

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

    let motion = match storage.wasd_held {
        (true, false, false, false) => (0.0, -1.0),
        (false, false, true, false) => (0.0, 1.0),
        (false, false, false, true) => (1.0, 0.0),
        (false, true, false, false) => (-1.0, 0.0),
        _ => (0.0, 0.0),
    };

    camera.position.0 += motion.0 * delta_time * 10.0;
    camera.position.1 += motion.1 * delta_time * 10.0;

    //println!("scale:{}",scale);
    //println!("camera position:{:?}",camera.position);

    let screen_width_as_world_units = 2.0 / camera.scale;
    let screen_height_as_world_units = 2.0 / camera.scale / scale;

    //println!("screen width:{}, screen height:{}",screen_width_as_world_units,screen_height_as_world_units);

    // let bottom_left_world_position_of_screen = (
    //     (camera.position.0 - (screen_width_as_world_units * 0.5)).floor() as i32,
    //     (camera.position.1 - (screen_height_as_world_units * 0.5)).floor() as i32,
    // );
    // let top_right_world_position_of_screen = (
    //     (camera.position.0 + (screen_width_as_world_units * 0.5)).floor() as i32,
    //     (camera.position.1 + (screen_height_as_world_units * 0.5)).floor() as i32,
    // );

    //println!("bottom left: {:?}, top right: {:?}",bottom_left_world_position_of_screen,top_right_world_position_of_screen);

    let mut simple_rendering_count = 0u32;

    for x in (camera.position.0 - (screen_width_as_world_units * 0.5)).floor() as i32 - 1
        ..(camera.position.0 + (screen_width_as_world_units * 0.5)).ceil() as i32 + 1
    {
        for y in (camera.position.1 - (screen_height_as_world_units * 0.5)).floor() as i32 - 1
            ..(camera.position.1 + (screen_height_as_world_units * 0.5)).ceil() as i32 + 1
        {
            if x < 0 || y < 0 {
                continue;
            }

            let map_object =
                storage.map_objects[full_index_from_full_position((x as u32, y as u32))];

            match map_object {
                biomes::MapObject::RandomPattern(i) => {
                    let random_pattern_map_object = &biomes::RANDOM_PATTERN_MAP_OBJECTS[i as usize];
                    // println!(
                    //     "Position:({},{}), Map Object:{:?}",
                    //     x, y, random_pattern_map_object,
                    // );

                    let vertex_start = (simple_rendering_count * 4) as usize;
                    let index_start = (simple_rendering_count * 6) as usize;

                    vertices[vertex_start] = vertex_data::VertexData {
                        // top right
                        position: [
                            x as f32 + (0.5 * random_pattern_map_object.rendering_size.0),
                            y as f32 + (0.5 * random_pattern_map_object.rendering_size.1),
                        ],
                        uv: [
                            random_pattern_map_object.uv.0 + SPRITE_SIZE.0,
                            random_pattern_map_object.uv.1 + SPRITE_SIZE.1,
                        ],
                    };

                    vertices[vertex_start + 1] = vertex_data::VertexData {
                        // bottom right
                        position: [
                            x as f32 + (0.5 * random_pattern_map_object.rendering_size.0),
                            y as f32 + (-0.5 * random_pattern_map_object.rendering_size.1),
                        ],
                        uv: [
                            random_pattern_map_object.uv.0 + SPRITE_SIZE.0,
                            random_pattern_map_object.uv.1,
                        ],
                    };

                    vertices[vertex_start + 2] = vertex_data::VertexData {
                        // top left
                        position: [
                            x as f32 + (-0.5 * random_pattern_map_object.rendering_size.0),
                            y as f32 + (0.5 * random_pattern_map_object.rendering_size.1),
                        ],
                        uv: [
                            random_pattern_map_object.uv.0,
                            random_pattern_map_object.uv.1 + SPRITE_SIZE.1,
                        ],
                    };

                    vertices[vertex_start + 3] = vertex_data::VertexData {
                        // bottom left
                        position: [
                            x as f32 + (-0.5 * random_pattern_map_object.rendering_size.0),
                            y as f32 + (-0.5 * random_pattern_map_object.rendering_size.1),
                        ],
                        uv: [
                            random_pattern_map_object.uv.0,
                            random_pattern_map_object.uv.1,
                        ],
                    };

                    indices[index_start] = vertex_start as u16;
                    indices[index_start + 1] = vertex_start as u16 + 1;
                    indices[index_start + 2] = vertex_start as u16 + 2;

                    indices[index_start + 3] = vertex_start as u16 + 1;
                    indices[index_start + 4] = vertex_start as u16 + 3;
                    indices[index_start + 5] = vertex_start as u16 + 2;

                    simple_rendering_count += 1;
                }
                biomes::MapObject::SimplexPattern(i) => {
                    let simplex_pattern_map_object =
                        &biomes::SIMPLEX_PATTERN_MAP_OBJECTS[i as usize];

                    let vertex_start = (simple_rendering_count * 4) as usize;
                    let index_start = (simple_rendering_count * 6) as usize;

                    vertices[vertex_start] = vertex_data::VertexData {
                        // top right
                        position: [
                            x as f32 + (0.5 * simplex_pattern_map_object.rendering_size.0),
                            y as f32 + (0.5 * simplex_pattern_map_object.rendering_size.1),
                        ],
                        uv: [
                            simplex_pattern_map_object.uv.0 + SPRITE_SIZE.0,
                            simplex_pattern_map_object.uv.1 + SPRITE_SIZE.1,
                        ],
                    };

                    vertices[vertex_start + 1] = vertex_data::VertexData {
                        // bottom right
                        position: [
                            x as f32 + (0.5 * simplex_pattern_map_object.rendering_size.0),
                            y as f32 + (-0.5 * simplex_pattern_map_object.rendering_size.1),
                        ],
                        uv: [
                            simplex_pattern_map_object.uv.0 + SPRITE_SIZE.0,
                            simplex_pattern_map_object.uv.1,
                        ],
                    };

                    vertices[vertex_start + 2] = vertex_data::VertexData {
                        // top left
                        position: [
                            x as f32 + (-0.5 * simplex_pattern_map_object.rendering_size.0),
                            y as f32 + (0.5 * simplex_pattern_map_object.rendering_size.1),
                        ],
                        uv: [
                            simplex_pattern_map_object.uv.0,
                            simplex_pattern_map_object.uv.1 + SPRITE_SIZE.1,
                        ],
                    };

                    vertices[vertex_start + 3] = vertex_data::VertexData {
                        // bottom left
                        position: [
                            x as f32 + (-0.5 * simplex_pattern_map_object.rendering_size.0),
                            y as f32 + (-0.5 * simplex_pattern_map_object.rendering_size.1),
                        ],
                        uv: [
                            simplex_pattern_map_object.uv.0,
                            simplex_pattern_map_object.uv.1,
                        ],
                    };

                    indices[index_start] = vertex_start as u16;
                    indices[index_start + 1] = vertex_start as u16 + 1;
                    indices[index_start + 2] = vertex_start as u16 + 2;

                    indices[index_start + 3] = vertex_start as u16 + 1;
                    indices[index_start + 4] = vertex_start as u16 + 3;
                    indices[index_start + 5] = vertex_start as u16 + 2;

                    simple_rendering_count += 1;
                }
                biomes::MapObject::SimplexSmoothedPattern(_) => {
                    todo!();
                }
                biomes::MapObject::None => {}
            }
        }
    }

    *index_count = simple_rendering_count * 6;
}

pub fn late_update(storage: &mut Storage, delta_time: f32, average_fps: f32) {
    match storage.chunk_receiver.try_recv() {
        Ok(chunk) => {
            println!("Got chunk!");
            let starting_index = full_index_from_full_position(chunk.1);
            storage.map_objects[starting_index..starting_index + CHUNK_WIDTH_SQUARED as usize]
                .copy_from_slice(&chunk.0);
        }
        Err(TryRecvError::Empty) => {}
        Err(TryRecvError::Disconnected) => {
            panic!("Something got disconnected from the chunk receivers and senders!")
        }
    }
}

pub fn on_keyboard_input(storage: &mut Storage, input: KeyboardInput) {
    if let Some(key_code) = input.virtual_keycode {
        match key_code {
            VirtualKeyCode::W => storage.wasd_held.0 = is_pressed(input.state),
            VirtualKeyCode::A => storage.wasd_held.1 = is_pressed(input.state),
            VirtualKeyCode::S => storage.wasd_held.2 = is_pressed(input.state),
            VirtualKeyCode::D => storage.wasd_held.3 = is_pressed(input.state),
            _ => (),
        }
    }
}

fn is_pressed(state: ElementState) -> bool {
    match state {
        ElementState::Pressed => true,
        ElementState::Released => false,
    }
}

pub struct Storage {
    // This is for the user's stuff. The event loop should not touch this.
    wasd_held: (bool, bool, bool, bool),
    main_seed: u32,
    percent_range: Uniform<u8>,
    biome_noise: (OpenSimplex, OpenSimplex),
    map_objects: [biomes::MapObject; FULL_GRID_WIDTH_SQUARED as usize],
    chunk_sender: Sender<(
        [biomes::MapObject; CHUNK_WIDTH_SQUARED as usize],
        (u32, u32),
    )>,
    chunk_receiver: Receiver<(
        [biomes::MapObject; CHUNK_WIDTH_SQUARED as usize],
        (u32, u32),
    )>,
}

// Block generation:

fn generate_chunk(storage: &Storage, chunk_position: (u32, u32)) {
    let chunk_sender = storage.chunk_sender.clone();

    let biome_noise = storage.biome_noise;
    let percent_range = storage.percent_range;
    let main_seed = storage.main_seed;

    thread::Builder::new()
        .stack_size(CHUNK_WIDTH_SQUARED as usize * 1000)
        .name("chunk generation thread".into())
        .spawn(move || {
            let full_pos_start = (
                chunk_position.0 * CHUNK_WIDTH as u32,
                chunk_position.1 * CHUNK_WIDTH as u32,
            );
            let mut rng = thread_rng();
            let mut chunk_array = [biomes::MapObject::None; CHUNK_WIDTH_SQUARED as usize];

            for x in 0..CHUNK_WIDTH as u32 {
                for y in 0..CHUNK_WIDTH as u32 {
                    let full_pos = (full_pos_start.0 + x, full_pos_start.1 + y);
                    chunk_array[index_from_position(full_pos, CHUNK_WIDTH as u32) as usize] =
                        generate_position(
                            full_pos,
                            &mut rng,
                            biome_noise,
                            percent_range,
                            main_seed,
                        );
                }
            }

            chunk_sender.send((chunk_array, full_pos_start))
        })
        .unwrap();
}

fn generate_position(
    position: (u32, u32),
    mut rng: &mut ThreadRng,
    biome_noise: (OpenSimplex, OpenSimplex),
    percent_range: Uniform<u8>,
    main_seed: u32,
) -> biomes::MapObject {
    let position_as_float_array = [position.0 as f64, position.1 as f64];

    let biome = &biomes::BIOMES[biomes::get_biome(
        biome_noise.0.get(position_as_float_array),
        biome_noise.1.get(position_as_float_array),
    )];

    let mut map_object = biomes::MapObject::None;
    let mut highest_priority = 0u8;

    for i in biome.random_pattern.starting_index..biome.random_pattern.length {
        let random_pattern_map_object = &biomes::RANDOM_PATTERN_MAP_OBJECTS[i as usize];
        if random_pattern_map_object.priority > highest_priority
            && percent_range.sample(&mut rng) < random_pattern_map_object.chance
        {
            map_object = biomes::MapObject::RandomPattern(i);
            highest_priority = random_pattern_map_object.priority
        }
    }

    for i in biome.simplex_pattern.starting_index..biome.simplex_pattern.length {
        let simplex_pattern_map_object = &biomes::SIMPLEX_PATTERN_MAP_OBJECTS[i as usize];
        let simplex_noise = OpenSimplex::new(main_seed + simplex_pattern_map_object.seed as u32)
            .get([
                position_as_float_array[0] * simplex_pattern_map_object.noise_scale,
                position_as_float_array[1] * simplex_pattern_map_object.noise_scale,
            ]);
        if simplex_pattern_map_object.priority > highest_priority
            && percent_range.sample(&mut rng) < simplex_pattern_map_object.chance
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
            OpenSimplex::new(main_seed + simplex_smoothed_pattern_map_object.seed as u32).get([
                position_as_float_array[0] * simplex_smoothed_pattern_map_object.noise_scale,
                position_as_float_array[1] * simplex_smoothed_pattern_map_object.noise_scale,
            ]);
        if simplex_smoothed_pattern_map_object.priority > highest_priority
            && percent_range.sample(&mut rng) < simplex_smoothed_pattern_map_object.chance
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
        full_position.0 / CHUNK_WIDTH as u32,
        full_position.1 / CHUNK_WIDTH as u32,
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

fn position_from_index<T>(index: T, width: T) -> (T, T)
where
    T: Rem<T, Output = T> + Div<T, Output = T> + Copy,
{
    (index % width, index / width)
}

pub struct Camera {
    pub scale: f32,
    pub position: (f32, f32),
}
