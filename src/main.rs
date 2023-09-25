use std::{sync::Arc, time::Instant};
use vulkano::{
    buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, BufferError},
    command_buffer::{
        allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
        RenderPassBeginInfo, SubpassContents,
    },
    device::{
        physical::PhysicalDeviceType, Device, DeviceCreateInfo, DeviceExtensions, QueueCreateInfo,
        QueueFlags,
    },
    image::{view::ImageView, ImageAccess, ImageUsage, SwapchainImage},
    instance::{Instance, InstanceCreateInfo},
    memory::allocator::{AllocationCreateInfo, MemoryUsage, StandardMemoryAllocator},
    pipeline::{
        graphics::{
            input_assembly::InputAssemblyState,
            vertex_input::Vertex,
            viewport::{Viewport, ViewportState},
        },
        GraphicsPipeline,
    },
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass},
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

use vulkano::buffer::subbuffer::BufferWriteGuard;

fn main()
{
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

    let vertices = [
        VertexData{position:[-1.0, 1.0]},
        VertexData{position:[1.0, 1.0]},
        VertexData{position:[1.0, -1.0]},
        ];

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
        vertices,
    )
    .unwrap();

    let indices: [u16; 3] = [0,1,2];

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
        indices,
    )
    .unwrap();
    // end of creating buffers

    // start creating shaders
    mod vertex_shader {
        vulkano_shaders::shader! {
            ty: "vertex",
            src: r"
                #version 450

                layout(location = 0) in vec2 position;

                void main() {
                    gl_Position = vec4(position, 0.0, 1.0);
                }
            ",
        }
    }

    mod fragment_shader {
        vulkano_shaders::shader! {
            ty: "fragment",
            src: r"
                #version 450

                layout(location = 0) out vec4 f_color;

                void main() {
                    f_color = vec4(1.0, 1.0, 0.0, 1.0);
                }
            ",
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

    // start creating pipeline
    let pipeline = GraphicsPipeline::start()
        .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
        .vertex_input_state(VertexData::per_vertex())
        .input_assembly_state(InputAssemblyState::new())
        .vertex_shader(vertex_shader.entry_point("main").unwrap(), ())
        .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
        .fragment_shader(fragment_shader.entry_point("main").unwrap(), ())
        .build(device.clone())
        .unwrap();
    // end creating pipeline

    // start creating viewport
    let mut viewport = Viewport {
        origin: [0.0, 0.0],
        dimensions: [0.0, 0.0],
        depth_range: 0.0..1.0,
    };
    // end creating viewport

    // start creating framebuffers
    let mut framebuffers = window_size_dependent_setup(&images, render_pass.clone(), &mut viewport);
    // end creating framebuffers

    // start creating allocator for command buffer
    let command_buffer_allocator = StandardCommandBufferAllocator::new(device.clone(), Default::default());
    // end creating allocator for command buffer

    let mut recreate_swapchain = false; // sometimes the swapchain is broken, and need to be fixed

    let mut previous_frame_end = Some(sync::now(device.clone()).boxed()); // store previous frame

    let mut delta_time_sum = 0.0;
    let mut average_fps = 0.0;
    let mut frame_count = 0.0;
    let mut delta_time = 0.0;
    let mut time = Instant::now();

    let mut skip_update = false; // Sometimes when the window is being resized or moved, you can't access the buffers and as such must skip an update.

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

            Event::RedrawEventsCleared => { // This should run once per frame.
                let window = surface.object().unwrap().downcast_ref::<Window>().unwrap();

                let dimensions = window.inner_size();
                if dimensions.width == 0 || dimensions.height == 0 { // If the window is 0 in size, don't bother drawing the frame.
                    return;
                }

                previous_frame_end.as_mut().unwrap().cleanup_finished(); // clean up memory

                let vertex_writer = vertex_buffer.write();

                skip_update = match vertex_writer
                {
                    Ok(_) => skip_update,
                    Err(BufferError::InUseByDevice) => true,
                    Err(e) => panic!("couldn't write to the vertex buffer: {e}"),
                };

                let index_writer = index_buffer.write();

                skip_update = match index_writer
                {
                    Ok(_) => skip_update,
                    Err(BufferError::InUseByDevice) => true,
                    Err(e) => panic!("couldn't write to the index buffer: {e}"),
                };

                if skip_update
                {
                    println!("Skipping update!");
                    skip_update = false;
                }
                else
                {
                    update(vertex_writer.unwrap(), index_writer.unwrap(), delta_time, average_fps); // call update once per frame

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

                if recreate_swapchain { // When the window resizes we need to recreate everything dependent on the window size.
                    let (new_swapchain, new_images) = match swapchain.recreate(SwapchainCreateInfo {
                        image_extent: dimensions.into(),
                        ..swapchain.create_info()
                    }) {
                        Ok(r) => r,
                        Err(SwapchainCreationError::ImageExtentNotSupported {..}) => return,
                        Err(e) => panic!("failed to recreate swapchain: {e}"),
                    };

                    swapchain = new_swapchain;

                    framebuffers = window_size_dependent_setup(
                        &new_images,
                        render_pass.clone(),
                        &mut viewport,
                    );

                    recreate_swapchain = false;
                }

                // Aquire image to draw on.
                let (image_index, suboptimal, acquire_future) = match acquire_next_image(swapchain.clone(), None) {
                    Ok(r) => r,
                    Err(AcquireError::OutOfDate) => {
                        recreate_swapchain = true;
                        return;
                    }
                    Err(e) => panic!("failed to acquire next image: {e}"),
                };

                if suboptimal { // Sometimes the image will be messed up. Recreate the swapchain if it is.
                    recreate_swapchain = true;
                }

                let mut builder = AutoCommandBufferBuilder::primary( // Command buffers hold the list of commands.
                    &command_buffer_allocator,
                    queue.queue_family_index(),
                    CommandBufferUsage::OneTimeSubmit,
                )
                .unwrap();

                builder.begin_render_pass( // begin the render pass so we can later draw
                    RenderPassBeginInfo {
                        clear_values: vec![Some([0.0, 0.0, 1.0, 1.0].into())], // Sets background colour.
                        ..RenderPassBeginInfo::framebuffer(
                            framebuffers[image_index as usize].clone()
                        )
                    },
                    SubpassContents::Inline,
                )
                .unwrap()
                .set_viewport(0, [viewport.clone()])
                .bind_pipeline_graphics(pipeline.clone())
                .bind_vertex_buffers(0, vertex_buffer.clone())
                .bind_index_buffer(index_buffer.clone())
                .draw_indexed(indices.len() as u32, 1, 0, 0, 0)
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
            }
            _ => (),
        }
    })
    // end event loop
}

fn window_size_dependent_setup(
    images: &[Arc<SwapchainImage>],
    render_pass: Arc<RenderPass>,
    viewport: &mut Viewport,
) -> Vec<Arc<Framebuffer>> {
    let dimensions = images[0].dimensions().width_height();
    viewport.dimensions = [dimensions[0] as f32, dimensions[1] as f32];

    images.iter()
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
        .collect::<Vec<_>>()
}

fn update(
    mut vertices: BufferWriteGuard<'_, [VertexData]>,
    mut indices: BufferWriteGuard<'_, [u16]>,
    delta_time: f32,
    average_fps: f32,
)
{
    //println!("delta time: {}", delta_time);
    println!("average fps: {}", average_fps);
    vertices[0].position[1] -= 1.0 * delta_time;
}

#[derive(BufferContents, Vertex)]
    #[repr(C)]
    struct VertexData {
        #[format(R32G32_SFLOAT)]
        position: [f32; 2]
    }