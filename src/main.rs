use std::{io::Cursor, sync::Arc, time::Instant};
use vulkano::{
    buffer::{
        allocator::{SubbufferAllocator, SubbufferAllocatorCreateInfo},
        Buffer, BufferCreateInfo, BufferError, BufferUsage,
    },
    command_buffer::{
        allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
        RenderPassBeginInfo, SubpassContents,
    },
    descriptor_set::{
        allocator::StandardDescriptorSetAllocator, PersistentDescriptorSet, WriteDescriptorSet,
    },
    device::{
        physical::PhysicalDeviceType, Device, DeviceCreateInfo, DeviceExtensions, DeviceOwned,
        QueueCreateInfo, QueueFlags,
    },
    image::{
        view::ImageView, AttachmentImage, ImageAccess, ImageUsage, ImmutableImage, SwapchainImage,
    },
    instance::{Instance, InstanceCreateInfo},
    memory::allocator::{AllocationCreateInfo, MemoryUsage, StandardMemoryAllocator},
    pipeline::{
        graphics::{
            color_blend::ColorBlendState,
            depth_stencil::{CompareOp, DepthState, DepthStencilState},
            input_assembly::InputAssemblyState,
            vertex_input::Vertex,
            viewport::{Viewport, ViewportState},
        },
        GraphicsPipeline, Pipeline, PipelineBindPoint, StateMode,
    },
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass},
    sampler::{Filter, SamplerAddressMode},
    shader::ShaderModule,
    swapchain::{
        acquire_next_image, AcquireError, Swapchain, SwapchainCreateInfo, SwapchainCreationError,
        SwapchainPresentInfo,
    },
    sync::{self, FlushError, GpuFuture},
    VulkanLibrary,
};
use vulkano_win::VkSurfaceBuild;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use vulkano::format::Format;
use vulkano::image::ImageDimensions;
use vulkano::image::MipmapsCount;
use vulkano::sampler::Sampler;
use vulkano::sampler::SamplerCreateInfo;

use vulkano::command_buffer::PrimaryCommandBufferAbstract;

use vulkano::buffer::Subbuffer;

mod vertex_data;

mod events;

mod biomes;

mod marching_squares;

mod menus;

mod ui;

mod collision;

mod chunks;

const DEPTH_FORMAT: Format = Format::D24_UNORM_S8_UINT; // TODO: work out what this should be

