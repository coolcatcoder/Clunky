use noise::NoiseFn;
use noise::OpenSimplex;
use rand::distributions::{Distribution, Uniform};
use rand::rngs::ThreadRng;
use rand::thread_rng;
use std::ops::{Add, Div, Mul, Rem};
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Instant;
use winit::event::ElementState;

use crate::biomes;
use crate::collision;
use crate::marching_squares;
use crate::menus;
use crate::perks_and_curses;
use crate::ui;
use crate::vertex_data;

pub const FULL_GRID_WIDTH: u32 = CHUNK_WIDTH as u32 * 50; //100;
pub const FULL_GRID_WIDTH_SQUARED: u32 = FULL_GRID_WIDTH * FULL_GRID_WIDTH;

pub const CHUNK_WIDTH: u16 = 64;
pub const CHUNK_WIDTH_SQUARED: u16 = CHUNK_WIDTH * CHUNK_WIDTH;
//const CHUNK_WIDTH_LOG2: u16 = (u16::BITS - CHUNK_WIDTH.leading_zeros()) as u16;

pub const CHUNK_GRID_WIDTH: u32 = FULL_GRID_WIDTH / CHUNK_WIDTH as u32;
pub const CHUNK_GRID_WIDTH_SQUARED: u32 = CHUNK_GRID_WIDTH * CHUNK_GRID_WIDTH;

pub const FIXED_UPDATE_TIME_STEP: f32 = 0.004;
pub const MAX_SUBSTEPS: u32 = 150;

pub const MAX_VERTICES: usize = CHUNK_WIDTH_SQUARED as usize * 4 * 100;
pub const MAX_INDICES: usize = CHUNK_WIDTH_SQUARED as usize * 6 * 100;
pub const MAX_INSTANCES: usize = CHUNK_WIDTH_SQUARED as usize * 100;

pub fn start(render_storage: &mut RenderStorage) -> UserStorage {
    render_storage.camera.scale = 0.12;

    render_storage.brightness = 2.5;

    let (generation_sender, generation_receiver) = mpsc::channel();

    let available_parallelism = thread::available_parallelism().unwrap().get();

    let mut user_storage = UserStorage {
        wasd_held: (false, false, false, false),
        zoom_held: (false, false),
        show_debug: false,
        main_seed: 0,
        percent_range: Uniform::new(0u8, 100),
        biome_noise: (OpenSimplex::new(0), OpenSimplex::new(0)),
        chunks_generated: vec![false; 0],
        details: [
            Detail {
                scale: 1,
                offset: (0.0, 0.0),
            },
            Detail {
                scale: 2,
                offset: (-0.25, -0.25),
            },
            Detail {
                scale: 3,
                offset: (0.0, 0.0),
            },
        ],
        map_objects: [
            vec![biomes::MapObject::None; 0],
            vec![biomes::MapObject::None; 0],
            vec![biomes::MapObject::None; 0],
        ],
        generation_sender,
        generation_receiver,
        available_parallelism,
        map_objects_per_thread: CHUNK_WIDTH_SQUARED as usize / available_parallelism,
        player: Player {
            aabb: collision::AabbCentred {
                position: (0.0, 0.0),
                size: (0.0, 0.0),
            },
            previous_position: (0.0, 0.0),
            sprinting: false,
            collision_debug: false,
            statistics: biomes::Statistics {
                strength: 0,
                health: 0,
                stamina: 0,
            },
            starting_statistics: biomes::Statistics {
                strength: 1,
                health: 1,
                stamina: 100,
            },
        },
        stop_watch: Instant::now(),
        fixed_time_passed: 0.0,
        multithread_rendering: false,
        chunk_generation: 0,
        menu: menus::STARTING_MENU,
        screen_texts: vec![],
        screen_buttons: vec![],
        screen_toggleable_buttons: vec![],
        perks_and_curses: perks_and_curses::PerksAndCurses {
            cost: 0,
            one_time_perks_owned: vec![],
            one_time_curses_owned: vec![],
            offered_perks: vec![],
            offered_curses: vec![],
        },
    };

    let check = user_storage.map_objects_per_thread * available_parallelism;

    println!("Available Parallelism: {}, Assumed generation per thread: {}, Check: {}, Correct Version: {}", available_parallelism, user_storage.map_objects_per_thread, check, CHUNK_WIDTH_SQUARED);

    assert!(check == CHUNK_WIDTH_SQUARED as usize);

    (menus::STARTING_MENU.get_data().start)(&mut user_storage, render_storage);

    user_storage
}

