use noise::OpenSimplex;
use rand::distributions::Bernoulli;
use rand::distributions::{Distribution, Uniform};
use rand::thread_rng;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::mpsc::TryRecvError;
use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, KeyboardInput, MouseButton, VirtualKeyCode};

use crate::events;
use crate::ui;
use crate::vertex_data;
use crate::{biomes, collision};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Menu {
    // add a pause screen that doesn't call ALIVE.end() and when it changes back to Alive, then it doesn't run ALIVE.start(). This will work.
    TitleScreen, // horrible name
    Alive,
    Paused,
    Dead,
    PerksAndCurses,
}

pub struct MenuData {
    pub start: fn(&mut events::UserStorage, &mut events::RenderStorage),
    pub update: fn(&mut events::UserStorage, &mut events::RenderStorage, f32, f32),
    pub end: fn(&mut events::UserStorage, &mut events::RenderStorage),
    pub on_keyboard_input: fn(&mut events::UserStorage, &mut events::RenderStorage, KeyboardInput),
    pub on_window_resize: fn(&mut events::UserStorage, &mut events::RenderStorage),
    pub on_cursor_moved:
        fn(&mut events::UserStorage, &mut events::RenderStorage, PhysicalPosition<f64>),
    pub on_mouse_input:
        fn(&mut events::UserStorage, &mut events::RenderStorage, ElementState, MouseButton),
}

pub const TITLE_SCREEN: MenuData = MenuData {
    start: |user_storage: &mut events::UserStorage, render_storage: &mut events::RenderStorage| {
        (TITLE_SCREEN.on_window_resize)(user_storage, render_storage);
    },
    update: |user_storage: &mut events::UserStorage,
             render_storage: &mut events::RenderStorage,
             _delta_time: f32,
             _average_fps: f32| {
        render_storage.vertex_count_ui = 0;
        render_storage.index_count_ui = 0;

        ui::render_screen_texts(render_storage, &user_storage.screen_texts);
        ui::render_screen_buttons(render_storage, &user_storage.screen_buttons)
    },
    end: |_user_storage: &mut events::UserStorage, _render_storage: &mut events::RenderStorage| {},
    on_keyboard_input: |user_storage: &mut events::UserStorage,
                        render_storage: &mut events::RenderStorage,
                        input: KeyboardInput| {
        if let Some(key_code) = input.virtual_keycode {
            match key_code {
                VirtualKeyCode::Return => {
                    if events::is_pressed(input.state) {
                        user_storage.menu = Menu::Alive;
                        (ALIVE.start)(user_storage, render_storage);
                    }
                }
                _ => (),
            }
        }
    },
    on_window_resize: |user_storage: &mut events::UserStorage,
                       render_storage: &mut events::RenderStorage| {
        let screen_width = 2.0 / render_storage.aspect_ratio;

        user_storage.screen_texts = vec![ui::ScreenText::new(
            (screen_width * -0.5 + screen_width * 0.1, -0.5),
            (0.25, 0.5),
            0.125,
            "No Title! Press Enter!",
            [1.0, 0.0, 1.0, 1.0],
        )];

        user_storage.screen_buttons = vec![
            ui::ScreenButton::new(
                collision::AabbCentred {
                    position: (0.0, 0.0),
                    size: (screen_width / 2.0, 0.2),
                },
                (0.0 * ui::TEXT_SPRITE_SIZE.0, 3.0 * ui::TEXT_SPRITE_SIZE.1),
                [1.0, 0.0, 1.0, 1.0],
                [0.0, 1.0, 1.0, 1.0],
                |user_storage: &mut events::UserStorage, render_storage: &mut events::RenderStorage| {
                    user_storage.menu = Menu::Alive;
                    (ALIVE.start)(user_storage,render_storage);
                },
            ),
            ui::ScreenButton::new(
                collision::AabbCentred {
                    position: (0.0, 0.3),
                    size: (screen_width / 2.0, 0.2),
                },
                (1.0 * ui::TEXT_SPRITE_SIZE.0, 3.0 * ui::TEXT_SPRITE_SIZE.1),
                [1.0, 0.0, 1.0, 1.0],
                [0.0, 1.0, 1.0, 1.0],
                ALIVE.start,
            ),
            ui::ScreenButton::new(
                collision::AabbCentred {
                    position: (0.0, 0.6),
                    size: (screen_width / 2.0, 0.2),
                },
                (2.0 * ui::TEXT_SPRITE_SIZE.0, 3.0 * ui::TEXT_SPRITE_SIZE.1),
                [1.0, 0.0, 1.0, 1.0],
                [0.0, 1.0, 1.0, 1.0],
                ALIVE.start,
            ),
            ui::ScreenButton::new(
                collision::AabbCentred {
                    position: (0.0, 0.9),
                    size: (screen_width / 2.0, 0.2),
                },
                (3.0 * ui::TEXT_SPRITE_SIZE.0, 3.0 * ui::TEXT_SPRITE_SIZE.1),
                [1.0, 0.0, 1.0, 1.0],
                [0.0, 1.0, 1.0, 1.0],
                ALIVE.start,
            ),
        ];
    },
    on_cursor_moved: |user_storage: &mut events::UserStorage,
                      render_storage: &mut events::RenderStorage,
                      position: PhysicalPosition<f64>| {
        let mouse_position = (
            events::rerange(
                (0.0, render_storage.window_size[0] as f32),
                (-1.0, 1.0),
                position.x as f32,
            ) / render_storage.aspect_ratio,
            events::rerange(
                (0.0, render_storage.window_size[1] as f32),
                (-1.0, 1.0),
                position.y as f32,
            ),
        );

        ui::hover_screen_buttons(
            render_storage,
            &mut user_storage.screen_buttons,
            mouse_position,
        )
    },
    on_mouse_input: |user_storage: &mut events::UserStorage,
                     render_storage: &mut events::RenderStorage,
                     state: ElementState,
                     button: MouseButton| {
        println!("{:?}, {:?}", state, button);

        if state == ElementState::Released && button == MouseButton::Left {
            ui::process_hovered_screen_buttons(user_storage, render_storage);//, user_storage.screen_buttons);
        }
    },
};

