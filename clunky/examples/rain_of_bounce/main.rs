use std::collections::HashMap;

use clunky::{
    lost_code::{is_pressed, FixedUpdate, MaxSubsteps},
    math::{add_3d, Matrix4},
    physics::physics_3d::{
        aabb::AabbCentredOrigin,
        //bodies::{Body, ImmovableCuboid},
        solver::{self, CpuSolver},
        verlet::Particle,
    },
    shaders::instanced_simple_lit_colour_3d::{self, Camera},
};
use common_renderer::{CommonRenderer, WindowConfig};
use engine::{EngineEvent, PhysicsEvent, SimpleEngine};
use rand::{thread_rng, Rng};
use winit::{
    dpi::PhysicalPosition,
    event::{DeviceEvent, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
    window::{Fullscreen, WindowId},
};

use body::{Body, Creature as CreatureBody};

use creature_types::{Burgle, CreatureType};

mod body;
mod common_renderer;
mod creature_types;
mod engine;

type Engine = SimpleEngine<CommonRenderer<Body>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct CreatureIndex(usize);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct BodyIndex(usize);

const GRID_MIN: [isize; 3] = [-200, -3, -200];
const GRID_MAX: [isize; 3] = [200, 10, 200];
const GRID_SUBDIVISIONS: [usize; 3] = [15, 1, 15];

const FIXED_DELTA_TIME: f64 = 0.03;

struct CreaturesManager {
    creature_controlled_by_window: HashMap<WindowId, CreatureIndex>,
    creature_selection_window: Option<WindowId>,

    creatures: Vec<CreatureType>,
    captured_creatures: Vec<CreatureIndex>,
}

struct Settings {
    mouse_sensitivity: f32,
}

struct Game {
    settings: Settings,

    creatures_manager: CreaturesManager,

    paused: bool,
    window_focused: Option<WindowId>,

    capture_key_held: bool,
    // Usize is index into bodies within capture range. BodyIndex is to make certain that it is still the same body, and hasn't accidentally changed.
    capture_index_selected: Option<(usize, BodyIndex)>,
    bodies_within_capture_range: Option<Vec<BodyIndex>>,
}

impl Game {
    fn new() -> Self {
        Game {
            settings: Settings {
                mouse_sensitivity: 1.0,
            },

            creatures_manager: CreaturesManager {
                creature_controlled_by_window: HashMap::new(),
                creature_selection_window: None,

                creatures: vec![],
                captured_creatures: vec![],
            },

            paused: false,
            window_focused: None,

            capture_key_held: false,
            capture_index_selected: None,
            bodies_within_capture_range: None,
        }
    }

    fn focused_creature(&self) -> &CreatureType {
        let focused_window = self.window_focused.unwrap();

        let creature_index = *self
            .creatures_manager
            .creature_controlled_by_window
            .get(&focused_window)
            .unwrap();

        &self.creatures_manager.creatures[creature_index.0]
    }

    fn focused_creature_mut(&mut self) -> &mut CreatureType {
        let focused_window = self.window_focused.unwrap();

        let creature_index = *self
            .creatures_manager
            .creature_controlled_by_window
            .get(&focused_window)
            .unwrap();

        &mut self.creatures_manager.creatures[creature_index.0]
    }
}

fn main() {
    println!("TODO:\nStore which grid actually have collision in them, so you don't have to loop over them all.");

    let mut game = Game::new();

    let physics_config = solver::Config {
        ..solver::Config::size_from_min_max_with_subdivisions(GRID_MIN, GRID_MAX, GRID_SUBDIVISIONS)
    };

    let physics_simulation = CpuSolver::new(physics_config);

    let renderer_config = common_renderer::Config {
        starting_windows: vec![WindowConfig {
            camera: Default::default(),
            window_descriptor: Default::default(),
            swapchain_create_info_modify: |_| {},
        }],
        ..Default::default()
    };

    let config = engine::Config {
        physics_fixed_update: FixedUpdate::new(FIXED_DELTA_TIME, MaxSubsteps::WarnAt(100)),
        renderer_config,
        close_everything_on_window_close: false,
        ..Default::default()
    };

    let mut engine: Engine = SimpleEngine::new(config, physics_simulation);

    let temp_event_loop = engine.temporary_event_loop_storage.take().unwrap();

    game.creatures_manager
        .creatures
        .push(CreatureType::Burgle(Burgle::new(
            &mut engine,
            [0.0; 3],
            [0.5, 1.0, 0.5],
            [1.5; 3],
            [1.0, 0.0, 1.0, 1.0],
            CreatureIndex(game.creatures_manager.creatures.len()),
        )));

    game.creatures_manager
        .captured_creatures
        .push(CreatureIndex(0));

    game.creatures_manager.creature_controlled_by_window.insert(
        engine.windows_manager.primary_window_id().unwrap(),
        CreatureIndex(0),
    );

    // 2 player test
    game.creatures_manager
        .creatures
        .push(CreatureType::Burgle(Burgle::new(
            &mut engine,
            [0.0; 3],
            [0.5, 1.0, 0.5],
            [1.5; 3],
            [1.0, 1.0, 0.0, 1.0],
            CreatureIndex(game.creatures_manager.creatures.len()),
        )));

    /*
    game.creatures_manager
        .captured_creatures
        .push(CreatureIndex(1));
    */

    let mut starting_renderer = common_renderer::Methods::new(&mut engine);

    starting_renderer.add_cuboid_colour(instanced_simple_lit_colour_3d::Instance::new(
        [1.0; 4],
        Matrix4::from_translation([GRID_MIN[0] as f32, GRID_MIN[1] as f32, GRID_MIN[2] as f32]),
    ));

    starting_renderer.add_cuboid_colour(instanced_simple_lit_colour_3d::Instance::new(
        [1.0; 4],
        Matrix4::from_translation([GRID_MAX[0] as f32, GRID_MAX[1] as f32, GRID_MAX[2] as f32]),
    ));

    starting_renderer.add_cuboid_colour(instanced_simple_lit_colour_3d::Instance::new(
        [1.0; 4],
        Matrix4::from_translation([2.0, 0.0, 0.0]),
    ));

    /*
    let second_player_window =
        starting_renderer.create_window(WindowConfig::default(), &temp_event_loop);

    game.creatures_manager
        .creature_controlled_by_window
        .insert(second_player_window, CreatureIndex(1));
    */

    engine.temporary_event_loop_storage = Some(temp_event_loop);

    let floor = AabbCentredOrigin {
        position: [0.0, 1.0, 0.0],
        half_size: [
            (GRID_MAX[0] - GRID_MIN[0]) as f32 * 0.5,
            0.5,
            (GRID_MAX[2] - GRID_MIN[2]) as f32 * 0.5,
        ],
    };
    engine
        .physics
        .bodies
        .push(Body::ImmovableCuboid(floor.clone()));

    let mut starting_renderer = common_renderer::Methods::new(&mut engine);
    starting_renderer.add_cuboid_colour_from_aabb(floor, [1.0, 0.0, 1.0, 1.0]);

    engine.run(move |event, event_loop, control_flow, engine| match event {
        EngineEvent::WinitEvent(Event::WindowEvent {
            window_id,
            event: WindowEvent::CloseRequested,
            ..
        }) => {
            let mut renderer = common_renderer::Methods::new(engine);
            renderer.remove_window(window_id);

            if let Some(creature_selection_window) =
                game.creatures_manager.creature_selection_window
            {
                if creature_selection_window == window_id {
                    game.creatures_manager.creature_selection_window = None;
                    return;
                }
            }

            game.creatures_manager
                .creature_controlled_by_window
                .remove(&window_id);
        }

        EngineEvent::WinitEvent(Event::MainEventsCleared) => {
            on_update(&mut game, engine)
        }

        EngineEvent::PhysicsEvent(PhysicsEvent::BeforeTick) => {
            on_physics_fixed_update_before_physics_tick(&mut game, engine)
        }

        EngineEvent::PhysicsEvent(PhysicsEvent::AfterTick) => {
            on_physics_fixed_update_after_physics_tick(&mut game, engine)
        }

        EngineEvent::WinitEvent(Event::WindowEvent {
            event: WindowEvent::KeyboardInput { input, .. },
            ..
        }) => {
            on_keyboard_input(input, control_flow, engine, &mut game, event_loop);
        }

        EngineEvent::WinitEvent(Event::WindowEvent {
            window_id,
            event: WindowEvent::Focused(focus),
            ..
        }) => {
            if focus {
                game.window_focused = Some(window_id);
            } else {
                game.window_focused = None;
            }
        }

        EngineEvent::WinitEvent(Event::DeviceEvent {
            event: DeviceEvent::Motion { axis, value },
            ..
        }) => {
            let Some(window_focused) = game.window_focused else {
                return;
            };

            let window_renderer = engine
                .windows_manager
                .get_renderer_mut(window_focused)
                .unwrap();

            if game.paused {
                window_renderer.window().set_cursor_visible(true);
                return;
            }

            let value = value as f32 * game.settings.mouse_sensitivity;

            match axis {
                0 => game.focused_creature_mut().on_mouse_movement([0.0, value]),
                1 => game.focused_creature_mut().on_mouse_movement([value, 0.0]),
                _ => unreachable!("We do not support mice with more than 2 dimensions. I don't even know if they exist.")
            }

            let window_extent = window_renderer.window_size();

            window_renderer
                .window()
                .set_cursor_position(PhysicalPosition::new(
                    window_extent[0] / 2.0,
                    window_extent[1] / 2.0,
                ))
                .unwrap();
            window_renderer.window().set_cursor_visible(false);
        }

        EngineEvent::WinitEvent(Event::DeviceEvent {
            event: DeviceEvent::MouseWheel { delta: _ },
            ..
        }) => {}

        _ => (),
    })
}

// TODO: because we can't access Game in this function, we can either have a custom event enum for the engine, or we can just add a generic to the engine, and pass any random struct in, for use here or elsewhere.
fn on_physics_fixed_update_before_physics_tick(game: &mut Game, engine: &mut Engine) {
    let Some(window_focused) = game.window_focused else {
        return;
    };

    let Some(creature_index) = game
        .creatures_manager
        .creature_controlled_by_window
        .get(&window_focused)
    else {
        return;
    };

    let creature = &mut game.creatures_manager.creatures[creature_index.0];

    creature.on_physics_fixed_update_before_physics_tick_when_focused(&mut engine.physics.bodies);

    /*
    let motion = match game.movement.wasd_held {
        [true, false, false, false] => (0.0, -1.0),
        [false, false, true, false] => (0.0, 1.0),
        [false, false, false, true] => (1.0, 0.0),
        [false, true, false, false] => (-1.0, 0.0),

        [true, true, false, false] => (-0.7, -0.7),
        [true, false, false, true] => (0.7, -0.7),

        [false, true, true, false] => (-0.7, 0.7),
        [false, false, true, true] => (0.7, 0.7),

        _ => (0.0, 0.0),
    };

    /*
    let speed = match (sprinting, *jump_held, player.grounded) {
        (false, true, true) | (false, false, true) | (false, true, false) => 25.0,
        (true, true, true) | (true, false, true) | (false, false, false) | (true, true, false) => {
            50.0
        }
        (true, false, false) => 100.0,
    };
    */

    let speed = match game.movement.sprinting {
        true => 50.0,
        false => 25.0,
    };

    let real_motion = (-motion.0 * speed, motion.1 * speed);

    let camera = camera(window_focused, engine);

    let y_rotation_cos = camera.rotation[1].to_radians().cos();
    let y_rotation_sin = camera.rotation[1].to_radians().sin();

    let real_motion = (
        real_motion.0 * y_rotation_cos - real_motion.1 * y_rotation_sin,
        real_motion.1 * y_rotation_cos + real_motion.0 * y_rotation_sin,
    );

    //let creature = creature(*creature_index, engine);
    let creature = game.creatures_manager.creatures[creature_index.0];

    // Replace this garbage, as the creature might have different speeds, or just entirely different controls.
    creature.accelerate([real_motion.0 as f32, 0.0, real_motion.1 as f32], &mut engine.physics.bodies);

    if game.movement.jump_held {
        //if game.player().grounded {
        if true {
            //game.player().particle.accelerate([0.0, -500.0, 0.0]);
            creature.particle.accelerate([0.0, -100.0, 0.0]);
        }
    }

    let horizontal_dampening = if creature.grounded { 0.8 } else { 0.95 }; // grounded originally 0.8

    creature.dampening = [horizontal_dampening, 1.0, horizontal_dampening];
    // y 0.98 originally
    */
}

fn on_physics_fixed_update_after_physics_tick(game: &mut Game, engine: &mut Engine) {}

fn on_update(game: &mut Game, engine: &mut Engine) {
    if game.capture_key_held {
        if let Some(focused_window) = game.window_focused {
            // Accounts for it perhaps being the creature selection window.
            if let Some(creature_index) = game
                .creatures_manager
                .creature_controlled_by_window
                .get(&focused_window)
            {
                let creature = &game.creatures_manager.creatures[creature_index.0];

                let mut bodies_within_capture_range =
                    creature.get_bodies_within_capture_range(&engine.physics.bodies);
                let mut uncaptured_bodies_within_capture_range =
                    Vec::with_capacity(bodies_within_capture_range.len());

                for body_index in bodies_within_capture_range {
                    if !game
                        .creatures_manager
                        .captured_creatures
                        .contains(&engine.physics.bodies[body_index.0].owner().unwrap())
                    {
                        uncaptured_bodies_within_capture_range.push(body_index);
                    }
                }

                bodies_within_capture_range = uncaptured_bodies_within_capture_range;
                bodies_within_capture_range.sort();

                // Check if the index.0 is invalid in any way, then always search to see if index.1 is still in the bodies, and if so, correct index.0
                if let Some(capture_index_selected) = game.capture_index_selected {
                    let mut invalid_index = false;

                    if capture_index_selected.0 >= bodies_within_capture_range.len() {
                        invalid_index = true;
                    } else if bodies_within_capture_range[capture_index_selected.0]
                        != capture_index_selected.1
                    {
                        invalid_index = true;
                    }

                    if invalid_index {
                        if let Ok(new_index) =
                            bodies_within_capture_range.binary_search(&capture_index_selected.1)
                        {
                            game.capture_index_selected =
                                Some((new_index, capture_index_selected.1));
                        } else {
                            game.capture_index_selected = None;
                        }
                    }
                }

                game.bodies_within_capture_range = Some(bodies_within_capture_range);
            }
        }
    }

    for (window_id, creature_index) in &game.creatures_manager.creature_controlled_by_window {
        let creature = &game.creatures_manager.creatures[creature_index.0];

        // Really unfortunate.
        let mut cloned_camera = camera(*window_id, engine).clone();
        creature.update_camera(&mut cloned_camera, &engine.physics.bodies);

        let camera = camera(*window_id, engine);
        *camera = cloned_camera;

        camera.light_position[0] = camera.position[0];
        camera.light_position[2] = camera.position[2];
    }
}

enum Action {
    HorizontalMovement([f32; 2]),
    VerticalMovement(f32),

    Capture(bool),

    SpeedModifier(f32),

    PrimaryInteract,
    SecondaryInteract,

    MoveSelection(i8),
    // Add unknown with some type that could take keyboard inputs and other input types. Perhaps enum?
}

impl Action {
    //fn from_keyboard_input(input: KeyboardInput, manager: InputManager) -> Self {

    //}
}

fn on_keyboard_input(
    input: KeyboardInput,
    control_flow: &mut ControlFlow,
    engine: &mut Engine,
    game: &mut Game,
    event_loop: &EventLoopWindowTarget<()>,
) {
    if let Some(key_code) = input.virtual_keycode {
        let focused_window = game.window_focused.unwrap();
        let window_renderer = engine
            .windows_manager
            .get_renderer_mut(focused_window)
            .unwrap();

        let focused_creature_index = *game
            .creatures_manager
            .creature_controlled_by_window
            .get(&focused_window)
            .unwrap();

        game.creatures_manager.creatures[focused_creature_index.0].on_keyboard_input(input);

        match key_code {
            VirtualKeyCode::Backslash => {
                if is_pressed(input.state) {
                    if let None = window_renderer.window().fullscreen() {
                        window_renderer
                            .window()
                            .set_fullscreen(Some(Fullscreen::Borderless(None)));
                    } else {
                        window_renderer.window().set_fullscreen(None);
                    }
                }
            }

            VirtualKeyCode::Tab => {
                if game.capture_key_held && is_pressed(input.state) {
                    let Some(bodies_within_capture_range) = &game.bodies_within_capture_range
                    else {
                        println!("This seems impossible, but perhaps it isn't.");
                        return;
                    };
                    println!(
                        "bodies_within_capture_range: {:?}",
                        bodies_within_capture_range
                    );

                    if bodies_within_capture_range.len() != 0 {
                        if let Some(ref mut capture_index_selected) =
                            &mut game.capture_index_selected
                        {
                            // Some way to backwards would be nice?
                            capture_index_selected.0 += 1;
                            capture_index_selected.0 %= bodies_within_capture_range.len() + 1;

                            if capture_index_selected.0 == bodies_within_capture_range.len() {
                                game.capture_index_selected = None;
                            } else {
                                capture_index_selected.1 =
                                    bodies_within_capture_range[capture_index_selected.0];
                            }
                        } else {
                            game.capture_index_selected = Some((0, bodies_within_capture_range[0]));
                        }

                        println!("capture_index_selected: {:?}", game.capture_index_selected);
                    }
                }
            }

            VirtualKeyCode::C => {
                game.capture_key_held = is_pressed(input.state);

                if !game.capture_key_held {
                    println!("Capture key released!");
                    println!("capture_index_selected: {:?}", game.capture_index_selected);

                    if let Some(capture_index_selected) = game.capture_index_selected {
                        let capture_creature_index = engine.physics.bodies
                            [capture_index_selected.1 .0]
                            .owner()
                            .unwrap();

                        // Wrangle the borrow checker into lettings us mutably attempt_capture on the capture creature and mutably give it the spreader.
                        // This feels like it could be simplified.
                        let (capture, spreader) =
                            if focused_creature_index.0 > capture_creature_index.0 {
                                let (lhs, rhs) = game
                                    .creatures_manager
                                    .creatures
                                    .split_at_mut(focused_creature_index.0);

                                (&mut lhs[capture_creature_index.0], &mut rhs[0])
                            } else {
                                let (lhs, rhs) = game
                                    .creatures_manager
                                    .creatures
                                    .split_at_mut(capture_creature_index.0);

                                (&mut rhs[0], &mut lhs[focused_creature_index.0])
                            };

                        if capture.attempt_capture(
                            spreader,
                            capture_index_selected.1,
                            &mut engine.physics.bodies,
                        ) {
                            game.creatures_manager
                                .captured_creatures
                                .push(capture_creature_index);

                            let new_window = renderer(engine)
                                .create_window(WindowConfig::default(), &event_loop);

                            game.creatures_manager
                                .creature_controlled_by_window
                                .insert(new_window, capture_creature_index);
                        }
                    }

                    game.capture_index_selected = None;
                    game.bodies_within_capture_range = None;
                } else {
                    //println!("Capture key pressed!");
                }
            }

            VirtualKeyCode::Delete => {
                if is_pressed(input.state) {
                    *control_flow = ControlFlow::Exit;
                }
            }

            VirtualKeyCode::X => {
                if is_pressed(input.state) {
                    println!("fps: {}", engine.fps_tracker().average_fps());
                    println!("bodies: {}", engine.physics.bodies.len());
                }
            }

            VirtualKeyCode::P => {
                if is_pressed(input.state) {
                    // Unwrapping should be safe, as you have to be focused to input a keycode.
                    let window_id = game.window_focused.unwrap();

                    if Some(window_id) == game.creatures_manager.creature_selection_window {
                        return;
                    }

                    let creature_index = *game
                        .creatures_manager
                        .creature_controlled_by_window
                        .get(&window_id)
                        .unwrap();

                    println!("Creature Index: {}", creature_index.0);

                    //println!("creature: {:?}", creature(creature_index, engine));
                    //println!("collisions: {:?}", collisions(creature_index, engine));

                    println!("camera: {:?}", camera(window_id, engine));
                }
            }

            VirtualKeyCode::Escape => {
                if is_pressed(input.state) {
                    game.paused = !game.paused;
                }
            }

            VirtualKeyCode::T => {
                for _ in 0..500 {
                    let index = engine.physics.bodies.len();

                    let mut rng = thread_rng();

                    engine.physics.bodies.push(Body::Creature(CreatureBody {
                        particle: Particle::from_position([
                            rng.gen_range(GRID_MIN[0] as f32..GRID_MAX[0] as f32),
                            -10.0,
                            rng.gen_range(GRID_MIN[2] as f32..GRID_MAX[2] as f32),
                        ]),
                        half_size: [0.5, 0.5, 0.5],

                        mass: 1.0,
                        dampening: [0.9, 1.0, 0.9],

                        grounded: false,

                        // This is allowed to be wrong. This control is just for testing physics.
                        owner: CreatureIndex(0),
                    }));

                    let mut renderer = common_renderer::Methods::new(engine);

                    renderer.add_cuboid_colour_from_body_index(index, [0.0, 1.0, 1.0, 1.0]);
                }
            }

            _ => (),
        }
    }
}

#[inline]
fn wasd_to_movement(wasd: [bool; 4]) -> [f32; 2] {
    match wasd {
        [true, false, false, false] => [0.0, -1.0],
        [false, false, true, false] => [0.0, 1.0],
        [false, false, false, true] => [-1.0, 0.0],
        [false, true, false, false] => [1.0, 0.0],

        [true, true, false, false] => [0.7, -0.7],
        [true, false, false, true] => [-0.7, -0.7],

        [false, true, true, false] => [0.7, 0.7],
        [false, false, true, true] => [-0.7, 0.7],

        _ => [0.0, 0.0],
    }
}

#[inline]
fn movement_in_direction(movement: [f32; 2], rotation_x_y: [f32; 2]) -> [f32; 2] {
    let y_rotation_cos = rotation_x_y[1].to_radians().cos();
    let y_rotation_sin = rotation_x_y[1].to_radians().sin();

    [
        movement[0] * y_rotation_cos - movement[1] * y_rotation_sin,
        movement[1] * y_rotation_cos + movement[0] * y_rotation_sin,
    ]
}

#[inline]
fn renderer(engine: &mut Engine) -> common_renderer::Methods<Body> {
    common_renderer::Methods::new(engine)
}

// fn creature(index: usize, engine: &mut Engine) -> &mut CreatureBody {
//     let Body::Creature(creature) = &mut engine.physics.bodies[index] else {
//         panic!("index: {index}")
//     };
//     creature
// }

// fn collisions(index: usize, engine: &mut Engine) -> &[usize] {
//     let Body::TriggerImmovableCuboid {
//         aabb: _,
//         collisions,
//     } = &engine.physics.bodies[index + 1]
//     else {
//         panic!("index: {index}")
//     };
//     collisions
// }

fn camera(window_id: WindowId, engine: &mut Engine) -> &mut Camera {
    &mut engine
        .renderer_storage
        .get_window_specific(window_id)
        .unwrap()
        .camera
}

/*
#![feature(vec_push_within_capacity)]
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{sync::Arc, time::Instant};

use clunky::{
    buffer_contents::{self, Colour3DInstance},
    lost_code::{is_pressed, FixedUpdate, FpsTracker, MaxSubsteps},
    math::{self, Matrix4, Radians},
    meshes,
    physics::physics_3d::{
        aabb::{AabbCentredOrigin, AabbMinMax},
        bodies::{Body, CollisionRecorderCuboid, Body},
        solver::{self, CpuSolver, OutsideOfGridBoundsBehaviour},
        verlet::{
            bodies::{Cuboid, Player},
            Particle,
        },
    },
    rendering::draw_instanced,
    shaders::colour_3d_instanced_shaders::{self, Camera},
};
use gltf::RenderObject;
use rand::{rngs::ThreadRng, thread_rng, Rng};
use vulkano::{
    buffer::{
        allocator::{SubbufferAllocator, SubbufferAllocatorCreateInfo},
        Buffer, BufferCreateInfo, BufferUsage, Subbuffer,
    },
    command_buffer::{
        allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
        RenderPassBeginInfo,
    },
    descriptor_set::{
        allocator::StandardDescriptorSetAllocator, PersistentDescriptorSet, WriteDescriptorSet,
    },
    device::Device,
    format::{ClearValue, Format},
    image::{view::ImageView, Image, ImageCreateInfo, ImageType, ImageUsage},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
    pipeline::{graphics::viewport::Viewport, GraphicsPipeline, Pipeline, PipelineBindPoint},
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass},
};
use vulkano_util::{
    context::{VulkanoConfig, VulkanoContext},
    renderer::VulkanoWindowRenderer,
    window::VulkanoWindows,
};
use winit::{
    dpi::PhysicalPosition,
    event::{DeviceEvent, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Fullscreen,
};

use vulkano::sync::GpuFuture;

mod engine;
mod gltf;

// Small secret... I have no idea how the grid in my physics engine works.
const COLLISION_GRID_SIZE: [usize; 3] = [60, 30, 60];
const COLLISION_GRID_ORIGIN: [f32; 3] = [-300.0, -200.0, -300.0];
const COLLISION_GRID_CELL_SIZE: [usize; 3] = [10, 10, 10];

const INITIAL_BODY_CAPACITY: usize = 100;

const FIXED_DELTA_TIME: f32 = 0.025; // usually I would do 0.04
const MAX_SUBSTEPS: u32 = 200;

const MAX_RAINDROPS: usize = 15_000;

fn main() {
    let context = VulkanoContext::new(VulkanoConfig::default());
    let event_loop = EventLoop::new();
    let mut windows_manager = VulkanoWindows::default();
    windows_manager.create_window(&event_loop, &context, &Default::default(), |_| {});

    let render_pass = vulkano::single_pass_renderpass!(
        context.device().clone(),
        attachments: {
            color: {
                format: windows_manager.get_primary_renderer().unwrap().swapchain_format(),
                samples: 1,
                load_op: Clear,
                store_op: Store,
            },
            depth: {
                format: Format::D32_SFLOAT,
                samples: 1,
                load_op: Clear,
                store_op: Store,
            }
        },
        pass: {
            color: [color],
            depth_stencil: {depth},
        },
    )
    .unwrap();

    let pipelines = create_pipelines(context.device(), &render_pass);
    let allocators = create_allocators(context.device(), context.memory_allocator());

    let mut viewport = Viewport {
        offset: [0.0, 0.0],
        extent: [0.0, 0.0],
        depth_range: 0.0..=1.0,
    };

    let mut fps_tracker = FpsTracker::<f32>::new();

    //let fps_cap: Option<f32> = None;

    let mut rain = Rain::new();
    let mut game = create_game(&context.memory_allocator());

    let mut fixed_update_runner =
        FixedUpdate::new(FIXED_DELTA_TIME, MaxSubsteps::WarnAt(MAX_SUBSTEPS));

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => {
            *control_flow = ControlFlow::Exit;
        }

        Event::WindowEvent {
            event: WindowEvent::Resized(..) | WindowEvent::ScaleFactorChanged { .. },
            window_id,
        } => {
            let window_renderer = windows_manager.get_renderer_mut(window_id).unwrap();
            window_renderer.resize();
            viewport.extent = window_renderer.window_size();
            game.camera.aspect_ratio = window_renderer.aspect_ratio();
        }

        Event::MainEventsCleared => {
            fixed_update_runner.update(|| fixed_update(&mut game));

            update(&mut game, &mut rain);

            render(
                &context,
                &mut windows_manager,
                &allocators,
                &render_pass,
                &game,
                &viewport,
                &pipelines,
            );
            fps_tracker.update();
            //println!("{}", fps_tracker.average_fps());
        }

        Event::WindowEvent {
            event: WindowEvent::KeyboardInput { input, .. },
            ..
        } => {
            on_keyboard_input(
                input,
                control_flow,
                &fps_tracker,
                &mut windows_manager.get_primary_renderer_mut().unwrap(),
                &mut game,
            );
        }

        Event::DeviceEvent {
            event: DeviceEvent::Motion { axis, value },
            ..
        } => {
            let window_renderer = windows_manager.get_primary_renderer_mut().unwrap();
            if game.paused || !window_renderer.window().has_focus() {
                return;
            }

            match axis {
                0 => game.camera.rotation[1] -= value as f32 * game.mouse_sensitivity,
                1 => game.camera.rotation[0] -= value as f32 * game.mouse_sensitivity,
                _ => (),
            }

            let window_extent = window_renderer.window_size();

            window_renderer
                .window()
                .set_cursor_position(PhysicalPosition::new(
                    window_extent[0] / 2.0,
                    window_extent[1] / 2.0,
                ))
                .unwrap();
            window_renderer.window().set_cursor_visible(false);
        }

        Event::DeviceEvent {
            event: DeviceEvent::MouseWheel { delta: _ },
            ..
        } => {}

        _ => (),
    });
}

fn render(
    context: &VulkanoContext,
    windows_manager: &mut VulkanoWindows,
    allocators: &Allocators,
    render_pass: &Arc<RenderPass>,
    game: &Game,
    viewport: &Viewport,
    pipelines: &Pipelines,
) {
    let window_renderer = windows_manager.get_primary_renderer_mut().unwrap();

    let future = window_renderer.acquire().unwrap();

    let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
        &allocators.command_buffer_allocator,
        context.graphics_queue().queue_family_index(),
        CommandBufferUsage::OneTimeSubmit,
    )
    .unwrap();

    // Creating a depth buffer and a frame buffer every frame is very very bad. Not avoidable until next vulkano version.

    let depth_buffer_view = ImageView::new_default(
        Image::new(
            context.memory_allocator().clone(),
            ImageCreateInfo {
                image_type: ImageType::Dim2d,
                format: Format::D32_SFLOAT,
                extent: window_renderer.swapchain_image_view().image().extent(),
                usage: ImageUsage::TRANSIENT_ATTACHMENT | ImageUsage::DEPTH_STENCIL_ATTACHMENT,
                ..Default::default()
            },
            AllocationCreateInfo::default(),
        )
        .unwrap(),
    )
    .unwrap();

    let framebuffer = Framebuffer::new(
        render_pass.clone(),
        FramebufferCreateInfo {
            attachments: vec![window_renderer.swapchain_image_view(), depth_buffer_view],
            ..Default::default()
        },
    )
    .unwrap();

    let camera_uniform = allocators.subbuffer_allocator.allocate_sized().unwrap();
    *camera_uniform.write().unwrap() = game.camera.to_uniform();

    command_buffer_builder
        .begin_render_pass(
            RenderPassBeginInfo {
                clear_values: vec![
                    // Sets background colour.
                    Some([0.0, 0.0, 1.0, 1.0].into()),
                    Some(ClearValue::Depth(1.0)),
                ],
                ..RenderPassBeginInfo::framebuffer(framebuffer)
            },
            Default::default(),
        )
        .unwrap()
        .set_viewport(0, [viewport.clone()].into_iter().collect())
        .unwrap()
        .bind_pipeline_graphics(pipelines.colour_pipeline.clone())
        .unwrap()
        .bind_descriptor_sets(
            PipelineBindPoint::Graphics,
            pipelines.colour_pipeline.layout().clone(),
            0,
            vec![PersistentDescriptorSet::new(
                &allocators.descriptor_set_allocator,
                pipelines
                    .colour_pipeline
                    .layout()
                    .set_layouts()
                    .get(0)
                    .unwrap()
                    .clone(),
                [WriteDescriptorSet::buffer(0, camera_uniform)],
                [],
            )
            .unwrap()],
        )
        .unwrap();

    draw_instanced(
        &mut command_buffer_builder,
        &game.cuboid_buffers.instance_buffer,
        &game.cuboid_buffers.vertex_buffer,
        &game.cuboid_buffers.index_buffer,
        &allocators.subbuffer_allocator,
    );

    command_buffer_builder
        .end_render_pass(Default::default())
        .unwrap();

    let command_buffer = command_buffer_builder.build().unwrap();

    window_renderer.present(
        future
            .then_execute(context.graphics_queue().clone(), command_buffer)
            .unwrap()
            .boxed(),
        false,
    );
}

struct Allocators {
    command_buffer_allocator: StandardCommandBufferAllocator,
    subbuffer_allocator: SubbufferAllocator,
    descriptor_set_allocator: StandardDescriptorSetAllocator,
}

fn create_allocators(
    device: &Arc<Device>,
    memory_allocator: &Arc<StandardMemoryAllocator>,
) -> Allocators {
    Allocators {
        command_buffer_allocator: StandardCommandBufferAllocator::new(
            device.clone(),
            Default::default(),
        ),
        subbuffer_allocator: SubbufferAllocator::new(
            memory_allocator.clone(),
            SubbufferAllocatorCreateInfo {
                buffer_usage: BufferUsage::UNIFORM_BUFFER
                    | BufferUsage::VERTEX_BUFFER
                    | BufferUsage::INDEX_BUFFER,
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
        ),
        descriptor_set_allocator: StandardDescriptorSetAllocator::new(
            device.clone(),
            Default::default(),
        ),
    }
}

struct Pipelines {
    colour_pipeline: Arc<GraphicsPipeline>,
}

fn create_pipelines(device: &Arc<Device>, render_pass: &Arc<RenderPass>) -> Pipelines {
    Pipelines {
        colour_pipeline: colour_3d_instanced_shaders::create_pipeline(
            device.clone(),
            Subpass::from(render_pass.clone(), 0).unwrap(),
        ),
    }
}

// Can we have some sort of smoke erupting from the volcano?
enum Stage {
    Calm(u32),
    LargeChunks(u32),
    FineRain(u32),
    Todo(u32),
}

// Todo, rename to volcano.
struct Rain {
    rng: ThreadRng,
    stage: Stage,
    drop_indices: Vec<(usize, usize)>,
    stage_ticker: FixedUpdate<f32>,
    basic_drop: FixedUpdate<f32>,
}

impl Rain {
    fn new() -> Rain {
        Rain {
            rng: thread_rng(),
            stage: Stage::Calm(0),
            drop_indices: Vec::with_capacity(MAX_RAINDROPS),

            stage_ticker: FixedUpdate::new(1.0, MaxSubsteps::Infinite),
            basic_drop: FixedUpdate::new(0.1_f32, MaxSubsteps::ReturnAt(10)),
        }
    }
    fn run(&mut self, game: &mut Game) {
        return;
        if self.drop_indices.len() >= MAX_RAINDROPS - 1000 {
            //if false {
            //println!("Removing rain!");
            for _ in 0..50 {
                //game.physics.bodies.swap_remove(self.drop_indices[0]); // still might be possible
                game.physics.bodies[self.drop_indices[0].0] = Body::None;
                //game.objects_to_render.swap_remove(self.drop_indices[0].1); // once again, might be possible, with a bit of work
                game.objects_to_render[self.drop_indices[0].1] = RenderObject::None;
                self.drop_indices.remove(0); // May be slow.
            }
        }

        self.stage_ticker.update(|| match &mut self.stage {
            Stage::Calm(seconds) => *seconds += 1,
            Stage::LargeChunks(seconds) => *seconds += 1,
            Stage::FineRain(seconds) => *seconds += 1,
            Stage::Todo(seconds) => *seconds += 1,
        });

        self.basic_drop.update(|| {
            if let Stage::FineRain(_) = self.stage {
                for _ in 0..30 {
                    self.drop_indices
                        .push_within_capacity((
                            game.physics.bodies.len(),
                            game.objects_to_render.len(),
                        ))
                        .unwrap();
                    game.objects_to_render.push(RenderObject::Cuboid {
                        body_index: game.physics.bodies.len(),
                        colour: [
                            self.rng.gen_range(0.0..1.0),
                            self.rng.gen_range(0.0..1.0),
                            self.rng.gen_range(0.0..1.0),
                            1.0,
                        ],
                    });
                    game.physics.bodies.push(Body::Cuboid(Cuboid {
                        particle: Particle::from_position([
                            self.rng.gen_range(-250.0..250.0),
                            -300.0,
                            self.rng.gen_range(-250.0..250.0),
                        ]),
                        half_size: [
                            self.rng.gen_range(2.0..6.0),
                            self.rng.gen_range(2.0..6.0),
                            self.rng.gen_range(2.0..6.0),
                        ],
                    }));
                }
            }
        });
    }
}

struct PlantGrower {
    grow_zones: Vec<AabbMinMax<f32>>,
    new_plant_ticker: FixedUpdate<f32>,
}

struct DayNightCycle {
    start: Option<Instant>,
    level: usize,
}

struct TitleMenu {}

struct CreatureSelectionMenu {}

enum Menu {
    Title(TitleMenu),
    CreatureSelection(CreatureSelectionMenu),
}

struct Game {
    rng: ThreadRng,

    mouse_sensitivity: f32,
    wasd_held: [bool; 4],
    jump_held: bool,
    sprinting: bool,
    paused: bool,
    camera: Camera,

    physics: CpuSolver<f32, Body<f32>>,
    objects_to_render: Vec<gltf::RenderObject>,

    plant_grower: PlantGrower,

    day_night_cycle: DayNightCycle,

    cuboid_buffers: ColourBuffers,
}

impl Game {
    fn player(&mut self) -> &mut Player<f32> {
        let Body::Player(ref mut player) = self.physics.bodies[0] else {
            unreachable!("The player will always be index 0 of bodies.")
        };
        player
    }

    fn tick_plant_grower(&mut self) {
        self.plant_grower.new_plant_ticker.update(|| {
            println!("tick");
            let index = self.rng.gen_range(0..self.plant_grower.grow_zones.len());
            let grow_zone = &self.plant_grower.grow_zones[index];
            let position = [
                self.rng.gen_range(grow_zone.min[0]..grow_zone.max[0]),
                self.rng.gen_range(grow_zone.min[1]..grow_zone.max[1]),
                self.rng.gen_range(grow_zone.min[2]..grow_zone.max[2]),
            ];

            self.objects_to_render.push(RenderObject::Cuboid {
                body_index: self.physics.bodies.len(),
                colour: [0.0, 1.0, 0.0, 1.0],
            });

            // Rather just have 1 collision recorder for the player
            self.physics
                .bodies
                .push(Body::CollisionRecorderCuboid(
                    CollisionRecorderCuboid {
                        aabb: AabbCentredOrigin {
                            position,
                            half_size: [0.5; 3],
                        },
                        save_collision: |_body| true,
                        stored_collider_index: None,
                    },
                ));
        });
    }
}

fn create_game(memory_allocator: &Arc<StandardMemoryAllocator>) -> Game {
    let mut game = Game {
        rng: thread_rng(),

        mouse_sensitivity: 1.0,
        wasd_held: [false; 4],
        jump_held: false,
        sprinting: true,
        paused: false,
        camera: Camera {
            position: [-184.70149, 2.0, -147.17622],
            rotation: [4.0, 523.0, 0.0],

            ambient_strength: 0.3,
            specular_strength: 0.5,
            light_colour: [0.5; 3],
            light_position: [0.0, -10.0, 0.0],

            near_distance: 0.01,
            far_distance: 250.0,
            aspect_ratio: 0.0,
            fov_y: Radians(std::f32::consts::FRAC_PI_2),
        },

        physics: CpuSolver::new(
            solver::Config{
                gravity: [0.0, 50.0, 0.0],
                dampening: [0.8, 1.0, 0.8],
                grid_size: COLLISION_GRID_SIZE,
                grid_origin: COLLISION_GRID_ORIGIN,
                cell_size: COLLISION_GRID_CELL_SIZE,
                outside_of_grid_bounds_behaviour: OutsideOfGridBoundsBehaviour::ContinueUpdating, // TODO: replace with none instead
                bodies: Vec::with_capacity(INITIAL_BODY_CAPACITY),
            },
        ),
        objects_to_render: Vec::with_capacity(INITIAL_BODY_CAPACITY),

        plant_grower: PlantGrower {
            grow_zones: vec![],
            new_plant_ticker: FixedUpdate::new(3.0, MaxSubsteps::Infinite),
        },
        day_night_cycle: DayNightCycle {
            start: None,
            level: 0,
        },

        cuboid_buffers: ColourBuffers {
            vertex_buffer: Buffer::from_iter(
                memory_allocator.clone(),
                BufferCreateInfo {
                    usage: BufferUsage::VERTEX_BUFFER,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                        | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                },
                meshes::CUBE_VERTICES.to_owned(), // TODO: this might be slow
            )
            .unwrap(),
            index_buffer: Buffer::from_iter(
                memory_allocator.clone(),
                BufferCreateInfo {
                    usage: BufferUsage::INDEX_BUFFER,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                        | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                },
                meshes::CUBE_INDICES.to_owned(), // TODO: this might be slow
            )
            .unwrap(),
            instance_buffer: vec![],
        },
    };

    game.physics.bodies.push(Body::Player(Player {
        particle: Particle::from_position([-184.70149, 3.0, -147.17622]),
        mass: 30.0,
        friction: 5.0,
        restitution: 0.5,
        half_size: [0.5, 1.0, 0.5],
        dampening: [0.0, 0.0, 0.0],
        grounded: false,
    }));

    let mut scenes = gltf::load_scenes();
    let world = &mut scenes.scenes[0];

    game.objects_to_render = world.render_objects.drain(..).collect();

    game.physics.bodies.append(&mut world.bodies);

    game.plant_grower.grow_zones = world.grow_zones.clone();

    game
}

struct ColourBuffers {
    vertex_buffer: Subbuffer<[buffer_contents::Basic3DVertex]>,
    index_buffer: Subbuffer<[u32]>,
    instance_buffer: Vec<buffer_contents::Colour3DInstance>,
}

fn fixed_update(game: &mut Game) {
    let motion = match game.wasd_held {
        [true, false, false, false] => (0.0, -1.0),
        [false, false, true, false] => (0.0, 1.0),
        [false, false, false, true] => (1.0, 0.0),
        [false, true, false, false] => (-1.0, 0.0),

        [true, true, false, false] => (-0.7, -0.7),
        [true, false, false, true] => (0.7, -0.7),

        [false, true, true, false] => (-0.7, 0.7),
        [false, false, true, true] => (0.7, 0.7),

        _ => (0.0, 0.0),
    };

    /*
    let speed = match (sprinting, *jump_held, player.grounded) {
        (false, true, true) | (false, false, true) | (false, true, false) => 25.0,
        (true, true, true) | (true, false, true) | (false, false, false) | (true, true, false) => {
            50.0
        }
        (true, false, false) => 100.0,
    };
    */

    let speed = match game.sprinting {
        true => 50.0,
        false => 25.0,
    };

    let real_motion = (-motion.0 * speed, motion.1 * speed);

    let y_rotation_cos = game.camera.rotation[1].to_radians().cos();
    let y_rotation_sin = game.camera.rotation[1].to_radians().sin();

    let real_motion = (
        real_motion.0 * y_rotation_cos - real_motion.1 * y_rotation_sin,
        real_motion.1 * y_rotation_cos + real_motion.0 * y_rotation_sin,
    );

    game.player()
        .particle
        .accelerate([real_motion.0 as f32, 0.0, real_motion.1 as f32]);

    let horizontal_dampening = if game.player().grounded { 0.8 } else { 0.95 }; // grounded originally 0.8

    game.player().dampening = [horizontal_dampening, 1.0, horizontal_dampening]; // y 0.98 originally

    game.physics.update_extra_experimental(FIXED_DELTA_TIME);

    if game.jump_held {
        //if game.player().grounded {
        if true {
            //game.player().particle.accelerate([0.0, -500.0, 0.0]);
            game.player().particle.accelerate([0.0, -100.0, 0.0]);
        }
    }

    game.camera.position = math::add_3d(game.player().particle.position, [0.0, -1.0, 0.0]);

    game.camera.light_position[0] = game.camera.position[0];
    game.camera.light_position[2] = game.camera.position[2];
}

