use std::sync::Arc;

use clunky::{
    lost_code::{FixedUpdate, MaxSubsteps},
    physics::physics_3d::{
        bodies::CommonBody,
        solver::{self, CpuSolver},
    },
    shaders::colour_3d_instanced_shaders::Camera,
};
use vulkano::{
    buffer::{
        allocator::{SubbufferAllocator, SubbufferAllocatorCreateInfo},
        BufferUsage,
    },
    command_buffer::allocator::StandardCommandBufferAllocator,
    descriptor_set::allocator::StandardDescriptorSetAllocator,
    device::Device,
    memory::allocator::{MemoryTypeFilter, StandardMemoryAllocator},
    pipeline::graphics::viewport::Viewport,
};
use vulkano_util::{
    context::{VulkanoConfig, VulkanoContext},
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
    pub allocators: Allocators,

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

    renderer: R,

    accessible_to_renderer: AccessibleToRenderer,
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
    pub fn init<F>(config: Config<R>, create_renderer: fn(&mut AccessibleToRenderer, &EventLoop<()>)->R, mut event_handler: F)
    where
        F: 'static
            + FnMut(Event<'_, ()>, &EventLoopWindowTarget<()>, &mut ControlFlow, &mut SimpleEngine<R>),
    {
        let event_loop = EventLoop::new();
        let mut engine = {
            let context = VulkanoContext::new(VulkanoConfig::default());
            let mut windows_manager = VulkanoWindows::default();
            windows_manager.create_window(&event_loop, &context, &Default::default(), |_| {});

            let allocators = Allocators::new(context.device(), context.memory_allocator());

            let mut accessible_to_renderer = AccessibleToRenderer {
                context,
                allocators,

                windows_manager,
            };

            Self {
                camera: config.starting_camera,

                physics: CpuSolver::new(config.physics_config),
                physics_fixed_update: Some(config.physics_fixed_update),
                on_physics_fixed_update: config.on_physics_fixed_update,

                renderer: create_renderer(&mut accessible_to_renderer, &event_loop),

                accessible_to_renderer,
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
                    let fixed_delta_time = physics_fixed_update.fixed_delta_time as f32;
                    physics_fixed_update.update(|| {
                        (engine.on_physics_fixed_update)(&mut engine);
                        engine.physics.update(fixed_delta_time);
                    });
                    engine.physics_fixed_update = Some(physics_fixed_update);

                    event_handler(event, target, control_flow, &mut engine);

                    engine
                        .renderer
                        .render(&mut engine.accessible_to_renderer);

                    return;
                }

                _ => (),
            }

            event_handler(event, target, control_flow, &mut engine);
        })
    }

    fn correct_window_size(&mut self, window_id: WindowId) {
        let window_renderer = self
            .accessible_to_renderer
            .windows_manager
            .get_renderer_mut(window_id)
            .unwrap();
        window_renderer.resize();
        //TODO Really suspicious:
        //self.accessible_while_drawing.viewport.extent = window_renderer.window_size();
        //self.camera.aspect_ratio = window_renderer.aspect_ratio();
    }
}

pub trait Renderer {
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
