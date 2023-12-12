use noise::OpenSimplex;
use rand::distributions::Bernoulli;
use rand::distributions::{Distribution, Uniform};
use rand::thread_rng;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::mpsc::TryRecvError;
use std::sync::{mpsc, Arc};
use vulkano::buffer::BufferUsage;
use vulkano::command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer};
use vulkano::pipeline::graphics::input_assembly::PrimitiveTopology;
use vulkano::pipeline::GraphicsPipeline;
use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, KeyboardInput, MouseButton, VirtualKeyCode};

use crate::chunks;
use crate::events::{self, UserStorage, CHUNK_GRID_WIDTH};
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
    pub create_pipelines:
        fn(&mut events::UserStorage, &mut crate::RenderStorage) -> Vec<Arc<GraphicsPipeline>>,
    pub on_draw: fn(
        &mut events::UserStorage,
        &mut crate::RenderStorage,
        &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
    ),
}

// impl MenuData {
//     const fn get_default() -> Self {
//         MenuData {
//             start: |_user_storage, _render_storage| {},
//             update: |_user_storage, _render_storage, _delta_time, _average_fps| {},
//             fixed_update: (f32::INFINITY, |_user_storage, _render_storage| {}),
//             end: |_user_storage, _render_storage| {},
//             on_keyboard_input: |_user_storage, _render_storage, _input| {},
//             on_window_resize: |_user_storage, _render_storage| {},
//             on_cursor_moved: |_user_storage, _render_storage, _position| {},
//             on_mouse_input: |_user_storage, _render_storage, _state, _button| {},
//             create_pipelines: |_user_storage, _render_storage| {vec![]},
//             on_draw: |_user_storage, _render_storage, _builder| {},
//         }
//     }
// }

// impl Default for MenuData {
//     fn default() -> Self {
//         MenuData::get_default()
//     }
// }

pub const TEST: MenuData = MenuData {
    start: |_user_storage, render_storage| {
        render_storage.entire_render_datas = vec![menu_rendering::EntireRenderData {
            render_buffers: menu_rendering::RenderBuffers {
                vertex_buffer: menu_rendering::VertexBuffer::ColourVertexBuffer(
                    menu_rendering::BufferTypes::RenderBuffer(menu_rendering::RenderBuffer::new(
                        vertex_data::ColourVertex {
                            position: [0.0, 0.0, 0.0],
                            colour: [0.0, 0.0, 0.0, 0.0],
                        },
                        4,
                        menu_rendering::EditFrequency::Rarely,
                        render_storage.memory_allocator.clone(),
                        BufferUsage::VERTEX_BUFFER,
                    )),
                ),
                index_buffer: menu_rendering::BufferTypes::RenderBuffer(
                    menu_rendering::RenderBuffer::new(
                        0,
                        6,
                        menu_rendering::EditFrequency::Rarely,
                        render_storage.memory_allocator.clone(),
                        BufferUsage::INDEX_BUFFER,
                    ),
                ),
                instance_buffer: None,
            },
            render_call: menu_rendering::RenderCall {
                vertex_shader: menu_rendering::VertexShader::SimpleTest,
                fragment_shader: menu_rendering::FragmentShader::SimpleTest,
                topology: PrimitiveTopology::TriangleStrip,
                depth: false,
            },
            descriptor_set_and_contained_buffers: None,
        }];

        // TODO: create macro for assuming a buffer is of a type
        let vertex_buffer: &mut menu_rendering::RenderBuffer<vertex_data::ColourVertex> = match &mut render_storage.entire_render_datas[0].render_buffers.vertex_buffer {
            menu_rendering::VertexBuffer::ColourVertexBuffer(ref mut vertex_buffer) => {
                if let menu_rendering::BufferTypes::RenderBuffer(ref mut vertex_buffer) = vertex_buffer {
                    vertex_buffer
                }
                else {
                    panic!()
                }
            }
            _ => panic!()
        };

        let index_buffer: &mut menu_rendering::RenderBuffer<u32> = if let menu_rendering::BufferTypes::RenderBuffer(ref mut index_buffer) = &mut render_storage.entire_render_datas[0].render_buffers.index_buffer {
            index_buffer
        }
        else {
            panic!()
        };

        vertex_buffer.buffer[0] = vertex_data::ColourVertex {
            // top left
            position: [-0.5, 0.5, 0.0],
            colour: [1.0, 0.0, 0.0, 1.0],
        };
        vertex_buffer.buffer[1] = vertex_data::ColourVertex {
            // top right
            position: [0.5, 0.5, 0.0],
            colour: [1.0, 0.0, 0.0, 1.0],
        };
        vertex_buffer.buffer[2] = vertex_data::ColourVertex {
            // bottom left
            position: [-0.5, -0.5, 0.0],
            colour: [1.0, 0.0, 0.0, 1.0],
        };
        vertex_buffer.buffer[3] = vertex_data::ColourVertex {
            // bottom right
            position: [0.5, -0.5, 0.0],
            colour: [1.0, 0.0, 0.0, 1.0],
        };

        index_buffer.buffer[0] = 0;
        index_buffer.buffer[1] = 2;
        index_buffer.buffer[2] = 3;

        index_buffer.buffer[3] = 0;
        index_buffer.buffer[4] = 3;
        index_buffer.buffer[5] = 1;

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
    fixed_update: (f32::INFINITY, |_user_storage, _render_storage| {}),
    end: |_user_storage, _render_storage| {},
    on_keyboard_input: |_user_storage, _render_storage, _input| {},
    on_window_resize: |_user_storage, _render_storage| {},
    on_cursor_moved: |_user_storage, _render_storage, _position| {},
    on_mouse_input: |_user_storage, _render_storage, _state, _button| {},
    create_pipelines: |_user_storage, _render_storage| vec![],
    on_draw: |_user_storage, _render_storage, _builder| {},
};

pub const TEST3D: MenuData = MenuData {
    start: |_user_storage, _render_storage| {},
    update: |_user_storage, _render_storage, _delta_time, _average_fps| {},
    fixed_update: (f32::INFINITY, |_user_storage, _render_storage| {}),
    end: |_user_storage, _render_storage| {},
    on_keyboard_input: |_user_storage, _render_storage, _input| {},
    on_window_resize: |_user_storage, _render_storage| {},
    on_cursor_moved: |_user_storage, _render_storage, _position| {},
    on_mouse_input: |_user_storage, _render_storage, _state, _button| {},
    create_pipelines: |_user_storage, _render_storage| vec![],
    on_draw: |_user_storage, _render_storage, _builder| {},
};