fn main() {
    let instance = get_instance();

    let event_loop = EventLoop::new();
    let surface = WindowBuilder::new()
        .build_vk_surface(&event_loop, instance.clone())
        .unwrap();

    let device_extensions = DeviceExtensions {
        khr_swapchain: true,
        ..DeviceExtensions::empty()
    };
    let (physical_device, queue_family_index) = instance
        .enumerate_physical_devices()
        .unwrap()
        .filter(|p| p.supported_extensions().contains(&device_extensions))
        .filter_map(|p| {
            p.queue_family_properties()
                .iter()
                .enumerate()
                .position(|(i, q)| {
                    q.queue_flags.intersects(QueueFlags::GRAPHICS)
                        && p.surface_support(i as u32, &surface).unwrap_or(false)
                })
                .map(|i| (p, i as u32))
        })
        .min_by_key(|(p, _)| match p.properties().device_type {
            PhysicalDeviceType::DiscreteGpu => 0,
            PhysicalDeviceType::IntegratedGpu => 1,
            PhysicalDeviceType::VirtualGpu => 2,
            PhysicalDeviceType::Cpu => 3,
            PhysicalDeviceType::Other => 4,
            _ => 5,
        })
        .unwrap();

    println!(
        "Using device: {} (type: {:?})",
        physical_device.properties().device_name,
        physical_device.properties().device_type,
    );

    let (device, mut queues) = Device::new(
        physical_device,
        DeviceCreateInfo {
            enabled_extensions: device_extensions,
            queue_create_infos: vec![QueueCreateInfo {
                queue_family_index,
                ..Default::default()
            }],
            ..Default::default()
        },
    )
    .unwrap();

    let queue = queues.next().unwrap();

    let (mut swapchain, images) = {
        let surface_capabilities = device
            .physical_device()
            .surface_capabilities(&surface, Default::default())
            .unwrap();
        let image_format = Some(
            device
                .physical_device()
                .surface_formats(&surface, Default::default())
                .unwrap()[0]
                .0,
        );
        let window = surface.object().unwrap().downcast_ref::<Window>().unwrap();

        window.set_title("I Don't Know!");
        window.set_maximized(true);

        Swapchain::new(
            device.clone(),
            surface.clone(),
            SwapchainCreateInfo {
                min_image_count: surface_capabilities.min_image_count,
                image_format,
                image_extent: window.inner_size().into(),
                image_usage: ImageUsage::COLOR_ATTACHMENT,
                composite_alpha: surface_capabilities
                    .supported_composite_alpha
                    .into_iter()
                    .next()
                    .unwrap(),
                ..Default::default()
            },
        )
        .unwrap()
    };
    // end of initialization

    // start creating buffers
    let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));

    let (vertex_buffers_map, index_buffers_map) = create_buffers_map(&memory_allocator);

    let (vertex_buffers_ui, index_buffers_ui) = create_buffers_ui(&memory_allocator);

    let uniform_buffer_main = SubbufferAllocator::new(
        memory_allocator.clone(),
        SubbufferAllocatorCreateInfo {
            buffer_usage: BufferUsage::UNIFORM_BUFFER,
            ..Default::default()
        },
    );
    // end of creating buffers

    // start creating allocator for command buffer
    let command_buffer_allocator =
        StandardCommandBufferAllocator::new(device.clone(), Default::default());
    // end creating allocator for command buffer

    let mut uploads = AutoCommandBufferBuilder::primary(
        &command_buffer_allocator,
        queue.queue_family_index(),
        CommandBufferUsage::OneTimeSubmit,
    )
    .unwrap();

    let sprites_map = {
        let png_bytes = include_bytes!("sprite_sheet.png").to_vec();
        let cursor = Cursor::new(png_bytes);
        let decoder = png::Decoder::new(cursor);
        let mut reader = decoder.read_info().unwrap();
        let info = reader.info();
        let dimensions = ImageDimensions::Dim2d {
            width: info.width,
            height: info.height,
            array_layers: 1,
        };
        let mut image_data = Vec::new();
        image_data.resize((info.width * info.height * 4) as usize, 0);
        reader.next_frame(&mut image_data).unwrap();

        let image = ImmutableImage::from_iter(
            &memory_allocator,
            image_data,
            dimensions,
            MipmapsCount::One,
            Format::R8G8B8A8_SRGB,
            &mut uploads,
        )
        .unwrap();
        ImageView::new_default(image).unwrap()
    };

    let sprites_text = {
        let png_bytes = include_bytes!("Text.png").to_vec();
        let cursor = Cursor::new(png_bytes);
        let decoder = png::Decoder::new(cursor);
        let mut reader = decoder.read_info().unwrap();
        let info = reader.info();
        let dimensions = ImageDimensions::Dim2d {
            width: info.width,
            height: info.height,
            array_layers: 1,
        };
        let mut image_data = Vec::new();
        image_data.resize((info.width * info.height * 4) as usize, 0);
        reader.next_frame(&mut image_data).unwrap();

        let image = ImmutableImage::from_iter(
            &memory_allocator,
            image_data,
            dimensions,
            MipmapsCount::One,
            Format::R8G8B8A8_SRGB,
            &mut uploads,
        )
        .unwrap();
        ImageView::new_default(image).unwrap()
    };

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

    // start creating shaders
    //let (vertex_shader_map, fragment_shader_map, vertex_shader_text, fragment_shader_text) = create_shaders(device);

    mod vertex_shader_map {
        vulkano_shaders::shader! {
            ty: "vertex",
            path: "src/map_shaders/vertex_shader.glsl",
        }
    }

    mod fragment_shader_map {
        vulkano_shaders::shader! {
            ty: "fragment",
            path: "src/map_shaders/fragment_shader.glsl",
        }
    }

    let vertex_shader_map = vertex_shader_map::load(device.clone()).unwrap();
    let fragment_shader_map = fragment_shader_map::load(device.clone()).unwrap();

    mod vertex_shader_ui {
        vulkano_shaders::shader! {
            ty: "vertex",
            path: "src/ui_shaders/vertex_shader.glsl",
        }
    }

    mod fragment_shader_ui {
        vulkano_shaders::shader! {
            ty: "fragment",
            path: "src/ui_shaders/fragment_shader.glsl",
        }
    }

    let vertex_shader_ui = vertex_shader_ui::load(device.clone()).unwrap();
    let fragment_shader_ui = fragment_shader_ui::load(device.clone()).unwrap();

    // end of creating shaders

    // start creating render pass
    let render_pass = vulkano::single_pass_renderpass!(
        device.clone(),
        attachments: {
            color: {
                load: Clear,
                store: Store,
                format: swapchain.image_format(),
                samples: 1,
            },
            depth: {
                load: Clear,
                store: DontCare,
                format: DEPTH_FORMAT,
                samples: 1,
            }
        },
        pass: {
            color: [color],
            depth_stencil: {depth},
        },
    )
    .unwrap();
    // end of creating render pass

    let (mut pipeline_map, mut pipeline_ui, mut framebuffers) = window_size_dependent_setup(
        &memory_allocator,
        &vertex_shader_map,
        &fragment_shader_map,
        &vertex_shader_ui,
        &fragment_shader_ui,
        &images,
        render_pass.clone(),
    );

    let descriptor_set_allocator = StandardDescriptorSetAllocator::new(device.clone());

    let mut recreate_swapchain = false; // sometimes the swapchain is broken, and need to be fixed

    //let mut previous_frame_end = Some(sync::now(device.clone()).boxed()); // store previous frame
    let mut previous_frame_end = Some(
        uploads
            .build()
            .unwrap()
            .execute(queue.clone())
            .unwrap()
            .boxed(),
    );

    let layout_map = pipeline_map.layout().set_layouts().get(1).unwrap();
    let layout_ui = pipeline_ui.layout().set_layouts().get(1).unwrap();

    let set_sprites_map = PersistentDescriptorSet::new(
        &descriptor_set_allocator,
        layout_map.clone(),
        [WriteDescriptorSet::image_view_sampler(
            1,
            sprites_map,
            sampler.clone(),
        )],
    )
    .unwrap();

    let set_sprites_text = PersistentDescriptorSet::new(
        &descriptor_set_allocator,
        layout_ui.clone(),
        [WriteDescriptorSet::image_view_sampler(
            1,
            sprites_text,
            sampler,
        )],
    )
    .unwrap();

    let mut delta_time_sum = 0.0;
    let mut average_fps = 0.0;
    let mut frame_count = 0.0;
    let mut delta_time = 0.0;
    let mut time = Instant::now();

    let mut vertex_counts_map = [0u32, 0u32];
    let mut index_counts_map = [0u32, 0u32];

    let mut vertex_counts_ui = [0u32, 0u32];
    let mut index_counts_ui = [0u32, 0u32];

    let mut render_storage = events::RenderStorage {
        vertices_map: vec![
            vertex_data::MapVertex {
                position: [0.0, 0.0, 0.0],
                uv: [0.0, 0.0],
            };
            vertex_buffers_map[0].len() as usize
        ],
        vertex_count_map: 0,
        indices_map: vec![0u32; index_buffers_map[0].len() as usize],
        index_count_map: 0,
        vertices_ui: vec![
            vertex_data::UIVertex {
                position: [0.0, 0.0],
                uv: [0.0, 0.0],
                colour: [0.0, 0.0, 0.0, 0.0],
            };
            vertex_buffers_ui[0].len() as usize
        ],
        vertex_count_ui: 0,
        indices_ui: vec![0u32; index_buffers_ui[0].len() as usize],
        index_count_ui: 0,
        aspect_ratio: 0.0,
        camera: events::Camera {
            scale: 1.0,
            position: (0.0, 0.0),
        },
        brightness: 0.0,
        frame_count: 0,
        starting_time: Instant::now(),
        window_size: [0, 0],
    };

    let mut user_storage = events::start(&mut render_storage);

    // start event loop
    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }

            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                ..
            } => {
                recreate_swapchain = true;
            }

            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { input, .. },
                ..
            } => {
                events::on_keyboard_input(&mut user_storage, &mut render_storage, input);
            }

            Event::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                ..
            } => {
                events::on_cursor_moved(&mut user_storage, &mut render_storage, position);
            }

            Event::WindowEvent {
                event: WindowEvent::MouseInput { state, button, .. },
                ..
            } => {
                events::on_mouse_input(&mut user_storage, &mut render_storage, state, button);
            }

            Event::RedrawEventsCleared => {
                // This should run once per frame.
                let window = surface.object().unwrap().downcast_ref::<Window>().unwrap();

                let dimensions = window.inner_size();

                if dimensions.width == 0 || dimensions.height == 0 {
                    // If the window is 0 in size, don't bother drawing the frame.
                    return;
                }

                previous_frame_end.as_mut().unwrap().cleanup_finished(); // clean up memory

                render_storage.aspect_ratio =
                    swapchain.image_extent()[1] as f32 / swapchain.image_extent()[0] as f32;

                let previous_window_size = render_storage.window_size;
                render_storage.window_size = swapchain.image_extent();

                if previous_window_size != render_storage.window_size {
                    events::on_window_resize(&mut user_storage, &mut render_storage);
                }

                events::update(
                    &mut user_storage,
                    &mut render_storage,
                    //vertex_writer.unwrap(),
                    //index_writer.unwrap(),
                    //&mut index_count,
                    //swapchain.image_extent()[1] as f32 / swapchain.image_extent()[0] as f32,
                    delta_time,
                    average_fps,
                    //&mut camera,
                    //&mut brightness,
                ); // call update once per frame

                update_buffers_map(
                    &mut render_storage,
                    &vertex_buffers_map,
                    &mut vertex_counts_map,
                    &index_buffers_map,
                    &mut index_counts_map,
                );

                update_buffers_ui(
                    &mut render_storage,
                    &vertex_buffers_ui,
                    &mut vertex_counts_ui,
                    &index_buffers_ui,
                    &mut index_counts_ui,
                );

                if recreate_swapchain {
                    // When the window resizes we need to recreate everything dependent on the window size.
                    let (new_swapchain, new_images) =
                        match swapchain.recreate(SwapchainCreateInfo {
                            image_extent: dimensions.into(),
                            ..swapchain.create_info()
                        }) {
                            Ok(r) => r,
                            Err(SwapchainCreationError::ImageExtentNotSupported { .. }) => return,
                            Err(e) => panic!("failed to recreate swapchain: {e}"),
                        };

                    swapchain = new_swapchain;

                    let (new_pipeline_map, new_pipeline_ui, new_framebuffers) =
                        window_size_dependent_setup(
                            &memory_allocator,
                            &vertex_shader_map,
                            &fragment_shader_map,
                            &vertex_shader_ui,
                            &fragment_shader_ui,
                            &new_images,
                            render_pass.clone(),
                        );

                    pipeline_map = new_pipeline_map;
                    pipeline_ui = new_pipeline_ui;
                    framebuffers = new_framebuffers;

                    recreate_swapchain = false;
                }

                let uniform_buffer_subbuffer = {
                    let uniform_data: vertex_shader_map::Data = vertex_shader_map::Data {
                        scale: swapchain.image_extent()[1] as f32
                            / swapchain.image_extent()[0] as f32,
                        camera_scale: render_storage.camera.scale,
                        camera_position: [
                            render_storage.camera.position.0,
                            render_storage.camera.position.1,
                        ],
                        brightness: render_storage.brightness,
                    };

                    let subbuffer = uniform_buffer_main.allocate_sized().unwrap();

                    *subbuffer.write().unwrap() = uniform_data;

                    subbuffer
                };

                let layout_main = pipeline_map.layout().set_layouts().get(0).unwrap();
                let set_main = PersistentDescriptorSet::new(
                    &descriptor_set_allocator,
                    layout_main.clone(),
                    [WriteDescriptorSet::buffer(0, uniform_buffer_subbuffer)],
                )
                .unwrap();

                // Aquire image to draw on.
                let (image_index, suboptimal, acquire_future) =
                    match acquire_next_image(swapchain.clone(), None) {
                        Ok(r) => r,
                        Err(AcquireError::OutOfDate) => {
                            recreate_swapchain = true;
                            return;
                        }
                        Err(e) => panic!("failed to acquire next image: {e}"),
                    };

                if suboptimal {
                    // Sometimes the image will be messed up. Recreate the swapchain if it is.
                    recreate_swapchain = true;
                }

                let mut builder = AutoCommandBufferBuilder::primary(
                    // Command buffers hold the list of commands.
                    &command_buffer_allocator,
                    queue.queue_family_index(),
                    CommandBufferUsage::OneTimeSubmit,
                )
                .unwrap();

                builder
                    .begin_render_pass(
                        // begin the render pass so we can later draw
                        RenderPassBeginInfo {
                            clear_values: vec![Some([0.0, 0.0, 1.0, 1.0].into()), Some((1.0,0u32).into())], // Sets background colour and something else.
                            ..RenderPassBeginInfo::framebuffer(
                                framebuffers[image_index as usize].clone(),
                            )
                        },
                        SubpassContents::Inline,
                    )
                    .unwrap()
                    //.set_viewport(0, [viewport.clone()])
                    .bind_pipeline_graphics(pipeline_map.clone())
                    .bind_descriptor_sets(
                        PipelineBindPoint::Graphics,
                        pipeline_map.layout().clone(),
                        0,
                        vec![set_main.clone(), set_sprites_map.clone()],
                    )
                    .bind_vertex_buffers(
                        0,
                        vertex_buffers_map[render_storage.frame_count as usize % 2].clone(),
                    )
                    .bind_index_buffer(
                        index_buffers_map[render_storage.frame_count as usize % 2].clone(),
                    )
                    .draw_indexed(
                        index_counts_map[render_storage.frame_count as usize % 2],
                        1,
                        0,
                        0,
                        0,
                    )
                    .unwrap()
                    .bind_pipeline_graphics(pipeline_ui.clone()) // start of text pipeline
                    .bind_descriptor_sets(
                        PipelineBindPoint::Graphics,
                        pipeline_ui.layout().clone(),
                        0,
                        vec![set_main, set_sprites_text.clone()],
                    )
                    .bind_vertex_buffers(
                        0,
                        vertex_buffers_ui[render_storage.frame_count as usize % 2].clone(),
                    )
                    .bind_index_buffer(
                        index_buffers_ui[render_storage.frame_count as usize % 2].clone(),
                    )
                    .draw_indexed(
                        index_counts_ui[render_storage.frame_count as usize % 2],
                        1,
                        0,
                        0,
                        0,
                    )
                    .unwrap()
                    .end_render_pass()
                    .unwrap();

                let command_buffer = builder.build().unwrap(); // Finish building the command buffer.

                let future = previous_frame_end // Stop the gpu from freezing.
                    .take()
                    .unwrap()
                    .join(acquire_future)
                    .then_execute(queue.clone(), command_buffer)
                    .unwrap()
                    .then_swapchain_present(
                        queue.clone(),
                        SwapchainPresentInfo::swapchain_image_index(swapchain.clone(), image_index),
                    )
                    .then_signal_fence_and_flush();

                match future {
                    Ok(future) => {
                        previous_frame_end = Some(future.boxed());
                    }

                    Err(FlushError::OutOfDate) => {
                        recreate_swapchain = true;
                        previous_frame_end = Some(sync::now(device.clone()).boxed());
                    }

                    Err(e) => {
                        match e {
                            FlushError::ResourceAccessError { error, use_ref } => {
                                let use_ref = use_ref.unwrap();
                                println!("ResourceAccessError: {}, More info: command_index:{}, command_name:{}", error, use_ref.command_index, use_ref.command_name);
                                recreate_swapchain = true;
                                previous_frame_end = Some(sync::now(device.clone()).boxed());
                            }
                            _ => {
                                panic!("failed to flush future: {e}");
                            }
                        }
                    }
                }

                render_storage.frame_count += 1;

                // Start calculating time.
                if delta_time_sum > 1.0 {
                    average_fps = frame_count / delta_time_sum;
                    frame_count = 0.0;
                    delta_time_sum = 0.0;
                }

                delta_time = time.elapsed().as_secs_f32();
                delta_time_sum += delta_time;
                frame_count += 1.0;
                time = Instant::now();
                // End calculating time.
            }
            _ => (),
        }
    })
    // end event loop
}

