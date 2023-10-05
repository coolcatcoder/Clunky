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
    image::{view::ImageView, ImageAccess, ImageUsage, ImmutableImage, SwapchainImage},
    instance::{Instance, InstanceCreateInfo},
    memory::allocator::{AllocationCreateInfo, MemoryUsage, StandardMemoryAllocator},
    pipeline::{
        graphics::{
            color_blend::ColorBlendState,
            input_assembly::InputAssemblyState,
            vertex_input::Vertex,
            viewport::{Viewport, ViewportState},
        },
        GraphicsPipeline, Pipeline, PipelineBindPoint,
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

mod vertex_data;

mod events;

mod biomes;

fn main() {
    println!("start of main() before fake_main()");
    fake_main();
}

fn fake_main() {
    // start initialization
    let library = VulkanLibrary::new().unwrap();
    let required_extensions = vulkano_win::required_extensions(&library);
    let instance = Instance::new(
        library,
        InstanceCreateInfo {
            enabled_extensions: required_extensions,
            enumerate_portability: true,
            ..Default::default()
        },
    )
    .unwrap();

    let event_loop = EventLoop::new();
    let surface = WindowBuilder::new()
        .build_vk_surface(&event_loop, instance.clone())
        .unwrap();

    let device_extensions = DeviceExtensions {
        khr_swapchain: true,
        ..DeviceExtensions::empty()
    };
    println!("Last known point.");
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
    println!("you shouldn't see this");

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

    println!("Starting vertex and index allocation");

    let vertex_buffer = Buffer::from_iter(
        &memory_allocator,
        BufferCreateInfo {
            usage: BufferUsage::VERTEX_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            usage: MemoryUsage::Upload,
            ..Default::default()
        },
        *events::STARTING_VERTICES,
    )
    .unwrap();

    let index_buffer = Buffer::from_iter(
        &memory_allocator,
        BufferCreateInfo {
            usage: BufferUsage::INDEX_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            usage: MemoryUsage::Upload,
            ..Default::default()
        },
        *events::STARTING_INDICES,
    )
    .unwrap();

    println!("finished creating vertices and indices");

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

    let texture = {
        let png_bytes = include_bytes!("image_img.png").to_vec();
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
            mag_filter: Filter::Linear,
            min_filter: Filter::Linear,
            address_mode: [SamplerAddressMode::Repeat; 3],
            ..Default::default()
        },
    )
    .unwrap();

    // start creating shaders
    mod vertex_shader {
        vulkano_shaders::shader! {
            ty: "vertex",
            path: "src/vertex_shader.glsl",
        }
    }

    mod fragment_shader {
        vulkano_shaders::shader! {
            ty: "fragment",
            path: "src/fragment_shader.glsl",
        }
    }

    let vertex_shader = vertex_shader::load(device.clone()).unwrap();
    let fragment_shader = fragment_shader::load(device.clone()).unwrap();
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
        },
        pass: {
            color: [color],
            depth_stencil: {},
        },
    )
    .unwrap();
    // end of creating render pass

    let (mut pipeline, mut framebuffers) = window_size_dependent_setup(
        &memory_allocator,
        &vertex_shader,
        &fragment_shader,
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

    let layout_images = pipeline.layout().set_layouts().get(1).unwrap();
    let set_images = PersistentDescriptorSet::new(
        &descriptor_set_allocator,
        layout_images.clone(),
        [WriteDescriptorSet::image_view_sampler(1, texture, sampler)],
    )
    .unwrap();

    let mut delta_time_sum = 0.0;
    let mut average_fps = 0.0;
    let mut frame_count = 0.0;
    let mut delta_time = 0.0;
    let mut time = Instant::now();

    let mut skip_update = false; // Sometimes when the window is being resized or moved, you can't access the buffers and as such must skip an update.

    let mut index_count = events::STARTING_INDEX_COUNT;

    let mut camera = events::Camera {
        scale: 1.0,
        position: (0.0, 0.0),
    };

    let mut storage = events::start(&mut camera);

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

            Event::RedrawEventsCleared => {
                // This should run once per frame.
                let window = surface.object().unwrap().downcast_ref::<Window>().unwrap();

                let dimensions = window.inner_size();
                if dimensions.width == 0 || dimensions.height == 0 {
                    // If the window is 0 in size, don't bother drawing the frame.
                    return;
                }

                previous_frame_end.as_mut().unwrap().cleanup_finished(); // clean up memory

                let vertex_writer = vertex_buffer.write();

                skip_update = match vertex_writer {
                    Ok(_) => skip_update,
                    Err(BufferError::InUseByDevice) => true,
                    Err(e) => panic!("couldn't write to the vertex buffer: {e}"),
                };

                let index_writer = index_buffer.write();

                skip_update = match index_writer {
                    Ok(_) => skip_update,
                    Err(BufferError::InUseByDevice) => true,
                    Err(e) => panic!("couldn't write to the index buffer: {e}"),
                };

                if skip_update {
                    println!("Skipping update!");
                } else {
                    events::update(
                        &mut storage,
                        vertex_writer.unwrap(),
                        index_writer.unwrap(),
                        &mut index_count,
                        swapchain.image_extent()[0] as f32 / swapchain.image_extent()[1] as f32,
                        delta_time,
                        average_fps,
                        &mut camera,
                    ); // call update once per frame
                }

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

                    let (new_pipeline, new_framebuffers) = window_size_dependent_setup(
                        &memory_allocator,
                        &vertex_shader,
                        &fragment_shader,
                        &new_images,
                        render_pass.clone(),
                    );

                    pipeline = new_pipeline;
                    framebuffers = new_framebuffers;

                    recreate_swapchain = false;
                }

                let uniform_buffer_subbuffer = {
                    let uniform_data: vertex_shader::Data = vertex_shader::Data {
                        scale: swapchain.image_extent()[0] as f32
                            / swapchain.image_extent()[1] as f32,
                        camera_scale: camera.scale,
                        camera_position: [camera.position.0, camera.position.1],
                    };

                    let subbuffer = uniform_buffer_main.allocate_sized().unwrap();

                    *subbuffer.write().unwrap() = uniform_data;

                    subbuffer
                };

                let layout_main = pipeline.layout().set_layouts().get(0).unwrap();
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
                            clear_values: vec![Some([0.0, 0.0, 1.0, 1.0].into())], // Sets background colour.
                            ..RenderPassBeginInfo::framebuffer(
                                framebuffers[image_index as usize].clone(),
                            )
                        },
                        SubpassContents::Inline,
                    )
                    .unwrap()
                    //.set_viewport(0, [viewport.clone()])
                    .bind_pipeline_graphics(pipeline.clone())
                    .bind_descriptor_sets(
                        PipelineBindPoint::Graphics,
                        pipeline.layout().clone(),
                        0,
                        vec![set_main, set_images.clone()],
                    )
                    // .bind_descriptor_sets(
                    //     PipelineBindPoint::Graphics,
                    //     pipeline.layout().clone(),
                    //     0,
                    //     set_images.clone(),
                    // )
                    .bind_vertex_buffers(0, vertex_buffer.clone())
                    .bind_index_buffer(index_buffer.clone())
                    .draw_indexed(index_count, 1, 0, 0, 0)
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
                        panic!("failed to flush future: {e}");
                    }
                }

                if skip_update {
                    skip_update = false;
                } else {
                    events::late_update(&mut storage, delta_time, average_fps); // The goal of late update should be to do cpu work while the gpu is doing the hard work of rendering everything. This should save performance if done right.

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
            }
            _ => (),
        }
    })
    // end event loop
}

