use std::{sync::Arc, time::Instant};

use clunky::{
    buffer_contents,
    lost_code::{is_pressed, FixedUpdate, FpsTracker},
    math::{self, Degrees, Matrix4, Radians},
    meshes,
    physics::physics_3d::{
        aabb::AabbCentredOrigin, bodies::{CommonBody, ImmovableCuboid}, solver::{CpuSolver, OutsideOfGridBoundsBehaviour}, verlet::{
            bodies::{Cuboid, Player},
            Particle,
        }
    },
    rendering::{self, draw_instanced, load_images, ImageBytes},
    shaders,
};
use rand::{
    distributions::{Bernoulli, Distribution, Uniform},
    rngs::ThreadRng,
    Rng,
};
use vulkano::{
    buffer::{
        allocator::{SubbufferAllocator, SubbufferAllocatorCreateInfo},
        Buffer, BufferCreateInfo, BufferUsage, Subbuffer,
    },
    command_buffer::{
        allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
        PrimaryCommandBufferAbstract, RenderPassBeginInfo,
    },
    descriptor_set::{
        allocator::StandardDescriptorSetAllocator, PersistentDescriptorSet, WriteDescriptorSet,
    },
    device::{Device, DeviceExtensions, QueueFlags},
    format::{ClearValue, Format},
    image::{
        sampler::{Filter, Sampler, SamplerAddressMode, SamplerCreateInfo},
        view::ImageView,
        Image, ImageCreateInfo, ImageType, ImageUsage,
    },
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
    pipeline::{
        graphics::{
            color_blend::{AttachmentBlend, ColorBlendAttachmentState, ColorBlendState},
            depth_stencil::{CompareOp, DepthState, DepthStencilState},
            input_assembly::{InputAssemblyState, PrimitiveTopology},
            multisample::MultisampleState,
            rasterization::{CullMode, FrontFace, RasterizationState},
            vertex_input::{Vertex, VertexDefinition},
            viewport::{Scissor, Viewport, ViewportState},
            GraphicsPipelineCreateInfo,
        },
        layout::PipelineDescriptorSetLayoutCreateInfo,
        GraphicsPipeline, Pipeline, PipelineBindPoint, PipelineLayout,
        PipelineShaderStageCreateInfo,
    },
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass},
    swapchain::{acquire_next_image, SwapchainCreateInfo, SwapchainPresentInfo},
    sync::{self, GpuFuture},
    Validated, VulkanError,
};
use winit::{
    dpi::PhysicalPosition,
    event::{DeviceEvent, Event, KeyboardInput, MouseScrollDelta, VirtualKeyCode, WindowEvent},
    event_loop::ControlFlow,
    window::{Fullscreen, Window},
};

const FIXED_DELTA_TIME: f32 = 0.04;
const MAX_SUBSTEPS: u32 = 200;
const TESTING_BOX_AMOUNT: usize = 0; //2000;