fn window_size_dependent_setup(
    memory_allocator: &StandardMemoryAllocator,
    vertex_shader_map: &ShaderModule,
    fragment_shader_map: &ShaderModule,
    vertex_shader_ui: &ShaderModule,
    fragment_shader_ui: &ShaderModule,
    images: &[Arc<SwapchainImage>],
    render_pass: Arc<RenderPass>,
) -> (
    Arc<GraphicsPipeline>,
    Arc<GraphicsPipeline>,
    Vec<Arc<Framebuffer>>,
) {
    let dimensions = images[0].dimensions().width_height();

    let framebuffers = images
        .iter()
        .map(|image| {
            let view = ImageView::new_default(image.clone()).unwrap();
            let depth = ImageView::new_default(
                AttachmentImage::transient(memory_allocator, dimensions, DEPTH_FORMAT).unwrap(),
            )
            .unwrap();
            Framebuffer::new(
                render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![view, depth],
                    ..Default::default()
                },
            )
            .unwrap()
        })
        .collect::<Vec<_>>();

    let subpass = Subpass::from(render_pass.clone(), 0).unwrap();
    let pipeline_map = GraphicsPipeline::start()
        .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
        .vertex_input_state(vertex_data::MapVertex::per_vertex())
        .input_assembly_state(InputAssemblyState::new())
        .vertex_shader(vertex_shader_map.entry_point("main").unwrap(), ())
        .viewport_state(ViewportState::viewport_fixed_scissor_irrelevant([
            Viewport {
                origin: [0.0, 0.0],
                dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                depth_range: 0.0..1.0,
            },
        ]))
        .fragment_shader(fragment_shader_map.entry_point("main").unwrap(), ())
        //.depth_stencil_simple_depth() // TODO: mess around with
        //.depth_write(true)
        .depth_stencil_state(DepthStencilState {
            depth: Some(DepthState {
                enable_dynamic: false,
                write_enable: StateMode::<bool>::Fixed(true),
                compare_op: StateMode::<CompareOp>::Fixed(CompareOp::Less),
            }),
            depth_bounds: None,
            // depth_bounds: Some(DepthBoundsState {
            //     enable_dynamic: false,
            //     bounds: StateMode::<RangeInclusive<f32>>::Fixed(0.0..=1.0),
            // }),
            stencil: None,
        })
        .color_blend_state(ColorBlendState::new(subpass.num_color_attachments()).blend_alpha())
        .build(memory_allocator.device().clone())
        .unwrap();

    let pipeline_ui = GraphicsPipeline::start()
        .render_pass(Subpass::from(render_pass, 0).unwrap())
        .vertex_input_state(vertex_data::UIVertex::per_vertex())
        .input_assembly_state(InputAssemblyState::new())
        .vertex_shader(vertex_shader_ui.entry_point("main").unwrap(), ())
        .viewport_state(ViewportState::viewport_fixed_scissor_irrelevant([
            Viewport {
                origin: [0.0, 0.0],
                dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                depth_range: 0.0..1.0,
            },
        ]))
        .fragment_shader(fragment_shader_ui.entry_point("main").unwrap(), ())
        .color_blend_state(ColorBlendState::new(subpass.num_color_attachments()).blend_alpha())
        .build(memory_allocator.device().clone())
        .unwrap();

    (pipeline_map, pipeline_ui, framebuffers)
}