pub const ALIVE: MenuData = MenuData {
    start: |user_storage: &mut events::UserStorage, render_storage: &mut events::RenderStorage| {
        let mut rng = thread_rng();

        let seed_range = Uniform::new(0u32, 1000);
        let player_size_range = Uniform::new(0.25, 10.0);

        if Bernoulli::new(0.1).unwrap().sample(&mut rng) {
            user_storage.player.aabb.size = (
                player_size_range.sample(&mut rng),
                player_size_range.sample(&mut rng),
            )
        } else {
            user_storage.player.aabb.size = (0.7, 0.7);
        }

        user_storage.main_seed = seed_range.sample(&mut rng);

        user_storage.biome_noise = (
            OpenSimplex::new(seed_range.sample(&mut rng)),
            OpenSimplex::new(seed_range.sample(&mut rng)),
        );

        user_storage.chunks_generated = vec![false; events::CHUNK_GRID_WIDTH_SQUARED as usize];

        user_storage.map_objects = [
            // * scale**2
            vec![biomes::MapObject::None; events::FULL_GRID_WIDTH_SQUARED as usize],
            vec![biomes::MapObject::None; events::FULL_GRID_WIDTH_SQUARED as usize * 4],
            vec![biomes::MapObject::None; events::FULL_GRID_WIDTH_SQUARED as usize * 9],
        ];

        let safe_position = events::get_safe_position(user_storage);

        user_storage.player.aabb.position = (safe_position.0 as f32, safe_position.1 as f32);
        user_storage.player.previous_position = user_storage.player.aabb.position;

        let starting_chunk = (
            safe_position.0 / events::CHUNK_WIDTH as u32,
            safe_position.1 / events::CHUNK_WIDTH as u32,
        );

        events::generate_chunk(&user_storage, starting_chunk);

        user_storage.chunks_generated
            [events::index_from_position(starting_chunk, events::CHUNK_GRID_WIDTH) as usize] = true;

        user_storage.player.sprinting = false;

        user_storage.player.statistics = biomes::Statistics {
            strength: 1,
            health: 1,
            stamina: 100,
        };

        render_storage.camera.position = user_storage.player.aabb.position;

        user_storage.fixed_time_passed = render_storage.starting_time.elapsed().as_secs_f32();
        user_storage.wasd_held = (false, false, false, false);

        user_storage.map_objects[0][events::full_index_from_full_position((10, 10), 1)] =
            biomes::MapObject::RandomPattern(0);

        user_storage.map_objects[0][events::full_index_from_full_position((7, 8), 1)] =
            biomes::MapObject::RandomPattern(1);

        user_storage.map_objects[0][events::full_index_from_full_position((5, 8), 1)] =
            biomes::MapObject::RandomPattern(2);

        (ALIVE.on_window_resize)(user_storage, render_storage);

        // all lines below this one, and before the end of the function, should be removed, they are debug
        user_storage.player.aabb.position.0 = 15.0;
        user_storage.player.aabb.position.1 = 15.0;
        user_storage.player.previous_position = user_storage.player.aabb.position;
    },
    update: |user_storage: &mut events::UserStorage,
             render_storage: &mut events::RenderStorage,
             delta_time: f32,
             average_fps: f32| {
        let seconds_since_start = render_storage.starting_time.elapsed().as_secs_f32();

        let mut substeps = 0;

        while user_storage.fixed_time_passed < seconds_since_start {
            events::fixed_update(user_storage, render_storage);
            user_storage.fixed_time_passed += events::FIXED_UPDATE_TIME_STEP;

            substeps += 1;

            if substeps > events::MAX_SUBSTEPS {
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

        render_storage.camera.position = user_storage.player.aabb.position;

        render_storage.vertex_count_map = 0;
        render_storage.index_count_map = 0;

        match user_storage.multithread_rendering {
            true => {
                let (render_sender, render_receiver): (
                    Sender<(Vec<vertex_data::MapVertex>, u32, Vec<u32>, u32)>,
                    Receiver<(Vec<vertex_data::MapVertex>, u32, Vec<u32>, u32)>,
                ) = mpsc::channel();

                for detail_index in 0..user_storage.details.len() {
                    #[allow(deprecated)]
                    events::render_map(
                        user_storage,
                        render_storage,
                        detail_index as u8,
                        &render_sender,
                    );
                }

                drop(render_sender);

                for render_data in render_receiver {
                    //println!("{:?}", render_data.0);
                    render_storage.vertices_map[render_storage.vertex_count_map as usize
                        ..render_storage.vertex_count_map as usize + render_data.1 as usize]
                        .copy_from_slice(&render_data.0[0..render_data.1 as usize]);
                    render_storage.indices_map[render_storage.index_count_map as usize
                        ..render_storage.index_count_map as usize + render_data.3 as usize]
                        .copy_from_slice(&render_data.2[0..render_data.3 as usize]);
                    render_storage.indices_map[render_storage.index_count_map as usize
                        ..render_storage.index_count_map as usize + render_data.3 as usize]
                        .iter_mut()
                        .for_each(|x| *x += render_storage.vertex_count_map);

                    render_storage.vertex_count_map += render_data.1;
                    render_storage.index_count_map += render_data.3;
                }
            }
            false => {
                for detail_index in 0..user_storage.details.len() {
                    #[allow(deprecated)]
                    events::render_map_single_threaded(
                        user_storage,
                        render_storage,
                        detail_index as u8,
                    );
                }
            }
        }

        events::render_player(user_storage, render_storage);

        render_storage.vertex_count_ui = 0;
        render_storage.index_count_ui = 0;

        if user_storage.menu == Menu::Dead {
            (DEAD.on_window_resize)(user_storage, render_storage);
            return;
        }

        (ALIVE.on_window_resize)(user_storage, render_storage); // TODO: make it so I don't call this every frame...
        if user_storage.show_debug {
            let screen_width = 2.0 / render_storage.aspect_ratio;

            user_storage.screen_texts.append(&mut vec![
                ui::ScreenText::new(
                    (screen_width * -0.5 + screen_width * 0.1, 0.4),
                    (0.05, 0.1),
                    0.025,
                    format!("substeps: {}", substeps).as_str(),
                    [1.0, 0.0, 1.0, 1.0],
                ),
                ui::ScreenText::new(
                    (screen_width * -0.5 + screen_width * 0.1, 0.7),
                    (0.05, 0.1),
                    0.025,
                    format!("average_fps: {}", average_fps).as_str(),
                    [1.0, 0.0, 1.0, 1.0],
                ),
                ui::ScreenText::new(
                    (screen_width * -0.5 + screen_width * 0.1, 0.8),
                    (0.05, 0.1),
                    0.025,
                    format!("delta_time: {}", delta_time).as_str(),
                    [1.0, 0.0, 1.0, 1.0],
                ),
            ]);
        }

        ui::render_screen_texts(render_storage, &user_storage.screen_texts);

        match user_storage.generation_receiver.try_recv() {
            Ok(generation) => {
                if !(generation.1 + generation.0.len()
                    > user_storage.map_objects[generation.2 as usize].len())
                {
                    // user_storage.map_objects[generation.2 as usize]
                    //     [generation.1..generation.1 + generation.0.len()]
                    //     .copy_from_slice(generation.0.as_slice());

                    for i in 0..generation.0.len() {
                        if match user_storage.map_objects[generation.2 as usize][i + generation.1] {
                            biomes::MapObject::RandomPattern(i) => {
                                biomes::RANDOM_PATTERN_MAP_OBJECTS[i as usize].priority
                            }
                            biomes::MapObject::SimplexPattern(i) => {
                                biomes::SIMPLEX_PATTERN_MAP_OBJECTS[i as usize].priority
                            }
                            biomes::MapObject::SimplexSmoothedPattern(i, _) => {
                                biomes::SIMPLEX_SMOOTHED_PATTERN_MAP_OBJECTS[i as usize].priority
                            }
                            biomes::MapObject::None => 0,
                        } == 255
                        {
                            continue;
                        }
                        user_storage.map_objects[generation.2 as usize][i + generation.1] =
                            generation.0[i];
                    }
                } else {
                    println!("??????");
                }
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {
                panic!("Something got disconnected from the chunk receivers and senders!")
            }
        }

        for x in -1..2 {
            for y in -1..2 {
                let player_chunk_position_dangerous = (
                    user_storage.player.aabb.position.0 as i32 / events::CHUNK_WIDTH as i32 + x,
                    user_storage.player.aabb.position.1 as i32 / events::CHUNK_WIDTH as i32 + y,
                );

                if player_chunk_position_dangerous.0 < 0 || player_chunk_position_dangerous.1 < 0 {
                    continue;
                }

                let player_chunk_position = (
                    player_chunk_position_dangerous.0 as u32,
                    player_chunk_position_dangerous.1 as u32,
                );

                let player_chunk_index =
                    events::index_from_position(player_chunk_position, events::CHUNK_GRID_WIDTH)
                        as usize;

                if player_chunk_index >= user_storage.chunks_generated.len() {
                    continue;
                }

                if !user_storage.chunks_generated[player_chunk_index] {
                    events::generate_chunk(&user_storage, player_chunk_position);
                    user_storage.chunks_generated[player_chunk_index] = true;
                }
            }
        }
    },
    end: |_user_storage: &mut events::UserStorage, _render_storage: &mut events::RenderStorage| {},
    on_keyboard_input: |user_storage: &mut events::UserStorage,
                        render_storage: &mut events::RenderStorage,
                        input: KeyboardInput| {
        if let Some(key_code) = input.virtual_keycode {
            match key_code {
                VirtualKeyCode::W => user_storage.wasd_held.0 = events::is_pressed(input.state),
                VirtualKeyCode::A => user_storage.wasd_held.1 = events::is_pressed(input.state),
                VirtualKeyCode::S => user_storage.wasd_held.2 = events::is_pressed(input.state),
                VirtualKeyCode::D => user_storage.wasd_held.3 = events::is_pressed(input.state),
                VirtualKeyCode::F => {
                    if events::is_pressed(input.state) {
                        user_storage.player.sprinting = !user_storage.player.sprinting;
                    }
                }
                VirtualKeyCode::R => {
                    if events::is_pressed(input.state) {
                        match user_storage.chunk_generation {
                            0 => {
                                events::generate_chunk_old(
                                    &user_storage,
                                    (
                                        (user_storage.player.aabb.position.0
                                            / events::CHUNK_WIDTH as f32)
                                            .floor() as u32,
                                        (user_storage.player.aabb.position.1
                                            / events::CHUNK_WIDTH as f32)
                                            .floor() as u32,
                                    ),
                                );
                            }

                            1 => {
                                events::generate_chunk(
                                    &user_storage,
                                    (
                                        (user_storage.player.aabb.position.0
                                            / events::CHUNK_WIDTH as f32)
                                            .floor() as u32,
                                        (user_storage.player.aabb.position.1
                                            / events::CHUNK_WIDTH as f32)
                                            .floor() as u32,
                                    ),
                                );
                            }

                            _ => {}
                        }
                    }
                }
                VirtualKeyCode::E => {
                    if events::is_pressed(input.state) {
                        user_storage.player.collision_debug = !user_storage.player.collision_debug;
                    }
                }
                VirtualKeyCode::Up => user_storage.zoom_held.0 = events::is_pressed(input.state),
                VirtualKeyCode::Down => user_storage.zoom_held.1 = events::is_pressed(input.state),

                VirtualKeyCode::V => {
                    if events::is_pressed(input.state) {
                        user_storage.show_debug = !user_storage.show_debug;
                        (ALIVE.on_window_resize)(user_storage, render_storage);
                    }
                }

                VirtualKeyCode::G => {
                    if events::is_pressed(input.state) {
                        user_storage.multithread_rendering = !user_storage.multithread_rendering;
                    }
                }

                VirtualKeyCode::Minus => {
                    if events::is_pressed(input.state) {
                        user_storage.chunk_generation -= 1;
                    }
                }
                VirtualKeyCode::Equals => {
                    if events::is_pressed(input.state) {
                        user_storage.chunk_generation += 1;
                    }
                }
                VirtualKeyCode::Escape => {
                    if events::is_pressed(input.state) {
                        user_storage.menu = Menu::Paused;
                        (PAUSED.on_window_resize)(user_storage, render_storage);
                    }
                }
                _ => (),
            }
        }
    },
    on_window_resize: |user_storage: &mut events::UserStorage,
                       render_storage: &mut events::RenderStorage| {
        let screen_width = 2.0 / render_storage.aspect_ratio;

        user_storage.screen_texts = vec![
            ui::ScreenText::new(
                (screen_width * -0.5 + screen_width * 0.1, -0.8),
                (0.05, 0.1),
                0.025,
                format!("Health: {}", user_storage.player.statistics.health).as_str(),
                [1.0, 0.0, 1.0, 1.0],
            ),
            ui::ScreenText::new(
                (screen_width * -0.5 + screen_width * 0.1, -0.7),
                (0.05, 0.1),
                0.025,
                format!("Stamina: {}%", user_storage.player.statistics.stamina).as_str(),
                [1.0, 0.0, 1.0, 1.0],
            ),
            ui::ScreenText::new(
                (screen_width * -0.5 + screen_width * 0.1, -0.6),
                (0.05, 0.1),
                0.025,
                format!("Strength: {}", user_storage.player.statistics.strength).as_str(),
                [1.0, 0.0, 1.0, 1.0],
            ),
        ];

        if user_storage.show_debug {
            user_storage.screen_texts.append(&mut vec![
                ui::ScreenText::new(
                    (screen_width * -0.5 + screen_width * 0.1, 0.2),
                    (0.05, 0.1),
                    0.025,
                    format!("chunk_generation: {}", user_storage.chunk_generation).as_str(),
                    [1.0, 0.0, 1.0, 1.0],
                ),
                ui::ScreenText::new(
                    (screen_width * -0.5 + screen_width * 0.1, 0.3),
                    (0.05, 0.1),
                    0.025,
                    format!(
                        "multithread_rendering: {}",
                        user_storage.multithread_rendering
                    )
                    .as_str(),
                    [1.0, 0.0, 1.0, 1.0],
                ),
                ui::ScreenText::new(
                    (screen_width * -0.5 + screen_width * 0.1, 0.5),
                    (0.05, 0.1),
                    0.025,
                    format!(
                        "player.position: ({},{})",
                        user_storage.player.aabb.position.0, user_storage.player.aabb.position.1
                    )
                    .as_str(),
                    [1.0, 0.0, 1.0, 1.0],
                ),
                ui::ScreenText::new(
                    (screen_width * -0.5 + screen_width * 0.1, 0.6),
                    (0.05, 0.1),
                    0.025,
                    format!(
                        "player.size: ({},{})",
                        user_storage.player.aabb.size.0, user_storage.player.aabb.size.1
                    )
                    .as_str(),
                    [1.0, 0.0, 1.0, 1.0],
                ),
            ])
        }
    },
    on_cursor_moved: |_user_storage: &mut events::UserStorage,
                      _render_storage: &mut events::RenderStorage,
                      _position: PhysicalPosition<f64>| {},
    on_mouse_input: |_user_storage: &mut events::UserStorage,
                     _render_storage: &mut events::RenderStorage,
                     _state: ElementState,
                     _button: MouseButton| {},
};

pub const PAUSED: MenuData = MenuData {
    start: |_user_storage: &mut events::UserStorage,
            _render_storage: &mut events::RenderStorage| {},
    update: |user_storage: &mut events::UserStorage,
             render_storage: &mut events::RenderStorage,
             _delta_time: f32,
             _average_fps: f32| {
        render_storage.vertex_count_ui = 0;
        render_storage.index_count_ui = 0;

        ui::render_screen_texts(render_storage, &user_storage.screen_texts);
    },
    end: |_user_storage: &mut events::UserStorage, _render_storage: &mut events::RenderStorage| {},
    on_keyboard_input: |user_storage: &mut events::UserStorage,
                        render_storage: &mut events::RenderStorage,
                        input: KeyboardInput| {
        if let Some(key_code) = input.virtual_keycode {
            match key_code {
                VirtualKeyCode::Escape => {
                    if events::is_pressed(input.state) {
                        user_storage.menu = Menu::Alive;
                        user_storage.fixed_time_passed =
                            render_storage.starting_time.elapsed().as_secs_f32();
                        user_storage.wasd_held = (false, false, false, false);
                    }
                }
                _ => (),
            }
        }
    },
    on_window_resize: |user_storage: &mut events::UserStorage,
                       render_storage: &mut events::RenderStorage| {
        let screen_width = 2.0 / render_storage.aspect_ratio;

        user_storage.screen_texts = vec![ui::ScreenText::new(
            (screen_width * -0.5 + screen_width * 0.1, -0.5),
            (0.25, 0.5),
            0.125,
            "Paused!",
            [1.0, 0.0, 1.0, 1.0],
        )];
    },
    on_cursor_moved: |_user_storage: &mut events::UserStorage,
                      _render_storage: &mut events::RenderStorage,
                      _position: PhysicalPosition<f64>| {},
    on_mouse_input: |_user_storage: &mut events::UserStorage,
                     _render_storage: &mut events::RenderStorage,
                     _state: ElementState,
                     _button: MouseButton| {},
};

pub const DEAD: MenuData = MenuData {
    start: |_user_storage: &mut events::UserStorage,
            _render_storage: &mut events::RenderStorage| {},
    update: |user_storage: &mut events::UserStorage,
             render_storage: &mut events::RenderStorage,
             _delta_time: f32,
             _average_fps: f32| {
        render_storage.vertex_count_ui = 0;
        render_storage.index_count_ui = 0;

        ui::render_screen_texts(render_storage, &user_storage.screen_texts)
    },
    end: |_user_storage: &mut events::UserStorage, _render_storage: &mut events::RenderStorage| {},
    on_keyboard_input: |user_storage: &mut events::UserStorage,
                        render_storage: &mut events::RenderStorage,
                        input: KeyboardInput| {
        if let Some(key_code) = input.virtual_keycode {
            match key_code {
                VirtualKeyCode::Return => {
                    if events::is_pressed(input.state) {
                        user_storage.menu = Menu::Alive;
                        (ALIVE.start)(user_storage, render_storage);
                    }
                }
                _ => (),
            }
        }
    },
    on_window_resize: |user_storage: &mut events::UserStorage,
                       render_storage: &mut events::RenderStorage| {
        let screen_width = 2.0 / render_storage.aspect_ratio;

        user_storage.screen_texts = vec![ui::ScreenText::new(
            (screen_width * -0.5 + screen_width * 0.1, -0.5),
            (0.25, 0.5),
            0.125,
            "Dead! Press Enter!",
            [1.0, 0.0, 1.0, 1.0],
        )];
    },

    on_cursor_moved: |_user_storage: &mut events::UserStorage,
                      _render_storage: &mut events::RenderStorage,
                      _position: PhysicalPosition<f64>| {},
    on_mouse_input: |_user_storage: &mut events::UserStorage,
                     _render_storage: &mut events::RenderStorage,
                     _state: ElementState,
                     _button: MouseButton| {},
};