fn main() {
    //let pain = rust_gpu_easier::wow!(/home/coolcatcoder/Documents/GitHub/Clunky/clunky/src/test_out.rs);
    //println!("{}", pain);
    let (event_loop, window, _surface, device, queue, mut swapchain, swapchain_images) =
        rendering::initiate_general(
            QueueFlags::GRAPHICS | QueueFlags::COMPUTE,
            DeviceExtensions {
                khr_swapchain: true,
                ..DeviceExtensions::empty()
            },
        );

    let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone())); // TODO: work out which of the allocators we actually want to give away

    let command_buffer_allocator =
        StandardCommandBufferAllocator::new(device.clone(), Default::default());

    window.set_fullscreen(Some(Fullscreen::Borderless(None)));

    let descriptor_set_allocator =
        StandardDescriptorSetAllocator::new(device.clone(), Default::default());
    let subbuffer_allocator = SubbufferAllocator::new(
        memory_allocator.clone(),
        SubbufferAllocatorCreateInfo {
            buffer_usage: BufferUsage::UNIFORM_BUFFER
                | BufferUsage::VERTEX_BUFFER
                | BufferUsage::INDEX_BUFFER, // TODO: work out if it is ok to lie about this
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        },
    );

    let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
        &command_buffer_allocator,
        queue.queue_family_index(),
        CommandBufferUsage::OneTimeSubmit,
    )
    .unwrap();

    let sprites = load_images(
        [ImageBytes::Png(include_bytes!(
            "../src/sprites/moon_wax_tree.png"
        ))],
        &memory_allocator,
        &mut command_buffer_builder,
    );

    let sampler = Sampler::new(
        device.clone(),
        SamplerCreateInfo {
            mag_filter: Filter::Nearest,
            min_filter: Filter::Nearest,
            address_mode: [SamplerAddressMode::Repeat; 3],
            ..Default::default()
        },
    )
    .unwrap();

    let render_pass = vulkano::single_pass_renderpass!(
        device.clone(),
        attachments: {
            color: {
                format: swapchain.image_format(),
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

    let (mut pipelines, mut framebuffers) =
        window_size_dependent_setup(&memory_allocator, &swapchain_images, &render_pass, &device);

    let mut recreate_swapchain = false;
    let mut window_extent = window.inner_size();
    let mut aspect_ratio = window_extent.width as f32 / window_extent.height as f32;

    let mut previous_frame_end = Some(
        command_buffer_builder
            .build()
            .unwrap()
            .execute(queue.clone())
            .unwrap()
            .boxed(),
    );

    let mut box_buffers = ColourBuffers {
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
    };

    let mut sphere_buffers = ColourBuffers {
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
            meshes::SPHERE_VERTICES.to_owned(), // TODO: this might be slow
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
            meshes::SPHERE_INDICES.to_owned(), // TODO: this might be slow
        )
        .unwrap(),
        instance_buffer: vec![],
    };

    let mut moon_wax_tree_buffers = UvBuffers {
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
            meshes::MOON_WAX_TREE_VERTICES.to_owned(), // TODO: this might be slow
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
            meshes::MOON_WAX_TREE_INDICES.to_owned(), // TODO: this might be slow
        )
        .unwrap(),
        instance_buffer: vec![],
    };

    let mut simple_grass_buffers = ColourBuffers {
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
            meshes::SIMPLE_GRASS_VERTICES.to_owned(), // TODO: this might be slow
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
            meshes::SIMPLE_GRASS_INDICES.to_owned(), // TODO: this might be slow
        )
        .unwrap(),
        instance_buffer: vec![],
    };

    let mut camera_uniform = shaders::colour_3d_instanced_shaders::vertex_shader::CameraData3D {
        position: [0.0, 0.0, 0.0],

        ambient_strength: 0.3,
        specular_strength: 0.5.into(),
        light_colour: [0.5, 0.5, 0.5].into(),
        light_position: [0.0, 1250.0, 0.0].into(), // Y is up here, I don't know. I don't understand. Well I vaguely understand though. This is the translation of the entire world, rather than the light. A rename could be beneficial.

        camera_to_clip: math::Matrix4::IDENTITY_AS_2D_ARRAY,
        world_to_camera: math::Matrix4::IDENTITY_AS_2D_ARRAY,
    };

    let mut wasd_held = [false; 4];
    let mut sprinting = true;
    let mut jump_held = false;

    let mut rotation = [0.0f32; 3];

    let sky_top = generate_islands_circle_technique(
        30,
        -1000.0..-750.0,
        2.0,
        1.0..50.0,
        0.5..1.0,
        0.5..2.0,
        50,
        [0.5, 0.5, 0.0, 1.0],
        sky_middle_get_overall_island_type,
        sky_middle_create_per_piece_type,
    );

    let sky_middle = generate_islands_circle_technique(
        30,
        -750.0..-250.0,
        1.0,
        1.0..50.0,
        0.5..1.0,
        0.5..2.0,
        50,
        [0.0, 1.0, 0.0, 1.0],
        sky_middle_get_overall_island_type,
        sky_middle_create_per_piece_type,
    );

    let sky_bottom = generate_islands_circle_technique(
        10,
        -250.0..-0.0,
        0.75,
        10.0..100.0,
        5.0..10.0,
        0.3..3.0,
        20,
        [0.5, 0.5, 0.5, 1.0],
        sky_bottom_get_overall_island_type,
        sky_bottom_create_per_piece_type,
    );

    let islands = Islands {
        sky_top: sky_top.0,
        sky_middle: sky_middle.0,
        sky_bottom: sky_bottom.0,
    };

    let mut bodies =
        Vec::with_capacity(1 + sky_top.1.len() + sky_middle.1.len() + sky_bottom.1.len());

    bodies.push(CommonBody::Player(Player {
        particle: Particle::from_position([0.0, -1050.0, 0.0]),
        mass: 30.0,
        friction: 5.0,
        restitution: 0.5,
        half_size: [0.5, 1.0, 0.5],
        dampening: [0.0, 0.0, 0.0],
        grounded: false,
    }));

    let mut rng = rand::thread_rng();

    let mut test_box_instances = Vec::with_capacity(TESTING_BOX_AMOUNT);

    for _ in 0..TESTING_BOX_AMOUNT {
        let position = [
            rng.gen_range(-900.0..900.0),
            rng.gen_range(-1000.0..-900.0),
            rng.gen_range(-900.0..900.0),
        ];

        bodies.push(CommonBody::Cuboid(Cuboid {
            particle: Particle::from_position(position),
            half_size: [1.0, 1.0, 1.0],
        }));

        test_box_instances.push(buffer_contents::Colour3DInstance::new(
            [1.0, 0.0, 0.0, 1.0],
            Matrix4::from_translation(position),
        ));
    }

    for aabb in sky_top.1 {
        bodies.push(CommonBody::ImmovableCuboid(ImmovableCuboid { aabb }));
    }

    for aabb in sky_middle.1 {
        bodies.push(CommonBody::ImmovableCuboid(ImmovableCuboid { aabb }));
    }

    for aabb in sky_bottom.1 {
        bodies.push(CommonBody::ImmovableCuboid(ImmovableCuboid { aabb }));
    }

    let mut verlet_solver = CpuSolver::new(
        [0.0, 50.0, 0.0],
        [0.8, 1.0, 0.8],
        [8, 15, 8],
        [-1000.0, -2000.0, -1000.0],
        [250, 300, 250],
        OutsideOfGridBoundsBehaviour::ContinueUpdating,
        bodies,
    );

    let mut altitude = Altitude::BelowAll; // Purposefully wrong, so that everything gets updated on the first frame.

    let mut mouse_sensitivity = 1.0;

    let mut frames_since_start = 0u64;

    let mut fixed_update_runner = FixedUpdate::new(FIXED_DELTA_TIME);

    let mut fps_tracker = FpsTracker::<f32>::new();

    let time_since_start = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }

            Event::WindowEvent {
                event: WindowEvent::Resized(extent),
                ..
            } => {
                recreate_swapchain = true;
                window_extent = extent; // Whispers in the darkness claim that this should not be trusted.
                aspect_ratio = extent.width as f32 / extent.height as f32;
            }

            Event::MainEventsCleared => {
                // game stuff
                {
                    fixed_update_runner.update(MAX_SUBSTEPS, || {
                        fixed_update(
                            &mut wasd_held,
                            &mut jump_held,
                            &mut sprinting,
                            &mut rotation,
                            &mut verlet_solver,
                            &mut altitude,
                            &mut box_buffers,
                            &mut sphere_buffers,
                            &mut moon_wax_tree_buffers,
                            &mut simple_grass_buffers,
                            &islands,
                            &mut camera_uniform,
                            &aspect_ratio,
                        )
                    });

                    for i in 0..TESTING_BOX_AMOUNT {
                        let CommonBody::Cuboid(ref test_box) = verlet_solver.bodies[i + 1] else {
                            panic!();
                        };

                        test_box_instances[i] = buffer_contents::Colour3DInstance::new(
                            [1.0, 0.0, 0.0, 1.0],
                            Matrix4::from_translation(test_box.particle.position).multiply(
                                Matrix4::from_scale(math::mul_3d_by_1d(test_box.half_size, 2.0)),
                            ),
                        );
                    }

                    let strength = ((time_since_start.elapsed().as_secs_f32() / 60.0 / 3.0
                        + std::f32::consts::FRAC_PI_2)
                        .sin()
                        + 1.0)
                        / 2.0; // use desmos before modifying this
                               //println!("light strength: {strength}");

                    camera_uniform.light_colour = [strength; 3].into();

                    update();
                }

                // rendering
                {
                    if window_extent.width == 0 || window_extent.height == 0 {
                        // If the window size is 0 in any dimensions, why bother rendering? You won't see anything.
                        return;
                    }

                    previous_frame_end.as_mut().unwrap().cleanup_finished(); // Cleans up memory.

                    if recreate_swapchain {
                        let (new_swapchain, new_swapchain_images) = swapchain
                            .recreate(SwapchainCreateInfo {
                                image_extent: window_extent.into(),
                                ..swapchain.create_info()
                            })
                            .expect("Expected swapchain to always be able to be recreated.");

                        swapchain = new_swapchain;

                        (pipelines, framebuffers) = window_size_dependent_setup(
                            &memory_allocator,
                            &new_swapchain_images,
                            &render_pass,
                            &device,
                        );

                        recreate_swapchain = false;
                    }

                    // Acquire image to draw on.
                    let (image_index, suboptimal, acquire_future) = match acquire_next_image(
                        swapchain.clone(),
                        None,
                    )
                    .map_err(Validated::unwrap)
                    {
                        Ok(r) => r,
                        Err(VulkanError::OutOfDate) => {
                            recreate_swapchain = true;
                            return;
                        }
                        Err(e) => panic!("Failed to acquire next image: {e}"),
                    };

                    if suboptimal {
                        // Sometimes the image will be messed up. Recreate the swapchain if it is.
                        recreate_swapchain = true;
                    }

                    let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
                        &command_buffer_allocator,
                        queue.queue_family_index(),
                        CommandBufferUsage::OneTimeSubmit,
                    )
                    .unwrap();

                    command_buffer_builder
                        .begin_render_pass(
                            RenderPassBeginInfo {
                                clear_values: vec![
                                    Some([0.0, 0.0, 1.0, 1.0].into()),
                                    //some(ClearValue::)
                                    Some(ClearValue::Depth(1.0)),
                                ], // Sets background colour and something else, likely depth buffer.
                                ..RenderPassBeginInfo::framebuffer(
                                    framebuffers[image_index as usize].clone(),
                                )
                            },
                            Default::default(),
                        )
                        .unwrap();

                    let uniform_buffer = subbuffer_allocator.allocate_sized().unwrap();
                    *uniform_buffer.write().unwrap() = camera_uniform;

                    command_buffer_builder
                        .bind_descriptor_sets(
                            PipelineBindPoint::Graphics,
                            pipelines.colour_pipeline.layout().clone(),
                            0,
                            vec![PersistentDescriptorSet::new(
                                &descriptor_set_allocator,
                                pipelines
                                    .colour_pipeline
                                    .layout()
                                    .set_layouts()
                                    .get(0)
                                    .unwrap()
                                    .clone(),
                                [WriteDescriptorSet::buffer(0, uniform_buffer.clone())],
                                [],
                            )
                            .unwrap()],
                        )
                        .unwrap();

                    command_buffer_builder
                        .bind_pipeline_graphics(pipelines.colour_pipeline.clone())
                        .unwrap();

                    if test_box_instances.len() != 0 {
                        draw_instanced(
                            &mut command_buffer_builder,
                            &test_box_instances,
                            &box_buffers.vertex_buffer,
                            &box_buffers.index_buffer,
                            &subbuffer_allocator,
                        );
                    }
                    if box_buffers.instance_buffer.len() != 0 {
                        draw_instanced(
                            &mut command_buffer_builder,
                            &box_buffers.instance_buffer,
                            &box_buffers.vertex_buffer,
                            &box_buffers.index_buffer,
                            &subbuffer_allocator,
                        );
                    }
                    if sphere_buffers.instance_buffer.len() != 0 {
                        draw_instanced(
                            &mut command_buffer_builder,
                            &sphere_buffers.instance_buffer,
                            &sphere_buffers.vertex_buffer,
                            &sphere_buffers.index_buffer,
                            &subbuffer_allocator,
                        );
                    }
                    if simple_grass_buffers.instance_buffer.len() != 0 {
                        draw_instanced(
                            &mut command_buffer_builder,
                            &simple_grass_buffers.instance_buffer,
                            &simple_grass_buffers.vertex_buffer,
                            &simple_grass_buffers.index_buffer,
                            &subbuffer_allocator,
                        );
                    }

                    command_buffer_builder
                        .bind_pipeline_graphics(pipelines.uv_pipeline.clone())
                        .unwrap();

                    command_buffer_builder
                        .bind_descriptor_sets(
                            PipelineBindPoint::Graphics,
                            pipelines.uv_pipeline.layout().clone(),
                            0,
                            vec![
                                PersistentDescriptorSet::new(
                                    &descriptor_set_allocator,
                                    pipelines
                                        .uv_pipeline
                                        .layout()
                                        .set_layouts()
                                        .get(0)
                                        .unwrap()
                                        .clone(),
                                    [WriteDescriptorSet::buffer(0, uniform_buffer.clone())],
                                    [],
                                )
                                .unwrap(),
                                PersistentDescriptorSet::new(
                                    &descriptor_set_allocator,
                                    pipelines
                                        .uv_pipeline
                                        .layout()
                                        .set_layouts()
                                        .get(1)
                                        .unwrap()
                                        .clone(),
                                    [
                                        WriteDescriptorSet::sampler(0, sampler.clone()),
                                        WriteDescriptorSet::image_view(1, sprites[0].clone()),
                                    ],
                                    [],
                                )
                                .unwrap(),
                            ],
                        )
                        .unwrap();

                    if moon_wax_tree_buffers.instance_buffer.len() != 0 {
                        draw_instanced(
                            &mut command_buffer_builder,
                            &moon_wax_tree_buffers.instance_buffer,
                            &moon_wax_tree_buffers.vertex_buffer,
                            &moon_wax_tree_buffers.index_buffer,
                            &subbuffer_allocator,
                        );
                    }

                    command_buffer_builder
                        .end_render_pass(Default::default())
                        .unwrap();

                    let command_buffer = command_buffer_builder.build().unwrap();

                    let future = previous_frame_end
                        .take()
                        .unwrap()
                        .join(acquire_future)
                        .then_execute(queue.clone(), command_buffer)
                        .unwrap()
                        .then_swapchain_present(
                            queue.clone(),
                            SwapchainPresentInfo::swapchain_image_index(
                                swapchain.clone(),
                                image_index,
                            ),
                        )
                        .then_signal_fence_and_flush();

                    match future.map_err(Validated::unwrap) {
                        Ok(future) => {
                            previous_frame_end = Some(future.boxed());
                        }
                        Err(VulkanError::OutOfDate) => {
                            recreate_swapchain = true;
                            previous_frame_end = Some(sync::now(device.clone()).boxed());
                        }
                        Err(e) => {
                            panic!("Failed to flush future: {e}");
                        }
                    }
                }

                frames_since_start += 1;

                fps_tracker.update();
            }

            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { input, .. },
                ..
            } => {
                on_keyboard_input(
                    input,
                    &mut wasd_held,
                    &mut sprinting,
                    &mut jump_held,
                    &window,
                    control_flow,
                    fps_tracker.average_fps(),
                );
            }

            Event::DeviceEvent {
                event: DeviceEvent::Motion { axis, value },
                ..
            } => {
                if !window.has_focus() {
                    return;
                }

                match axis {
                    0 => rotation[1] += value as f32 * mouse_sensitivity,
                    1 => rotation[0] -= value as f32 * mouse_sensitivity,
                    _ => (),
                }

                window
                    .set_cursor_position(PhysicalPosition::new(
                        window_extent.width / 2,
                        window_extent.height / 2,
                    ))
                    .unwrap();
                window.set_cursor_visible(false);
            }

            Event::DeviceEvent {
                event: DeviceEvent::MouseWheel { delta },
                ..
            } => {
                if !window.has_focus() {
                    return;
                }

                mouse_sensitivity += if let MouseScrollDelta::PixelDelta(delta) = delta {
                    delta.y as f32
                } else if let MouseScrollDelta::LineDelta(_, delta_y) = delta {
                    delta_y
                } else {
                    unreachable!()
                }
            }

            _ => (),
        }
    })
}

