#![feature(const_fn_floating_point_arithmetic)] // Required for math for now.
#![feature(test)]

use std::{sync::Arc, time::Instant};
use vulkano::{
    buffer::{
        allocator::{SubbufferAllocator, SubbufferAllocatorCreateInfo},
        Buffer, BufferCreateInfo, BufferUsage,
    },
    command_buffer::{
        allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
        CopyBufferToImageInfo, RenderPassBeginInfo,
    },
    descriptor_set::{
        allocator::StandardDescriptorSetAllocator, PersistentDescriptorSet, WriteDescriptorSet,
    },
    device::{
        physical::PhysicalDeviceType, Device, DeviceCreateInfo, DeviceExtensions, QueueCreateInfo,
        QueueFlags,
    },
    format::ClearValue,
    image::sampler::{Filter, SamplerAddressMode},
    image::{view::ImageView, ImageCreateInfo, ImageType, ImageUsage},
    instance::{Instance, InstanceCreateFlags, InstanceCreateInfo},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
    pipeline::{
        graphics::{
            color_blend::{AttachmentBlend, ColorBlendAttachmentState, ColorBlendState},
            depth_stencil::{CompareOp, DepthState, DepthStencilState},
            input_assembly::InputAssemblyState,
            multisample::MultisampleState,
            rasterization::RasterizationState,
            vertex_input::VertexDefinition,
            viewport::{Scissor, Viewport, ViewportState},
            GraphicsPipelineCreateInfo,
        },
        layout::PipelineDescriptorSetLayoutCreateInfo,
        GraphicsPipeline, Pipeline, PipelineBindPoint, PipelineLayout,
        PipelineShaderStageCreateInfo,
    },
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass},
    swapchain::{
        acquire_next_image, Surface, Swapchain, SwapchainCreateInfo, SwapchainPresentInfo,
    },
    sync::{self, GpuFuture},
    DeviceSize, Validated, VulkanError, VulkanLibrary,
};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use vulkano::format::Format;
use vulkano::image::sampler::Sampler;
use vulkano::image::sampler::SamplerCreateInfo;

use vulkano::command_buffer::PrimaryCommandBufferAbstract;

use vulkano::image::Image;

use vulkano::device::DeviceOwned;

mod buffer_contents;

mod menus;

mod lost_code;

mod menu_rendering;

#[allow(dead_code)]
mod math;

mod meshes;

#[allow(dead_code)]
mod physics;

#[allow(dead_code)]
mod random_generation;

mod colour_3d_instanced_vertex_shader {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/shaders/colour_3d_instanced_shaders/vertex_shader.vert",
    }
}

mod colour_3d_instanced_fragment_shader {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/colour_3d_instanced_shaders/fragment_shader.frag",
    }
}

mod colour_2d_vertex_shader {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/shaders/colour_2d_shaders/vertex_shader.vert",
    }
}

mod colour_2d_fragment_shader {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/colour_2d_shaders/fragment_shader.frag",
    }
}

mod uv_2d_vertex_shader {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/shaders/uv_2d_shaders/vertex_shader.vert",
    }
}

mod uv_2d_fragment_shader {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/uv_2d_shaders/fragment_shader.frag",
    }
}

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

mod vertex_shader_ui {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/ui_shaders/vertex_shader.glsl",
    }
}

mod fragment_shader_ui {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/ui_shaders/fragment_shader.frag",
    }
}

mod vertex_shader_test {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/test_shaders/vertex_shader.glsl",
    }
}

mod fragment_shader_test {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/test_shaders/fragment_shader.glsl",
    }
}

const DEPTH_FORMAT: Format = Format::D16_UNORM; // TODO: work out what this should be