fn get_instance() -> Arc<vulkano::instance::Instance> {
    let library = VulkanLibrary::new().unwrap();
    let required_extensions = vulkano_win::required_extensions(&library);
    Instance::new(
        library,
        InstanceCreateInfo {
            enabled_extensions: required_extensions,
            enumerate_portability: true,
            ..Default::default()
        },
    )
    .unwrap()
}

fn create_buffers_map(
    memory_allocator: &StandardMemoryAllocator,
) -> (
    [Subbuffer<[vertex_data::MapVertex]>; 2],
    [Subbuffer<[u32]>; 2],
) {
    let vertex_buffers_map = [
        Buffer::from_iter(
            memory_allocator,
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                usage: MemoryUsage::Upload,
                ..Default::default()
            },
            //*events::STARTING_VERTICES,
            vec![
                vertex_data::MapVertex {
                    position: [0.0, 0.0, 0.0],
                    uv: [0.0, 0.0],
                };
                events::MAX_VERTICES
            ],
        )
        .unwrap(),
        Buffer::from_iter(
            memory_allocator,
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                usage: MemoryUsage::Upload,
                ..Default::default()
            },
            //*events::STARTING_VERTICES,
            vec![
                vertex_data::MapVertex {
                    position: [0.0, 0.0, 0.0],
                    uv: [0.0, 0.0],
                };
                events::MAX_VERTICES
            ],
        )
        .unwrap(),
    ];

    let index_buffers_map = [
        Buffer::from_iter(
            memory_allocator,
            BufferCreateInfo {
                usage: BufferUsage::INDEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                usage: MemoryUsage::Upload,
                ..Default::default()
            },
            //*events::STARTING_INDICES,
            vec![0u32; events::MAX_INDICES],
        )
        .unwrap(),
        Buffer::from_iter(
            memory_allocator,
            BufferCreateInfo {
                usage: BufferUsage::INDEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                usage: MemoryUsage::Upload,
                ..Default::default()
            },
            //*events::STARTING_INDICES,
            vec![0u32; events::MAX_INDICES],
        )
        .unwrap(),
    ];

    (vertex_buffers_map, index_buffers_map)
}