fn fixed_update(
    wasd_held: &mut [bool; 4],
    jump_held: &mut bool,
    sprinting: &mut bool,
    rotation: &mut [f32; 3],
    verlet_solver: &mut CpuSolver<f32, CommonBody<f32>>,
    altitude: &mut Altitude,
    box_buffers: &mut ColourBuffers,
    sphere_buffers: &mut ColourBuffers,
    moon_wax_tree_buffers: &mut UvBuffers,
    simple_grass_buffers: &mut ColourBuffers,
    islands: &Islands,
    camera_uniform: &mut shaders::colour_3d_instanced_shaders::vertex_shader::CameraData3D,
    aspect_ratio: &f32,
) {
    let CommonBody::Player(player) = &mut verlet_solver.bodies[0] else {
        unreachable!();
    };

    let motion = match wasd_held {
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

    let speed = match (sprinting, *jump_held, player.grounded) {
        (false, true, true) | (false, false, true) | (false, true, false) => 25.0,
        (true, true, true) | (true, false, true) | (false, false, false) | (true, true, false) => {
            50.0
        }
        (true, false, false) => 100.0,
    };

    let real_motion = (motion.0 * speed, motion.1 * speed);

    let y_rotation_cos = rotation[1].to_radians().cos();
    let y_rotation_sin = rotation[1].to_radians().sin();

    let real_motion = (
        real_motion.0 * y_rotation_cos - real_motion.1 * y_rotation_sin,
        real_motion.1 * y_rotation_cos + real_motion.0 * y_rotation_sin,
    );

    player
        .particle
        .accelerate([real_motion.0, 0.0, real_motion.1]);

    let horizontal_dampening = if player.grounded { 0.8 } else { 0.95 }; // grounded originally 0.8

    player.dampening = [horizontal_dampening, 1.0, horizontal_dampening]; // y 0.98 originally

    verlet_solver.update(FIXED_DELTA_TIME); // This function is not the slow one.

    let CommonBody::Player(player) = &mut verlet_solver.bodies[0] else {
        unreachable!();
    };

    if *jump_held {
        if player.grounded {
            player.particle.accelerate([0.0, -1000.0, 0.0]);
        } else {
            if wasd_held[0] || wasd_held[1] || wasd_held[2] || wasd_held[3] {
                player.particle.accelerate([0.0, -50.0, 0.0]);
            } else {
                player.particle.accelerate([0.0, -300.0, 0.0]);
            }
        }
    }

    let previous_altitude = *altitude;
    *altitude = Altitude::get_altitude(player.particle.position[1]);

    if *altitude != previous_altitude {
        (
            box_buffers.instance_buffer,
            sphere_buffers.instance_buffer,
            moon_wax_tree_buffers.instance_buffer,
            simple_grass_buffers.instance_buffer,
        ) = islands.update_altitude_and_get_instances(*altitude);
    }

    camera_uniform.position = [
        player.particle.position[0],
        player.particle.position[1] - 1.0,
        player.particle.position[2],
    ];

    camera_uniform.world_to_camera = Matrix4::from_scale([1.0, 1.0, 1.0])
        .multiply(Matrix4::from_angle_x(Degrees(rotation[0]).to_radians()))
        .multiply(Matrix4::from_angle_y(Degrees(rotation[1]).to_radians()))
        .multiply(Matrix4::from_angle_z(Degrees(rotation[2]).to_radians()))
        .multiply(Matrix4::from_translation([
            -player.particle.position[0],
            -(player.particle.position[1] - 1.0),
            -player.particle.position[2],
        ]))
        .as_2d_array();

    camera_uniform.camera_to_clip = Matrix4::from_perspective(
        Radians(std::f32::consts::FRAC_PI_2),
        *aspect_ratio,
        0.01,
        1000.0,
    )
    .as_2d_array();
}

fn update() {}
struct Islands {
    sky_top: Layer,
    sky_middle: Layer,
    sky_bottom: Layer,
}

impl Islands {
    fn altitude_to_layer(&self, altitude: Altitude) -> &Layer {
        match altitude {
            Altitude::AboveAll => {
                panic!("Currently this altitude does not have a layer, this may change.")
            }
            Altitude::SkyTop => &self.sky_top,
            Altitude::SkyMiddle => &self.sky_middle,
            Altitude::SkyBottom => &self.sky_bottom,
            Altitude::BelowAll => {
                panic!("Currently this altitude does not have a layer, this may change.")
            }
        }
    }

    fn update_altitude_and_get_instances(
        &self,
        altitude: Altitude,
    ) -> (
        Vec<buffer_contents::Colour3DInstance>, // box
        Vec<buffer_contents::Colour3DInstance>, // sphere
        Vec<buffer_contents::Uv3DInstance>,     // moon wax tree
        Vec<buffer_contents::Colour3DInstance>, // simple grass
    ) {
        let altitude_index = altitude.to_u8();

        let mut box_instances = vec![];
        let mut sphere_instances = vec![];

        let mut moon_wax_tree_instances = vec![];
        let mut simple_grass_instances = vec![];

        if altitude_index != 0 {
            if altitude_index != 1 {
                let layer = self.altitude_to_layer(Altitude::from_u8(altitude_index - 1));

                box_instances.append(&mut layer.box_instances.clone());
                sphere_instances.append(&mut layer.sphere_instances.clone());

                moon_wax_tree_instances.append(&mut layer.moon_wax_tree_instances.clone());
                simple_grass_instances.append(&mut layer.simple_grass_instances.clone());
            }

            if altitude_index != 4 {
                let layer = self.altitude_to_layer(altitude);

                box_instances.append(&mut layer.box_instances.clone());
                sphere_instances.append(&mut layer.sphere_instances.clone());

                moon_wax_tree_instances.append(&mut layer.moon_wax_tree_instances.clone());
                simple_grass_instances.append(&mut layer.simple_grass_instances.clone());
            }
        }

        if altitude_index != 3 && altitude_index != 4 {
            let layer = self.altitude_to_layer(Altitude::from_u8(altitude_index + 1));

            box_instances.append(&mut layer.box_instances.clone());
            sphere_instances.append(&mut layer.sphere_instances.clone());

            moon_wax_tree_instances.append(&mut layer.moon_wax_tree_instances.clone());
            simple_grass_instances.append(&mut layer.simple_grass_instances.clone());
        }

        if box_instances.len() == 0 {
            box_instances.push(buffer_contents::Colour3DInstance::new(
                [0.0, 0.0, 0.0, 0.0],
                math::Matrix4::from_scale([0.0, 0.0, 0.0]),
            ));
        }
        if sphere_instances.len() == 0 {
            sphere_instances.push(buffer_contents::Colour3DInstance::new(
                [0.0, 0.0, 0.0, 0.0],
                math::Matrix4::from_scale([0.0, 0.0, 0.0]),
            ));
        }

        if moon_wax_tree_instances.len() == 0 {
            moon_wax_tree_instances.push(buffer_contents::Uv3DInstance::new(
                [0.0, 0.0],
                math::Matrix4::from_scale([0.0, 0.0, 0.0]),
            ));
        }
        if simple_grass_instances.len() == 0 {
            simple_grass_instances.push(buffer_contents::Colour3DInstance::new(
                [0.0, 0.0, 0.0, 0.0],
                math::Matrix4::from_scale([0.0, 0.0, 0.0]),
            ));
        }

        (
            box_instances,
            sphere_instances,
            moon_wax_tree_instances,
            simple_grass_instances,
        )
    }
}

struct Layer {
    box_instances: Vec<buffer_contents::Colour3DInstance>,
    sphere_instances: Vec<buffer_contents::Colour3DInstance>,

    moon_wax_tree_instances: Vec<buffer_contents::Uv3DInstance>,

    simple_grass_instances: Vec<buffer_contents::Colour3DInstance>,
}

#[derive(PartialEq, Clone, Copy, Debug)]
enum Altitude {
    AboveAll,

    SkyTop,
    SkyMiddle,
    SkyBottom,

    BelowAll,
}

impl Altitude {
    fn get_altitude(y: f32) -> Altitude {
        if y < -1000.0 {
            Altitude::AboveAll
        } else if y < -750.0 {
            Altitude::SkyTop
        } else if y < -250.0 {
            Altitude::SkyMiddle
        } else if y < -0.0 {
            Altitude::SkyBottom
        } else {
            Altitude::BelowAll
        }
    }

    const fn to_u8(&self) -> u8 {
        match self {
            Altitude::AboveAll => 0,

            Altitude::SkyTop => 1,
            Altitude::SkyMiddle => 2,
            Altitude::SkyBottom => 3,

            Altitude::BelowAll => 4,
        }
    }

    const fn from_u8(index: u8) -> Altitude {
        match index {
            0 => Altitude::AboveAll,

            1 => Altitude::SkyTop,
            2 => Altitude::SkyMiddle,
            3 => Altitude::SkyBottom,

            4 => Altitude::BelowAll,
            _ => panic!("Index does not map to an altitude."),
        }
    }
}

fn on_keyboard_input(
    input: KeyboardInput,
    wasd_held: &mut [bool; 4],
    sprinting: &mut bool,
    jump_held: &mut bool,
    window: &Arc<Window>,
    control_flow: &mut ControlFlow,
    average_fps: f32,
) {
    if let Some(key_code) = input.virtual_keycode {
        match key_code {
            VirtualKeyCode::W => wasd_held[0] = is_pressed(input.state),
            VirtualKeyCode::A => wasd_held[1] = is_pressed(input.state),
            VirtualKeyCode::S => wasd_held[2] = is_pressed(input.state),
            VirtualKeyCode::D => wasd_held[3] = is_pressed(input.state),

            VirtualKeyCode::Backslash => {
                if is_pressed(input.state) {
                    if let None = window.fullscreen() {
                        window.set_fullscreen(Some(Fullscreen::Borderless(None)));
                    } else {
                        window.set_fullscreen(None);
                    }
                }
            }

            VirtualKeyCode::F => {
                if is_pressed(input.state) {
                    *sprinting = !*sprinting;
                }
            }

            VirtualKeyCode::Space => *jump_held = is_pressed(input.state),

            VirtualKeyCode::Delete => {
                if is_pressed(input.state) {
                    *control_flow = ControlFlow::Exit;
                }
            }

            VirtualKeyCode::X => {
                if is_pressed(input.state) {
                    println!("{average_fps}");
                }
            }
            _ => (),
        }
    }
}

struct ColourBuffers {
    vertex_buffer: Subbuffer<[buffer_contents::Basic3DVertex]>,
    index_buffer: Subbuffer<[u32]>,
    instance_buffer: Vec<buffer_contents::Colour3DInstance>,
}

struct UvBuffers {
    vertex_buffer: Subbuffer<[buffer_contents::Uv3DVertex]>,
    index_buffer: Subbuffer<[u32]>,
    instance_buffer: Vec<buffer_contents::Uv3DInstance>,
}

fn window_size_dependent_setup(
    memory_allocator: &Arc<StandardMemoryAllocator>,
    swapchain_images: &[Arc<Image>],
    render_pass: &Arc<RenderPass>,
    device: &Arc<Device>,
) -> (Pipelines, Vec<Arc<Framebuffer>>) {
    let swapchain_image_extent = swapchain_images[0].extent();

    let depth_buffer_view = ImageView::new_default(
        Image::new(
            memory_allocator.clone(),
            ImageCreateInfo {
                image_type: ImageType::Dim2d,
                format: Format::D32_SFLOAT,
                extent: swapchain_image_extent,
                usage: ImageUsage::TRANSIENT_ATTACHMENT | ImageUsage::DEPTH_STENCIL_ATTACHMENT,
                ..Default::default()
            },
            AllocationCreateInfo::default(),
        )
        .unwrap(),
    )
    .unwrap();

    let framebuffers = swapchain_images
        .iter()
        .map(|swapchain_image| {
            let swapchain_image_view = ImageView::new_default(swapchain_image.clone()).unwrap();
            Framebuffer::new(
                render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![swapchain_image_view, depth_buffer_view.clone()],
                    ..Default::default()
                },
            )
            .unwrap()
        })
        .collect::<Vec<_>>();

    let pipelines = Pipelines {
        uv_pipeline: get_uv_pipeline(device, swapchain_image_extent, render_pass),
        colour_pipeline: get_colour_pipeline(device, swapchain_image_extent, render_pass),
    };

    (pipelines, framebuffers)
}