fn update(game: &mut Game, rain: &mut Rain) {
    rain.run(game);
    game.tick_plant_grower();

    game.cuboid_buffers.instance_buffer.clear();

    // Actual plan:
    // Step 1: Remove non-renderables, like None physics objects.
    // Step 2: Collect in parallel an instance from all the render_objects

    game.cuboid_buffers.instance_buffer = game
        .objects_to_render
        .par_iter()
        .filter_map(|render_object| match render_object {
            RenderObject::None => None,
            RenderObject::Cuboid { body_index, colour } => {
                let body = &game.physics.bodies[*body_index];
                Some(Colour3DInstance::new(
                    *colour,
                    Matrix4::from_translation(body.position_unchecked())
                        * Matrix4::from_scale(body.size().unwrap()),
                ))
            }
            RenderObject::CuboidNoPhysics(instance) => Some(instance.clone()),
        })
        //.collect_into_vec(&mut game.cuboid_buffers.instance_buffer);
        .collect();
}

fn on_keyboard_input(
    input: KeyboardInput,
    control_flow: &mut ControlFlow,
    fps_tracker: &FpsTracker<f32>,
    window_renderer: &mut VulkanoWindowRenderer,
    game: &mut Game,
) {
    if let Some(key_code) = input.virtual_keycode {
        match key_code {
            VirtualKeyCode::W => game.wasd_held[0] = is_pressed(input.state),
            VirtualKeyCode::A => game.wasd_held[1] = is_pressed(input.state),
            VirtualKeyCode::S => game.wasd_held[2] = is_pressed(input.state),
            VirtualKeyCode::D => game.wasd_held[3] = is_pressed(input.state),

            VirtualKeyCode::Backslash => {
                if is_pressed(input.state) {
                    if let None = window_renderer.window().fullscreen() {
                        window_renderer
                            .window()
                            .set_fullscreen(Some(Fullscreen::Borderless(None)));
                    } else {
                        window_renderer.window().set_fullscreen(None);
                    }
                }
            }

            VirtualKeyCode::F => {
                if is_pressed(input.state) {
                    game.sprinting = !game.sprinting;
                }
            }

            VirtualKeyCode::Space => game.jump_held = is_pressed(input.state),

            VirtualKeyCode::Delete => {
                if is_pressed(input.state) {
                    *control_flow = ControlFlow::Exit;
                }
            }

            VirtualKeyCode::X => {
                if is_pressed(input.state) {
                    println!("fps: {}", fps_tracker.average_fps());
                    println!("bodies: {}", game.physics.bodies.len());
                }
            }

            VirtualKeyCode::P => {
                if is_pressed(input.state) {
                    println!("player: {:?}", game.player());
                    println!("camera: {:?}", game.camera);
                }
            }

            VirtualKeyCode::Escape => {
                if is_pressed(input.state) {
                    game.paused = !game.paused;
                }
            }

            VirtualKeyCode::T => {
                if is_pressed(input.state) {
                    for _ in 0..3000 {
                        let mut rng = thread_rng();
                        game.objects_to_render.push(RenderObject::Cuboid {
                            body_index: game.physics.bodies.len(),
                            colour: [
                                rng.gen_range(0.0..1.0),
                                rng.gen_range(0.0..1.0),
                                rng.gen_range(0.0..1.0),
                                1.0,
                            ],
                        });
                        game.physics.bodies.push(Body::Cuboid(Cuboid {
                            particle: Particle::from_position([
                                rng.gen_range(-250.0..250.0),
                                -300.0,
                                rng.gen_range(-250.0..250.0),
                            ]),
                            half_size: [
                                rng.gen_range(0.5..3.0),
                                rng.gen_range(0.5..3.0),
                                rng.gen_range(0.5..3.0),
                            ],
                        }));
                    }
                }
            }
            _ => (),
        }
    }
}
*/
