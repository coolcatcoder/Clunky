use cgmath::Deg;
use cgmath::Matrix4;
use cgmath::Rad;
use cgmath::SquareMatrix;
use vulkano::buffer::BufferUsage;
use vulkano::pipeline::graphics::input_assembly::PrimitiveTopology;
use vulkano::pipeline::graphics::rasterization::CullMode;
use vulkano::pipeline::graphics::rasterization::FrontFace;
use winit::dpi::PhysicalPosition;
use winit::event::DeviceEvent;
use winit::event::Event;
use winit::event::KeyboardInput;
use winit::event::VirtualKeyCode;
use winit::event::WindowEvent;
use winit::window::Fullscreen;

use crate::buffer_contents;
use crate::events;
use crate::lost_code::is_pressed;
use crate::menu_rendering;
use crate::menu_rendering::BufferTypes;
use crate::menu_rendering::EditFrequency;
use crate::menu_rendering::FrequentAccessRenderBuffer;
use crate::menu_rendering::InstanceBuffer;
use crate::menu_rendering::RenderBuffer;
use crate::menu_rendering::UniformBuffer;
use crate::menu_rendering::VertexBuffer;
use crate::menus;
use crate::meshes;

pub const MENU: menus::Data = menus::Data {
    start: |user_storage, render_storage| {
        render_storage.entire_render_datas = vec![
            menu_rendering::EntireRenderData {
                render_buffers: menu_rendering::RenderBuffers {
                    vertex_buffer: menu_rendering::VertexBuffer::Basic3D(
                        menu_rendering::BufferTypes::RenderBuffer(
                            menu_rendering::RenderBuffer::new(
                                buffer_contents::Basic3DVertex {
                                    position: [0.0, 0.0, 0.0],
                                    normal: [0.0, 0.0, 0.0],
                                },
                                meshes::CUBE_VERTICES.len(),
                                menu_rendering::EditFrequency::Rarely,
                                render_storage.memory_allocator.clone(),
                                BufferUsage::VERTEX_BUFFER,
                            ),
                        ),
                    ),
                    index_buffer: Some(BufferTypes::RenderBuffer(RenderBuffer::new(
                        0,
                        meshes::CUBE_INDICES.len(),
                        EditFrequency::Rarely,
                        render_storage.memory_allocator.clone(),
                        BufferUsage::INDEX_BUFFER,
                    ))),
                    instance_buffer: Some(InstanceBuffer::Colour3D(
                        BufferTypes::FrequentAccessRenderBuffer(FrequentAccessRenderBuffer {
                            buffer: Vec::from(meshes::test_scene::CUBE_COLOUR_3D_INSTANCES),
                        }),
                    )),
                    shader_accessible_buffers: Some(menu_rendering::ShaderAccessibleBuffers {
                        uniform_buffer: Some(menu_rendering::UniformBuffer::CameraData3D(
                            menu_rendering::BufferTypes::FrequentAccessRenderBuffer(
                                menu_rendering::FrequentAccessRenderBuffer {
                                    buffer: vec![
                                        {
                                            let view = Matrix4::from_angle_z(Deg(90.0));

                                            //let scale = Matrix4::from_scale(0.01);
                                            let scale = Matrix4::identity();

                                            crate::colour_3d_instanced_vertex_shader::CameraData3D {
                                                position: user_storage.camera_3d_position.into(),

                                                ambient_strength: 0.1,
                                                specular_strength: 0.5.into(),
                                                light_colour: [1.0, 1.0, 1.0].into(),
                                                light_position: [0.0, -5.0, 0.0].into(),

                                                camera_to_clip: cgmath::perspective(
                                                    Rad(std::f32::consts::FRAC_PI_2),
                                                    render_storage.other_aspect_ratio,
                                                    0.01,
                                                    100.0,
                                                )
                                                .into(),
                                                world_to_camera: (view * scale).into(),
                                            }
                                        };
                                        1
                                    ],
                                },
                            ),
                        )),
                        image: None,
                    }),
                },
                settings: menu_rendering::Settings {
                    vertex_shader: menu_rendering::VertexShader::Colour3DInstanced,
                    fragment_shader: menu_rendering::FragmentShader::Colour3DInstanced,
                    topology: PrimitiveTopology::TriangleList,
                    depth: true,
                    cull_mode: CullMode::Back,
                    front_face: FrontFace::Clockwise,
                },
            },
            menu_rendering::EntireRenderData {
                render_buffers: menu_rendering::RenderBuffers {
                    vertex_buffer: menu_rendering::VertexBuffer::Basic3D(
                        menu_rendering::BufferTypes::RenderBuffer(
                            menu_rendering::RenderBuffer::new(
                                buffer_contents::Basic3DVertex {
                                    position: [0.0, 0.0, 0.0],
                                    normal: [0.0, 0.0, 0.0],
                                },
                                meshes::SPHERE_VERTICES.len(),
                                menu_rendering::EditFrequency::Rarely,
                                render_storage.memory_allocator.clone(),
                                BufferUsage::VERTEX_BUFFER,
                            ),
                        ),
                    ),
                    index_buffer: Some(BufferTypes::RenderBuffer(RenderBuffer::new(
                        0,
                        meshes::SPHERE_INDICES.len(),
                        EditFrequency::Rarely,
                        render_storage.memory_allocator.clone(),
                        BufferUsage::INDEX_BUFFER,
                    ))),
                    instance_buffer: Some(InstanceBuffer::Colour3D(
                        BufferTypes::FrequentAccessRenderBuffer(FrequentAccessRenderBuffer {
                            buffer: Vec::from(meshes::test_scene::SPHERE_COLOUR_3D_INSTANCES),
                        }),
                    )),
                    shader_accessible_buffers: Some(menu_rendering::ShaderAccessibleBuffers {
                        uniform_buffer: Some(menu_rendering::UniformBuffer::CameraData3D(
                            menu_rendering::BufferTypes::FrequentAccessRenderBuffer(
                                menu_rendering::FrequentAccessRenderBuffer {
                                    buffer: vec![
                                        {
                                            let view = Matrix4::from_angle_z(Deg(90.0));

                                            //let scale = Matrix4::from_scale(0.01);
                                            let scale = Matrix4::identity();

                                            crate::colour_3d_instanced_vertex_shader::CameraData3D {
                                                position: user_storage.camera_3d_position.into(),

                                                ambient_strength: 0.1,
                                                specular_strength: 0.5.into(),
                                                light_colour: [1.0, 1.0, 1.0].into(),
                                                light_position: [0.0, -5.0, 0.0].into(),

                                                camera_to_clip: cgmath::perspective(
                                                    Rad(std::f32::consts::FRAC_PI_2),
                                                    render_storage.other_aspect_ratio,
                                                    0.01,
                                                    100.0,
                                                )
                                                .into(),
                                                world_to_camera: (view * scale).into(),
                                            }
                                        };
                                        1
                                    ],
                                },
                            ),
                        )),
                        image: None,
                    }),
                },
                settings: menu_rendering::Settings {
                    vertex_shader: menu_rendering::VertexShader::Colour3DInstanced,
                    fragment_shader: menu_rendering::FragmentShader::Colour3DInstanced,
                    topology: PrimitiveTopology::TriangleList,
                    depth: true,
                    cull_mode: CullMode::Back,
                    front_face: FrontFace::Clockwise,
                },
            },
        ];

        let entire_render_data_0 = &mut render_storage.entire_render_datas[0];

        // TODO: create macro for assuming a buffer is of a type

        let VertexBuffer::Basic3D(vertex_buffer) =
            &mut entire_render_data_0.render_buffers.vertex_buffer
        else {
            panic!()
        };
        let BufferTypes::RenderBuffer(vertex_buffer) = vertex_buffer else {
            panic!()
        };

        vertex_buffer.buffer.copy_from_slice(meshes::CUBE_VERTICES);

        vertex_buffer.element_count = meshes::CUBE_VERTICES.len();
        vertex_buffer.update_buffer = true;

        let Some(BufferTypes::RenderBuffer(index_buffer)) =
            &mut entire_render_data_0.render_buffers.index_buffer
        else {
            panic!()
        };

        index_buffer.buffer.copy_from_slice(meshes::CUBE_INDICES);

        index_buffer.element_count = meshes::CUBE_INDICES.len();
        index_buffer.update_buffer = true;

        let entire_render_data_1 = &mut render_storage.entire_render_datas[1];

        let VertexBuffer::Basic3D(vertex_buffer) =
            &mut entire_render_data_1.render_buffers.vertex_buffer
        else {
            panic!()
        };
        let BufferTypes::RenderBuffer(vertex_buffer) = vertex_buffer else {
            panic!()
        };

        vertex_buffer
            .buffer
            .copy_from_slice(meshes::SPHERE_VERTICES);

        vertex_buffer.element_count = meshes::SPHERE_VERTICES.len();
        vertex_buffer.update_buffer = true;

        let Some(BufferTypes::RenderBuffer(index_buffer)) =
            &mut entire_render_data_1.render_buffers.index_buffer
        else {
            panic!()
        };

        index_buffer.buffer.copy_from_slice(meshes::SPHERE_INDICES);

        index_buffer.element_count = meshes::SPHERE_INDICES.len();
        index_buffer.update_buffer = true;

        render_storage.force_run_window_dependent_setup = true;
    },
    update: |_user_storage, _render_storage, _delta_time, _average_fps| {},
    fixed_update: (0.04, |user_storage, render_storage| {
        let entire_render_data_0 = &mut render_storage.entire_render_datas[0];

        // let Some(instance_buffer) = &mut entire_render_data.render_buffers.instance_buffer else {
        //     panic!()
        // };
        // let InstanceBuffer::Colour3D(instance_buffer) = instance_buffer else {
        //     panic!()
        // };
        // let BufferTypes::FrequentAccessRenderBuffer(instance_buffer) = instance_buffer else {
        //     panic!()
        // };

        // instance_buffer.buffer[0] = buffer_contents::Colour3DInstance {
        //     model_to_world_0: translation.x.into(),
        //     model_to_world_1: translation.y.into(),
        //     model_to_world_2: translation.z.into(),
        //     model_to_world_3: translation.w.into(),
        //     colour: [1.0, 0.0, 0.0, 1.0],
        // }

        let Some(uniform_buffer) = &mut entire_render_data_0
            .render_buffers
            .shader_accessible_buffers
        else {
            panic!()
        };
        let Some(uniform_buffer) = &mut uniform_buffer.uniform_buffer else {
            panic!()
        };
        let UniformBuffer::CameraData3D(uniform_buffer) = uniform_buffer else {
            panic!()
        };
        let BufferTypes::FrequentAccessRenderBuffer(uniform_buffer) = uniform_buffer else {
            panic!()
        };

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

        let speed = match user_storage.sprinting {
            true => 3.0,
            false => 1.0,
        };

        let real_motion = (
            -motion.0 * speed * MENU.fixed_update.0,
            -motion.1 * speed * MENU.fixed_update.0,
        );

        let y_rotation_cos = (user_storage.camera_3d_rotation[1].to_radians()).cos();
        let y_rotation_sin = (user_storage.camera_3d_rotation[1].to_radians()).sin();

        let real_motion = (
            real_motion.0 * y_rotation_cos - real_motion.1 * y_rotation_sin,
            real_motion.1 * y_rotation_cos + real_motion.0 * y_rotation_sin,
        );

        user_storage.camera_3d_position[0] += real_motion.0;
        user_storage.camera_3d_position[2] += real_motion.1;

        uniform_buffer.buffer[0].position = user_storage.camera_3d_position.into();

        uniform_buffer.buffer[0].world_to_camera =
            (Matrix4::from_angle_x(Deg(user_storage.camera_3d_rotation[0]))
                * Matrix4::from_angle_y(Deg(user_storage.camera_3d_rotation[1]))
                * Matrix4::from_translation(user_storage.camera_3d_position.into()))
            .into();

        uniform_buffer.buffer[0].camera_to_clip = cgmath::perspective(
            Rad(std::f32::consts::FRAC_PI_2),
            render_storage.other_aspect_ratio,
            0.01,
            100.0,
        )
        .into();

        let temp_uniform_copy = uniform_buffer.buffer[0]; // This is temp because having 2 uniform buffers in this case is not requires, and probably costs us quite a bit of performance and memory. Fix asap with reusable buffers or give up and have good performance only available to those who want to do everything manually.

        let entire_render_data_1 = &mut render_storage.entire_render_datas[1];

        let Some(uniform_buffer) = &mut entire_render_data_1
            .render_buffers
            .shader_accessible_buffers
        else {
            panic!()
        };
        let Some(uniform_buffer) = &mut uniform_buffer.uniform_buffer else {
            panic!()
        };
        let UniformBuffer::CameraData3D(uniform_buffer) = uniform_buffer else {
            panic!()
        };
        let BufferTypes::FrequentAccessRenderBuffer(uniform_buffer) = uniform_buffer else {
            panic!()
        };

        uniform_buffer.buffer[0] = temp_uniform_copy;
    }),
    handle_events: |user_storage, render_storage, event| match event {
        Event::WindowEvent {
            event: WindowEvent::KeyboardInput { input, .. },
            ..
        } => {
            on_keyboard_input(user_storage, render_storage, input);
        }
        Event::DeviceEvent {
            event: DeviceEvent::Motion { axis, value },
            ..
        } => {
            if !render_storage.window.has_focus() {
                return;
            }

            match axis {
                0 => user_storage.camera_3d_rotation[1] += value as f32 * user_storage.sensitivity,
                1 => user_storage.camera_3d_rotation[0] -= value as f32 * user_storage.sensitivity,
                _ => (),
            }

            let window_size = render_storage.window.inner_size();
            render_storage
                .window
                .set_cursor_position(PhysicalPosition::new(
                    window_size.width / 2,
                    window_size.height / 2,
                ))
                .unwrap();
            render_storage.window.set_cursor_visible(false);
        }
        _ => {}
    },
    create_pipelines: |_user_storage, _render_storage| vec![],
    on_draw: |_user_storage, _render_storage, _builder| {},
    end: |_user_storage, _render_storage| {},
};

