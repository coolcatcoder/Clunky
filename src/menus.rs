use noise::OpenSimplex;
use rand::distributions::Bernoulli;
use rand::distributions::{Distribution, Uniform};
use rand::thread_rng;
use vulkano::command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer};
use vulkano::pipeline::GraphicsPipeline;
use std::sync::{mpsc, Arc};
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::mpsc::TryRecvError;
use vulkano::pipeline::graphics::input_assembly::PrimitiveTopology;
use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, KeyboardInput, MouseButton, VirtualKeyCode};

use crate::chunks;
use crate::events::{self, CHUNK_GRID_WIDTH, UserStorage};
use crate::lost_code;
use crate::menu_rendering;
use crate::perks_and_curses;
use crate::ui;
use crate::vertex_data;
use crate::{biomes, collision};

pub const STARTING_MENU: Menu = Menu::Test3D;

pub const PNG_BYTES_LIST: [&[u8]; 1] = [include_bytes!("sprite_sheet.png").as_slice()];

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Menu {
    Test,
    Test3D,
}

impl Menu {
    pub const fn get_data(&self) -> MenuData {
        match *self {
            Menu::Test => TEST,
            Menu::Test3D => TEST3D,
        }
    }
}

pub struct MenuData {
    pub start: fn(&mut events::UserStorage, &mut crate::RenderStorage),
    pub update: fn(&mut events::UserStorage, &mut crate::RenderStorage, f32, f32),
    pub fixed_update: (f32, fn(&mut events::UserStorage, &mut crate::RenderStorage)),
    pub end: fn(&mut events::UserStorage, &mut crate::RenderStorage),
    pub on_keyboard_input: fn(&mut events::UserStorage, &mut crate::RenderStorage, KeyboardInput),
    pub on_window_resize: fn(&mut events::UserStorage, &mut crate::RenderStorage),
    pub on_cursor_moved:
        fn(&mut events::UserStorage, &mut crate::RenderStorage, PhysicalPosition<f64>),
    pub on_mouse_input:
        fn(&mut events::UserStorage, &mut crate::RenderStorage, ElementState, MouseButton),
    pub create_pipelines: fn(&mut events::UserStorage, &mut crate::RenderStorage) -> Vec<Arc<GraphicsPipeline>>,
    pub on_draw: fn(&mut events::UserStorage, &mut crate::RenderStorage, &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>),
}

impl Default for MenuData {
    fn default() -> Self {
        MenuData {
            start: |_user_storage, _render_storage| {},
            update: |_user_storage, _render_storage, _delta_time, _average_fps| {},
            fixed_update: (f32::INFINITY, |_user_storage, _render_storage| {}),
            end: |_user_storage, _render_storage| {},
            on_keyboard_input: |_user_storage, _render_storage, _input| {},
            on_window_resize: |_user_storage, _render_storage| {},
            on_cursor_moved: |_user_storage, _render_storage, _position| {},
            on_mouse_input: |_user_storage, _render_storage, _state, _button| {},
            create_pipelines: |_user_storage, _render_storage| {vec![]},
            on_draw: |_user_storage, _render_storage, _builder| {},
        }
    }
}

pub const TEST: MenuData = MenuData {
    start: |_user_storage, _render_storage| {
        /*
        println!("Test Menu Start");

        render_storage.vertices_test[0] = vertex_data::TestVertex {
            // top left
            position: [-0.5, 0.5],
            uv: [0.0, biomes::SPRITE_SIZE.1],
        };
        render_storage.vertices_test[1] = vertex_data::TestVertex {
            // top right
            position: [0.5, 0.5],
            uv: [biomes::SPRITE_SIZE.0, biomes::SPRITE_SIZE.1],
        };
        render_storage.vertices_test[2] = vertex_data::TestVertex {
            // bottom left
            position: [-0.5, -0.5],
            uv: [0.0, 0.0],
        };
        render_storage.vertices_test[3] = vertex_data::TestVertex {
            // bottom right
            position: [0.5, -0.5],
            uv: [biomes::SPRITE_SIZE.0, 0.0],
        };

        render_storage.indices_test[0] = 0;
        render_storage.indices_test[1] = 2;
        render_storage.indices_test[2] = 3;

        render_storage.indices_test[3] = 0;
        render_storage.indices_test[4] = 3;
        render_storage.indices_test[5] = 1;

        render_storage.instances_test[0] = vertex_data::TestInstance {
            position_offset: [0.0, 0.0, 0.8],
            scale: [1.0, 1.0],
            uv_centre: [0.0, 0.0],
        };

        render_storage.instances_test[1] = vertex_data::TestInstance {
            position_offset: [0.0, 1.0, 0.5],
            scale: [3.0, 3.0],
            uv_centre: [6.0 * biomes::SPRITE_SIZE.0, 0.0],
        };

        render_storage.vertex_count_test = 4;
        render_storage.index_count_test = 6;
        render_storage.instance_count_test = 2;

        render_storage.update_vertices_test = true;
        render_storage.update_indices_test = true;
        render_storage.update_instances_test = true;
        */
    },
    update: |_user_storage, _render_storage, _delta_time, _average_fps| {},
    end: |_user_storage, _render_storage| {},
    on_keyboard_input: |_user_storage, _render_storage, _input| {},
    on_window_resize: |_user_storage, _render_storage| {},
    on_cursor_moved: |_user_storage, _render_storage, _position| {},
    on_mouse_input: |_user_storage, _render_storage, _state, _button| {},
    ..Default::default()
};

pub const TEST3D: MenuData = MenuData {
    start: |_user_storage, _render_storage| {},
    update: |_user_storage, _render_storage, _delta_time, _average_fps| {},
    end: |_user_storage, _render_storage| {},
    on_keyboard_input: |_user_storage, _render_storage, _input| {},
    on_window_resize: |_user_storage, _render_storage| {},
    on_cursor_moved: |_user_storage, _render_storage, _position| {},
    on_mouse_input: |_user_storage, _render_storage, _state, _button| {},
    ..Default::default()
};
