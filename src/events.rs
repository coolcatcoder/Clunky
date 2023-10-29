use noise::NoiseFn;
use noise::OpenSimplex;
use rand::distributions::Bernoulli;
use rand::distributions::{Distribution, Uniform};
use rand::rngs::ThreadRng;
use rand::thread_rng;
use std::ops::{Add, Div, Mul, Rem};
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::mpsc::TryRecvError;
use std::thread;
use std::time::Instant;
use winit::event::ElementState;
use winit::event::KeyboardInput;
use winit::event::VirtualKeyCode;

use crate::biomes;
use crate::vertex_data;

const TEXT_SPRITE_SIZE: (f32, f32) = (1.0 / 30.0, 1.0 / 5.0);

const FULL_GRID_WIDTH: u32 = CHUNK_WIDTH as u32 * 100;
const FULL_GRID_WIDTH_SQUARED: u32 = FULL_GRID_WIDTH * FULL_GRID_WIDTH; // 256**2

const CHUNK_WIDTH: u16 = 128;
pub const CHUNK_WIDTH_SQUARED: u16 = CHUNK_WIDTH * CHUNK_WIDTH;
//const CHUNK_WIDTH_LOG2: u16 = (u16::BITS - CHUNK_WIDTH.leading_zeros()) as u16;

const CHUNK_GRID_WIDTH: u32 = FULL_GRID_WIDTH / CHUNK_WIDTH as u32;
const CHUNK_GRID_WIDTH_SQUARED: u32 = CHUNK_GRID_WIDTH * CHUNK_GRID_WIDTH;

const FIXED_UPDATE_TIME_STEP: f32 = 0.003;
const MAX_SUBSTEPS: u32 = 150;