struct Pipelines {
    uv_pipeline: Arc<GraphicsPipeline>,
    colour_pipeline: Arc<GraphicsPipeline>,
}

fn get_colour_pipeline(
    device: &Arc<Device>,
    extent: [u32; 3],
    render_pass: &Arc<RenderPass>,
) -> Arc<GraphicsPipeline> {
    let vertex_shader_entrance =
        shaders::colour_3d_instanced_shaders::vertex_shader::load(device.clone())
            .unwrap()
            .entry_point("main")
            .unwrap();
    let fragment_shader_entrance =
        shaders::colour_3d_instanced_shaders::fragment_shader::load(device.clone())
            .unwrap()
            .entry_point("main")
            .unwrap();

    let vertex_input_state = [
        buffer_contents::Basic3DVertex::per_vertex(),
        buffer_contents::Colour3DInstance::per_instance(),
    ]
    .definition(&vertex_shader_entrance.info().input_interface)
    .unwrap();

    let stages = [
        PipelineShaderStageCreateInfo::new(vertex_shader_entrance),
        PipelineShaderStageCreateInfo::new(fragment_shader_entrance),
    ];

    let layout = PipelineLayout::new(
        device.clone(),
        PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
            .into_pipeline_layout_create_info(device.clone())
            .unwrap(),
    )
    .unwrap();

    let subpass = Subpass::from(render_pass.clone(), 0).unwrap();
    GraphicsPipeline::new(
        device.clone(),
        None,
        GraphicsPipelineCreateInfo {
            stages: stages.into_iter().collect(),
            vertex_input_state: Some(vertex_input_state),
            input_assembly_state: Some(InputAssemblyState {
                topology: PrimitiveTopology::TriangleList,
                ..Default::default()
            }),
            viewport_state: Some(ViewportState {
                viewports: [Viewport {
                    offset: [0.0, 0.0],
                    extent: [extent[0] as f32, extent[1] as f32],
                    depth_range: 0.0f32..=1.0,
                }]
                .into(),
                scissors: [Scissor {
                    offset: [0, 0],
                    extent: [extent[0], extent[1]],
                }]
                .into(),
                ..Default::default()
            }),
            rasterization_state: Some(RasterizationState {
                cull_mode: CullMode::Back,
                front_face: FrontFace::CounterClockwise,
                ..Default::default()
            }),
            multisample_state: Some(MultisampleState::default()),
            color_blend_state: Some(ColorBlendState::with_attachment_states(
                subpass.num_color_attachments(),
                ColorBlendAttachmentState {
                    blend: Some(AttachmentBlend::alpha()),
                    ..Default::default()
                },
            )),
            depth_stencil_state: Some(DepthStencilState {
                depth: Some(DepthState {
                    write_enable: true,
                    compare_op: CompareOp::Less,
                }),
                depth_bounds: None,
                stencil: None,
                ..Default::default()
            }),
            subpass: Some(subpass.into()),
            ..GraphicsPipelineCreateInfo::layout(layout)
        },
    )
    .unwrap()
}

