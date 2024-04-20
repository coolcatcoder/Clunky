use std::sync::{mpsc::channel, Arc};

use clunky::{
    buffer_contents::{self, Colour3DInstance},
    lost_code::{FixedUpdate, MaxSubsteps},
    math::Matrix4,
    physics::physics_3d::{
        bodies::{Body, CommonBody},
        solver::{self, CpuSolver},
    },
    rendering::draw_instanced,
    shaders::colour_3d_instanced_shaders::{self, Camera},
};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use vulkano::{
    buffer::{
        allocator::{SubbufferAllocator, SubbufferAllocatorCreateInfo},
        BufferContents, BufferUsage, Subbuffer,
    },
    command_buffer::{
        allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
        RenderPassBeginInfo,
    },
    descriptor_set::{
        allocator::StandardDescriptorSetAllocator, PersistentDescriptorSet, WriteDescriptorSet,
    },
    device::Device,
    format::{ClearValue, Format},
    image::{view::ImageView, Image, ImageCreateInfo, ImageType, ImageUsage},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
    pipeline::{graphics::viewport::Viewport, GraphicsPipeline, Pipeline, PipelineBindPoint},
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass},
    sync::GpuFuture,
};
use vulkano_util::{
    context::{VulkanoConfig, VulkanoContext},
    renderer::VulkanoWindowRenderer,
    window::VulkanoWindows,
};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
    window::WindowId,
};

/*
pub struct interesting {
    //pub camera: Camera, Don't know how to do yet, perhaps have it be a generic, for example: T::Camera with T impling 3d shader type or something.

    //physics: CpuSolver<f32, CommonBody<f32>>, Need a solver trait perhaps, and then have this be generic.
    //objects_to_render: Vec<gltf::RenderObject>,
    buffers: Buffers,
}
*/

/// A necessary evil, so that we avoid recursive generics in ExperimentalDrawCalls.
pub struct AccessibleToRenderer {
    pub context: VulkanoContext,
    pub render_pass: Arc<RenderPass>,
    pub allocators: Allocators,

    pub viewport: Viewport,
    pub windows_manager: VulkanoWindows,
}

/// This is designed for this game, and this game only. I'm hopeful I'll find a way to make it generic oneday.
pub struct SimpleEngine<R: Renderer> {
    // Camera should be a generic, based on a trait that allows conversion to some universal properties thing, that we can send to any shader.
    pub camera: Camera,

    pub physics: CpuSolver<f32, CommonBody<f32>>,
    // If option is None then you cannot access fixed update.
    // This is required to avoid potential double mutability.
    physics_fixed_update: Option<FixedUpdate<f64>>,
    on_physics_fixed_update: fn(&mut SimpleEngine<R>),

    experimental_draw_calls: R,

    accessible_while_drawing: AccessibleToRenderer,
}

pub struct Config<R: Renderer> {
    pub starting_camera: Camera,

    pub physics_config: solver::Config<f32, CommonBody<f32>>,
    pub physics_fixed_update: FixedUpdate<f64>,
    pub on_physics_fixed_update: fn(&mut SimpleEngine<R>),
}

impl<R: Renderer> Default for Config<R> {
    fn default() -> Self {
        Self {
            starting_camera: Default::default(),

            physics_config: Default::default(),
            physics_fixed_update: FixedUpdate::new(0.035, MaxSubsteps::WarnAt(85)),
            on_physics_fixed_update: |_| {},
        }
    }
}

impl<R: Renderer + 'static> SimpleEngine<R> {
    pub fn init<F>(config: Config<R>, mut event_handler: F)
    where
        F: 'static
            + FnMut(Event<'_, ()>, &EventLoopWindowTarget<()>, &mut ControlFlow, &mut SimpleEngine<R>),
    {
        let event_loop = EventLoop::new();
        let mut engine = {
            let context = VulkanoContext::new(VulkanoConfig::default());
            let mut windows_manager = VulkanoWindows::default();
            windows_manager.create_window(&event_loop, &context, &Default::default(), |_| {});

            let render_pass = vulkano::single_pass_renderpass!(
                context.device().clone(),
                attachments: {
                    color: {
                        format: windows_manager.get_primary_renderer().unwrap().swapchain_format(),
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

            let allocators = Allocators::new(context.device(), context.memory_allocator());

            let mut viewport = Viewport {
                offset: [0.0, 0.0],
                extent: [0.0, 0.0],
                depth_range: 0.0..=1.0,
            };

            let mut accessible_while_drawing = AccessibleToRenderer {
                context,
                render_pass,
                allocators,

                viewport,
                windows_manager,
            };

            Self {
                camera: config.starting_camera,

                physics: CpuSolver::new(config.physics_config),
                physics_fixed_update: Some(config.physics_fixed_update),
                on_physics_fixed_update: config.on_physics_fixed_update,

                experimental_draw_calls: R::new(&mut accessible_while_drawing),

                accessible_while_drawing,
            }
        };

        event_loop.run(move |event, target, control_flow| {
            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    *control_flow = ControlFlow::Exit;
                }

                Event::WindowEvent {
                    event: WindowEvent::Resized(..) | WindowEvent::ScaleFactorChanged { .. },
                    window_id,
                } => {
                    engine.correct_window_size(window_id);
                }

                Event::MainEventsCleared => {
                    let mut physics_fixed_update = engine.physics_fixed_update.take().unwrap();
                    physics_fixed_update.update(|| {
                        (engine.on_physics_fixed_update)(&mut engine);
                        engine.physics.update(todo!());
                    });
                    engine.physics_fixed_update = Some(physics_fixed_update);

                    event_handler(event, target, control_flow, &mut engine);

                    engine
                        .experimental_draw_calls
                        .render(&mut engine.accessible_while_drawing);

                    return;
                }

                _ => (),
            }

            event_handler(event, target, control_flow, &mut engine);
        })
    }

    fn correct_window_size(&mut self, window_id: WindowId) {
        let window_renderer = self
            .accessible_while_drawing
            .windows_manager
            .get_renderer_mut(window_id)
            .unwrap();
        window_renderer.resize();
        //TODO Really suspicious:
        self.accessible_while_drawing.viewport.extent = window_renderer.window_size();
        self.camera.aspect_ratio = window_renderer.aspect_ratio();
    }
}

pub trait Renderer {
    fn new(accessible_to_renderer: &mut AccessibleToRenderer) -> Self;
    fn render(&mut self, accessible_to_renderer: &mut AccessibleToRenderer);
}

pub struct Allocators {
    pub command_buffer_allocator: StandardCommandBufferAllocator,
    pub subbuffer_allocator: SubbufferAllocator,
    pub descriptor_set_allocator: StandardDescriptorSetAllocator,
}

impl Allocators {
    fn new(device: &Arc<Device>, memory_allocator: &Arc<StandardMemoryAllocator>) -> Self {
        Self {
            command_buffer_allocator: StandardCommandBufferAllocator::new(
                device.clone(),
                Default::default(),
            ),
            subbuffer_allocator: SubbufferAllocator::new(
                memory_allocator.clone(),
                SubbufferAllocatorCreateInfo {
                    buffer_usage: BufferUsage::UNIFORM_BUFFER
                        | BufferUsage::VERTEX_BUFFER
                        | BufferUsage::INDEX_BUFFER,
                    memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                        | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                },
            ),
            descriptor_set_allocator: StandardDescriptorSetAllocator::new(
                device.clone(),
                Default::default(),
            ),
        }
    }
}