fn window_size_dependent_setup(
    memory_allocator: &StandardMemoryAllocator,
    vertex_shader: &ShaderModule,
    fragment_shader: &ShaderModule,
    images: &[Arc<SwapchainImage>],
    render_pass: Arc<RenderPass>,
) -> (Arc<GraphicsPipeline>, Vec<Arc<Framebuffer>>) {
    let dimensions = images[0].dimensions().width_height();

    let framebuffers = images
        .iter()
        .map(|image| {
            let view = ImageView::new_default(image.clone()).unwrap();
            Framebuffer::new(
                render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![view],
                    ..Default::default()
                },
            )
            .unwrap()
        })
        .collect::<Vec<_>>();

    let subpass = Subpass::from(render_pass.clone(), 0).unwrap();
    let pipeline = GraphicsPipeline::start()
        .render_pass(Subpass::from(render_pass, 0).unwrap())
        .vertex_input_state(vertex_data::VertexData::per_vertex())
        .input_assembly_state(InputAssemblyState::new())
        .vertex_shader(vertex_shader.entry_point("main").unwrap(), ())
        .viewport_state(ViewportState::viewport_fixed_scissor_irrelevant([
            Viewport {
                origin: [0.0, 0.0],
                dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                depth_range: 0.0..1.0,
            },
        ]))
        .fragment_shader(fragment_shader.entry_point("main").unwrap(), ())
        .color_blend_state(ColorBlendState::new(subpass.num_color_attachments()).blend_alpha())
        .build(memory_allocator.device().clone())
        .unwrap();

    (pipeline, framebuffers)
}