fn get_uv_pipeline(
    device: &Arc<Device>,
    extent: [u32; 3],
    render_pass: &Arc<RenderPass>,
) -> Arc<GraphicsPipeline> {
    let vertex_shader_entrance = shaders::uv_3d_instanced_vertex_shader::load(device.clone())
        .unwrap()
        .entry_point("main")
        .unwrap();
    let fragment_shader_entrance = shaders::uv_3d_instanced_fragment_shader::load(device.clone())
        .unwrap()
        .entry_point("main")
        .unwrap();

    let vertex_input_state = [
        buffer_contents::Uv3DVertex::per_vertex(),
        buffer_contents::Uv3DInstance::per_instance(),
    ]
    .definition(&vertex_shader_entrance.info().input_interface)
    .unwrap();

    let stages = [
        PipelineShaderStageCreateInfo::new(vertex_shader_entrance),
        PipelineShaderStageCreateInfo::new(fragment_shader_entrance),
    ];

    let layout = PipelineLayout::new(
        device.clone(),
        PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
            .into_pipeline_layout_create_info(device.clone())
            .unwrap(),
    )
    .unwrap();

    let subpass = Subpass::from(render_pass.clone(), 0).unwrap();
    GraphicsPipeline::new(
        device.clone(),
        None,
        GraphicsPipelineCreateInfo {
            stages: stages.into_iter().collect(),
            vertex_input_state: Some(vertex_input_state),
            input_assembly_state: Some(InputAssemblyState {
                topology: PrimitiveTopology::TriangleList,
                ..Default::default()
            }),
            viewport_state: Some(ViewportState {
                viewports: [Viewport {
                    offset: [0.0, 0.0],
                    extent: [extent[0] as f32, extent[1] as f32],
                    depth_range: 0.0f32..=1.0,
                }]
                .into(),
                scissors: [Scissor {
                    offset: [0, 0],
                    extent: [extent[0], extent[1]],
                }]
                .into(),
                ..Default::default()
            }),
            rasterization_state: Some(RasterizationState {
                cull_mode: CullMode::Back,
                front_face: FrontFace::CounterClockwise,
                ..Default::default()
            }),
            multisample_state: Some(MultisampleState::default()),
            color_blend_state: Some(ColorBlendState::with_attachment_states(
                subpass.num_color_attachments(),
                ColorBlendAttachmentState {
                    blend: Some(AttachmentBlend::alpha()),
                    ..Default::default()
                },
            )),
            depth_stencil_state: Some(DepthStencilState {
                depth: Some(DepthState {
                    write_enable: true,
                    compare_op: CompareOp::Less,
                }),
                depth_bounds: None,
                stencil: None,
                ..Default::default()
            }),
            subpass: Some(subpass.into()),
            ..GraphicsPipelineCreateInfo::layout(layout)
        },
    )
    .unwrap()
}

