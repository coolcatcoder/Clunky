use std::collections::HashMap;

use clunky::{
    lost_code::{is_pressed, FixedUpdate, FpsTracker, MaxSubsteps},
    math::{remap, Matrix4},
    physics::{
        physics_3d::{
            aabb::AabbCentredOrigin,
            //bodies::{Body, ImmovableCuboid},
            solver::{self, CpuSolver},
        },
        PhysicsSimulation,
    },
    shaders::{
        instanced_simple_lit_colour_3d::{self, Camera},
        instanced_text_sdf, instanced_unlit_uv_2d_stretch,
    },
};
use common_renderer::{bits_has, CommonRenderer};
use engine::SimpleEngine;
use gilrs::{EventType, Gilrs};
use renderer::{Camera3D, Renderer, WindowConfig, WindowVariety};
use vulkano::swapchain::PresentMode;
use vulkano_util::window::WindowDescriptor;
use winit::{
    dpi::PhysicalPosition,
    event::{DeviceEvent, Event, KeyboardInput, MouseButton, WindowEvent},
    event_loop::{EventLoop, EventLoopWindowTarget},
    window::{Fullscreen, WindowId},
};

use body::Body;

use creature_types::{Burgle, CreatureType};

mod body;
mod common_renderer;
mod creature_types;
mod engine;
mod renderer;
mod menus;

type Engine = SimpleEngine<CommonRenderer<Body>>;
type Physics = CpuSolver<f32, Body>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct CreatureIndex(usize);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct BodyIndex(usize);

const GRID_MIN: [isize; 3] = [-200, -3, -200];
const GRID_MAX: [isize; 3] = [200, 10, 200];
const GRID_SUBDIVISIONS: [usize; 3] = [15, 1, 15];

const FIXED_DELTA_TIME: f32 = 0.03;

struct CreaturesManager {
    creature_controlled_by_window: HashMap<WindowId, CreatureIndex>,

    creatures: Vec<CreatureType>,
    captured_creatures: Vec<CreatureIndex>,

    creature_selection_window: WindowId,
}

struct Settings {
    mouse_sensitivity: f32,
}

struct Reality {
    creatures_manager: CreaturesManager,

    physics_simulation: Physics,

    actions: RealityActions,
}