pub fn start(render_storage: &mut RenderStorage) -> UserStorage {
    render_storage.camera.scale = 0.12;

    render_storage.brightness = 2.5;

    let mut rng = thread_rng();

    let seed_range = Uniform::new(0u32, 1000);
    let player_size_range = Uniform::new(0.25, 10.0);

    let mut player_size = (0.7, 0.7);

    if Bernoulli::new(0.1).unwrap().sample(&mut rng) {
        player_size = (
            player_size_range.sample(&mut rng),
            player_size_range.sample(&mut rng),
        )
    }

    let (generation_sender, generation_receiver) = mpsc::channel();

    let available_parallelism = thread::available_parallelism().unwrap().get();

    let mut user_storage = UserStorage {
        wasd_held: (false, false, false, false),
        zoom_held: (false, false),
        show_debug: false,
        main_seed: seed_range.sample(&mut rng),
        percent_range: Uniform::new(0u8, 100),
        biome_noise: (
            OpenSimplex::new(seed_range.sample(&mut rng)),
            OpenSimplex::new(seed_range.sample(&mut rng)),
        ),
        chunks_generated: vec![false; CHUNK_GRID_WIDTH_SQUARED as usize],
        details: [
            Detail {
                scale: 1,
                offset: (0.0, 0.0),
            },
            Detail {
                scale: 2,
                offset: (-0.25, -0.25),
            },
        ],
        map_objects: [
            vec![biomes::MapObject::None; FULL_GRID_WIDTH_SQUARED as usize],
            vec![biomes::MapObject::None; FULL_GRID_WIDTH_SQUARED as usize * 4],
        ],
        generation_sender,
        generation_receiver,
        available_parallelism,
        map_objects_per_thread: CHUNK_WIDTH_SQUARED as usize / available_parallelism,
        player: Player {
            position: (10.0, 10.0),
            previous_position: (5.0, 5.0),
            sprinting: false,
            collision_debug: false,
            size: player_size,
            statistics: biomes::Statistics {
                strength: 1,
                health: 30,
                stamina: 100,
            },
        },
        stop_watch: Instant::now(),
        fixed_time_passed: 0.0,
    };

    render_storage.camera.position = user_storage.player.position;

    let check = user_storage.map_objects_per_thread * available_parallelism;

    println!("Available Parallelism: {}, Assumed generation per thread: {}, Check: {}, Correct Version: {}", available_parallelism, user_storage.map_objects_per_thread, check, CHUNK_WIDTH_SQUARED);

    assert!(check == CHUNK_WIDTH_SQUARED as usize);

    let safe_position = get_safe_position(&mut user_storage);

    user_storage.player.position = (safe_position.0 as f32, safe_position.1 as f32);

    let starting_chunk = (
        safe_position.0 / CHUNK_WIDTH as u32,
        safe_position.1 / CHUNK_WIDTH as u32,
    );

    generate_chunk(&user_storage, starting_chunk);

    user_storage.chunks_generated[index_from_position(starting_chunk, CHUNK_GRID_WIDTH) as usize] =
        true;

    //let pain = transform_biomes![TESTING_BIOME, TESTING_BIOME];

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
    // TODO: work out how the hell to get a fixed update working...

    let seconds_since_start = render_storage.starting_time.elapsed().as_secs_f32();

    let mut substeps = 0;

    while user_storage.fixed_time_passed < seconds_since_start {
        fixed_update(user_storage);
        user_storage.fixed_time_passed += FIXED_UPDATE_TIME_STEP;

        substeps += 1;

        if substeps > MAX_SUBSTEPS {
            println!(
                "Too many substeps per frame. Entered performance sinkhole. Substeps: {}",
                substeps
            )
        }
    }

    let zoom_motion = match user_storage.zoom_held {
        (true, false) => -1.0,
        (false, true) => 1.0,
        _ => 0.0,
    };

    let camera_speed = match user_storage.player.sprinting {
        // bit jank, but it works
        false => 0.01,
        true => 0.1,
    };

    render_storage.camera.scale += zoom_motion * delta_time * (camera_speed);

    render_storage.camera.position = user_storage.player.position;

    render_storage.vertex_count_map = 0;
    render_storage.index_count_map = 0;

    for detail_index in 0..user_storage.details.len() {
        render_map(user_storage, render_storage, detail_index as u8);
    }

    render_player(user_storage, render_storage);

    render_storage.vertex_count_text = 0;
    render_storage.index_count_text = 0;

    let screen_width = 2.0 / render_storage.aspect_ratio;

    draw_text(
        render_storage,
        (screen_width * -0.5 + screen_width * 0.1, -0.8),
        (0.05, 0.1),
        0.025,
        format!("Health: {}", user_storage.player.statistics.health).as_str(),
    );

    draw_text(
        render_storage,
        (screen_width * -0.5 + screen_width * 0.1, -0.7),
        (0.05, 0.1),
        0.025,
        format!("Stamina: {}%", user_storage.player.statistics.stamina).as_str(),
    );

    draw_text(
        render_storage,
        (screen_width * -0.5 + screen_width * 0.1, -0.6),
        (0.05, 0.1),
        0.025,
        format!("@ Strength: {}", user_storage.player.statistics.strength).as_str(),
    );

    if user_storage.show_debug {
        draw_text(
            render_storage,
            (screen_width * -0.5 + screen_width * 0.1, 0.4),
            (0.05, 0.1),
            0.025,
            format!("substeps: {}", substeps).as_str(),
        );

        draw_text(
            render_storage,
            (screen_width * -0.5 + screen_width * 0.1, 0.5),
            (0.05, 0.1),
            0.025,
            format!(
                "player.position: ({},{})",
                user_storage.player.position.0, user_storage.player.position.1
            )
            .as_str(),
        );

        draw_text(
            render_storage,
            (screen_width * -0.5 + screen_width * 0.1, 0.6),
            (0.05, 0.1),
            0.025,
            format!(
                "player.size: ({},{})",
                user_storage.player.size.0, user_storage.player.size.1
            )
            .as_str(),
        );

        draw_text(
            render_storage,
            (screen_width * -0.5 + screen_width * 0.1, 0.7),
            (0.05, 0.1),
            0.025,
            format!("average_fps: {}", average_fps).as_str(),
        );

        draw_text(
            render_storage,
            (screen_width * -0.5 + screen_width * 0.1, 0.8),
            (0.05, 0.1),
            0.025,
            format!("delta_time: {}", delta_time).as_str(),
        );
    }

    match user_storage.generation_receiver.try_recv() {
        Ok(generation) => {
            user_storage.map_objects[generation.2 as usize]
                [generation.1..generation.1 + generation.0.len()]
                .copy_from_slice(generation.0.as_slice());
        }
        Err(TryRecvError::Empty) => {}
        Err(TryRecvError::Disconnected) => {
            panic!("Something got disconnected from the chunk receivers and senders!")
        }
    }

    for x in -1..2 {
        for y in -1..2 {
            let player_chunk_position = (
                (user_storage.player.position.0 as i32 / CHUNK_WIDTH as i32 + x) as u32,
                (user_storage.player.position.1 as i32 / CHUNK_WIDTH as i32 + y) as u32,
            );
            let player_chunk_index =
                index_from_position(player_chunk_position, CHUNK_GRID_WIDTH) as usize;

            if !user_storage.chunks_generated[player_chunk_index] {
                generate_chunk(&user_storage, player_chunk_position);
                user_storage.chunks_generated[player_chunk_index] = true;
            }
        }
    }
}