fn generate_islands_circle_technique<T: Copy>(
    quantity: u32,
    vertical_position: std::ops::Range<f32>,
    squish: f32,
    x_scale: std::ops::Range<f32>,
    y_scale: std::ops::Range<f32>,
    z_scale_compared_to_x: std::ops::Range<f32>,
    max_pieces: u16,
    colour: [f32; 4],
    get_overall_island_type: fn(&mut Layer, &mut ThreadRng) -> T,
    create_per_piece_type: fn(&mut Layer, &mut ThreadRng, [f32; 3], [f32; 3], T),
) -> (Layer, Vec<AabbCentredOrigin<f32>>) {
    let mut layer = Layer {
        box_instances: vec![],
        sphere_instances: vec![],

        moon_wax_tree_instances: vec![],
        simple_grass_instances: vec![],
    };

    let mut aabbs = vec![];

    let mut rng = rand::thread_rng();

    let horizontal_position_range = Uniform::from(-1000.0..1000.0);
    let vertical_position_range = Uniform::from(vertical_position);

    let x_scale_range = Uniform::from(x_scale);
    let no_bias_bool = Bernoulli::new(0.5).unwrap();
    let z_scale_gain = Uniform::from(1.0..z_scale_compared_to_x.end);
    let z_scale_lose = Uniform::from(z_scale_compared_to_x.start..1.0);
    let vertical_scale_range = Uniform::from(y_scale);

    let island_pieces_range = Uniform::from(1..max_pieces);

    let rotation_range = Uniform::from(0.0..360.0);

    for _ in 0..quantity {
        let island_pieces = island_pieces_range.sample(&mut rng);

        let mut previous_position = [
            horizontal_position_range.sample(&mut rng),
            vertical_position_range.sample(&mut rng),
            horizontal_position_range.sample(&mut rng),
        ];

        let x_scale = x_scale_range.sample(&mut rng);

        let z_scale = if no_bias_bool.sample(&mut rng) {
            z_scale_gain.sample(&mut rng)
        } else {
            z_scale_lose.sample(&mut rng)
        };

        let mut previous_scale = [
            x_scale,
            vertical_scale_range.sample(&mut rng),
            x_scale * z_scale,
        ];

        let island_type = get_overall_island_type(&mut layer, &mut rng);

        for _ in 0..island_pieces {
            let rotation: f32 = rotation_range.sample(&mut rng);
            let offset = math::rotate_2d(
                [(previous_scale[0] + previous_scale[2]) * squish, 0.0], // 1.0 is on the edge for squish
                rotation.to_radians(),
            );

            previous_position[0] += offset[0];
            previous_position[2] += offset[1];

            let x_scale = x_scale_range.sample(&mut rng);

            let z_scale = if no_bias_bool.sample(&mut rng) {
                z_scale_gain.sample(&mut rng)
            } else {
                z_scale_lose.sample(&mut rng)
            };

            previous_scale = [
                x_scale,
                vertical_scale_range.sample(&mut rng),
                x_scale * z_scale,
            ];

            layer
                .box_instances
                .push(buffer_contents::Colour3DInstance::new(
                    colour,
                    math::Matrix4::from_translation(previous_position)
                        .multiply(math::Matrix4::from_scale(previous_scale))
                        .multiply(Matrix4::from_scale([2.0, 2.0, 2.0])), // Double scale mul is there only for debugging what the aabbs look like with box instances.
                ));
            aabbs.push(AabbCentredOrigin {
                position: previous_position,
                half_size: previous_scale,
            });

            create_per_piece_type(
                &mut layer,
                &mut rng,
                previous_position,
                previous_scale,
                island_type,
            );
        }
    }

    (layer, aabbs)
}

