use std::sync::Arc;
use vulkano::command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer};
use vulkano::pipeline::graphics::input_assembly::PrimitiveTopology;
use vulkano::pipeline::GraphicsPipeline;
use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, KeyboardInput, MouseButton};
use crate::menu_rendering;
use crate::buffer_contents;
use crate::events;

pub const STARTING_MENU: Menu = Menu::Example1;

pub const PNG_BYTES_LIST: [&[u8]; 1] = [include_bytes!("sprite_sheet.png").as_slice()];

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Menu {
    Example1,
    Example3D,
}

impl Menu {
    pub const fn get_data(&self) -> MenuData {
        match *self {
            Menu::Example1 => EXAMPLE1,
            Menu::Example3D => EXAMPLE3D,
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

pub const EXAMPLE1: MenuData = MenuData {
    start: |_user_storage, render_storage| {
        println!("hello world");
        
        render_storage.entire_render_datas = vec![menu_rendering::EntireRenderData {
            render_buffers: menu_rendering::RenderBuffers {
                vertex_buffer: menu_rendering::VertexBuffer::Colour(
                    // menu_rendering::BufferTypes::RenderBuffer(menu_rendering::RenderBuffer::new(
                    //     vertex_data::ColourVertex {
                    //         position: [0.0, 0.0, 0.0],
                    //         colour: [0.0, 0.0, 0.0, 0.0],
                    //     },
                    //     7,
                    //     menu_rendering::EditFrequency::Rarely,
                    //     render_storage.memory_allocator.clone(),
                    //     BufferUsage::VERTEX_BUFFER,
                    // )),
                    menu_rendering::BufferTypes::FrequentAccessRenderBuffer(menu_rendering::FrequentAccessRenderBuffer { buffer: vec![buffer_contents::ColourVertex {position: [0.0, 0.0, 0.0], colour: [0.0, 0.0, 0.0, 0.0],}; 7] })
                ),
                index_buffer: None,
                instance_buffer: None,
                shader_accessible_buffers: None,
                // shader_accessible_buffers: Some(menu_rendering::ShaderAccessibleBuffers {
                //     //uniform_buffer: Some(menu_rendering::UniformBuffer::Test(()))
                //     uniform_buffer: None,
                // }),
            },
            render_call: menu_rendering::RenderCall {
                vertex_shader: menu_rendering::VertexShader::SimpleTest,
                fragment_shader: menu_rendering::FragmentShader::SimpleTest,
                topology: PrimitiveTopology::TriangleStrip,
                depth: true,
            },
        }];

        let entire_render_data = &mut render_storage.entire_render_datas[0];

        // TODO: create macro for assuming a buffer is of a type

        let vertex_buffer = match &mut entire_render_data.render_buffers.vertex_buffer {
            menu_rendering::VertexBuffer::Colour(ref mut vertex_buffer) => {
                if let menu_rendering::BufferTypes::FrequentAccessRenderBuffer(ref mut vertex_buffer) = vertex_buffer {
                    vertex_buffer
                }
                else {
                    panic!()
                }
            }
            _ => panic!()
        };

        vertex_buffer.buffer[0] = buffer_contents::ColourVertex {
            // top left
            position: [-0.5, 0.5, 0.0],
            colour: [1.0, 0.0, 0.0, 1.0],
        };
        vertex_buffer.buffer[1] = buffer_contents::ColourVertex {
            // top right
            position: [0.5, 0.5, 0.0],
            colour: [1.0, 0.0, 0.0, 1.0],
        };
        vertex_buffer.buffer[2] = buffer_contents::ColourVertex {
            // bottom left
            position: [-0.5, -0.5, 0.0],
            colour: [1.0, 0.0, 0.0, 1.0],
        };
        vertex_buffer.buffer[3] = buffer_contents::ColourVertex {
            // bottom right
            position: [0.5, -0.5, 0.0],
            colour: [1.0, 0.0, 0.0, 1.0],
        };

        vertex_buffer.buffer[4] = buffer_contents::ColourVertex {
            position: [0.0, -1.0, 0.0],
            colour: [1.0, 0.0, 1.0, 1.0],
        };

        vertex_buffer.buffer[5] = buffer_contents::ColourVertex {
            position: [1.0, -1.0, 0.0],
            colour: [0.0, 1.0, 1.0, 1.0],
        };

        vertex_buffer.buffer[6] = buffer_contents::ColourVertex {
            position: [1.0, 0.0, 0.8],
            colour: [1.0, 1.0, 0.0, 1.0],
        };

        render_storage.force_run_window_dependent_setup = true;
    },
    update: |_user_storage, _render_storage, _delta_time, _average_fps| {},
    fixed_update: (0.04, |_user_storage, render_storage| {
        let entire_render_data = &mut render_storage.entire_render_datas[0];

        let vertex_buffer = match &mut entire_render_data.render_buffers.vertex_buffer {
            menu_rendering::VertexBuffer::Colour(ref mut vertex_buffer) => {
                if let menu_rendering::BufferTypes::FrequentAccessRenderBuffer(ref mut vertex_buffer) = vertex_buffer {
                    vertex_buffer
                }
                else {
                    panic!()
                }
            }
            _ => panic!()
        };

        vertex_buffer.buffer[6].position[1] += 0.5 * EXAMPLE1.fixed_update.0;
        vertex_buffer.buffer[6].position[1] = vertex_buffer.buffer[6].position[1] % 1.0;
    }),
    end: |_user_storage, _render_storage| {},
    on_keyboard_input: |_user_storage, _render_storage, _input| {},
    on_window_resize: |_user_storage, _render_storage| {},
    on_cursor_moved: |_user_storage, _render_storage, _position| {},
    on_mouse_input: |_user_storage, _render_storage, _state, _button| {},
    create_pipelines: |_user_storage, _render_storage| vec![],
    on_draw: |_user_storage, _render_storage, _builder| {},
};

pub const EXAMPLE3D: MenuData = MenuData {
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