pub fn fixed_update(user_storage: &mut UserStorage) {
    let motion = match user_storage.wasd_held {
        (true, false, false, false) => (0.0, -1.0),
        (false, false, true, false) => (0.0, 1.0),
        (false, false, false, true) => (1.0, 0.0),
        (false, true, false, false) => (-1.0, 0.0),
        _ => (0.0, 0.0),
    };

    let speed = match user_storage.player.sprinting {
        false => 5.0,
        true => 10.0,
    };

    user_storage.player.previous_position = user_storage.player.position;

    user_storage.player.position.0 += motion.0 * FIXED_UPDATE_TIME_STEP * speed;
    user_storage.player.position.1 += motion.1 * FIXED_UPDATE_TIME_STEP * speed;

    if !user_storage.player.collision_debug {
        for detail_index in 0..user_storage.details.len() {
            let detail = user_storage.details[detail_index];

            let rounded_player_position_scaled = (
                (user_storage.player.position.0 * detail.scale as f32).round() as i32,
                (user_storage.player.position.1 * detail.scale as f32).round() as i32,
            );

            let ceil_player_half_size_scaled = (
                (user_storage.player.size.0 * 0.5 * detail.scale as f32).ceil() as i32,
                (user_storage.player.size.1 * 0.5 * detail.scale as f32).ceil() as i32,
            );

            for x in -ceil_player_half_size_scaled.0..ceil_player_half_size_scaled.0 + 1 {
                for y in -ceil_player_half_size_scaled.1..ceil_player_half_size_scaled.1 + 1 {
                    collide(
                        user_storage,
                        (
                            (rounded_player_position_scaled.0 + x) as u32,
                            (rounded_player_position_scaled.1 + y) as u32,
                        ),
                        detail_index as u8,
                    )
                }
            }
        }
    }

    user_storage.player.statistics.stamina = user_storage.player.statistics.stamina.min(100);

    if user_storage.stop_watch.elapsed().as_secs_f32() >= 0.25 {
        user_storage.stop_watch = Instant::now();

        user_storage.player.statistics.stamina -= 1;

        if user_storage.player.statistics.stamina < 0 {
            user_storage.player.statistics.health -= 1;
        }
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

            VirtualKeyCode::V => {
                if is_pressed(input.state) {
                    user_storage.show_debug = !user_storage.show_debug;
                }
            }
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

#[derive(Debug, Copy, Clone)]
struct Detail {
    scale: u32, // This is unintuitive. Basically how many of these blocks become 1 block.
    offset: (f32, f32),
}

pub struct UserStorage {
    // This is for the user's stuff. The event loop should not touch this.
    wasd_held: (bool, bool, bool, bool),
    zoom_held: (bool, bool),
    show_debug: bool,
    main_seed: u32,
    percent_range: Uniform<u8>,
    biome_noise: (OpenSimplex, OpenSimplex),
    chunks_generated: Vec<bool>,
    details: [Detail; 2],
    map_objects: [Vec<biomes::MapObject>; 2],
    generation_sender: Sender<(Vec<biomes::MapObject>, usize, u8)>,
    generation_receiver: Receiver<(Vec<biomes::MapObject>, usize, u8)>,
    available_parallelism: usize,
    map_objects_per_thread: usize,
    player: Player,
    stop_watch: Instant,
    fixed_time_passed: f32,
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
    pub starting_time: Instant,
}

fn generate_chunk(user_storage: &UserStorage, chunk_position: (u32, u32)) {
    let biome_noise = user_storage.biome_noise;
    let percent_range = user_storage.percent_range;
    let main_seed = user_storage.main_seed;

    let details = user_storage.details;

    let full_position_start_unscaled = (
        chunk_position.0 * CHUNK_WIDTH as u32,
        chunk_position.1 * CHUNK_WIDTH as u32,
    );

    let map_objects_per_thread = user_storage.map_objects_per_thread;

    for t in 0..user_storage.available_parallelism {
        let generation_sender = user_storage.generation_sender.clone();
        thread::Builder::new()
            .name("Generation Thread".into())
            .spawn(move || {
                for detail_index in 0..details.len() {
                    let detail = details[detail_index];
                    let mut generation_array = vec![
                        biomes::MapObject::None;
                        map_objects_per_thread
                            * (detail.scale * detail.scale) as usize
                    ];

                    let full_position_start = (
                        full_position_start_unscaled.0 * detail.scale as u32,
                        full_position_start_unscaled.1 * detail.scale as u32,
                    );

                    for i in 0..map_objects_per_thread * (detail.scale * detail.scale) as usize {
                        let local_position = position_from_index(
                            (i + (t
                                * map_objects_per_thread
                                * (detail.scale * detail.scale) as usize))
                                as u32,
                            CHUNK_WIDTH as u32 * detail.scale as u32,
                        );
                        let full_position = (
                            full_position_start.0 + local_position.0,
                            full_position_start.1 + local_position.1,
                        );

                        generation_array[i] = generate_position(
                            full_position,
                            detail_index as u8,
                            detail.scale,
                            detail.offset,
                            &mut thread_rng(),
                            biome_noise,
                            percent_range,
                            main_seed,
                        );
                    }

                    let full_index_start =
                        full_index_from_full_position(full_position_start, detail.scale as u32);

                    generation_sender
                        .send((
                            generation_array,
                            full_index_start
                                + (t * map_objects_per_thread
                                    * (detail.scale * detail.scale) as usize),
                            detail_index as u8,
                        ))
                        .unwrap()
                }
            })
            .unwrap();
    }
}

fn generate_position(
    position: (u32, u32),
    detail: u8,
    scale: u32,
    offset: (f32, f32),
    mut rng: &mut ThreadRng,
    biome_noise: (OpenSimplex, OpenSimplex),
    percent_range: Uniform<u8>,
    main_seed: u32,
) -> biomes::MapObject {
    let position_as_float_array_descaled = [
        position.0 as f64 / scale as f64 + offset.0 as f64,
        position.1 as f64 / scale as f64 + offset.1 as f64,
    ]; // returning to true world space

    let biome_position = [
        position_as_float_array_descaled[0] * biomes::BIOME_SCALE.0,
        position_as_float_array_descaled[1] * biomes::BIOME_SCALE.1,
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
            && detail == random_pattern_map_object.detail
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
                position_as_float_array_descaled[0] * simplex_pattern_map_object.noise_scale,
                position_as_float_array_descaled[1] * simplex_pattern_map_object.noise_scale,
            ]);
        if simplex_pattern_map_object.priority > highest_priority
            && detail == simplex_pattern_map_object.detail
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
                position_as_float_array_descaled[0]
                    * simplex_smoothed_pattern_map_object.noise_scale,
                position_as_float_array_descaled[1]
                    * simplex_smoothed_pattern_map_object.noise_scale,
            ]);
        if simplex_smoothed_pattern_map_object.priority > highest_priority
            && detail == simplex_smoothed_pattern_map_object.detail
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