#[derive(Clone, Copy)]
enum SkyMiddleIslandTypes {
    TallForest,
    SmallForest,
    Plains,
}

fn sky_middle_get_overall_island_type(
    _layer: &mut Layer,
    rng: &mut ThreadRng,
) -> SkyMiddleIslandTypes {
    match rng.gen_range(0..10) {
        0 | 1 => SkyMiddleIslandTypes::TallForest,
        2 | 3 | 4 => SkyMiddleIslandTypes::SmallForest,
        5 | 6 | 7 | 8 | 9 => SkyMiddleIslandTypes::Plains,
        _ => unreachable!(),
    }
}

fn sky_middle_create_per_piece_type(
    layer: &mut Layer,
    rng: &mut ThreadRng,
    position: [f32; 3],
    scale: [f32; 3],
    overall_island_type: SkyMiddleIslandTypes,
) {
    match overall_island_type {
        SkyMiddleIslandTypes::TallForest => {
            if rng.gen() {
                return;
            }

            let trunk_thickness = rng.gen_range(1.0..5.0);
            let tree_scale = [trunk_thickness, rng.gen_range(1.0..10.0), trunk_thickness];
            let tree_position = [position[0], position[1] - tree_scale[1] * 0.5, position[2]];

            layer
                .moon_wax_tree_instances
                .push(buffer_contents::Uv3DInstance::new(
                    [0.0, 0.0],
                    math::Matrix4::from_translation(tree_position)
                        .multiply(math::Matrix4::from_angle_y(
                            math::Degrees(rng.gen_range(0.0..360.0)).to_radians(),
                        ))
                        .multiply(math::Matrix4::from_scale(tree_scale)),
                    //.multiply(math::Matrix4::from_scale([1.0, 1.0, 1.0])),
                ));
        }

        SkyMiddleIslandTypes::SmallForest => {
            if rng.gen() {
                let trunk_thickness = rng.gen_range(0.5..1.5);
                let tree_scale = [trunk_thickness, rng.gen_range(0.5..3.0), trunk_thickness];
                let tree_position = [position[0], position[1] - tree_scale[1] * 0.5, position[2]];

                layer
                    .moon_wax_tree_instances
                    .push(buffer_contents::Uv3DInstance::new(
                        [0.0, 0.0],
                        math::Matrix4::from_translation(tree_position)
                            .multiply(math::Matrix4::from_angle_y(
                                math::Degrees(rng.gen_range(0.0..360.0)).to_radians(),
                            ))
                            .multiply(math::Matrix4::from_scale(tree_scale)),
                    ));
            }
        }

        SkyMiddleIslandTypes::Plains => {
            let grass_amount = (scale[0] * scale[2] / 2.0) as u32; // roughly right

            for _ in 0..grass_amount {
                let grass_position = [
                    rng.gen_range((position[0] - scale[0])..(position[0] + scale[0])),
                    position[1] - scale[1],
                    rng.gen_range((position[2] - scale[2])..(position[2] + scale[2])),
                ];

                let a = grass_position[0] - position[0];
                let b = grass_position[2] - position[2];

                let semi_major_axis = scale[0] * 0.8;
                let semi_minor_axis = scale[2] * 0.8;

                if ((a * a) / (semi_major_axis * semi_major_axis))
                    + ((b * b) / (semi_minor_axis * semi_minor_axis))
                    <= 1.0
                {
                    layer
                        .simple_grass_instances
                        .push(buffer_contents::Colour3DInstance::new(
                            [0.5, 0.0, 1.0, 1.0],
                            math::Matrix4::from_translation(grass_position)
                                .multiply(math::Matrix4::from_angle_y(
                                    math::Degrees(rng.gen_range(0.0..360.0)).to_radians(),
                                ))
                                .multiply(math::Matrix4::from_scale([1.0, 1.0, 1.0])),
                        ));
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
enum SkyBottomIslandTypes {
    RubbledPlains,
    _Lake,
}

fn sky_bottom_get_overall_island_type(
    _layer: &mut Layer,
    _rng: &mut ThreadRng,
) -> SkyBottomIslandTypes {
    SkyBottomIslandTypes::RubbledPlains
}

fn sky_bottom_create_per_piece_type(
    _layer: &mut Layer,
    _rng: &mut ThreadRng,
    _position: [f32; 3],
    _scale: [f32; 3],
    _overall_island_type: SkyBottomIslandTypes,
) {
}