fn on_keyboard_input(
    user_storage: &mut events::UserStorage,
    render_storage: &mut crate::RenderStorage,
    input: KeyboardInput,
) {
    if let Some(key_code) = input.virtual_keycode {
        match key_code {
            VirtualKeyCode::W => user_storage.wasd_held.0 = is_pressed(input.state),
            VirtualKeyCode::A => user_storage.wasd_held.1 = is_pressed(input.state),
            VirtualKeyCode::S => user_storage.wasd_held.2 = is_pressed(input.state),
            VirtualKeyCode::D => user_storage.wasd_held.3 = is_pressed(input.state),
            VirtualKeyCode::Up => user_storage.zoom_held.0 = is_pressed(input.state),
            VirtualKeyCode::Down => user_storage.zoom_held.1 = is_pressed(input.state),

            VirtualKeyCode::Backslash => {
                if is_pressed(input.state) {
                    if let None = render_storage.window.fullscreen() {
                        render_storage
                            .window
                            .set_fullscreen(Some(Fullscreen::Borderless(None)));
                    } else {
                        render_storage.window.set_fullscreen(None);
                    }
                }
            }

            VirtualKeyCode::F => {
                if is_pressed(input.state) {
                    user_storage.sprinting = !user_storage.sprinting;
                }
            }
            _ => (),
        }
    }
}