fn create_buffers_ui(
    memory_allocator: &StandardMemoryAllocator,
) -> (
    [Subbuffer<[vertex_data::UIVertex]>; 2],
    [Subbuffer<[u32]>; 2],
) {
    let vertex_buffers_ui = [
        Buffer::from_iter(
            memory_allocator,
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                usage: MemoryUsage::Upload,
                ..Default::default()
            },
            vec![
                vertex_data::UIVertex {
                    position: [0.0, 0.0],
                    uv: [0.0, 0.0],
                    colour: [0.0, 0.0, 0.0, 0.0],
                };
                events::CHUNK_WIDTH_SQUARED as usize * 4 * 30
            ],
        )
        .unwrap(),
        Buffer::from_iter(
            memory_allocator,
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                usage: MemoryUsage::Upload,
                ..Default::default()
            },
            vec![
                vertex_data::UIVertex {
                    position: [0.0, 0.0],
                    uv: [0.0, 0.0],
                    colour: [0.0, 0.0, 0.0, 0.0],
                };
                events::CHUNK_WIDTH_SQUARED as usize * 4 * 30
            ],
        )
        .unwrap(),
    ];

    let index_buffers_ui = [
        Buffer::from_iter(
            memory_allocator,
            BufferCreateInfo {
                usage: BufferUsage::INDEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                usage: MemoryUsage::Upload,
                ..Default::default()
            },
            vec![0u32; events::CHUNK_WIDTH_SQUARED as usize * 6 * 30],
        )
        .unwrap(),
        Buffer::from_iter(
            memory_allocator,
            BufferCreateInfo {
                usage: BufferUsage::INDEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                usage: MemoryUsage::Upload,
                ..Default::default()
            },
            vec![0u32; events::CHUNK_WIDTH_SQUARED as usize * 6 * 30],
        )
        .unwrap(),
    ];

    (vertex_buffers_ui, index_buffers_ui)
}