impl Reality {
    fn new(game: &mut Game, event_loop: &EventLoopWindowTarget<()>) -> Self {
        let selection_window = game.renderer.create_window(
            &event_loop,
            &WindowConfig {
                variety: WindowVariety::Selection,
                window_descriptor: WindowDescriptor {
                    ..Default::default()
                },
                swapchain_create_info_modify: |_| {},
            },
        );

        let physics_config = solver::Config {
            ..solver::Config::size_from_min_max_with_subdivisions(
                GRID_MIN,
                GRID_MAX,
                GRID_SUBDIVISIONS,
            )
        };

        let mut reality = Self {
            creatures_manager: CreaturesManager {
                creature_controlled_by_window: HashMap::new(),
                creature_selection_window: selection_window,

                creatures: vec![],
                captured_creatures: vec![],
            },

            physics_simulation: CpuSolver::new(physics_config),

            actions: Default::default()
        };
    
        game.renderer.selection_menu_uv_instances_mut().push(
            instanced_unlit_uv_2d_stretch::Instance::new(
                [0.0, 0.0],
                0.0,
                glam::Affine2::from_translation([1.0, 0.0].into()),
            ),
        );
    
        game.renderer
            .selection_menu_text_instances_mut()
            .push(instanced_text_sdf::Instance::new(
                [0.0, 0.0],
                [1.0, 0.0, 1.0, 1.0],
                0.01,
                0.2,
                glam::Affine2::from_translation([0.0, 0.0].into())
                    * glam::Affine2::from_scale([0.25, 0.25].into()),
            ));
        //text_rendering::blah();
    
        let test_burgle_window = game.renderer.create_window(
            &event_loop,
            &WindowConfig {
                variety: WindowVariety::Creature(Camera3D {
                    ..Default::default()
                }),
                window_descriptor: WindowDescriptor {
                    present_mode: PresentMode::Fifo,
                    transparent: true,
                    decorations: false,
                    ..Default::default()
                },
                swapchain_create_info_modify: |_| {},
            },
        );
    
        reality.creatures_manager
            .creatures
            .push(CreatureType::Burgle(Burgle::new(
                &mut game.renderer,
                &mut reality.physics_simulation.bodies,
                [0.0; 3],
                [0.5, 1.0, 0.5],
                [1.5; 3],
                [1.0, 0.0, 1.0, 1.0],
                CreatureIndex(reality.creatures_manager.creatures.len()),
            )));
    
            reality.creatures_manager
            .captured_creatures
            .push(CreatureIndex(0));
    
            reality.creatures_manager
            .creature_controlled_by_window
            .insert(test_burgle_window, CreatureIndex(0));
    
        // 2 player test
        reality.creatures_manager
            .creatures
            .push(CreatureType::Burgle(Burgle::new(
                &mut game.renderer,
                &mut reality.physics_simulation.bodies,
                [0.0; 3],
                [0.5, 1.0, 0.5],
                [1.5; 3],
                [1.0, 1.0, 0.0, 1.0],
                CreatureIndex(reality.creatures_manager.creatures.len()),
            )));
    
        /*
        game.creatures_manager
            .captured_creatures
            .push(CreatureIndex(1));
        */
    
        game.renderer
            .add_cuboid_colour(instanced_simple_lit_colour_3d::Instance::new(
                [1.0; 4],
                Matrix4::from_translation([GRID_MIN[0] as f32, GRID_MIN[1] as f32, GRID_MIN[2] as f32]),
            ));
    
        game.renderer
            .add_cuboid_colour(instanced_simple_lit_colour_3d::Instance::new(
                [1.0; 4],
                Matrix4::from_translation([GRID_MAX[0] as f32, GRID_MAX[1] as f32, GRID_MAX[2] as f32]),
            ));
    
        game.renderer
            .add_cuboid_colour(instanced_simple_lit_colour_3d::Instance::new(
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
    
        let floor = AabbCentredOrigin {
            position: [0.0, 1.0, 0.0],
            half_size: [
                (GRID_MAX[0] - GRID_MIN[0]) as f32 * 0.5,
                0.5,
                (GRID_MAX[2] - GRID_MIN[2]) as f32 * 0.5,
            ],
        };
        reality.physics_simulation
            .bodies
            .push(Body::ImmovableCuboid(floor.clone()));
    
        game.renderer
            .add_cuboid_colour_from_aabb(floor, [1.0, 0.0, 1.0, 1.0]);

        reality
    }
}

struct Game {
    renderer: Renderer,

    physics_fixed_update: Option<FixedUpdate<f32>>,

    fps: FpsTracker<f32>,

    settings: Settings,

    window_focused: Option<WindowId>,

    actions: GameActions,
    input_manager: InputManager,

    reality: Option<Reality>,

    menu_window: Option<WindowId>,
}

impl Game {
    fn new() -> (Self, EventLoop<()>) {
        let (renderer, event_loop) = Renderer::new();

        let game = Game {
            renderer,

            physics_fixed_update: Some(FixedUpdate::new(
                FIXED_DELTA_TIME,
                MaxSubsteps::WarnAt(100),
            )),

            fps: FpsTracker::new(),

            settings: Settings {
                mouse_sensitivity: 1.0,
            },

            window_focused: None,

            actions: Default::default(),
            input_manager: InputManager {
                gilrs: Gilrs::new().unwrap(),

                active_binding: 0,
                bindings: vec![Bindings::ScanCodeBindings(Default::default())],
            },

            reality: None,

            // TODO: this shouldn't be none, this should be the main menu. This currently doesn't exist though.
            menu_window: None
        };
        (game, event_loop)
    }
}

fn main() {
    println!("TODO:\nStore which grid actually have collision in them, so you don't have to loop over them all.");
    println!("TODO:\nUse ahash.");

    let blah = bits_has(0b010, 0b110);
    println!("{blah}");

    let (mut game, event_loop) = Game::new();

    game.reality = Some(Reality::new(&mut game, &event_loop));

    event_loop.run(move |event, event_loop, control_flow| match event {
        Event::WindowEvent {
            window_id,
            event: WindowEvent::CloseRequested,
            ..
        } => {
            let window_specific = game.renderer.window_specifics.get(&window_id).unwrap();

            if matches!(window_specific.variety, WindowVariety::Selection) {
                todo!("Display an are you sure window.");
            } else {
                game.renderer.remove_window(window_id);

                game.creatures_manager
                    .creature_controlled_by_window
                    .remove(&window_id);
            }
        }

        Event::WindowEvent {
            event: WindowEvent::Resized(..) | WindowEvent::ScaleFactorChanged { .. },
            window_id,
        } => {
            game.renderer.correct_window_size(window_id);

            if let Some(creature_selection_window) = game.creatures_manager.creature_selection_window {
                if window_id == creature_selection_window {
                    menus::TextManager::on_selection_menu_resize(game.renderer.windows_manager.get_renderer(window_id).unwrap().window().inner_size().into(), game.renderer.selection_menu_text_instances_mut(), &game.creatures_manager)
                }
            }
        }

        Event::MainEventsCleared => {
            let mut physics_fixed_update = game.physics_fixed_update.take().unwrap();
            physics_fixed_update.update(|| fixed_update(&mut game));
            game.physics_fixed_update = Some(physics_fixed_update);
            on_update(&mut game, event_loop);
            game.renderer.render(&game.physics_simulation.bodies);
            game.fps.update();
        }

        Event::WindowEvent {
            event: WindowEvent::KeyboardInput { input, .. },
            ..
        } => {
            let mut input_manager = game.input_manager.take().unwrap();
            let mut game_actions = game.actions.take().unwrap();
            if let Bindings::ScanCodeBindings(bindings) =
                &mut input_manager.bindings[input_manager.active_binding]
            {
                let creature_actions = game.focused_creature_mut().actions_mut();
                bindings.modify_actions_with_keyboard_input(
                    input,
                    &mut game_actions,
                    creature_actions,
                )
            }
            game.input_manager = Some(input_manager);
            game.actions = Some(game_actions);
        }

        Event::WindowEvent {
            window_id,
            event: WindowEvent::Focused(focus),
            ..
        } => {
            if focus {
                game.window_focused = Some(window_id);
            } else {
                game.window_focused = None;
            }
        }

        // Replace with mouse bindings struct.
        Event::WindowEvent {
            window_id,
            event: WindowEvent::MouseInput { state, button, .. },
            ..
        } => {
            if let Some(selection_window) = game.creatures_manager.creature_selection_window {
                if selection_window == window_id {
                    return
                }
            }
            
            match button {
            MouseButton::Left => {
                if is_pressed(state) {
                    game.focused_creature_mut().actions_mut().primary_interact = true;
                }
            }

            _ => (),
        }},

        Event::DeviceEvent {
            event: DeviceEvent::MouseMotion { delta },
            ..
        } => {
            let Some(window_focused) = game.window_focused else {
                return;
            };

            let window_renderer = game
                .renderer
                .windows_manager
                .get_renderer_mut(window_focused)
                .unwrap();

            if game.actions.as_ref().unwrap().paused {
                window_renderer.window().set_cursor_visible(true);
                return;
            }

            let delta = [
                delta.1 as f32 * game.settings.mouse_sensitivity,
                delta.0 as f32 * game.settings.mouse_sensitivity,
            ];

            //game.focused_creature_mut().on_mouse_motion(delta);

            let focused_window = game.window_focused.unwrap();

            let creature_index = *game
                .creatures_manager
                .creature_controlled_by_window
                .get(&focused_window)
                .unwrap();

            game.creatures_manager.creatures[creature_index.0].on_mouse_motion(delta);

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

        _ => (),
    })
}

fn fixed_update(game: &mut Game) {
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

    creature.on_physics_fixed_update_before_physics_tick_when_focused(
        &mut game.physics_simulation.bodies,
    );

    game.physics_simulation.update(FIXED_DELTA_TIME);
}

fn on_update(game: &mut Game, event_loop: &EventLoopWindowTarget<()>) {
    let mut input_manager = game.input_manager.take().unwrap();
    while let Some(event) = input_manager.gilrs.next_event() {

        //println!("event: {:?}", event);

        //let mut input_manager = game.input_manager.take().unwrap();
        let mut game_actions = game.actions.take().unwrap();
        if let Bindings::GamepadCodeBindings(bindings) =
            &mut input_manager.bindings[input_manager.active_binding]
        {
            let creature_actions = game.focused_creature_mut().actions_mut();
            bindings.modify_actions_with_gamepad_event_type(
                event.event,
                &mut game_actions,
                creature_actions,
            )
        }
        //game.input_manager = Some(input_manager);
        game.actions = Some(game_actions);
    }
    game.input_manager = Some(input_manager);

    // ((capture, capture's body index), spreader)
    let mut capture_attempts = vec![];
    for creature_index in &game.creatures_manager.captured_creatures {
        let creature = &mut game.creatures_manager.creatures[creature_index.0];

        creature.update(
            &game.physics_simulation.bodies,
            &game.creatures_manager.captured_creatures,
            &mut capture_attempts,
        );
    }
    for ((capture_index, body_index), spreader_index) in capture_attempts {
        // Wrangle the borrow checker into lettings us mutably attempt_capture on the capture creature and mutably give it the spreader.
        // This feels like it could be simplified.
        let (capture, spreader) = if spreader_index > capture_index {
            let (lhs, rhs) = game
                .creatures_manager
                .creatures
                .split_at_mut(spreader_index.0);

            (&mut lhs[capture_index.0], &mut rhs[0])
        } else {
            let (lhs, rhs) = game
                .creatures_manager
                .creatures
                .split_at_mut(capture_index.0);

            (&mut rhs[0], &mut lhs[spreader_index.0])
        };

        if capture.attempt_capture(spreader, body_index, &mut game.physics_simulation.bodies) {
            game.creatures_manager
                .captured_creatures
                .push(capture_index);

            let new_window = game
                .renderer
                .create_window(&event_loop, &WindowConfig::default());

            game.creatures_manager
                .creature_controlled_by_window
                .insert(new_window, capture_index);
        }
    }

    for (window_id, creature_index) in &game.creatures_manager.creature_controlled_by_window {
        let creature = &game.creatures_manager.creatures[creature_index.0];

        let WindowVariety::Creature(camera) = &mut game
            .renderer
            .window_specifics
            .get_mut(&window_id)
            .unwrap()
            .variety
        else {
            unreachable!()
        };

        creature.update_camera(camera, &game.physics_simulation.bodies);

        camera.light_position[0] = camera.position[0];
        camera.light_position[2] = camera.position[2];

        //println!("camera: {:?}", camera);
    }

    let mut game_actions = game.actions.take().unwrap();
    if let Some(focused_window) = game.window_focused {
        if game_actions.full_screen {
            game_actions.full_screen = false;

            let window = game
                .renderer
                .windows_manager
                .get_window(focused_window)
                .unwrap();

            window.set_fullscreen(if matches!(window.fullscreen(), None) {
                Some(Fullscreen::Borderless(None))
            } else {
                None
            });
        }

        if game_actions.log_fps {
            game_actions.log_fps = false;
            println!("fps: {}", game.fps.average_fps());
        }
    }
    game.actions = Some(game_actions);
}

#[derive(Default, Debug)]
enum ActionState {
    #[default]
    None,
    Do,
    End,
}

// So inflexible. If only there was a better way...
enum GamepadMovement<T> {
    Stick([T; 2]),
    Buttons([T; 4]),
}

struct GamepadCodeBindings {
    horizontal_movement: GamepadMovement<u32>,

    // Should be stick or buttons.
    up_movement: u32,
    down_movement: u32,

    capture: (u32, bool),

    speed_modifier: (u32, bool),

    primary_interact: u32,
    secondary_interact: u32,

    // Should be stick or buttons.
    positive_move_selection: u32,
    negative_move_selection: u32,

    full_screen: u32,
}

impl GamepadCodeBindings {
    #[rustfmt::skip]
    fn modify_actions_with_gamepad_event_type(
        &mut self,
        input: EventType,
        game_actions: &mut GameActions,
        creature_actions: &mut CreatureActions,
    ) {
        match input {
            EventType::ButtonPressed(_, code) => {
                let code = code.into_u32();
                println!("ButtonPressed({})", code);

                if code == self.up_movement {
                    creature_actions.movement[1] = -1.0;
                }

                else if code == self.capture.0 {
                    if self.capture.1 {
                        creature_actions.capture = match creature_actions.capture {
                            ActionState::Do => ActionState::End,
                            ActionState::None => ActionState::Do,
                            ActionState::End => {
                                println!("End should always be used. This should never happen.");
                                ActionState::End
                            }
                        };
                    } else {
                        creature_actions.capture = ActionState::Do;
                    }
                }

                else if code == self.speed_modifier.0 {
                    if self.speed_modifier.1 {
                        creature_actions.speed_modifier = if creature_actions.speed_modifier == 1.0 {
                            0.0
                        } else {
                            1.0
                        };
                    } else {
                        creature_actions.speed_modifier = 1.0;
                    }
                }

                else if code == self.positive_move_selection {
                    creature_actions.move_selection = 1;
                } else if code == self.negative_move_selection {
                    creature_actions.move_selection = -1;
                }
            }
            EventType::ButtonReleased(_, code) => {
                let code = code.into_u32();
                println!("ButtonReleased({})", code);

                if code == self.up_movement {
                    creature_actions.movement[1] = 0.0;
                }

                else if code == self.capture.0 && !self.capture.1 {
                    creature_actions.capture = ActionState::End;
                }

                else if code == self.speed_modifier.0 && !self.speed_modifier.1 {
                    creature_actions.speed_modifier = 0.0;
                }
            }

            EventType::AxisChanged(_, value, code) => {
                let code = code.into_u32();
                println!("AxisChanged(code:{},value:{})", code, value);

                if code == self.speed_modifier.0 && !self.speed_modifier.1 {
                    creature_actions.speed_modifier = remap(-value, -1.0..1.0, 0.0..1.0);
                }

                else if let GamepadMovement::Stick(horizontal_movement) = self.horizontal_movement {
                    if code == horizontal_movement[0] {
                        creature_actions.movement[0] = -value;
                    } else if code == horizontal_movement[1] {
                        creature_actions.movement[2] = -value;
                    }
                }
            }
            _ => (),
        }
    }

    fn default_attack() -> Self {
        Self {
            horizontal_movement: GamepadMovement::Stick([196608, 196609]),

            up_movement: 65824,
            down_movement: 0,

            capture: (65826, true),

            speed_modifier: (196610, false),
            //speed_modifier: (65832, true),
            primary_interact: 0,
            secondary_interact: 0,

            positive_move_selection: 65828,
            negative_move_selection: 65827,

            full_screen: 0,
        }
    }
}

impl Default for GamepadCodeBindings {
    fn default() -> Self {
        Self {
            horizontal_movement: GamepadMovement::Stick([196608, 196609]),

            up_movement: 65824,
            down_movement: 0,

            capture: (65826, true),

            speed_modifier: (196610, false),
            //speed_modifier: (65832, true),
            primary_interact: 0,
            secondary_interact: 0,

            positive_move_selection: 65828,
            negative_move_selection: 65827,

            full_screen: 0,
        }
    }
}

struct ScanCodeBindings {
    forwards_movement: u32,
    backwards_movement: u32,
    left_movement: u32,
    right_movement: u32,
    up_movement: u32,
    down_movement: u32,

    capture: u32,

    speed_modifier: u32,

    primary_interact: u32,
    secondary_interact: u32,

    positive_move_selection: u32,
    negative_move_selection: u32,

    full_screen: u32,
    pause: u32,

    log_fps: u32,
}

impl ScanCodeBindings {
    #[rustfmt::skip]
    fn modify_actions_with_keyboard_input(
        &mut self,
        input: KeyboardInput,
        game_actions: &mut GameActions,
        creature_actions: &mut CreatureActions,
    ) {
        //println!("scancode: {}", input.scancode);

        if input.scancode == self.forwards_movement {
            if is_pressed(input.state) {
                creature_actions.movement[2] = 1.0;
            } else {
                creature_actions.movement[2] = 0.0;
            }
        } else if input.scancode == self.backwards_movement {
            if is_pressed(input.state) {
                creature_actions.movement[2] = -1.0;
            } else {
                creature_actions.movement[2] = 0.0;
            }
        } else if input.scancode == self.left_movement {
            if is_pressed(input.state) {
                creature_actions.movement[0] = -1.0;
            } else {
                creature_actions.movement[0] = 0.0;
            }
        } else if input.scancode == self.right_movement {
            if is_pressed(input.state) {
                creature_actions.movement[0] = 1.0;
            } else {
                creature_actions.movement[0] = 0.0;
            }
        } else if input.scancode == self.up_movement {
            if is_pressed(input.state) {
                creature_actions.movement[1] = -1.0;
            } else {
                creature_actions.movement[1] = 0.0;
            }
        } else if input.scancode == self.down_movement {
            if is_pressed(input.state) {
                creature_actions.movement[1] = 1.0;
            } else {
                creature_actions.movement[1] = 0.0;
            }
        }

        else if input.scancode == self.capture {
            if is_pressed(input.state) {
                creature_actions.capture = ActionState::Do;
            } else {
                creature_actions.capture = ActionState::End;
            }
        }

        else if input.scancode == self.positive_move_selection {
            if is_pressed(input.state) {
                creature_actions.move_selection = 1;
            }
        } else if input.scancode == self.negative_move_selection {
            if is_pressed(input.state) {
                creature_actions.move_selection = -1;
            }
        }

        else if input.scancode == self.full_screen {
            if is_pressed(input.state) {
                game_actions.full_screen = true;
            }
        } else if input.scancode == self.pause {
            if is_pressed(input.state) {
                game_actions.paused = !game_actions.paused;
            }
        }

        else if input.scancode == self.log_fps {
            if is_pressed(input.state) {
                game_actions.log_fps = true;
            }
        }
    }
}

impl Default for ScanCodeBindings {
    fn default() -> Self {
        Self {
            forwards_movement: 17,
            backwards_movement: 31,
            left_movement: 30,
            right_movement: 32,
            up_movement: 57,
            down_movement: 42,

            capture: 46,

            speed_modifier: 33,

            primary_interact: 19,
            secondary_interact: 20,

            positive_move_selection: 16,
            negative_move_selection: 18,

            full_screen: 43,
            pause: 1,

            log_fps: 25,
        }
    }
}

enum Bindings {
    ScanCodeBindings(ScanCodeBindings),
    GamepadCodeBindings(GamepadCodeBindings),
}

struct InputManager {
    gilrs: Gilrs,

    active_binding: usize,
    // Vec so people can easily switch between bindings.
    bindings: Vec<Bindings>,
    //TODO: add creature specific bindings?

    //TODO: add default bindings when switching to controllers or keyboards, with specific names.
}

// Up to the bindings to use stuff as press and hold, or toggle.
#[derive(Default, Debug)]
struct CreatureActions {
    // Local space. Vertical should only be -1.0 or 1.0.
    movement: [f32; 3],

    capture: ActionState,

    // from 0-1
    speed_modifier: f32,

    primary_interact: bool,
    secondary_interact: bool,

    move_selection: i8,
}

#[derive(Default, Debug)]
struct GameActions {
    full_screen: bool,

    // Debug:
    log_fps: bool,
}

#[derive(Default, Debug)]
struct RealityActions {
    paused: bool,
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
fn rotate_2d(movement: [f32; 2], theta: f32) -> [f32; 2] {
    let theta = theta.to_radians();
    let theta_cos = theta.cos();
    let theta_sin = theta.sin();

    [
        movement[0] * theta_cos - movement[1] * theta_sin,
        movement[1] * theta_cos + movement[0] * theta_sin,
    ]
}

// Direction? Clockwise? Anticlockwise? I don't know!
fn rotate_about_y(vector: [f32; 3], theta: f32) -> [f32; 3] {
    let theta = theta.to_radians();
    let theta_cos = theta.cos();
    let theta_sin = theta.sin();

    [
        vector[2] * theta_sin + vector[0] * theta_cos,
        vector[1],
        vector[2] * theta_cos - vector[0] * theta_sin,
    ]
}

fn rotate_about_z(vector: [f32; 3], theta: f32) -> [f32; 3] {
    let theta = theta.to_radians();
    let theta_cos = theta.cos();
    let theta_sin = theta.sin();

    [
        vector[0] * theta_cos - vector[1] * theta_sin,
        vector[0] * theta_sin + vector[1] * theta_cos,
        vector[2],
    ]
}

fn rotate_about_x(vector: [f32; 3], theta: f32) -> [f32; 3] {
    let theta = theta.to_radians();
    let theta_cos = theta.cos();
    let theta_sin = theta.sin();

    [
        vector[0],
        vector[1] * theta_cos + vector[2] * theta_sin,
        vector[1] * theta_sin + vector[2] * theta_cos,
    ]
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
