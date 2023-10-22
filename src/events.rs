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
use winit::event::ElementState;
use winit::event::KeyboardInput;
use winit::event::VirtualKeyCode;

use crate::biomes;
use crate::vertex_data;

const TEXT_SPRITE_SIZE: (f32, f32) = (1.0 / 68.0, 1.0);

const FULL_GRID_WIDTH: u32 = CHUNK_WIDTH as u32 * 100;
const FULL_GRID_WIDTH_SQUARED: u32 = FULL_GRID_WIDTH * FULL_GRID_WIDTH; // 256**2

const CHUNK_WIDTH: u16 = 128;
pub const CHUNK_WIDTH_SQUARED: u16 = CHUNK_WIDTH * CHUNK_WIDTH;
//const CHUNK_WIDTH_LOG2: u16 = (u16::BITS - CHUNK_WIDTH.leading_zeros()) as u16;

const CHUNK_GRID_WIDTH: u32 = FULL_GRID_WIDTH / CHUNK_WIDTH as u32;

pub fn start(render_storage: &mut RenderStorage) -> UserStorage {
    render_storage.camera.scale = 0.12;

    render_storage.brightness = 2.5;

    let mut rng = thread_rng();
    let seed_range = Uniform::new(0u32, 1000);

    let (generation_sender, generation_receiver) = mpsc::channel();

    let available_parallelism = thread::available_parallelism().unwrap().get();

    let user_storage = UserStorage {
        wasd_held: (false, false, false, false),
        zoom_held: (false, false),
        main_seed: seed_range.sample(&mut rng),
        percent_range: Uniform::new(0u8, 100),
        biome_noise: (
            OpenSimplex::new(seed_range.sample(&mut rng)),
            OpenSimplex::new(seed_range.sample(&mut rng)),
        ),
        map_objects: vec![biomes::MapObject::None; FULL_GRID_WIDTH_SQUARED as usize],
        generation_sender,
        generation_receiver,
        available_parallelism,
        map_objects_per_thread: CHUNK_WIDTH_SQUARED as usize / available_parallelism,
        player: Player {
            position: (10.0, 10.0),
            previous_position: (5.0, 5.0),
            sprinting: false,
            collision_debug: false,
            size: (0.8, 0.8),
            strength: 1,
        },
    };

    render_storage.camera.position = user_storage.player.position;

    let check = user_storage.map_objects_per_thread * available_parallelism;

    println!("Available Parallelism: {}, Assumed generation per thread: {}, Check: {}, Correct Version: {}", available_parallelism, user_storage.map_objects_per_thread, check, CHUNK_WIDTH_SQUARED);

    assert!(check == CHUNK_WIDTH_SQUARED as usize);

    generate_chunk(&user_storage, (0, 0));

    user_storage
}

pub fn update(
    user_storage: &mut UserStorage,
    render_storage: &mut RenderStorage,
    //mut vertices: BufferWriteGuard<'_, [vertex_data::VertexData]>,
    //mut indices: BufferWriteGuard<'_, [u32]>,
    //index_count: &mut u32,
    //scale: f32,
    delta_time: f32,
    average_fps: f32,
    //camera: &mut Camera,
    //brightness: &mut f32,
) {
    //println!("delta time: {}", delta_time);
    //println!("average fps: {}", average_fps);

    let zoom_motion = match user_storage.zoom_held {
        (true, false) => -1.0,
        (false, true) => 1.0,
        _ => 0.0,
    };

    let motion = match user_storage.wasd_held {
        (true, false, false, false) => (0.0, -1.0),
        (false, false, true, false) => (0.0, 1.0),
        (false, false, false, true) => (1.0, 0.0),
        (false, true, false, false) => (-1.0, 0.0),
        _ => (0.0, 0.0),
    };

    let speed = match user_storage.player.sprinting {
        false => 10.0,
        true => 50.0,
    };

    user_storage.player.previous_position = user_storage.player.position;

    user_storage.player.position.0 += motion.0 * delta_time * speed;
    user_storage.player.position.1 += motion.1 * delta_time * speed;

    render_storage.camera.scale += zoom_motion * delta_time * (speed / 100.0);

    if !user_storage.player.collision_debug {
        collision_middle_top(user_storage);
        collision_right_top(user_storage);
        collision_left_top(user_storage);

        collision_middle_bottom(user_storage);
        collision_right_bottom(user_storage);
        collision_left_bottom(user_storage);

        collision_right_middle(user_storage);
        collision_left_middle(user_storage);

        //collision_middle_middle(user_storage); Doesn't work yet!
    }

    if user_storage.player.position.0 < 0.0 {
        user_storage.player.position.0 = 0.0;
    } else if user_storage.player.position.0 > FULL_GRID_WIDTH as f32 {
        user_storage.player.position.0 = FULL_GRID_WIDTH as f32;
    }
    if user_storage.player.position.1 < 0.0 {
        user_storage.player.position.1 = 0.0;
    } else if user_storage.player.position.1 > FULL_GRID_WIDTH as f32 {
        user_storage.player.position.1 = FULL_GRID_WIDTH as f32;
    }

    render_storage.camera.position = user_storage.player.position;

    render_map(user_storage, render_storage);
    render_player(user_storage, render_storage);

    render_storage.vertex_count_text = 0;
    render_storage.index_count_text = 0;

    let screen_width = 2.0 / render_storage.aspect_ratio;

    draw_text(
        render_storage,
        (screen_width * -0.5 + screen_width * 0.1, -0.7),
        (0.05, 0.1),
        0.025,
        format!("@ Strength: {}", user_storage.player.strength).as_str(),
    );
}

