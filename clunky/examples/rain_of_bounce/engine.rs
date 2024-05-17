use std::sync::Arc;

use clunky::{
    lost_code::{FixedUpdate, FpsTracker, MaxSubsteps},
    physics::PhysicsSimulation,
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

pub struct Redesign {
    // Physics
    // Option Renderer

}

/// This is designed for this game, and this game only. I'm hopeful I'll find a way to make it generic oneday.
pub struct SimpleEngine<R: Renderer> {
    pub physics: R::Physics,
    // If option is None then you cannot access fixed update.
    // This is required to avoid potential double mutability.
    physics_fixed_update: Option<FixedUpdate<f64>>,

    fps_tracker: FpsTracker<f32>,

    close_everything_on_window_close: bool,

    pub renderer_storage: R::Storage,

    //pub accessible_to_renderer: AccessibleToRenderer,
    pub context: VulkanoContext,
    pub allocators: Allocators,

    pub windows_manager: VulkanoWindows,

    pub temporary_event_loop_storage: Option<EventLoop<()>>,
}

/// Config for the simple engine.
pub struct Config {
    pub physics_fixed_update: FixedUpdate<f64>,

    pub close_everything_on_window_close: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            physics_fixed_update: FixedUpdate::new(0.035, MaxSubsteps::WarnAt(85)),

            close_everything_on_window_close: true,
        }
    }
}

/// An event. Not much more to say.
pub enum EngineEvent<'a, T: 'static> {
    WinitEvent(Event<'a, T>),
    PhysicsEvent(PhysicsEvent),
}

pub enum PhysicsEvent {
    BeforeTick,
    AfterTick,
}

impl<R: Renderer + 'static> SimpleEngine<R> {
    pub fn new(config: Config, physics_simulation: R::Physics) -> Self {
        let event_loop = EventLoop::new();
        {
            let context = VulkanoContext::new(VulkanoConfig::default());

            let allocators = Allocators::new(context.device(), context.memory_allocator());

            let windows_manager = VulkanoWindows::default();

            Self {
                physics: physics_simulation,
                physics_fixed_update: Some(config.physics_fixed_update),

                fps_tracker: FpsTracker::new(),

                close_everything_on_window_close: config.close_everything_on_window_close,

                renderer_storage: todo!(),

                context,
                allocators,

                windows_manager,

                temporary_event_loop_storage: Some(event_loop),
            }
        }
    }

    pub fn run<E>(self, mut event_handler: E) -> !
    where
        E: 'static
            + FnMut(
                EngineEvent<'_, ()>,
                &EventLoopWindowTarget<()>,
                &mut ControlFlow,
                &mut SimpleEngine<R>,
            ),
    {
        let mut engine = self;

        let event_loop = engine.temporary_event_loop_storage.take().unwrap();
        event_loop.run(move |event, target, control_flow| {
            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    if engine.close_everything_on_window_close {
                        *control_flow = ControlFlow::Exit;
                    }
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
                        event_handler(
                            EngineEvent::PhysicsEvent(PhysicsEvent::BeforeTick),
                            target,
                            control_flow,
                            &mut engine,
                        );
                        engine.physics.update(fixed_delta_time);
                        event_handler(
                            EngineEvent::PhysicsEvent(PhysicsEvent::AfterTick),
                            target,
                            control_flow,
                            &mut engine,
                        );
                    });
                    engine.physics_fixed_update = Some(physics_fixed_update);

                    event_handler(
                        EngineEvent::WinitEvent(event),
                        target,
                        control_flow,
                        &mut engine,
                    );

                    let mut renderer = R::new(&mut engine);
                    renderer.render();
                    //TODO:

                    //let mut renderer = engine.renderer.take().unwrap();

                    //renderer.render(&mut engine);

                    //engine.renderer = Some(renderer);

                    engine.fps_tracker.update();

                    return;
                }

                _ => (),
            }

            event_handler(
                EngineEvent::WinitEvent(event),
                target,
                control_flow,
                &mut engine,
            );
        })
    }

    pub fn get_renderer(&mut self) -> R {
        R::new(self)
    }

    fn correct_window_size(&mut self, window_id: WindowId) {
        let window_renderer = self.windows_manager.get_renderer_mut(window_id).unwrap();
        window_renderer.resize();
        self.get_renderer().correct_window_size(
            window_id,
            window_renderer.window_size(),
            window_renderer.aspect_ratio(),
        );
    }

    pub fn fps_tracker(&self) -> &FpsTracker<f32> {
        &self.fps_tracker
    }
}

pub trait RendererMethods {

}

pub trait Renderer
where
    Self: Sized,

    Self::Physics: PhysicsSimulation<f32>,
{
    // Bit annoying that the renderer needs to specify physics, but it is what it is.
    type Physics;
    type Storage;
    type Methods;
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