fn update_buffers_map(
    render_storage: &mut events::RenderStorage,
    vertex_buffers_map: &[Subbuffer<[vertex_data::MapVertex]>; 2],
    vertex_counts_map: &mut [u32; 2],
    index_buffers_map: &[Subbuffer<[u32]>; 2],
    index_counts_map: &mut [u32; 2],
) {
    let vertex_writer_map = vertex_buffers_map[render_storage.frame_count as usize % 2].write();

    match vertex_writer_map {
        Ok(mut writer) => {
            writer[0..render_storage.vertex_count_map as usize].copy_from_slice(
                &render_storage.vertices_map[0..render_storage.vertex_count_map as usize],
            );
            vertex_counts_map[render_storage.frame_count as usize % 2] =
                render_storage.vertex_count_map;
        }
        Err(BufferError::InUseByDevice) => {
            println!("Failed to update vertex buffer. Vertex buffer is being used by the device.")
        }
        Err(e) => panic!("couldn't write to the vertex buffer: {e}"),
    };

    let index_writer = index_buffers_map[render_storage.frame_count as usize % 2].write();

    match index_writer {
        Ok(mut writer) => {
            writer[0..render_storage.index_count_map as usize].copy_from_slice(
                &render_storage.indices_map[0..render_storage.index_count_map as usize],
            );
            index_counts_map[render_storage.frame_count as usize % 2] =
                render_storage.index_count_map;
        }
        Err(BufferError::InUseByDevice) => {
            println!("Failed to update index buffer. Index buffer is being used by the device.")
        }
        Err(e) => panic!("couldn't write to the index buffer: {e}"),
    };
}