fn main() {
    if meshes::DEBUG_VIEWER {
        println!("{}", meshes::DEBUG);
    }

    let (instance, event_loop) = get_instance_and_event_loop();

    let window = Arc::new(WindowBuilder::new().build(&event_loop).unwrap());

    let surface = Surface::from_window(instance.clone(), window.clone()).unwrap();

    // let surface = WindowBuilder::new()
    //     .build_vk_surface(&event_loop, instance.clone())
    //     .unwrap();

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
        let image_format = device
            .physical_device()
            .surface_formats(&surface, Default::default())
            .unwrap()[0]
            .0;

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

    let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone())); // TODO: Store in render storage.

    let mut render_storage = RenderStorage {
        aspect_ratio: 0.0,
        other_aspect_ratio: 0.01,
        frame_count: 0,
        starting_time: Instant::now(),
        window_size: [0, 0],
        menu: menus::STARTING_MENU,
        force_run_window_dependent_setup: false,
        entire_render_datas: vec![],
        buffer_allocator: SubbufferAllocator::new(
            memory_allocator.clone(),
            SubbufferAllocatorCreateInfo {
                buffer_usage: BufferUsage::UNIFORM_BUFFER
                    | BufferUsage::VERTEX_BUFFER
                    | BufferUsage::INDEX_BUFFER, // TODO: work out if it is ok to lie about this
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
        ),
        memory_allocator,
        descriptor_set_allocator: StandardDescriptorSetAllocator::new(
            device.clone(),
            Default::default(),
        ),
        fixed_time_passed: 0.0,

        window,
    };

    let mut user_storage = menus::start(&mut render_storage);

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

    let mut sprites = vec![];

    for png_bytes in menus::PNG_BYTES_LIST {
        sprites.push({
            let decoder = png::Decoder::new(png_bytes);
            let mut reader = decoder.read_info().unwrap();
            let info = reader.info();
            let extent = [info.width, info.height, 1];

            let upload_buffer = Buffer::new_slice(
                render_storage.memory_allocator.clone(),
                BufferCreateInfo {
                    usage: BufferUsage::TRANSFER_SRC,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::PREFER_HOST
                        | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                },
                (info.width * info.height * 4) as DeviceSize,
            )
            .unwrap();

            reader
                .next_frame(&mut upload_buffer.write().unwrap())
                .unwrap();

            let image = Image::new(
                render_storage.memory_allocator.clone(),
                ImageCreateInfo {
                    image_type: ImageType::Dim2d,
                    format: Format::R8G8B8A8_SRGB,
                    extent,
                    usage: ImageUsage::TRANSFER_DST | ImageUsage::SAMPLED,
                    ..Default::default()
                },
                AllocationCreateInfo::default(),
            )
            .unwrap();

            uploads
                .copy_buffer_to_image(CopyBufferToImageInfo::buffer_image(
                    upload_buffer,
                    image.clone(),
                ))
                .unwrap();

            ImageView::new_default(image).unwrap()
        })
    }

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

    // start creating render pass
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
                format: DEPTH_FORMAT,
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
    // end of creating render pass

    let (mut pipelines, mut framebuffers) = window_size_dependent_setup(
        render_storage.memory_allocator.clone(),
        &images,
        render_pass.clone(),
        &mut user_storage,
        &mut render_storage,
    );

    render_storage.descriptor_set_allocator =
        StandardDescriptorSetAllocator::new(device.clone(), Default::default());

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

    let mut delta_time_sum = 0.0;
    let mut average_fps = 0.0;
    let mut frame_count = 0.0;
    let mut delta_time = 0.0;
    let mut time = Instant::now();

    // start event loop
    event_loop.run(move |event, _, control_flow| { // TODO: to simplify menu coding, I should just pass event into a menu function called event_handler() or something, and it can deal with it from there.
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
                //let window = surface.object().unwrap().downcast_ref::<Window>().unwrap(); // TODO: work out why this existed?

                //let dimensions = window.inner_size();
                let image_extent: [u32; 2] = render_storage.window.inner_size().into();

                if image_extent.contains(&0) {
                    // If the window is 0 in size, don't bother drawing the frame.
                    return;
                }

                previous_frame_end.as_mut().unwrap().cleanup_finished(); // clean up memory

                render_storage.aspect_ratio =
                    swapchain.image_extent()[1] as f32 / swapchain.image_extent()[0] as f32;

                render_storage.other_aspect_ratio =
                    swapchain.image_extent()[0] as f32 / swapchain.image_extent()[1] as f32;

                //let previous_window_size = render_storage.window_size;
                render_storage.window_size = swapchain.image_extent();

                // TODO: this ancient artifact of code suggests that perhaps the resized window event might potentially not work correctly? Investigate.
                // if previous_window_size != render_storage.window_size {
                //     (render_storage.menu.get_data().on_window_resize)(
                //         &mut user_storage,
                //         &mut render_storage,
                //     );
                // }

                (render_storage.menu.get_data().update)(
                    &mut user_storage,
                    &mut render_storage,
                    delta_time,
                    average_fps,
                ); // call update once per frame

                let seconds_since_start = render_storage.starting_time.elapsed().as_secs_f32();

                let mut substeps = 0;

                while render_storage.fixed_time_passed < seconds_since_start {
                    (render_storage.menu.get_data().fixed_update.function)(&mut user_storage, &mut render_storage);
                    render_storage.fixed_time_passed += render_storage.menu.get_data().fixed_update.delta_time;

                    substeps += 1;

                    if substeps > render_storage.menu.get_data().fixed_update.max_substeps {
                        println!(
                            "Too many substeps per frame. Entered performance sinkhole. Substeps: {}",
                            substeps
                        )
                    }
                }

                update_buffers(
                    &mut render_storage,
                );

                if recreate_swapchain || render_storage.force_run_window_dependent_setup {
                    render_storage.force_run_window_dependent_setup = false;

                    // When the window resizes we need to recreate everything dependent on the window size.
                    let (new_swapchain, new_images) = swapchain
                        .recreate(SwapchainCreateInfo {
                            image_extent,
                            ..swapchain.create_info()
                        })
                        .expect("failed to recreate swapchain");

                    swapchain = new_swapchain;

                    let (new_pipelines, new_framebuffers) =
                        window_size_dependent_setup(
                            render_storage.memory_allocator.clone(),
                            &new_images,
                            render_pass.clone(),
                            &mut user_storage,
                            &mut render_storage,
                        );

                    pipelines = new_pipelines;
                    framebuffers = new_framebuffers;

                    recreate_swapchain = false;
                }

                // Aquire image to draw on.
                let (image_index, suboptimal, acquire_future) =
                    match acquire_next_image(swapchain.clone(), None).map_err(Validated::unwrap) {
                        Ok(r) => r,
                        Err(VulkanError::OutOfDate) => {
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
                            clear_values: vec![
                                Some([0.0, 0.0, 1.0, 1.0].into()),
                                //some(ClearValue::)
                                Some(ClearValue::Depth(1.0))
                            ], // Sets background colour and something else, likely depth buffer.
                            ..RenderPassBeginInfo::framebuffer(
                                framebuffers[image_index as usize].clone(),
                            )
                        },
                        //SubpassContents::Inline,
                        Default::default(),
                    )
                    .unwrap();

                (render_storage.menu.get_data().on_draw)(
                    &mut user_storage,
                    &mut render_storage,
                    &mut builder,
                );

                for i in 0..render_storage.entire_render_datas.len() {
                    let entire_render_data = &render_storage.entire_render_datas[i];
                    builder.bind_pipeline_graphics(pipelines[i].clone())
                    .unwrap();

                    // TODO: work out how to have descriptor sets here
                    if let Some(shader_accessible_buffers) = &entire_render_data.render_buffers.shader_accessible_buffers {
                        // TODO: fix this up

                        let layouts = pipelines[i].layout().set_layouts();

                        let mut sets = vec![];

                        if let Some(uniform_buffer) = &shader_accessible_buffers.uniform_buffer {
                            match uniform_buffer {
                                menu_rendering::UniformBuffer::CameraData2D(uniform_buffer) => {
                                    sets.push(PersistentDescriptorSet::new(
                                        &render_storage.descriptor_set_allocator,
                                        layouts.get(0).unwrap().clone(),
                                        [WriteDescriptorSet::buffer(0, uniform_buffer.get_cloned_buffer(&render_storage))],
                                        [],
                                    ).unwrap())
                                }
                                menu_rendering::UniformBuffer::CameraData3D(uniform_buffer) => {
                                    sets.push(PersistentDescriptorSet::new(
                                        &render_storage.descriptor_set_allocator,
                                        layouts.get(0).unwrap().clone(),
                                        [WriteDescriptorSet::buffer(0, uniform_buffer.get_cloned_buffer(&render_storage))],
                                        [],
                                    ).unwrap())
                                }
                            }
                        }

                        if let Some(image) = &shader_accessible_buffers.image {
                            sets.push(PersistentDescriptorSet::new(
                                &render_storage.descriptor_set_allocator,
                                layouts.get(1).unwrap().clone(),
                                [
                                    WriteDescriptorSet::sampler(0, sampler.clone()),
                                    WriteDescriptorSet::image_view(1, sprites[*image].clone()),
                                ],
                                [],
                            ).unwrap())
                        }

                        builder.bind_descriptor_sets(
                            PipelineBindPoint::Graphics,
                            pipelines[i].layout().clone(),
                            0,
                            sets,
                        ).unwrap();
                    }

                    let mut instance_count = 1;
                    let mut vertex_count = 0;
                    if let Some(instance_buffer) = &entire_render_data.render_buffers.instance_buffer {
                        menu_rendering::instance_buffer_generic_caller!(instance_buffer, |instance_buffer| {
                            //instance_count = instance_buffer.length(&render_storage);
                            instance_count = menu_rendering::buffer_types_length(instance_buffer, &render_storage);
                            let instance_buffer = instance_buffer.get_cloned_buffer(&render_storage);

                            menu_rendering::vertex_buffer_generic_caller!(&entire_render_data.render_buffers.vertex_buffer, |vertex_buffer| {
                                //vertex_count = vertex_buffer.len(&render_storage);
                                vertex_count = menu_rendering::buffer_types_length(vertex_buffer, &render_storage);
                                let vertex_buffer = vertex_buffer.get_cloned_buffer(&render_storage);
                                builder.bind_vertex_buffers(0, (vertex_buffer, instance_buffer)).unwrap();
                            });
                        });
                    }
                    else {
                        menu_rendering::vertex_buffer_generic_caller!(&entire_render_data.render_buffers.vertex_buffer, |vertex_buffer| {
                            vertex_count = menu_rendering::buffer_types_length(vertex_buffer, &render_storage);
                            let vertex_buffer = vertex_buffer.get_cloned_buffer(&render_storage);
                            builder.bind_vertex_buffers(0, vertex_buffer).unwrap();
                        });
                    }

                    if let Some(index_buffer) = &entire_render_data.render_buffers.index_buffer {
                        builder.bind_index_buffer(index_buffer.get_cloned_buffer(&render_storage)).unwrap()
                        .draw_indexed(
                            index_buffer.length(&render_storage) as u32,
                            instance_count as u32,
                            0,
                            0,
                            0,
                        )
                        .unwrap();
                    }
                    else {
                        builder.draw(
                            vertex_count as u32,
                            instance_count as u32,
                            0,
                            0,
                        )
                        .unwrap();
                    }
                }

                builder.end_render_pass(Default::default()).unwrap();

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

                match future.map_err(Validated::unwrap) {
                    Ok(future) => {
                        previous_frame_end = Some(future.boxed());
                    }

                    Err(VulkanError::OutOfDate) => {
                        recreate_swapchain = true;
                        previous_frame_end = Some(sync::now(device.clone()).boxed());
                    }

                    Err(e) => {
                        match e {
                            // VulkanError::ResourceAccessError { error, use_ref } => {
                            //     let use_ref = use_ref.unwrap();
                            //     println!("ResourceAccessError: {}, More info: command_index:{}, command_name:{}", error, use_ref.command_index, use_ref.command_name);
                            //     recreate_swapchain = true;
                            //     previous_frame_end = Some(sync::now(device.clone()).boxed());
                            // }
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
            _ => {
                (render_storage.menu.get_data().handle_events)(&mut user_storage, &mut render_storage, event);
            },
        }
    })
    // end event loop
}

fn window_size_dependent_setup(
    memory_allocator: Arc<StandardMemoryAllocator>,
    images: &[Arc<Image>],
    render_pass: Arc<RenderPass>,
    user_storage: &mut menus::UserStorage,
    render_storage: &mut RenderStorage,
) -> (Vec<Arc<GraphicsPipeline>>, Vec<Arc<Framebuffer>>) {
    let device = memory_allocator.device().clone();
    let extent = images[0].extent();

    let depth_buffer = ImageView::new_default(
        Image::new(
            memory_allocator.clone(),
            ImageCreateInfo {
                image_type: ImageType::Dim2d,
                format: DEPTH_FORMAT,
                extent,
                usage: ImageUsage::TRANSIENT_ATTACHMENT | ImageUsage::DEPTH_STENCIL_ATTACHMENT,
                ..Default::default()
            },
            AllocationCreateInfo::default(),
        )
        .unwrap(),
    )
    .unwrap();

    let framebuffers = images
        .iter()
        .map(|image| {
            let view = ImageView::new_default(image.clone()).unwrap();
            Framebuffer::new(
                render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![view, depth_buffer.clone()],
                    ..Default::default()
                },
            )
            .unwrap()
        })
        .collect::<Vec<_>>();

    let mut pipelines =
        (render_storage.menu.get_data().create_pipelines)(user_storage, render_storage);

    for entire_render_data in &mut render_storage.entire_render_datas {
        let vertex_shader_entrance = entire_render_data
            .settings
            .vertex_shader
            .load(device.clone())
            .unwrap()
            .entry_point("main")
            .unwrap();
        let fragment_shader_entrance = entire_render_data
            .settings
            .fragment_shader
            .load(device.clone())
            .unwrap()
            .entry_point("main")
            .unwrap();

        let vertex_input_state;

        if let Some(instance_buffer) = &entire_render_data.render_buffers.instance_buffer {
            vertex_input_state = [
                entire_render_data.render_buffers.vertex_buffer.per_vertex(),
                instance_buffer.per_instance(),
            ]
            .definition(&vertex_shader_entrance.info().input_interface)
            .unwrap();
        } else {
            vertex_input_state = entire_render_data
                .render_buffers
                .vertex_buffer
                .per_vertex()
                .definition(&vertex_shader_entrance.info().input_interface)
                .unwrap();
        }

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

        let depth_stencil_state = if entire_render_data.settings.depth {
            Some(DepthStencilState {
                depth: Some(DepthState {
                    write_enable: true,
                    compare_op: CompareOp::Less,
                }),
                depth_bounds: None,
                stencil: None,
                ..Default::default()
            })
        } else {
            Some(DepthStencilState {
                depth: Some(DepthState {
                    write_enable: false,
                    compare_op: CompareOp::Less,
                }),
                depth_bounds: None,
                stencil: None,
                ..Default::default()
            })
        };

        let subpass = Subpass::from(render_pass.clone(), 0).unwrap();
        pipelines.push(
            GraphicsPipeline::new(
                device.clone(),
                None,
                GraphicsPipelineCreateInfo {
                    stages: stages.into_iter().collect(),
                    vertex_input_state: Some(vertex_input_state),
                    input_assembly_state: Some(InputAssemblyState {
                        topology: entire_render_data.settings.topology,
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
                    //rasterization_state: Some(RasterizationState::default()),
                    rasterization_state: Some(RasterizationState {
                        cull_mode: entire_render_data.settings.cull_mode,
                        front_face: entire_render_data.settings.front_face,
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
                    depth_stencil_state,
                    subpass: Some(subpass.into()),
                    ..GraphicsPipelineCreateInfo::layout(layout)
                },
            )
            .unwrap(),
        );
    }

    (pipelines, framebuffers)
}

fn get_instance_and_event_loop() -> (Arc<vulkano::instance::Instance>, EventLoop<()>) {
    let library = VulkanLibrary::new().unwrap();
    let event_loop = EventLoop::new();
    let required_extensions = Surface::required_extensions(&event_loop);
    (
        Instance::new(
            library,
            InstanceCreateInfo {
                enabled_extensions: required_extensions,
                flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
                ..Default::default()
            },
        )
        .unwrap(),
        event_loop,
    )
}

fn update_buffers(render_storage: &mut RenderStorage) {
    for entire_render_data in &mut render_storage.entire_render_datas {
        if let Some(menu_rendering::BufferTypes::RenderBuffer(index_buffer)) =
            &mut entire_render_data.render_buffers.index_buffer
        {
            index_buffer.update(render_storage.frame_count);
        }

        menu_rendering::vertex_buffer_generic_caller!(
            &mut entire_render_data.render_buffers.vertex_buffer,
            |vertex_buffer: &mut _| {
                if let menu_rendering::BufferTypes::RenderBuffer(vertex_buffer) = vertex_buffer {
                    vertex_buffer.update(render_storage.frame_count);
                }
            }
        );

        if let Some(instance_buffer) = &mut entire_render_data.render_buffers.instance_buffer {
            menu_rendering::instance_buffer_generic_caller!(
                instance_buffer,
                |instance_buffer: &mut _| {
                    if let menu_rendering::BufferTypes::RenderBuffer(instance_buffer) =
                        instance_buffer
                    {
                        instance_buffer.update(render_storage.frame_count);
                    }
                }
            );
        }
    }
}

#[deprecated] // TODO: work out why this exists??? This certainly ain't going to be used by main. This feels like it was inteded for user storage?
pub struct Camera {
    pub scale: f32,
    pub position: (f32, f32),
}

pub struct RenderStorage {
    // TODO: Perhaps removing or refining what belongs in this struct.
    pub aspect_ratio: f32, // TODO: sort out these 2 different aspect ratios. Very messy.
    pub other_aspect_ratio: f32,
    pub frame_count: usize, // This will overflow after 2 years, assuming 60 fps.
    pub starting_time: Instant,
    pub window_size: [u32; 2],

    pub menu: menus::Menu,

    pub force_run_window_dependent_setup: bool,
    pub entire_render_datas: Vec<menu_rendering::EntireRenderData>, // Plural of data, must be datas, I'm so sorry.

    pub buffer_allocator: SubbufferAllocator,
    pub memory_allocator: Arc<StandardMemoryAllocator>,
    pub descriptor_set_allocator: StandardDescriptorSetAllocator,

    pub fixed_time_passed: f32,

    pub window: Arc<Window>,
}