pub fn late_update(user_storage: &mut UserStorage, delta_time: f32, average_fps: f32) {
    match user_storage.generation_receiver.try_recv() {
        Ok(generation) => {
            user_storage.map_objects[generation.1..generation.1 + generation.0.len()]
                .copy_from_slice(generation.0.as_slice());
        }
        Err(TryRecvError::Empty) => {}
        Err(TryRecvError::Disconnected) => {
            panic!("Something got disconnected from the chunk receivers and senders!")
        }
    }

    //generate_chunk(user_storage, user_storage.player.position/)
}

pub fn on_keyboard_input(user_storage: &mut UserStorage, input: KeyboardInput) {
    if let Some(key_code) = input.virtual_keycode {
        match key_code {
            VirtualKeyCode::W => user_storage.wasd_held.0 = is_pressed(input.state),
            VirtualKeyCode::A => user_storage.wasd_held.1 = is_pressed(input.state),
            VirtualKeyCode::S => user_storage.wasd_held.2 = is_pressed(input.state),
            VirtualKeyCode::D => user_storage.wasd_held.3 = is_pressed(input.state),
            VirtualKeyCode::F => {
                if is_pressed(input.state) {
                    user_storage.player.sprinting = !user_storage.player.sprinting;
                }
            }
            VirtualKeyCode::R => {
                if is_pressed(input.state) {
                    generate_chunk(
                        &user_storage,
                        (
                            (user_storage.player.position.0 / CHUNK_WIDTH as f32).floor() as u32,
                            (user_storage.player.position.1 / CHUNK_WIDTH as f32).floor() as u32,
                        ),
                    );
                }
            }
            VirtualKeyCode::E => {
                if is_pressed(input.state) {
                    user_storage.player.collision_debug = !user_storage.player.collision_debug;
                }
            }
            VirtualKeyCode::Up => user_storage.zoom_held.0 = is_pressed(input.state),
            VirtualKeyCode::Down => user_storage.zoom_held.1 = is_pressed(input.state),
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

pub struct UserStorage {
    // This is for the user's stuff. The event loop should not touch this.
    wasd_held: (bool, bool, bool, bool),
    zoom_held: (bool, bool),
    main_seed: u32,
    percent_range: Uniform<u8>,
    biome_noise: (OpenSimplex, OpenSimplex),
    map_objects: Vec<biomes::MapObject>,
    generation_sender: Sender<(Vec<biomes::MapObject>, usize)>,
    generation_receiver: Receiver<(Vec<biomes::MapObject>, usize)>,
    available_parallelism: usize,
    map_objects_per_thread: usize,
    player: Player,
}

pub struct RenderStorage {
    // TODO: Perhaps removing or refining what belongs in this struct.
    pub vertices_map: Vec<vertex_data::VertexData>,
    pub vertex_count_map: u32,
    pub indices_map: Vec<u32>,
    pub index_count_map: u32,

    pub vertices_text: Vec<vertex_data::VertexData>,
    pub vertex_count_text: u32,
    pub indices_text: Vec<u32>,
    pub index_count_text: u32,

    pub aspect_ratio: f32,
    pub camera: Camera,
    pub brightness: f32,
    pub frame_count: u32, // This will crash the game after 2 years, assuming 60 fps.
}

fn generate_chunk(user_storage: &UserStorage, chunk_position: (u32, u32)) {
    let biome_noise = user_storage.biome_noise;
    let percent_range = user_storage.percent_range;
    let main_seed = user_storage.main_seed;

    let full_position_start = (
        chunk_position.0 * CHUNK_WIDTH as u32,
        chunk_position.1 * CHUNK_WIDTH as u32,
    );

    let full_index_start = full_index_from_full_position(full_position_start);

    let map_objects_per_thread = user_storage.map_objects_per_thread;

    for t in 0..user_storage.available_parallelism {
        let generation_sender = user_storage.generation_sender.clone();
        thread::Builder::new()
            .name("Generation Thread".into())
            .spawn(move || {
                let mut generation_array = vec![biomes::MapObject::None; map_objects_per_thread];
                for i in 0..map_objects_per_thread {
                    let local_position = position_from_index(
                        (i + (t * map_objects_per_thread)) as u32,
                        CHUNK_WIDTH as u32,
                    );
                    let full_position = (
                        full_position_start.0 + local_position.0,
                        full_position_start.1 + local_position.1,
                    );
                    generation_array[i] = generate_position(
                        full_position,
                        &mut thread_rng(),
                        biome_noise,
                        percent_range,
                        main_seed,
                    )
                }

                generation_sender.send((
                    generation_array,
                    full_index_start + (t * map_objects_per_thread),
                ))
            })
            .unwrap();
    }
}

fn generate_position(
    position: (u32, u32),
    mut rng: &mut ThreadRng,
    biome_noise: (OpenSimplex, OpenSimplex),
    percent_range: Uniform<u8>,
    main_seed: u32,
) -> biomes::MapObject {
    let position_as_float_array = [position.0 as f64, position.1 as f64];

    let biome_position = [
        position_as_float_array[0] * biomes::BIOME_SCALE.0,
        position_as_float_array[1] * biomes::BIOME_SCALE.1,
    ];
    let biome = &biomes::BIOMES[biomes::get_biome((
        (biome_noise.0.get(biome_position) + 1.0) * 0.5,
        (biome_noise.1.get(biome_position) + 1.0) * 0.5,
    ))];

    let mut map_object = biomes::MapObject::None;
    let mut highest_priority = 0u8;

    for i in biome.random_pattern.starting_index
        ..biome.random_pattern.starting_index + biome.random_pattern.length
    {
        let random_pattern_map_object = &biomes::RANDOM_PATTERN_MAP_OBJECTS[i as usize];
        if random_pattern_map_object.priority > highest_priority
            && percent_range.sample(&mut rng) < random_pattern_map_object.chance
        {
            map_object = biomes::MapObject::RandomPattern(i);
            highest_priority = random_pattern_map_object.priority
        }
    }

    for i in biome.simplex_pattern.starting_index
        ..biome.simplex_pattern.starting_index + biome.simplex_pattern.length
    {
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

    for i in biome.simplex_smoothed_pattern.starting_index
        ..biome.simplex_smoothed_pattern.starting_index + biome.simplex_smoothed_pattern.length
    {
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

fn render_map(user_storage: &mut UserStorage, render_storage: &mut RenderStorage) {
    let screen_width_as_world_units =
        2.0 / render_storage.camera.scale / render_storage.aspect_ratio;
    let screen_height_as_world_units = 2.0 / render_storage.camera.scale;

    let mut simple_rendering_count = 0u32;

    for x in (render_storage.camera.position.0 - (screen_width_as_world_units * 0.5)).floor() as i32
        - 1
        ..(render_storage.camera.position.0 + (screen_width_as_world_units * 0.5)).ceil() as i32 + 1
    {
        for y in (render_storage.camera.position.1 - (screen_height_as_world_units * 0.5)).floor()
            as i32
            - 1
            ..(render_storage.camera.position.1 + (screen_height_as_world_units * 0.5)).ceil()
                as i32
                + 1
        {
            if x < 0 || y < 0 {
                continue;
            }

            let full_index = full_index_from_full_position((x as u32, y as u32));

            if full_index >= FULL_GRID_WIDTH_SQUARED as usize {
                println!("What the hell?");
                continue;
            }

            let map_object = user_storage.map_objects[full_index];

            match map_object {
                biomes::MapObject::RandomPattern(i) => {
                    let random_pattern_map_object = &biomes::RANDOM_PATTERN_MAP_OBJECTS[i as usize];
                    // println!(
                    //     "Position:({},{}), Map Object:{:?}",
                    //     x, y, random_pattern_map_object,
                    // );

                    let vertex_start = (simple_rendering_count * 4) as usize;
                    let index_start = (simple_rendering_count * 6) as usize;

                    render_storage.vertices_map[vertex_start] = vertex_data::VertexData {
                        // top right
                        position: [
                            x as f32 + (0.5 * random_pattern_map_object.rendering_size.0),
                            y as f32 + (0.5 * random_pattern_map_object.rendering_size.1),
                        ],
                        uv: [
                            random_pattern_map_object.uv.0 + biomes::SPRITE_SIZE.0,
                            random_pattern_map_object.uv.1 + biomes::SPRITE_SIZE.1,
                        ],
                    };

                    render_storage.vertices_map[vertex_start + 1] = vertex_data::VertexData {
                        // bottom right
                        position: [
                            x as f32 + (0.5 * random_pattern_map_object.rendering_size.0),
                            y as f32 + (-0.5 * random_pattern_map_object.rendering_size.1),
                        ],
                        uv: [
                            random_pattern_map_object.uv.0 + biomes::SPRITE_SIZE.0,
                            random_pattern_map_object.uv.1,
                        ],
                    };

                    render_storage.vertices_map[vertex_start + 2] = vertex_data::VertexData {
                        // top left
                        position: [
                            x as f32 + (-0.5 * random_pattern_map_object.rendering_size.0),
                            y as f32 + (0.5 * random_pattern_map_object.rendering_size.1),
                        ],
                        uv: [
                            random_pattern_map_object.uv.0,
                            random_pattern_map_object.uv.1 + biomes::SPRITE_SIZE.1,
                        ],
                    };

                    render_storage.vertices_map[vertex_start + 3] = vertex_data::VertexData {
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

                    render_storage.indices_map[index_start] = vertex_start as u32;
                    render_storage.indices_map[index_start + 1] = vertex_start as u32 + 1;
                    render_storage.indices_map[index_start + 2] = vertex_start as u32 + 2;

                    render_storage.indices_map[index_start + 3] = vertex_start as u32 + 1;
                    render_storage.indices_map[index_start + 4] = vertex_start as u32 + 3;
                    render_storage.indices_map[index_start + 5] = vertex_start as u32 + 2;

                    simple_rendering_count += 1;
                }
                biomes::MapObject::SimplexPattern(i) => {
                    let simplex_pattern_map_object =
                        &biomes::SIMPLEX_PATTERN_MAP_OBJECTS[i as usize];

                    let vertex_start = (simple_rendering_count * 4) as usize;
                    let index_start = (simple_rendering_count * 6) as usize;

                    render_storage.vertices_map[vertex_start] = vertex_data::VertexData {
                        // top right
                        position: [
                            x as f32 + (0.5 * simplex_pattern_map_object.rendering_size.0),
                            y as f32 + (0.5 * simplex_pattern_map_object.rendering_size.1),
                        ],
                        uv: [
                            simplex_pattern_map_object.uv.0 + biomes::SPRITE_SIZE.0,
                            simplex_pattern_map_object.uv.1 + biomes::SPRITE_SIZE.1,
                        ],
                    };

                    render_storage.vertices_map[vertex_start + 1] = vertex_data::VertexData {
                        // bottom right
                        position: [
                            x as f32 + (0.5 * simplex_pattern_map_object.rendering_size.0),
                            y as f32 + (-0.5 * simplex_pattern_map_object.rendering_size.1),
                        ],
                        uv: [
                            simplex_pattern_map_object.uv.0 + biomes::SPRITE_SIZE.0,
                            simplex_pattern_map_object.uv.1,
                        ],
                    };

                    render_storage.vertices_map[vertex_start + 2] = vertex_data::VertexData {
                        // top left
                        position: [
                            x as f32 + (-0.5 * simplex_pattern_map_object.rendering_size.0),
                            y as f32 + (0.5 * simplex_pattern_map_object.rendering_size.1),
                        ],
                        uv: [
                            simplex_pattern_map_object.uv.0,
                            simplex_pattern_map_object.uv.1 + biomes::SPRITE_SIZE.1,
                        ],
                    };

                    render_storage.vertices_map[vertex_start + 3] = vertex_data::VertexData {
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

                    render_storage.indices_map[index_start] = vertex_start as u32;
                    render_storage.indices_map[index_start + 1] = vertex_start as u32 + 1;
                    render_storage.indices_map[index_start + 2] = vertex_start as u32 + 2;

                    render_storage.indices_map[index_start + 3] = vertex_start as u32 + 1;
                    render_storage.indices_map[index_start + 4] = vertex_start as u32 + 3;
                    render_storage.indices_map[index_start + 5] = vertex_start as u32 + 2;

                    simple_rendering_count += 1;
                }
                biomes::MapObject::SimplexSmoothedPattern(_) => {
                    todo!();
                }
                biomes::MapObject::None => {}
            }
        }
    }

    render_storage.vertex_count_map = simple_rendering_count * 4;
    render_storage.index_count_map = simple_rendering_count * 6;
}

fn render_player(user_storage: &mut UserStorage, render_storage: &mut RenderStorage) {
    let vertex_start = render_storage.vertex_count_map as usize;
    let index_start = render_storage.index_count_map as usize;

    render_storage.vertices_map[vertex_start] = vertex_data::VertexData {
        // top right
        position: [
            user_storage.player.position.0 + user_storage.player.size.0 * 0.5,
            user_storage.player.position.1 + user_storage.player.size.1 * 0.5,
        ],
        uv: [biomes::SPRITE_SIZE.0, biomes::SPRITE_SIZE.1],
    };

    render_storage.vertices_map[vertex_start + 1] = vertex_data::VertexData {
        // bottom right
        position: [
            user_storage.player.position.0 + user_storage.player.size.0 * 0.5,
            user_storage.player.position.1 - user_storage.player.size.1 * 0.5,
        ],
        uv: [biomes::SPRITE_SIZE.0, 0.0],
    };

    render_storage.vertices_map[vertex_start + 2] = vertex_data::VertexData {
        // top left
        position: [
            user_storage.player.position.0 - user_storage.player.size.0 * 0.5,
            user_storage.player.position.1 + user_storage.player.size.1 * 0.5,
        ],
        uv: [0.0, biomes::SPRITE_SIZE.1],
    };

    render_storage.vertices_map[vertex_start + 3] = vertex_data::VertexData {
        // bottom left
        position: [
            user_storage.player.position.0 - user_storage.player.size.0 * 0.5,
            user_storage.player.position.1 - user_storage.player.size.1 * 0.5,
        ],
        uv: [0.0, 0.0],
    };

    render_storage.indices_map[index_start] = vertex_start as u32;
    render_storage.indices_map[index_start + 1] = vertex_start as u32 + 1;
    render_storage.indices_map[index_start + 2] = vertex_start as u32 + 2;

    render_storage.indices_map[index_start + 3] = vertex_start as u32 + 1;
    render_storage.indices_map[index_start + 4] = vertex_start as u32 + 3;
    render_storage.indices_map[index_start + 5] = vertex_start as u32 + 2;

    render_storage.vertex_count_map += 4;
    render_storage.index_count_map += 6;
}

fn collision_middle_top(user_storage: &mut UserStorage) {
    let top_position = user_storage.player.position.1.round() + 1.0;

    let full_position = (
        user_storage.player.position.0.round() as u32,
        top_position as u32,
    );

    let map_object = user_storage.map_objects[full_index_from_full_position(full_position)];

    match map_object {
        biomes::MapObject::RandomPattern(i) => {
            let random_pattern_map_object = &biomes::RANDOM_PATTERN_MAP_OBJECTS[i as usize];

            let bottom_of_top = top_position - (random_pattern_map_object.collision_size.1 * 0.5);

            if user_storage.player.position.1 + 0.5 * user_storage.player.size.1 > bottom_of_top {
                deal_with_collision(
                    user_storage,
                    (
                        user_storage.player.position.0,
                        bottom_of_top - 0.5 * user_storage.player.size.1,
                    ),
                    full_position,
                )
            }
        }
        biomes::MapObject::SimplexPattern(i) => {
            let simplex_pattern_map_object = &biomes::SIMPLEX_PATTERN_MAP_OBJECTS[i as usize];

            let bottom_of_top = top_position - (simplex_pattern_map_object.collision_size.1 * 0.5);

            if user_storage.player.position.1 + 0.5 * user_storage.player.size.1 > bottom_of_top {
                deal_with_collision(
                    user_storage,
                    (
                        user_storage.player.position.0,
                        bottom_of_top - 0.5 * user_storage.player.size.1,
                    ),
                    full_position,
                )
            }
        }
        biomes::MapObject::SimplexSmoothedPattern(i) => {
            todo!();
        }
        biomes::MapObject::None => {}
    }
}

fn collision_right_top(user_storage: &mut UserStorage) {
    let right_position = user_storage.player.position.0.round() + 1.0;
    let top_position = user_storage.player.position.1.round() + 1.0;

    let full_position = (right_position as u32, top_position as u32);

    let map_object = user_storage.map_objects[full_index_from_full_position(full_position)];

    match map_object {
        biomes::MapObject::RandomPattern(i) => {
            let random_pattern_map_object = &biomes::RANDOM_PATTERN_MAP_OBJECTS[i as usize];

            let left_of_right = right_position - (random_pattern_map_object.collision_size.0 * 0.5);
            let bottom_of_top = top_position - (random_pattern_map_object.collision_size.1 * 0.5);

            if user_storage.player.position.0 + 0.5 * user_storage.player.size.0 > left_of_right
                && user_storage.player.position.1 + 0.5 * user_storage.player.size.1 > bottom_of_top
            {
                deal_with_collision(
                    user_storage,
                    user_storage.player.previous_position,
                    full_position,
                )
            }
        }
        biomes::MapObject::SimplexPattern(i) => {
            let simplex_pattern_map_object = &biomes::SIMPLEX_PATTERN_MAP_OBJECTS[i as usize];

            let left_of_right =
                right_position - (simplex_pattern_map_object.collision_size.0 * 0.5);
            let bottom_of_top = top_position - (simplex_pattern_map_object.collision_size.1 * 0.5);

            if user_storage.player.position.0 + 0.5 * user_storage.player.size.0 > left_of_right
                && user_storage.player.position.1 + 0.5 * user_storage.player.size.1 > bottom_of_top
            {
                deal_with_collision(
                    user_storage,
                    user_storage.player.previous_position,
                    full_position,
                )
            }
        }
        biomes::MapObject::SimplexSmoothedPattern(i) => {
            todo!();
        }
        biomes::MapObject::None => {}
    }
}

fn collision_left_top(user_storage: &mut UserStorage) {
    let left_position = user_storage.player.position.0.round() - 1.0;
    let top_position = user_storage.player.position.1.round() + 1.0;

    let full_position = (left_position as u32, top_position as u32);

    let map_object = user_storage.map_objects[full_index_from_full_position(full_position)];

    match map_object {
        biomes::MapObject::RandomPattern(i) => {
            let random_pattern_map_object = &biomes::RANDOM_PATTERN_MAP_OBJECTS[i as usize];

            let right_of_left = left_position + (random_pattern_map_object.collision_size.0 * 0.5);
            let bottom_of_top = top_position - (random_pattern_map_object.collision_size.1 * 0.5);

            if user_storage.player.position.0 - 0.5 * user_storage.player.size.0 < right_of_left
                && user_storage.player.position.1 + 0.5 * user_storage.player.size.1 > bottom_of_top
            {
                deal_with_collision(
                    user_storage,
                    user_storage.player.previous_position,
                    full_position,
                )
            }
        }
        biomes::MapObject::SimplexPattern(i) => {
            let simplex_pattern_map_object = &biomes::SIMPLEX_PATTERN_MAP_OBJECTS[i as usize];

            let right_of_left = left_position + (simplex_pattern_map_object.collision_size.0 * 0.5);
            let bottom_of_top = top_position - (simplex_pattern_map_object.collision_size.1 * 0.5);

            if user_storage.player.position.0 - 0.5 * user_storage.player.size.0 < right_of_left
                && user_storage.player.position.1 + 0.5 * user_storage.player.size.1 > bottom_of_top
            {
                deal_with_collision(
                    user_storage,
                    user_storage.player.previous_position,
                    full_position,
                )
            }
        }
        biomes::MapObject::SimplexSmoothedPattern(i) => {
            todo!();
        }
        biomes::MapObject::None => {}
    }
}

fn collision_middle_bottom(user_storage: &mut UserStorage) {
    let bottom_position = user_storage.player.position.1.round() - 1.0;

    let full_position = (
        user_storage.player.position.0.round() as u32,
        bottom_position as u32,
    );

    let map_object = user_storage.map_objects[full_index_from_full_position(full_position)];

    match map_object {
        biomes::MapObject::RandomPattern(i) => {
            let random_pattern_map_object = &biomes::RANDOM_PATTERN_MAP_OBJECTS[i as usize];

            let top_of_bottom =
                bottom_position + (random_pattern_map_object.collision_size.1 * 0.5);

            if user_storage.player.position.1 - 0.5 * user_storage.player.size.1 < top_of_bottom {
                deal_with_collision(
                    user_storage,
                    (
                        user_storage.player.position.0,
                        top_of_bottom + 0.5 * user_storage.player.size.1,
                    ),
                    full_position,
                )
            }
        }
        biomes::MapObject::SimplexPattern(i) => {
            let simplex_pattern_map_object = &biomes::SIMPLEX_PATTERN_MAP_OBJECTS[i as usize];

            let top_of_bottom =
                bottom_position + (simplex_pattern_map_object.collision_size.1 * 0.5);

            if user_storage.player.position.1 - 0.5 * user_storage.player.size.1 < top_of_bottom {
                deal_with_collision(
                    user_storage,
                    (
                        user_storage.player.position.0,
                        top_of_bottom + 0.5 * user_storage.player.size.1,
                    ),
                    full_position,
                )
            }
        }
        biomes::MapObject::SimplexSmoothedPattern(i) => {
            todo!();
        }
        biomes::MapObject::None => {}
    }
}

fn collision_right_bottom(user_storage: &mut UserStorage) {
    let right_position = user_storage.player.position.0.round() + 1.0;
    let bottom_position = user_storage.player.position.1.round() - 1.0;

    let full_position = (right_position as u32, bottom_position as u32);

    let map_object = user_storage.map_objects[full_index_from_full_position(full_position)];

    match map_object {
        biomes::MapObject::RandomPattern(i) => {
            let random_pattern_map_object = &biomes::RANDOM_PATTERN_MAP_OBJECTS[i as usize];

            let left_of_right = right_position - (random_pattern_map_object.collision_size.0 * 0.5);
            let top_of_bottom =
                bottom_position + (random_pattern_map_object.collision_size.1 * 0.5);

            if user_storage.player.position.0 + 0.5 * user_storage.player.size.0 > left_of_right
                && user_storage.player.position.1 - 0.5 * user_storage.player.size.1 < top_of_bottom
            {
                deal_with_collision(
                    user_storage,
                    user_storage.player.previous_position,
                    full_position,
                )
            }
        }
        biomes::MapObject::SimplexPattern(i) => {
            let simplex_pattern_map_object = &biomes::SIMPLEX_PATTERN_MAP_OBJECTS[i as usize];

            let left_of_right =
                right_position - (simplex_pattern_map_object.collision_size.0 * 0.5);
            let top_of_bottom =
                bottom_position + (simplex_pattern_map_object.collision_size.1 * 0.5);

            if user_storage.player.position.0 + 0.5 * user_storage.player.size.0 > left_of_right
                && user_storage.player.position.1 - 0.5 * user_storage.player.size.1 < top_of_bottom
            {
                deal_with_collision(
                    user_storage,
                    user_storage.player.previous_position,
                    full_position,
                )
            }
        }
        biomes::MapObject::SimplexSmoothedPattern(i) => {
            todo!();
        }
        biomes::MapObject::None => {}
    }
}

fn collision_left_bottom(user_storage: &mut UserStorage) {
    let left_position = user_storage.player.position.0.round() - 1.0;
    let bottom_position = user_storage.player.position.1.round() - 1.0;

    let full_position = (left_position as u32, bottom_position as u32);

    let map_object = user_storage.map_objects[full_index_from_full_position(full_position)];

    match map_object {
        biomes::MapObject::RandomPattern(i) => {
            let random_pattern_map_object = &biomes::RANDOM_PATTERN_MAP_OBJECTS[i as usize];

            let right_of_left = left_position + (random_pattern_map_object.collision_size.0 * 0.5);
            let top_of_bottom =
                bottom_position + (random_pattern_map_object.collision_size.1 * 0.5);

            if user_storage.player.position.0 - 0.5 * user_storage.player.size.0 < right_of_left
                && user_storage.player.position.1 - 0.5 * user_storage.player.size.1 < top_of_bottom
            {
                deal_with_collision(
                    user_storage,
                    user_storage.player.previous_position,
                    full_position,
                )
            }
        }
        biomes::MapObject::SimplexPattern(i) => {
            let simplex_pattern_map_object = &biomes::SIMPLEX_PATTERN_MAP_OBJECTS[i as usize];

            let right_of_left = left_position + (simplex_pattern_map_object.collision_size.0 * 0.5);
            let top_of_bottom =
                bottom_position + (simplex_pattern_map_object.collision_size.1 * 0.5);

            if user_storage.player.position.0 - 0.5 * user_storage.player.size.0 < right_of_left
                && user_storage.player.position.1 - 0.5 * user_storage.player.size.1 < top_of_bottom
            {
                deal_with_collision(
                    user_storage,
                    user_storage.player.previous_position,
                    full_position,
                )
            }
        }
        biomes::MapObject::SimplexSmoothedPattern(i) => {
            todo!();
        }
        biomes::MapObject::None => {}
    }
}

fn collision_right_middle(user_storage: &mut UserStorage) {
    let right_position = user_storage.player.position.0.round() + 1.0;

    let full_position = (
        right_position as u32,
        user_storage.player.position.1.round() as u32,
    );

    let map_object = user_storage.map_objects[full_index_from_full_position(full_position)];

    match map_object {
        biomes::MapObject::RandomPattern(i) => {
            let random_pattern_map_object = &biomes::RANDOM_PATTERN_MAP_OBJECTS[i as usize];

            let left_of_right = right_position - (random_pattern_map_object.collision_size.0 * 0.5);

            if user_storage.player.position.0 + 0.5 * user_storage.player.size.0 > left_of_right {
                deal_with_collision(
                    user_storage,
                    (
                        left_of_right - 0.5 * user_storage.player.size.0,
                        user_storage.player.position.1,
                    ),
                    full_position,
                )
            }
        }
        biomes::MapObject::SimplexPattern(i) => {
            let simplex_pattern_map_object = &biomes::SIMPLEX_PATTERN_MAP_OBJECTS[i as usize];

            let left_of_right =
                right_position - (simplex_pattern_map_object.collision_size.0 * 0.5);

            if user_storage.player.position.0 + 0.5 * user_storage.player.size.0 > left_of_right {
                deal_with_collision(
                    user_storage,
                    (
                        left_of_right - 0.5 * user_storage.player.size.0,
                        user_storage.player.position.1,
                    ),
                    full_position,
                )
            }
        }
        biomes::MapObject::SimplexSmoothedPattern(i) => {
            todo!();
        }
        biomes::MapObject::None => {}
    }
}

fn collision_left_middle(user_storage: &mut UserStorage) {
    let left_position = user_storage.player.position.0.round() - 1.0;

    let full_position = (
        left_position as u32,
        user_storage.player.position.1.round() as u32,
    );

    let map_object = user_storage.map_objects[full_index_from_full_position(full_position)];

    match map_object {
        biomes::MapObject::RandomPattern(i) => {
            let random_pattern_map_object = &biomes::RANDOM_PATTERN_MAP_OBJECTS[i as usize];

            let right_of_left = left_position + (random_pattern_map_object.collision_size.0 * 0.5);

            if user_storage.player.position.0 - 0.5 * user_storage.player.size.0 < right_of_left {
                deal_with_collision(
                    user_storage,
                    (
                        right_of_left + 0.5 * user_storage.player.size.0,
                        user_storage.player.position.1,
                    ),
                    full_position,
                )
            }
        }
        biomes::MapObject::SimplexPattern(i) => {
            let simplex_pattern_map_object = &biomes::SIMPLEX_PATTERN_MAP_OBJECTS[i as usize];

            let right_of_left = left_position + (simplex_pattern_map_object.collision_size.0 * 0.5);

            if user_storage.player.position.0 - 0.5 * user_storage.player.size.0 < right_of_left {
                deal_with_collision(
                    user_storage,
                    (
                        right_of_left + 0.5 * user_storage.player.size.0,
                        user_storage.player.position.1,
                    ),
                    full_position,
                )
            }
        }
        biomes::MapObject::SimplexSmoothedPattern(i) => {
            todo!();
        }
        biomes::MapObject::None => {}
    }
}

fn collision_middle_middle(user_storage: &mut UserStorage) {
    let full_position = (
        user_storage.player.position.0 as u32,
        user_storage.player.position.1 as u32,
    );
    let map_object = user_storage.map_objects[full_index_from_full_position(full_position)];

    match map_object {
        biomes::MapObject::RandomPattern(i) => {
            todo!();
        }
        biomes::MapObject::SimplexPattern(i) => {
            todo!();
        }
        biomes::MapObject::SimplexSmoothedPattern(i) => {
            todo!();
        }
        biomes::MapObject::None => {}
    }
}
struct Player {
    position: (f32, f32),
    previous_position: (f32, f32),
    sprinting: bool,
    collision_debug: bool,
    size: (f32, f32),
    strength: u8,
}

fn deal_with_collision(
    user_storage: &mut UserStorage,
    fallback_position: (f32, f32),
    full_position: (u32, u32),
) {
    let map_object = &mut user_storage.map_objects[full_index_from_full_position(full_position)];

    match map_object {
        biomes::MapObject::RandomPattern(i) => {
            let random_pattern_map_object = &biomes::RANDOM_PATTERN_MAP_OBJECTS[*i as usize];

            match random_pattern_map_object.behaviour {
                biomes::CollisionBehaviour::None => {
                    user_storage.player.position = fallback_position; // Should none have collision or not?
                }
                biomes::CollisionBehaviour::Consume(strength) => {
                    if user_storage.player.strength > strength {
                        *map_object = biomes::MapObject::None;
                    } else {
                        user_storage.player.position = fallback_position;
                    }
                }
                biomes::CollisionBehaviour::Replace(strength, replacement_map_object) => {
                    if user_storage.player.strength > strength {
                        *map_object = replacement_map_object;
                    } else {
                        user_storage.player.position = fallback_position;
                    }
                }
            }
        }
        biomes::MapObject::SimplexPattern(i) => {
            let simplex_pattern_map_object = &biomes::SIMPLEX_PATTERN_MAP_OBJECTS[*i as usize];

            match simplex_pattern_map_object.behaviour {
                biomes::CollisionBehaviour::None => {
                    user_storage.player.position = fallback_position;
                }
                biomes::CollisionBehaviour::Consume(strength) => {
                    if user_storage.player.strength > strength {
                        *map_object = biomes::MapObject::None;
                    } else {
                        user_storage.player.position = fallback_position;
                    }
                }
                biomes::CollisionBehaviour::Replace(strength, replacement_map_object) => {
                    if user_storage.player.strength > strength {
                        *map_object = replacement_map_object;
                    } else {
                        user_storage.player.position = fallback_position;
                    }
                }
            }
        }
        biomes::MapObject::SimplexSmoothedPattern(i) => {
            todo!();
        }
        biomes::MapObject::None => {}
    }
}

fn draw_text(
    render_storage: &mut RenderStorage,
    mut position: (f32, f32),
    character_size: (f32, f32),
    letter_spacing: f32,
    text: &str,
) {
    for character in text.chars() {
        let uv = match character {
            '0' => (0.0, 0.0f32),
            '1' => (TEXT_SPRITE_SIZE.0 * 1.0, 0.0),
            '2' => (TEXT_SPRITE_SIZE.0 * 2.0, 0.0),
            '3' => (TEXT_SPRITE_SIZE.0 * 3.0, 0.0),
            '4' => (TEXT_SPRITE_SIZE.0 * 4.0, 0.0),
            '5' => (TEXT_SPRITE_SIZE.0 * 5.0, 0.0),
            '6' => (TEXT_SPRITE_SIZE.0 * 6.0, 0.0),
            '7' => (TEXT_SPRITE_SIZE.0 * 7.0, 0.0),
            '8' => (TEXT_SPRITE_SIZE.0 * 8.0, 0.0),
            '9' => (TEXT_SPRITE_SIZE.0 * 9.0, 0.0),
            'A' => (TEXT_SPRITE_SIZE.0 * 10.0, 0.0),
            'B' => (TEXT_SPRITE_SIZE.0 * 11.0, 0.0),
            'C' => (TEXT_SPRITE_SIZE.0 * 12.0, 0.0),
            'D' => (TEXT_SPRITE_SIZE.0 * 13.0, 0.0),
            'E' => (TEXT_SPRITE_SIZE.0 * 14.0, 0.0),
            'F' => (TEXT_SPRITE_SIZE.0 * 15.0, 0.0),
            'G' => (TEXT_SPRITE_SIZE.0 * 16.0, 0.0),
            'H' => (TEXT_SPRITE_SIZE.0 * 17.0, 0.0),
            'I' => (TEXT_SPRITE_SIZE.0 * 18.0, 0.0),
            'J' => (TEXT_SPRITE_SIZE.0 * 19.0, 0.0),
            'K' => (TEXT_SPRITE_SIZE.0 * 20.0, 0.0),
            'L' => (TEXT_SPRITE_SIZE.0 * 21.0, 0.0),
            'M' => (TEXT_SPRITE_SIZE.0 * 22.0, 0.0),
            'N' => (TEXT_SPRITE_SIZE.0 * 23.0, 0.0),
            'O' => (TEXT_SPRITE_SIZE.0 * 24.0, 0.0),
            'P' => (TEXT_SPRITE_SIZE.0 * 25.0, 0.0),
            'Q' => (TEXT_SPRITE_SIZE.0 * 26.0, 0.0),
            'R' => (TEXT_SPRITE_SIZE.0 * 27.0, 0.0),
            'S' => (TEXT_SPRITE_SIZE.0 * 28.0, 0.0),
            'T' => (TEXT_SPRITE_SIZE.0 * 29.0, 0.0),
            'U' => (TEXT_SPRITE_SIZE.0 * 30.0, 0.0),
            'V' => (TEXT_SPRITE_SIZE.0 * 31.0, 0.0),
            'W' => (TEXT_SPRITE_SIZE.0 * 32.0, 0.0),
            'X' => (TEXT_SPRITE_SIZE.0 * 33.0, 0.0),
            'Y' => (TEXT_SPRITE_SIZE.0 * 34.0, 0.0),
            'Z' => (TEXT_SPRITE_SIZE.0 * 35.0, 0.0),

            'a' => (TEXT_SPRITE_SIZE.0 * 36.0, 0.0),
            'b' => (TEXT_SPRITE_SIZE.0 * 37.0, 0.0),
            'c' => (TEXT_SPRITE_SIZE.0 * 38.0, 0.0),
            'd' => (TEXT_SPRITE_SIZE.0 * 39.0, 0.0),
            'e' => (TEXT_SPRITE_SIZE.0 * 40.0, 0.0),
            'f' => (TEXT_SPRITE_SIZE.0 * 41.0, 0.0),
            'g' => (TEXT_SPRITE_SIZE.0 * 42.0, 0.0),
            'h' => (TEXT_SPRITE_SIZE.0 * 43.0, 0.0),
            'i' => (TEXT_SPRITE_SIZE.0 * 44.0, 0.0),
            'j' => (TEXT_SPRITE_SIZE.0 * 45.0, 0.0),
            'k' => (TEXT_SPRITE_SIZE.0 * 46.0, 0.0),
            'l' => (TEXT_SPRITE_SIZE.0 * 47.0, 0.0),
            'm' => (TEXT_SPRITE_SIZE.0 * 48.0, 0.0),
            'n' => (TEXT_SPRITE_SIZE.0 * 49.0, 0.0),
            'o' => (TEXT_SPRITE_SIZE.0 * 50.0, 0.0),
            'p' => (TEXT_SPRITE_SIZE.0 * 51.0, 0.0),
            'q' => (TEXT_SPRITE_SIZE.0 * 52.0, 0.0),
            'r' => (TEXT_SPRITE_SIZE.0 * 53.0, 0.0),
            's' => (TEXT_SPRITE_SIZE.0 * 54.0, 0.0),
            't' => (TEXT_SPRITE_SIZE.0 * 55.0, 0.0),
            'u' => (TEXT_SPRITE_SIZE.0 * 56.0, 0.0),
            'v' => (TEXT_SPRITE_SIZE.0 * 57.0, 0.0),
            'w' => (TEXT_SPRITE_SIZE.0 * 58.0, 0.0),
            'x' => (TEXT_SPRITE_SIZE.0 * 59.0, 0.0),
            'y' => (TEXT_SPRITE_SIZE.0 * 60.0, 0.0),
            'z' => (TEXT_SPRITE_SIZE.0 * 61.0, 0.0),
            ' ' => (TEXT_SPRITE_SIZE.0 * 62.0, 0.0),
            ':' => (TEXT_SPRITE_SIZE.0 * 64.0, 0.0),
            '@' => (TEXT_SPRITE_SIZE.0 * 65.0, 0.0),
            _ => (TEXT_SPRITE_SIZE.0 * 63.0, 0.0),
        };

        let vertex_start = render_storage.vertex_count_text as usize;
        let index_start = render_storage.index_count_text as usize;

        render_storage.vertices_text[vertex_start] = vertex_data::VertexData {
            // top right
            position: [
                position.0 + character_size.0 * 0.5,
                position.1 + character_size.1 * 0.5,
            ],
            uv: [uv.0 + TEXT_SPRITE_SIZE.0, uv.1 + TEXT_SPRITE_SIZE.1],
        };

        render_storage.vertices_text[vertex_start + 1] = vertex_data::VertexData {
            // bottom right
            position: [
                position.0 + character_size.0 * 0.5,
                position.1 - character_size.1 * 0.5,
            ],
            uv: [uv.0 + TEXT_SPRITE_SIZE.0, uv.1],
        };

        render_storage.vertices_text[vertex_start + 2] = vertex_data::VertexData {
            // top left
            position: [
                position.0 - character_size.0 * 0.5,
                position.1 + character_size.1 * 0.5,
            ],
            uv: [uv.0, uv.1 + TEXT_SPRITE_SIZE.1],
        };

        render_storage.vertices_text[vertex_start + 3] = vertex_data::VertexData {
            // bottom left
            position: [
                position.0 - character_size.0 * 0.5,
                position.1 - character_size.1 * 0.5,
            ],
            uv: [uv.0, uv.1],
        };

        render_storage.indices_text[index_start] = vertex_start as u32;
        render_storage.indices_text[index_start + 1] = vertex_start as u32 + 1;
        render_storage.indices_text[index_start + 2] = vertex_start as u32 + 2;

        render_storage.indices_text[index_start + 3] = vertex_start as u32 + 1;
        render_storage.indices_text[index_start + 4] = vertex_start as u32 + 3;
        render_storage.indices_text[index_start + 5] = vertex_start as u32 + 2;

        render_storage.vertex_count_text += 4;
        render_storage.index_count_text += 6;

        position.0 += letter_spacing;
    }
}