fn update_buffers_ui(
    render_storage: &mut events::RenderStorage,
    vertex_buffers_ui: &[Subbuffer<[vertex_data::UIVertex]>; 2],
    vertex_counts_ui: &mut [u32; 2],
    index_buffers_ui: &[Subbuffer<[u32]>; 2],
    index_counts_ui: &mut [u32; 2],
) {
    let vertex_writer_ui = vertex_buffers_ui[render_storage.frame_count as usize % 2].write();

    match vertex_writer_ui {
        Ok(mut writer) => {
            writer[0..render_storage.vertex_count_ui as usize].copy_from_slice(
                &render_storage.vertices_ui[0..render_storage.vertex_count_ui as usize],
            );
            vertex_counts_ui[render_storage.frame_count as usize % 2] =
                render_storage.vertex_count_ui;
        }
        Err(BufferError::InUseByDevice) => {
            println!("Failed to update vertex buffer. Vertex buffer is being used by the device.")
        }
        Err(e) => panic!("couldn't write to the vertex buffer: {e}"),
    };

    let index_writer = index_buffers_ui[render_storage.frame_count as usize % 2].write();

    match index_writer {
        Ok(mut writer) => {
            writer[0..render_storage.index_count_ui as usize].copy_from_slice(
                &render_storage.indices_ui[0..render_storage.index_count_ui as usize],
            );
            index_counts_ui[render_storage.frame_count as usize % 2] =
                render_storage.index_count_ui;
        }
        Err(BufferError::InUseByDevice) => {
            println!("Failed to update index buffer. Index buffer is being used by the device.")
        }
        Err(e) => panic!("couldn't write to the index buffer: {e}"),
    };
}