pub fn fixed_update(user_storage: &mut UserStorage, render_storage: &mut RenderStorage) {
    let motion = match user_storage.wasd_held {
        (true, false, false, false) => (0.0, -1.0),
        (false, false, true, false) => (0.0, 1.0),
        (false, false, false, true) => (1.0, 0.0),
        (false, true, false, false) => (-1.0, 0.0),

        (true, true, false, false) => (-0.7, -0.7),
        (true, false, false, true) => (0.7, -0.7),

        (false, true, true, false) => (-0.7, 0.7),
        (false, false, true, true) => (0.7, 0.7),

        _ => (0.0, 0.0),
    };

    let speed = match user_storage.player.sprinting {
        false => 5.0,
        true => 10.0,
    };

    user_storage.player.previous_position = user_storage.player.aabb.position;

    user_storage.player.aabb.position.0 += motion.0 * FIXED_UPDATE_TIME_STEP * speed;
    user_storage.player.aabb.position.1 += motion.1 * FIXED_UPDATE_TIME_STEP * speed;

    if !user_storage.player.collision_debug {
        for detail_index in 0..user_storage.details.len() {
            let detail = user_storage.details[detail_index];

            let rounded_player_position_scaled = (
                (user_storage.player.aabb.position.0 * detail.scale as f32).round() as i32,
                (user_storage.player.aabb.position.1 * detail.scale as f32).round() as i32,
            );

            let ceil_player_half_size_scaled = (
                (user_storage.player.aabb.size.0 * 0.5 * detail.scale as f32).ceil() as i32,
                (user_storage.player.aabb.size.1 * 0.5 * detail.scale as f32).ceil() as i32,
            );

            for x in -ceil_player_half_size_scaled.0..ceil_player_half_size_scaled.0 + 1 {
                for y in -ceil_player_half_size_scaled.1..ceil_player_half_size_scaled.1 + 1 {
                    let total_x = (rounded_player_position_scaled.0 + x) as u32;
                    let total_y = (rounded_player_position_scaled.1 + y) as u32;

                    if total_x >= FULL_GRID_WIDTH * detail.scale
                        || total_y >= FULL_GRID_WIDTH * detail.scale
                    {
                        continue;
                    }

                    collide(
                        user_storage,
                        render_storage,
                        (total_x, total_y),
                        detail_index as u8,
                    )
                }
            }
        }
    }

    user_storage.player.statistics.stamina = user_storage.player.statistics.stamina.min(100);

    if !user_storage.player.collision_debug
        && user_storage.stop_watch.elapsed().as_secs_f32() >= 0.25
    {
        user_storage.stop_watch = Instant::now();

        user_storage.player.statistics.stamina -= 1;

        if user_storage.player.statistics.stamina < 0 {
            user_storage.player.statistics.health -= 1;
        }
    }

    if user_storage.player.statistics.health <= 0 {
        user_storage.menu = menus::Menu::Dead;
        (menus::DEAD.on_window_resize)(user_storage, render_storage);
        return;
    }

    if user_storage.player.aabb.position.0 < 0.0 {
        user_storage.player.aabb.position.0 = 0.0;
    } else if user_storage.player.aabb.position.0 > FULL_GRID_WIDTH as f32 {
        user_storage.player.aabb.position.0 = FULL_GRID_WIDTH as f32;
    }
    if user_storage.player.aabb.position.1 < 0.0 {
        user_storage.player.aabb.position.1 = 0.0;
    } else if user_storage.player.aabb.position.1 > FULL_GRID_WIDTH as f32 {
        user_storage.player.aabb.position.1 = FULL_GRID_WIDTH as f32;
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Detail {
    pub scale: u32, // This is unintuitive. Basically how many of these blocks become 1 block.
    pub offset: (f32, f32),
}

pub struct UserStorage {
    // This is for the user's stuff. The event loop should not touch this. Actually it will, but only for the menu. Should the menu be part of the render system then?
    pub wasd_held: (bool, bool, bool, bool),
    pub zoom_held: (bool, bool),
    pub show_debug: bool,
    pub main_seed: u32,
    pub percent_range: Uniform<u8>,
    pub biome_noise: (OpenSimplex, OpenSimplex),
    pub chunks_generated: Vec<bool>,
    pub details: [Detail; 3],
    pub map_objects: [Vec<biomes::MapObject>; 3],
    pub generation_sender: Sender<(Vec<biomes::MapObject>, usize, u8)>,
    pub generation_receiver: Receiver<(Vec<biomes::MapObject>, usize, u8)>,
    pub available_parallelism: usize,
    pub map_objects_per_thread: usize,
    pub player: Player,
    pub stop_watch: Instant,
    pub fixed_time_passed: f32,
    pub multithread_rendering: bool,
    pub chunk_generation: u8,
    pub menu: menus::Menu,
    pub screen_texts: Vec<ui::ScreenText>, // The plural of text is texts in this situation.
    pub screen_buttons: Vec<ui::ScreenButton>,
    pub screen_toggleable_buttons: Vec<ui::ScreenToggleableButton>,
    pub perks_and_curses: perks_and_curses::PerksAndCurses,
}

pub struct RenderStorage {
    // TODO: Perhaps removing or refining what belongs in this struct.
    pub vertices_map: Vec<vertex_data::MapVertex>,
    pub vertex_count_map: u32,
    pub indices_map: Vec<u32>,
    pub index_count_map: u32,

    pub vertices_ui: Vec<vertex_data::UIVertex>,
    pub vertex_count_ui: u32,
    pub indices_ui: Vec<u32>,
    pub index_count_ui: u32,

    pub vertices_test: Vec<vertex_data::TestVertex>,
    pub vertex_count_test: u32,
    pub update_vertices_test: bool,
    pub indices_test: Vec<u32>,
    pub index_count_test: u32,
    pub update_indices_test: bool,
    pub instances_test: Vec<vertex_data::TestInstance>,
    pub instance_count_test: u32,
    pub update_instances_test: bool,

    pub aspect_ratio: f32,
    pub camera: Camera,
    pub brightness: f32,
    pub frame_count: u32, // This will crash the game after 2 years, assuming 60 fps.
    pub starting_time: Instant,
    pub window_size: [u32; 2],
}

pub fn generate_chunk(user_storage: &UserStorage, chunk_position: (u32, u32)) {
    let biome_noise = user_storage.biome_noise;
    let percent_range = user_storage.percent_range;
    let main_seed = user_storage.main_seed;

    let details = user_storage.details;

    let full_position_start_unscaled = (
        chunk_position.0 * CHUNK_WIDTH as u32,
        chunk_position.1 * CHUNK_WIDTH as u32,
    );

    let generate_piece_of_chunk =
        move |amount_of_chunk_to_generate_unscaled: usize, // replace 'chunk' in name with 'mapobjects' perhaps?
              generation_offset_unscaled: usize,
              generation_sender: Sender<(Vec<biomes::MapObject>, usize, u8)>| {
            for detail_index in 0..details.len() {
                let detail = details[detail_index];
                let mut generation_array = vec![
                    biomes::MapObject::None;
                    amount_of_chunk_to_generate_unscaled
                        * (detail.scale * detail.scale) as usize
                ];

                let full_position_start = (
                    full_position_start_unscaled.0 * detail.scale as u32,
                    full_position_start_unscaled.1 * detail.scale as u32,
                );

                for i in
                    0..amount_of_chunk_to_generate_unscaled * (detail.scale * detail.scale) as usize
                {
                    let local_position = position_from_index(
                        (i + (generation_offset_unscaled * (detail.scale * detail.scale) as usize))
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
                            + (generation_offset_unscaled * (detail.scale * detail.scale) as usize),
                        detail_index as u8,
                    ))
                    .unwrap()
            }
        };

    let map_objects_per_thread =
        CHUNK_WIDTH_SQUARED as usize / (user_storage.available_parallelism - 1); // have fun messing around with this

    for thread_index in 0..user_storage.available_parallelism - 1 {
        let generation_sender = user_storage.generation_sender.clone();
        thread::Builder::new()
            .name("Generation Thread".into())
            .spawn(move || {
                generate_piece_of_chunk(
                    map_objects_per_thread,
                    thread_index * map_objects_per_thread,
                    generation_sender.clone(),
                )
            })
            .unwrap();
    }
}

pub fn generate_chunk_old(user_storage: &UserStorage, chunk_position: (u32, u32)) {
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
        // let simplex_noise =
        //     OpenSimplex::new(main_seed + simplex_smoothed_pattern_map_object.seed as u32).get([
        //         position_as_float_array_descaled[0]
        //             * simplex_smoothed_pattern_map_object.noise_scale,
        //         position_as_float_array_descaled[1]
        //             * simplex_smoothed_pattern_map_object.noise_scale,
        //     ]);

        let corners_noise = [
            // TODO: asap write somewhere what corners are for what positions, as I'm confused as hell
            OpenSimplex::new(main_seed + simplex_smoothed_pattern_map_object.seed as u32).get([
                // bottom left
                (position_as_float_array_descaled[0]
                    - simplex_smoothed_pattern_map_object.rendering_size.0 as f64 * 0.5)
                    * simplex_smoothed_pattern_map_object.noise_scale,
                (position_as_float_array_descaled[1]
                    - simplex_smoothed_pattern_map_object.rendering_size.1 as f64 * 0.5)
                    * simplex_smoothed_pattern_map_object.noise_scale,
            ]),
            OpenSimplex::new(main_seed + simplex_smoothed_pattern_map_object.seed as u32).get([
                // bottom right
                (position_as_float_array_descaled[0]
                    + simplex_smoothed_pattern_map_object.rendering_size.0 as f64 * 0.5)
                    * simplex_smoothed_pattern_map_object.noise_scale,
                (position_as_float_array_descaled[1]
                    - simplex_smoothed_pattern_map_object.rendering_size.1 as f64 * 0.5)
                    * simplex_smoothed_pattern_map_object.noise_scale,
            ]),
            OpenSimplex::new(main_seed + simplex_smoothed_pattern_map_object.seed as u32).get([
                // top right
                (position_as_float_array_descaled[0]
                    + simplex_smoothed_pattern_map_object.rendering_size.0 as f64 * 0.5)
                    * simplex_smoothed_pattern_map_object.noise_scale,
                (position_as_float_array_descaled[1]
                    + simplex_smoothed_pattern_map_object.rendering_size.1 as f64 * 0.5)
                    * simplex_smoothed_pattern_map_object.noise_scale,
            ]),
            OpenSimplex::new(main_seed + simplex_smoothed_pattern_map_object.seed as u32).get([
                // top left
                (position_as_float_array_descaled[0]
                    - simplex_smoothed_pattern_map_object.rendering_size.0 as f64 * 0.5)
                    * simplex_smoothed_pattern_map_object.noise_scale,
                (position_as_float_array_descaled[1]
                    + simplex_smoothed_pattern_map_object.rendering_size.1 as f64 * 0.5)
                    * simplex_smoothed_pattern_map_object.noise_scale,
            ]),
        ];

        let corners = [
            if corners_noise[0] > simplex_smoothed_pattern_map_object.acceptable_noise.0
                && corners_noise[0] < simplex_smoothed_pattern_map_object.acceptable_noise.1
            {
                true
            } else {
                false
            },
            if corners_noise[1] > simplex_smoothed_pattern_map_object.acceptable_noise.0
                && corners_noise[1] < simplex_smoothed_pattern_map_object.acceptable_noise.1
            {
                true
            } else {
                false
            },
            if corners_noise[2] > simplex_smoothed_pattern_map_object.acceptable_noise.0
                && corners_noise[2] < simplex_smoothed_pattern_map_object.acceptable_noise.1
            {
                true
            } else {
                false
            },
            if corners_noise[3] > simplex_smoothed_pattern_map_object.acceptable_noise.0
                && corners_noise[3] < simplex_smoothed_pattern_map_object.acceptable_noise.1
            {
                true
            } else {
                false
            },
        ];

        let has_correct_noise = corners[0] || corners[1] || corners[2] || corners[3];

        if simplex_smoothed_pattern_map_object.priority > highest_priority
            && detail == simplex_smoothed_pattern_map_object.detail
            && percent_range.sample(&mut rng) < simplex_smoothed_pattern_map_object.chance
            && has_correct_noise
        //&& simplex_noise > simplex_smoothed_pattern_map_object.acceptable_noise.0
        //&& simplex_noise < simplex_smoothed_pattern_map_object.acceptable_noise.1
        {
            let square_index = marching_squares::get_square_index(corners);

            if square_index != 0 {
                // if square index is 0, then no corners had the iso surface, meaning it basically shouldn't exist, even if the center had the iso surface. Potentially could create a slower but more accurate marching cubes by including center in the calculation of the square index.
                map_object = biomes::MapObject::SimplexSmoothedPattern(i, square_index);
                highest_priority = simplex_smoothed_pattern_map_object.priority;
            } else {
                panic!("Why is it 0??");
            }
        }
    }
    map_object
}

pub fn full_index_from_full_position(full_position: (u32, u32), scale: u32) -> usize {
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

pub fn index_from_position<T>(position: (T, T), width: T) -> T
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

#[deprecated]
pub fn render_map(
    user_storage: &mut UserStorage,
    render_storage: &mut RenderStorage,
    detail: u8,
    render_sender: &Sender<(Vec<vertex_data::MapVertex>, u32, Vec<u32>, u32)>,
) {
    let detail_scale = user_storage.details[detail as usize].scale;
    let float_detail_scale = detail_scale as f32;
    let detail_offset = user_storage.details[detail as usize].offset;

    let scaled_camera_position = (
        render_storage.camera.position.0 * float_detail_scale,
        render_storage.camera.position.1 * float_detail_scale,
    );

    let screen_width_as_world_units =
        2.0 / render_storage.camera.scale / render_storage.aspect_ratio * float_detail_scale + 5.0;
    let screen_height_as_world_units = 2.0 / render_storage.camera.scale * float_detail_scale + 5.0;

    let map_objects_per_thread = (screen_width_as_world_units * screen_height_as_world_units)
        .floor() as usize
        / user_storage.available_parallelism; // sketchy

    thread::scope(|scope| {
        for thread_index in 0..user_storage.available_parallelism {
            let map_objects_old = &user_storage.map_objects[detail as usize];
            //let map_objects = &user_storage.map_objects[detail as usize][]

            let render_sender = render_sender.clone();

            scope.spawn(move || {
                let mut vertices = vec![vertex_data::MapVertex{
                    position: [0.0,0.0,0.0],
                    uv: [0.0,0.0],
                };map_objects_per_thread*4];
                let mut indices = vec![0u32;map_objects_per_thread*6];

                let mut vertex_count = 0u32;
                let mut index_count = 0u32;

                for position_as_index in thread_index*map_objects_per_thread..(thread_index+1)*map_objects_per_thread {

                    let local_position = position_from_index(position_as_index, screen_width_as_world_units.floor() as usize); // sketchy

                    let (x,y) = (local_position.0 as i32 + scaled_camera_position.0 as i32 - (screen_width_as_world_units/2.0) as i32, local_position.1 as i32 + scaled_camera_position.1 as i32 - (screen_height_as_world_units/2.0) as i32); // even more sketchy

                    //println!("{},{}", x, y);

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

                    let map_object = map_objects_old[full_index];

                    let (rendering_size, uv, depth) = match map_object {
                        biomes::MapObject::RandomPattern(i) => {
                            let random_pattern_map_object = &biomes::RANDOM_PATTERN_MAP_OBJECTS[i as usize];
                            (
                                random_pattern_map_object.rendering_size,
                                random_pattern_map_object.uv,
                                random_pattern_map_object.depth,
                            )
                        }
                        biomes::MapObject::SimplexPattern(i) => {
                            let simplex_pattern_map_object =
                                &biomes::SIMPLEX_PATTERN_MAP_OBJECTS[i as usize];
                            (
                                simplex_pattern_map_object.rendering_size,
                                simplex_pattern_map_object.uv,
                                simplex_pattern_map_object.depth,
                            )
                        }
                        biomes::MapObject::SimplexSmoothedPattern(i,_) => {
                            let simplex_smoothed_pattern_map_object =
                                &biomes::SIMPLEX_SMOOTHED_PATTERN_MAP_OBJECTS[i as usize];
                            (
                                simplex_smoothed_pattern_map_object.rendering_size,
                                simplex_smoothed_pattern_map_object.uv,
                                simplex_smoothed_pattern_map_object.depth,
                            )
                        }
                        biomes::MapObject::None => {
                            continue;
                        }
                    };

                    let corrected_x = x as f32 / float_detail_scale + detail_offset.0;
                    let corrected_y = y as f32 / float_detail_scale + detail_offset.1;

                    let vertex_start = vertex_count as usize;
                    let index_start = index_count as usize;

                    vertices[vertex_start] = vertex_data::MapVertex {
                        // top right
                        position: [
                            corrected_x + (0.5 * rendering_size.0),
                            corrected_y + (0.5 * rendering_size.1),
                            depth,
                        ],
                        uv: [uv.0 + biomes::SPRITE_SIZE.0, uv.1 + biomes::SPRITE_SIZE.1],
                    };

                    vertices[vertex_start + 1] = vertex_data::MapVertex {
                        // bottom right
                        position: [
                            corrected_x + (0.5 * rendering_size.0),
                            corrected_y + (-0.5 * rendering_size.1),
                            depth,
                        ],
                        uv: [uv.0 + biomes::SPRITE_SIZE.0, uv.1],
                    };

                    vertices[vertex_start + 2] = vertex_data::MapVertex {
                        // top left
                        position: [
                            corrected_x + (-0.5 * rendering_size.0),
                            corrected_y + (0.5 * rendering_size.1),
                            depth,
                        ],
                        uv: [uv.0, uv.1 + biomes::SPRITE_SIZE.1],
                    };

                    vertices[vertex_start + 3] = vertex_data::MapVertex {
                        // bottom left
                        position: [
                            corrected_x + (-0.5 * rendering_size.0),
                            corrected_y + (-0.5 * rendering_size.1),
                            depth,
                        ],
                        uv: [uv.0, uv.1],
                    };

                    indices[index_start] = vertex_start as u32;
                    indices[index_start + 1] = vertex_start as u32 + 1;
                    indices[index_start + 2] = vertex_start as u32 + 2;

                    indices[index_start + 3] = vertex_start as u32 + 1;
                    indices[index_start + 4] = vertex_start as u32 + 3;
                    indices[index_start + 5] = vertex_start as u32 + 2;

                    vertex_count += 4;
                    index_count += 6;
                }
                render_sender.send((vertices, vertex_count, indices, index_count)).unwrap();
            });
        }
    });
}

#[deprecated]
pub fn render_map_single_threaded(
    user_storage: &mut UserStorage,
    render_storage: &mut RenderStorage,
    detail: u8,
) {
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
                continue; // Don't need to panic. Simply don't render it.
                          //panic!("Something has gone wrong with the index. It is beyond reasonable array bounds. full index: {}, bounds: {}", full_index, FULL_GRID_WIDTH_SQUARED * (detail_scale * detail_scale))
            }

            let map_object = user_storage.map_objects[detail as usize][full_index];

            let (rendering_size, uv, depth) = match map_object {
                biomes::MapObject::RandomPattern(i) => {
                    let random_pattern_map_object = &biomes::RANDOM_PATTERN_MAP_OBJECTS[i as usize];
                    (
                        random_pattern_map_object.rendering_size,
                        random_pattern_map_object.uv,
                        random_pattern_map_object.depth,
                    )
                }
                biomes::MapObject::SimplexPattern(i) => {
                    let simplex_pattern_map_object =
                        &biomes::SIMPLEX_PATTERN_MAP_OBJECTS[i as usize];
                    (
                        simplex_pattern_map_object.rendering_size,
                        simplex_pattern_map_object.uv,
                        simplex_pattern_map_object.depth,
                    )
                }
                biomes::MapObject::SimplexSmoothedPattern(i, square_index) => {
                    let simplex_smoothed_pattern_map_object =
                        &biomes::SIMPLEX_SMOOTHED_PATTERN_MAP_OBJECTS[i as usize];

                    let corrected_x = x as f32 / float_detail_scale + detail_offset.0;
                    let corrected_y = y as f32 / float_detail_scale + detail_offset.1;

                    let vertex_start = render_storage.vertex_count_map as usize;
                    let index_start = render_storage.index_count_map as usize;

                    let vertices = marching_squares::VERTEX_TABLE[square_index as usize];

                    for i in 0..vertices.len() {
                        let position = [
                            corrected_x
                                + (simplex_smoothed_pattern_map_object.rendering_size.0
                                    * 0.5
                                    * vertices[i].0 as f32),
                            corrected_y
                                + (simplex_smoothed_pattern_map_object.rendering_size.1
                                    * 0.5
                                    * vertices[i].1 as f32),
                            simplex_smoothed_pattern_map_object.depth,
                        ];

                        render_storage.vertices_map[vertex_start + i] = vertex_data::MapVertex {
                            position,
                            uv: [
                                rerange_from_neg_1_pos_1_range(
                                    (
                                        simplex_smoothed_pattern_map_object.uv.0,
                                        simplex_smoothed_pattern_map_object.uv.0
                                            + biomes::SPRITE_SIZE.0,
                                    ),
                                    vertices[i].0 as f32,
                                ),
                                rerange_from_neg_1_pos_1_range(
                                    (
                                        simplex_smoothed_pattern_map_object.uv.1,
                                        simplex_smoothed_pattern_map_object.uv.1
                                            + biomes::SPRITE_SIZE.1,
                                    ),
                                    vertices[i].1 as f32,
                                ),
                            ], // TODO: consider working out how to have larger "tiles" for the uv to be spread across. Perhaps by modulo-ing the real position by 3 or something, then doing some sort of multiplication with the rerange()-ed values.
                        }
                    }

                    let mut indices = marching_squares::INDEX_TABLE[square_index as usize].to_vec(); // TODO: investigate to_vec() and work out if there is a better way
                    indices.iter_mut().for_each(|x| *x += vertex_start as u32);

                    render_storage.indices_map[index_start..index_start + indices.len()]
                        .copy_from_slice(indices.as_slice());

                    render_storage.vertex_count_map += vertices.len() as u32;
                    render_storage.index_count_map += indices.len() as u32;

                    continue;
                }
                biomes::MapObject::None => {
                    continue;
                }
            };

            let corrected_x = x as f32 / float_detail_scale + detail_offset.0;
            let corrected_y = y as f32 / float_detail_scale + detail_offset.1;

            let vertex_start = render_storage.vertex_count_map as usize;
            let index_start = render_storage.index_count_map as usize;

            render_storage.vertices_map[vertex_start] = vertex_data::MapVertex {
                // top right
                position: [
                    corrected_x + (0.5 * rendering_size.0),
                    corrected_y + (0.5 * rendering_size.1),
                    depth,
                ],
                uv: [uv.0 + biomes::SPRITE_SIZE.0, uv.1 + biomes::SPRITE_SIZE.1],
            };

            render_storage.vertices_map[vertex_start + 1] = vertex_data::MapVertex {
                // bottom right
                position: [
                    corrected_x + (0.5 * rendering_size.0),
                    corrected_y + (-0.5 * rendering_size.1),
                    depth,
                ],
                uv: [uv.0 + biomes::SPRITE_SIZE.0, uv.1],
            };

            render_storage.vertices_map[vertex_start + 2] = vertex_data::MapVertex {
                // top left
                position: [
                    corrected_x + (-0.5 * rendering_size.0),
                    corrected_y + (0.5 * rendering_size.1),
                    depth,
                ],
                uv: [uv.0, uv.1 + biomes::SPRITE_SIZE.1],
            };

            render_storage.vertices_map[vertex_start + 3] = vertex_data::MapVertex {
                // bottom left
                position: [
                    corrected_x + (-0.5 * rendering_size.0),
                    corrected_y + (-0.5 * rendering_size.1),
                    depth,
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

pub fn render_player(user_storage: &mut UserStorage, render_storage: &mut RenderStorage) {
    let vertex_start = render_storage.vertex_count_map as usize;
    let index_start = render_storage.index_count_map as usize;

    render_storage.vertices_map[vertex_start] = vertex_data::MapVertex {
        // top right
        position: [
            user_storage.player.aabb.position.0 + user_storage.player.aabb.size.0 * 0.5,
            user_storage.player.aabb.position.1 + user_storage.player.aabb.size.1 * 0.5,
            0.0,
        ],
        uv: [biomes::SPRITE_SIZE.0, biomes::SPRITE_SIZE.1],
    };

    render_storage.vertices_map[vertex_start + 1] = vertex_data::MapVertex {
        // bottom right
        position: [
            user_storage.player.aabb.position.0 + user_storage.player.aabb.size.0 * 0.5,
            user_storage.player.aabb.position.1 - user_storage.player.aabb.size.1 * 0.5,
            0.0,
        ],
        uv: [biomes::SPRITE_SIZE.0, 0.0],
    };

    render_storage.vertices_map[vertex_start + 2] = vertex_data::MapVertex {
        // top left
        position: [
            user_storage.player.aabb.position.0 - user_storage.player.aabb.size.0 * 0.5,
            user_storage.player.aabb.position.1 + user_storage.player.aabb.size.1 * 0.5,
            0.0,
        ],
        uv: [0.0, biomes::SPRITE_SIZE.1],
    };

    render_storage.vertices_map[vertex_start + 3] = vertex_data::MapVertex {
        // bottom left
        position: [
            user_storage.player.aabb.position.0 - user_storage.player.aabb.size.0 * 0.5,
            user_storage.player.aabb.position.1 - user_storage.player.aabb.size.1 * 0.5,
            0.0,
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

fn collide(
    user_storage: &mut UserStorage,
    render_storage: &mut RenderStorage,
    full_position: (u32, u32),
    detail_index: u8,
) {
    // TODO: this is broken on detail 1, work out why, then fix it.
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
        biomes::MapObject::SimplexSmoothedPattern(i, _) => {
            biomes::SIMPLEX_SMOOTHED_PATTERN_MAP_OBJECTS[i as usize].collision_size
            // TODO: add proper collision handling for marching squares
        }
        biomes::MapObject::None => return,
    };

    if collision::detect_collision_aabb_centred(
        user_storage.player.aabb,
        collision::AabbCentred {
            position: (
                full_position.0 as f32 / detail.scale as f32 + detail.offset.0,
                full_position.1 as f32 / detail.scale as f32 + detail.offset.1,
            ),
            size: collision_size,
        },
    ) {
        deal_with_collision(
            user_storage,
            render_storage,
            user_storage.player.previous_position,
            full_position,
            detail_index,
        )
    }
}

pub struct Player {
    pub aabb: collision::AabbCentred,
    //pub position: (f32, f32),
    pub previous_position: (f32, f32),
    pub sprinting: bool,
    pub collision_debug: bool,
    //pub size: (f32, f32),
    pub statistics: biomes::Statistics,
    pub starting_statistics: biomes::Statistics,
}

fn deal_with_collision(
    user_storage: &mut UserStorage,
    render_storage: &mut RenderStorage,
    fallback_position: (f32, f32),
    full_position: (u32, u32),
    detail_index: u8,
) {
    let map_object = &mut user_storage.map_objects[detail_index as usize]
        [full_index_from_full_position(
            full_position,
            user_storage.details[detail_index as usize].scale,
        )];

    let behaviour = match map_object {
        biomes::MapObject::RandomPattern(i) => {
            biomes::RANDOM_PATTERN_MAP_OBJECTS[*i as usize].behaviour
        }
        biomes::MapObject::SimplexPattern(i) => {
            biomes::SIMPLEX_PATTERN_MAP_OBJECTS[*i as usize].behaviour
        }
        biomes::MapObject::SimplexSmoothedPattern(i, _) => {
            biomes::SIMPLEX_SMOOTHED_PATTERN_MAP_OBJECTS[*i as usize].behaviour
        }
        biomes::MapObject::None => biomes::CollisionBehaviour::None,
    };

    match behaviour {
        biomes::CollisionBehaviour::None => {}
        biomes::CollisionBehaviour::Consume(strength, statistics) => {
            if user_storage.player.statistics.strength > strength as i8 {
                *map_object = biomes::MapObject::None;
                user_storage.player.statistics += statistics;
            } else {
                user_storage.player.aabb.position = fallback_position;
            }
        }
        biomes::CollisionBehaviour::Replace(strength, statistics, replacement_map_object) => {
            if user_storage.player.statistics.strength > strength as i8 {
                *map_object = replacement_map_object;
                user_storage.player.statistics += statistics;
            } else {
                user_storage.player.aabb.position = fallback_position;
            }
        }
        biomes::CollisionBehaviour::RunCode(function_index) => {
            biomes::MAP_OBJECT_COLLISION_FUNCTIONS[function_index as usize](
                user_storage,
                render_storage,
                full_position,
                detail_index,
            );
        }
    }
}

// Not good. It can't account for randomly generated map objects, due to non deterministic generation. Also, super slow.
pub fn get_safe_position(user_storage: &mut UserStorage) -> (u32, u32) {
    let mut rng = thread_rng();
    let position_range = Uniform::new(0u32, FULL_GRID_WIDTH);

    let mut not_safe = true;
    let mut safe_position = (10u32, 10u32);

    while not_safe {
        safe_position = (
            position_range.sample(&mut rng),
            position_range.sample(&mut rng),
        );

        not_safe = false;

        for detail_index in 0..user_storage.details.len() {
            let detail = user_storage.details[detail_index];

            let rounded_player_position_scaled = (
                (safe_position.0 * detail.scale) as i32,
                (safe_position.1 * detail.scale) as i32,
            );

            let ceil_player_half_size_scaled = (
                (user_storage.player.aabb.size.0 * 0.5 * detail.scale as f32).ceil() as i32,
                (user_storage.player.aabb.size.1 * 0.5 * detail.scale as f32).ceil() as i32,
            );

            for x in -ceil_player_half_size_scaled.0..ceil_player_half_size_scaled.0 + 1 {
                for y in -ceil_player_half_size_scaled.1..ceil_player_half_size_scaled.1 + 1 {
                    let total_x = (rounded_player_position_scaled.0 + x) as u32;
                    let total_y = (rounded_player_position_scaled.1 + y) as u32;

                    if total_x >= FULL_GRID_WIDTH * detail.scale
                        || total_y >= FULL_GRID_WIDTH * detail.scale
                    {
                        continue;
                    }

                    not_safe = not_safe
                        || match generate_position(
                            (total_x, total_y),
                            detail_index as u8,
                            detail.scale,
                            detail.offset,
                            &mut rng,
                            user_storage.biome_noise,
                            user_storage.percent_range,
                            user_storage.main_seed,
                        ) {
                            biomes::MapObject::None => false,
                            _ => true,
                        };
                }
            }
        }
    }

    safe_position
}

pub fn rerange_from_neg_1_pos_1_range(desired_range: (f32, f32), value: f32) -> f32 {
    let slope = (desired_range.1 - desired_range.0) / (1.0 - -1.0);
    desired_range.0 + slope * (value - -1.0)
}

pub fn rerange(old_range: (f32, f32), desired_range: (f32, f32), value: f32) -> f32 {
    (((value - old_range.0) * (desired_range.1 - desired_range.0)) / (old_range.1 - old_range.0))
        + desired_range.0
}