fn full_index_from_full_position(full_position: (u32, u32), scale: u32) -> usize {
    let chunk_position = (
        full_position.0 / CHUNK_WIDTH as u32 / scale,
        full_position.1 / CHUNK_WIDTH as u32 / scale,
    );
    let chunk_index = index_from_position(chunk_position, CHUNK_GRID_WIDTH);
    let full_index_start = chunk_index * CHUNK_WIDTH_SQUARED as u32 * (scale * scale);

    let local_position = (
        full_position.0 % (CHUNK_WIDTH as u32 * scale),
        full_position.1 % (CHUNK_WIDTH as u32 * scale),
    );
    let local_index = index_from_position(local_position, CHUNK_WIDTH as u32 * scale);

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

fn render_map(user_storage: &mut UserStorage, render_storage: &mut RenderStorage, detail: u8) {
    let detail_scale = user_storage.details[detail as usize].scale;
    let float_detail_scale = detail_scale as f32;
    let detail_offset = user_storage.details[detail as usize].offset;

    let screen_width_as_world_units =
        2.0 / render_storage.camera.scale / render_storage.aspect_ratio * float_detail_scale;
    let screen_height_as_world_units = 2.0 / render_storage.camera.scale * float_detail_scale;

    for x in (render_storage.camera.position.0 * float_detail_scale
        - (screen_width_as_world_units * 0.5))
        .floor() as i32
        - 1
        ..(render_storage.camera.position.0 * float_detail_scale
            + (screen_width_as_world_units * 0.5))
            .ceil() as i32
            + 1
    {
        for y in (render_storage.camera.position.1 * float_detail_scale
            - (screen_height_as_world_units * 0.5))
            .floor() as i32
            - 1
            ..(render_storage.camera.position.1 * float_detail_scale
                + (screen_height_as_world_units * 0.5))
                .ceil() as i32
                + 1
        {
            if x < 0 || y < 0 {
                continue;
            }

            let full_index =
                full_index_from_full_position((x as u32, y as u32), detail_scale as u32);

            if full_index
                >= FULL_GRID_WIDTH_SQUARED as usize * (detail_scale * detail_scale) as usize
            {
                panic!("Something has gone wrong with the index. It is beyond reasonable array bounds. full index: {}, bounds: {}", full_index, FULL_GRID_WIDTH_SQUARED * (detail_scale * detail_scale))
            }

            let map_object = user_storage.map_objects[detail as usize][full_index];

            let (rendering_size, uv) = match map_object {
                biomes::MapObject::RandomPattern(i) => {
                    let random_pattern_map_object = &biomes::RANDOM_PATTERN_MAP_OBJECTS[i as usize];
                    (
                        random_pattern_map_object.rendering_size,
                        random_pattern_map_object.uv,
                    )
                }
                biomes::MapObject::SimplexPattern(i) => {
                    let simplex_pattern_map_object =
                        &biomes::SIMPLEX_PATTERN_MAP_OBJECTS[i as usize];
                    (
                        simplex_pattern_map_object.rendering_size,
                        simplex_pattern_map_object.uv,
                    )
                }
                biomes::MapObject::SimplexSmoothedPattern(_) => {
                    todo!();
                    //let simplex_smoothed_pattern_map_object = biomes::SIMPLEX_SMOOTHED_PATTERN_MAP_OBJECTS[i as usize];
                    //(simplex_smoothed_pattern_map_object.rendering_size, simplex_smoothed_pattern_map_object.uv)
                }
                biomes::MapObject::None => {
                    continue;
                }
            };

            let vertex_start = render_storage.vertex_count_map as usize;
            let index_start = render_storage.index_count_map as usize;

            render_storage.vertices_map[vertex_start] = vertex_data::VertexData {
                // top right
                position: [
                    x as f32 / float_detail_scale + detail_offset.0 + (0.5 * rendering_size.0),
                    y as f32 / float_detail_scale + detail_offset.1 + (0.5 * rendering_size.1),
                ],
                uv: [uv.0 + biomes::SPRITE_SIZE.0, uv.1 + biomes::SPRITE_SIZE.1],
            };

            render_storage.vertices_map[vertex_start + 1] = vertex_data::VertexData {
                // bottom right
                position: [
                    x as f32 / float_detail_scale + detail_offset.0 + (0.5 * rendering_size.0),
                    y as f32 / float_detail_scale + detail_offset.1 + (-0.5 * rendering_size.1),
                ],
                uv: [uv.0 + biomes::SPRITE_SIZE.0, uv.1],
            };

            render_storage.vertices_map[vertex_start + 2] = vertex_data::VertexData {
                // top left
                position: [
                    x as f32 / float_detail_scale + detail_offset.0 + (-0.5 * rendering_size.0),
                    y as f32 / float_detail_scale + detail_offset.1 + (0.5 * rendering_size.1),
                ],
                uv: [uv.0, uv.1 + biomes::SPRITE_SIZE.1],
            };

            render_storage.vertices_map[vertex_start + 3] = vertex_data::VertexData {
                // bottom left
                position: [
                    x as f32 / float_detail_scale + detail_offset.0 + (-0.5 * rendering_size.0),
                    y as f32 / float_detail_scale + detail_offset.1 + (-0.5 * rendering_size.1),
                ],
                uv: [uv.0, uv.1],
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
    }
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

fn detect_collision(
    position_1: (f32, f32),
    size_1: (f32, f32),
    position_2: (f32, f32),
    size_2: (f32, f32),
) -> bool {
    if (position_1.0 - position_2.0).abs() > size_1.0 * 0.5 + size_2.0 * 0.5 {
        return false;
    }
    if (position_1.1 - position_2.1).abs() > size_1.1 * 0.5 + size_2.1 * 0.5 {
        return false;
    }
    true
}

fn collide(user_storage: &mut UserStorage, full_position: (u32, u32), detail_index: u8) {
    let detail = user_storage.details[detail_index as usize];

    let map_object = user_storage.map_objects[detail_index as usize]
        [full_index_from_full_position(full_position, detail.scale)];

    let collision_size = match map_object {
        biomes::MapObject::RandomPattern(i) => {
            biomes::RANDOM_PATTERN_MAP_OBJECTS[i as usize].collision_size
        }
        biomes::MapObject::SimplexPattern(i) => {
            biomes::SIMPLEX_PATTERN_MAP_OBJECTS[i as usize].collision_size
        }
        biomes::MapObject::SimplexSmoothedPattern(_) => {
            todo!();
        }
        biomes::MapObject::None => return,
    };

    if detect_collision(
        user_storage.player.position,
        user_storage.player.size,
        (
            full_position.0 as f32 / detail.scale as f32 + detail.offset.0,
            full_position.1 as f32 / detail.scale as f32 + detail.offset.1,
        ), //TODO: probably add the offset to this. I'm fairly certain this won't work without offset.
        collision_size,
    ) {
        deal_with_collision(
            user_storage,
            user_storage.player.previous_position,
            full_position,
            detail_index,
        )
    }
}

struct Player {
    position: (f32, f32),
    previous_position: (f32, f32),
    sprinting: bool,
    collision_debug: bool,
    size: (f32, f32),
    statistics: biomes::Statistics,
}

fn deal_with_collision(
    user_storage: &mut UserStorage,
    fallback_position: (f32, f32),
    full_position: (u32, u32),
    detail: u8,
) {
    let map_object = &mut user_storage.map_objects[detail as usize]
        [full_index_from_full_position(full_position, user_storage.details[detail as usize].scale)];

    let behaviour = match map_object {
        biomes::MapObject::RandomPattern(i) => {
            biomes::RANDOM_PATTERN_MAP_OBJECTS[*i as usize].behaviour
        }
        biomes::MapObject::SimplexPattern(i) => {
            biomes::SIMPLEX_PATTERN_MAP_OBJECTS[*i as usize].behaviour
        }
        biomes::MapObject::SimplexSmoothedPattern(i) => {
            biomes::SIMPLEX_SMOOTHED_PATTERN_MAP_OBJECTS[*i as usize].behaviour
        }
        biomes::MapObject::None => biomes::CollisionBehaviour::None,
    };

    match behaviour {
        biomes::CollisionBehaviour::None => {}
        biomes::CollisionBehaviour::Consume(strength, statistics) => {
            if user_storage.player.statistics.strength > strength {
                *map_object = biomes::MapObject::None;
                user_storage.player.statistics += statistics;
            } else {
                user_storage.player.position = fallback_position;
            }
        }
        biomes::CollisionBehaviour::Replace(strength, statistics, replacement_map_object) => {
            if user_storage.player.statistics.strength > strength {
                *map_object = replacement_map_object;
                user_storage.player.statistics += statistics;
            } else {
                user_storage.player.position = fallback_position;
            }
        }
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
        let (uv, individual_letter_spacing) = match character {
            '0' => ((0.0, 0.0), 1.0f32),
            '1' => ((TEXT_SPRITE_SIZE.0 * 1.0, 0.0), 1.0),
            '2' => ((TEXT_SPRITE_SIZE.0 * 2.0, 0.0), 1.0),
            '3' => ((TEXT_SPRITE_SIZE.0 * 3.0, 0.0), 1.0),
            '4' => ((TEXT_SPRITE_SIZE.0 * 4.0, 0.0), 1.0),
            '5' => ((TEXT_SPRITE_SIZE.0 * 5.0, 0.0), 1.0),
            '6' => ((TEXT_SPRITE_SIZE.0 * 6.0, 0.0), 1.0),
            '7' => ((TEXT_SPRITE_SIZE.0 * 7.0, 0.0), 1.0),
            '8' => ((TEXT_SPRITE_SIZE.0 * 8.0, 0.0), 1.0),
            '9' => ((TEXT_SPRITE_SIZE.0 * 9.0, 0.0), 1.0),

            'A' => ((TEXT_SPRITE_SIZE.0 * 0.0, TEXT_SPRITE_SIZE.1 * 1.0), 1.0),
            'B' => ((TEXT_SPRITE_SIZE.0 * 1.0, TEXT_SPRITE_SIZE.1 * 1.0), 1.0),
            'C' => ((TEXT_SPRITE_SIZE.0 * 2.0, TEXT_SPRITE_SIZE.1 * 1.0), 1.0),
            'D' => ((TEXT_SPRITE_SIZE.0 * 3.0, TEXT_SPRITE_SIZE.1 * 1.0), 1.0),
            'E' => ((TEXT_SPRITE_SIZE.0 * 4.0, TEXT_SPRITE_SIZE.1 * 1.0), 1.0),
            'F' => ((TEXT_SPRITE_SIZE.0 * 5.0, TEXT_SPRITE_SIZE.1 * 1.0), 1.0),
            'G' => ((TEXT_SPRITE_SIZE.0 * 6.0, TEXT_SPRITE_SIZE.1 * 1.0), 1.0),
            'H' => ((TEXT_SPRITE_SIZE.0 * 7.0, TEXT_SPRITE_SIZE.1 * 1.0), 1.5),
            'I' => ((TEXT_SPRITE_SIZE.0 * 8.0, TEXT_SPRITE_SIZE.1 * 1.0), 1.0),
            'J' => ((TEXT_SPRITE_SIZE.0 * 9.0, TEXT_SPRITE_SIZE.1 * 1.0), 1.0),
            'K' => ((TEXT_SPRITE_SIZE.0 * 10.0, TEXT_SPRITE_SIZE.1 * 1.0), 1.0),
            'L' => ((TEXT_SPRITE_SIZE.0 * 11.0, TEXT_SPRITE_SIZE.1 * 1.0), 1.0),
            'M' => ((TEXT_SPRITE_SIZE.0 * 12.0, TEXT_SPRITE_SIZE.1 * 1.0), 1.0),
            'N' => ((TEXT_SPRITE_SIZE.0 * 13.0, TEXT_SPRITE_SIZE.1 * 1.0), 1.0),
            'O' => ((TEXT_SPRITE_SIZE.0 * 14.0, TEXT_SPRITE_SIZE.1 * 1.0), 1.0),
            'P' => ((TEXT_SPRITE_SIZE.0 * 15.0, TEXT_SPRITE_SIZE.1 * 1.0), 1.0),
            'Q' => ((TEXT_SPRITE_SIZE.0 * 16.0, TEXT_SPRITE_SIZE.1 * 1.0), 1.0),
            'R' => ((TEXT_SPRITE_SIZE.0 * 17.0, TEXT_SPRITE_SIZE.1 * 1.0), 1.0),
            'S' => ((TEXT_SPRITE_SIZE.0 * 18.0, TEXT_SPRITE_SIZE.1 * 1.0), 1.3),
            'T' => ((TEXT_SPRITE_SIZE.0 * 19.0, TEXT_SPRITE_SIZE.1 * 1.0), 1.0),
            'U' => ((TEXT_SPRITE_SIZE.0 * 20.0, TEXT_SPRITE_SIZE.1 * 1.0), 1.0),
            'V' => ((TEXT_SPRITE_SIZE.0 * 21.0, TEXT_SPRITE_SIZE.1 * 1.0), 1.0),
            'W' => ((TEXT_SPRITE_SIZE.0 * 22.0, TEXT_SPRITE_SIZE.1 * 1.0), 1.0),
            'X' => ((TEXT_SPRITE_SIZE.0 * 23.0, TEXT_SPRITE_SIZE.1 * 1.0), 1.0),
            'Y' => ((TEXT_SPRITE_SIZE.0 * 24.0, TEXT_SPRITE_SIZE.1 * 1.0), 1.0),
            'Z' => ((TEXT_SPRITE_SIZE.0 * 25.0, TEXT_SPRITE_SIZE.1 * 1.0), 1.0),

            'a' => ((TEXT_SPRITE_SIZE.0 * 0.0, TEXT_SPRITE_SIZE.1 * 2.0), 1.0),
            'b' => ((TEXT_SPRITE_SIZE.0 * 1.0, TEXT_SPRITE_SIZE.1 * 2.0), 1.0),
            'c' => ((TEXT_SPRITE_SIZE.0 * 2.0, TEXT_SPRITE_SIZE.1 * 2.0), 1.0),
            'd' => ((TEXT_SPRITE_SIZE.0 * 3.0, TEXT_SPRITE_SIZE.1 * 2.0), 1.0),
            'e' => ((TEXT_SPRITE_SIZE.0 * 4.0, TEXT_SPRITE_SIZE.1 * 2.0), 1.0),
            'f' => ((TEXT_SPRITE_SIZE.0 * 5.0, TEXT_SPRITE_SIZE.1 * 2.0), 0.5),
            'g' => ((TEXT_SPRITE_SIZE.0 * 6.0, TEXT_SPRITE_SIZE.1 * 2.0), 1.0),
            'h' => ((TEXT_SPRITE_SIZE.0 * 7.0, TEXT_SPRITE_SIZE.1 * 2.0), 1.0),
            'i' => ((TEXT_SPRITE_SIZE.0 * 8.0, TEXT_SPRITE_SIZE.1 * 2.0), 0.5),
            'j' => ((TEXT_SPRITE_SIZE.0 * 9.0, TEXT_SPRITE_SIZE.1 * 2.0), 1.0),
            'k' => ((TEXT_SPRITE_SIZE.0 * 10.0, TEXT_SPRITE_SIZE.1 * 2.0), 1.0),
            'l' => ((TEXT_SPRITE_SIZE.0 * 11.0, TEXT_SPRITE_SIZE.1 * 2.0), 0.5),
            'm' => ((TEXT_SPRITE_SIZE.0 * 12.0, TEXT_SPRITE_SIZE.1 * 2.0), 1.7),
            'n' => ((TEXT_SPRITE_SIZE.0 * 13.0, TEXT_SPRITE_SIZE.1 * 2.0), 1.3),
            'o' => ((TEXT_SPRITE_SIZE.0 * 14.0, TEXT_SPRITE_SIZE.1 * 2.0), 1.0),
            'p' => ((TEXT_SPRITE_SIZE.0 * 15.0, TEXT_SPRITE_SIZE.1 * 2.0), 1.0),
            'q' => ((TEXT_SPRITE_SIZE.0 * 16.0, TEXT_SPRITE_SIZE.1 * 2.0), 1.0),
            'r' => ((TEXT_SPRITE_SIZE.0 * 17.0, TEXT_SPRITE_SIZE.1 * 2.0), 0.5),
            's' => ((TEXT_SPRITE_SIZE.0 * 18.0, TEXT_SPRITE_SIZE.1 * 2.0), 1.0),
            't' => ((TEXT_SPRITE_SIZE.0 * 19.0, TEXT_SPRITE_SIZE.1 * 2.0), 0.5),
            'u' => ((TEXT_SPRITE_SIZE.0 * 20.0, TEXT_SPRITE_SIZE.1 * 2.0), 1.0),
            'v' => ((TEXT_SPRITE_SIZE.0 * 21.0, TEXT_SPRITE_SIZE.1 * 2.0), 1.0),
            'w' => ((TEXT_SPRITE_SIZE.0 * 22.0, TEXT_SPRITE_SIZE.1 * 2.0), 1.0),
            'x' => ((TEXT_SPRITE_SIZE.0 * 23.0, TEXT_SPRITE_SIZE.1 * 2.0), 1.0),
            'y' => ((TEXT_SPRITE_SIZE.0 * 24.0, TEXT_SPRITE_SIZE.1 * 2.0), 1.0),
            'z' => ((TEXT_SPRITE_SIZE.0 * 25.0, TEXT_SPRITE_SIZE.1 * 2.0), 1.0),

            ' ' => ((TEXT_SPRITE_SIZE.0 * 29.0, TEXT_SPRITE_SIZE.1 * 2.0), 1.0),
            ':' => ((TEXT_SPRITE_SIZE.0 * 12.0, TEXT_SPRITE_SIZE.1 * 0.0), 1.0),
            '-' => ((TEXT_SPRITE_SIZE.0 * 11.0, TEXT_SPRITE_SIZE.1 * 0.0), 1.0),
            '_' => ((TEXT_SPRITE_SIZE.0 * 13.0, TEXT_SPRITE_SIZE.1 * 0.0), 1.0),
            '.' => ((TEXT_SPRITE_SIZE.0 * 10.0, TEXT_SPRITE_SIZE.1 * 0.0), 0.5),
            '%' => ((TEXT_SPRITE_SIZE.0 * 17.0, TEXT_SPRITE_SIZE.1 * 0.0), 1.0),
            '(' => ((TEXT_SPRITE_SIZE.0 * 18.0, TEXT_SPRITE_SIZE.1 * 0.0), 0.5),
            ')' => ((TEXT_SPRITE_SIZE.0 * 19.0, TEXT_SPRITE_SIZE.1 * 0.0), 0.5),
            ',' => ((TEXT_SPRITE_SIZE.0 * 20.0, TEXT_SPRITE_SIZE.1 * 0.0), 0.5),

            '@' => ((TEXT_SPRITE_SIZE.0 * 0.0, TEXT_SPRITE_SIZE.1 * 3.0), 1.0),
            _ => ((TEXT_SPRITE_SIZE.0 * 14.0, 0.0), 1.0),
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

        position.0 += individual_letter_spacing * letter_spacing;
    }
}

// Absolute garbage, fix asap. This needs to account for player size, whether the block thinks it is safe (add a safe bool to all blocks), and detail.
fn get_safe_position(user_storage: &mut UserStorage) -> (u32, u32) {
    let mut rng = thread_rng();
    let position_range = Uniform::new(0u32, FULL_GRID_WIDTH);

    let mut safe = false;
    let mut safe_position = (10u32, 10u32);

    while !safe {
        safe_position = (
            position_range.sample(&mut rng),
            position_range.sample(&mut rng),
        );

        safe = match generate_position(
            safe_position,
            0,
            1,
            (0.0, 0.0),
            &mut rng,
            user_storage.biome_noise,
            user_storage.percent_range,
            user_storage.main_seed,
        ) {
            biomes::MapObject::None => true,
            _ => false,
        }
    }

    safe_position
}