// fn create_shaders(device: Arc<Device>) -> (Arc<ShaderModule>, Arc<ShaderModule>, Arc<ShaderModule>, Arc<ShaderModule>) {
//     mod vertex_shader_map {
//         vulkano_shaders::shader! {
//             ty: "vertex",
//             path: "src/vertex_shader.glsl",
//         }
//     }

//     mod fragment_shader_map {
//         vulkano_shaders::shader! {
//             ty: "fragment",
//             path: "src/fragment_shader.glsl",
//         }
//     }

//     let vertex_shader_map = vertex_shader_map::load(device.clone()).unwrap();
//     let fragment_shader_map = fragment_shader_map::load(device.clone()).unwrap();

//     mod vertex_shader_text {
//         vulkano_shaders::shader! {
//             ty: "vertex",
//             path: "src/vertex_shader.glsl",
//         }
//     }

//     mod fragment_shader_text {
//         vulkano_shaders::shader! {
//             ty: "fragment",
//             path: "src/fragment_shader.glsl",
//         }
//     }

//     let vertex_shader_text = vertex_shader_map::load(device.clone()).unwrap();
//     let fragment_shader_text = fragment_shader_map::load(device.clone()).unwrap();

//     (vertex_shader_map,fragment_shader_map,vertex_shader_text,fragment_shader_text)
// }
