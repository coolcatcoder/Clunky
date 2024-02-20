use std::sync::Arc;

use clunky::{
    buffer_contents::{self, Colour3DInstance},
    lost_code::FpsTracker,
    math::{self, Matrix4},
    physics::physics_3d::verlet::{bodies::CommonBody, CpuSolver, OutsideOfGridBoundsBehaviour},
    shaders::colour_3d_instanced_shaders,
};
use rand::{
    distributions::{Distribution, Uniform},
    rngs::ThreadRng,
};
use vulkano::{
    command_buffer::{
        allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
        RenderPassBeginInfo,
    },
    device::Device,
    format::{ClearValue, Format},
    image::{view::ImageView, Image, ImageCreateInfo, ImageType, ImageUsage},
    memory::allocator::AllocationCreateInfo,
    pipeline::GraphicsPipeline,
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass},
};
use vulkano_util::{
    context::{VulkanoConfig, VulkanoContext},
    renderer::VulkanoWindowRenderer,
    window::VulkanoWindows,
};
use winit::{
    event::{DeviceEvent, Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowId,
};

const DUNGEON_SIZE: usize = 10;
const ROOM_SIZE: [usize; 3] = [10, 10, 10];
const DOOR_WIDTH_HEIGHT_AND_THICKNESS: [f32; 3] = [2.0, 3.0, 1.5];

fn main() {
    let context = VulkanoContext::new(VulkanoConfig::default());
    let event_loop = EventLoop::new();
    let mut windows_manager = VulkanoWindows::default();
    let window_id =
        windows_manager.create_window(&event_loop, &context, &Default::default(), |_| {});

    let render_pass = vulkano::single_pass_renderpass!(
        context.device().clone(),
        attachments: {
            color: {
                format: windows_manager.get_renderer(window_id).unwrap().swapchain_format(),
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

    let pipelines = create_pipelines(context.device(), &render_pass);
    let allocators = create_allocators(context.device());

    let mut fps_tracker = FpsTracker::<f64>::new();

    let fps_cap: Option<f64> = None;

    let game = create_game();

    event_loop.run(move |event, _, control_flow| {
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
                let window_renderer = windows_manager.get_renderer_mut(window_id).unwrap();
                window_renderer.resize();

                //window_size_dependent_setup();
            }

            Event::MainEventsCleared => {
                render(&context, &mut windows_manager, window_id, &allocators, &render_pass);
                fps_tracker.update();
                println!("{}", fps_tracker.average_fps());
            }

            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { input, .. },
                ..
            } => {}

            Event::DeviceEvent {
                event: DeviceEvent::Motion { axis, value },
                ..
            } => {}

            Event::DeviceEvent {
                event: DeviceEvent::MouseWheel { delta },
                ..
            } => {}

            _ => (),
        }
    });
}

fn render(
    context: &VulkanoContext,
    windows_manager: &mut VulkanoWindows,
    window_id: WindowId,
    allocators: &Allocators,
    render_pass: &Arc<RenderPass>,
) {
    let window_renderer = windows_manager.get_renderer_mut(window_id).unwrap();
    let future = window_renderer.acquire().unwrap();

    let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
        &allocators.command_buffer_allocator,
        context.graphics_queue().queue_family_index(),
        CommandBufferUsage::OneTimeSubmit,
    )
    .unwrap();

    let depth_buffer_view = ImageView::new_default(
        Image::new(
            context.memory_allocator().clone(),
            ImageCreateInfo {
                image_type: ImageType::Dim2d,
                format: Format::D32_SFLOAT,
                extent: window_renderer.swapchain_image_view().image().extent(),
                usage: ImageUsage::TRANSIENT_ATTACHMENT | ImageUsage::DEPTH_STENCIL_ATTACHMENT,
                ..Default::default()
            },
            AllocationCreateInfo::default(),
        )
        .unwrap(),
    )
    .unwrap();

    let framebuffer = Framebuffer::new(
        render_pass.clone(),
        FramebufferCreateInfo {
            attachments: vec![window_renderer.swapchain_image_view(), depth_buffer_view],
            ..Default::default()
        }
    );

    /*
    command_buffer_builder
        .begin_render_pass(
            RenderPassBeginInfo {
                clear_values: vec![
                    Some([0.0, 0.0, 1.0, 1.0].into()),
                    Some(ClearValue::Depth(1.0)),
                ],
                ..RenderPassBeginInfo::framebuffer(
                    window_renderer.,
                )
            },
            Default::default(),
        )
        .unwrap();
    */
}

struct Allocators {
    command_buffer_allocator: StandardCommandBufferAllocator,
}

fn create_allocators(device: &Arc<Device>) -> Allocators {
    Allocators {
        command_buffer_allocator: StandardCommandBufferAllocator::new(
            device.clone(),
            Default::default(),
        ),
    }
}

struct Pipelines {
    colour_pipeline: Arc<GraphicsPipeline>,
}

fn create_pipelines(device: &Arc<Device>, render_pass: &Arc<RenderPass>) -> Pipelines {
    Pipelines {
        colour_pipeline: colour_3d_instanced_shaders::create_pipeline(
            device.clone(),
            Subpass::from(render_pass.clone(), 0).unwrap(),
        ),
    }
}

/*
fn window_size_dependent_setup(context: &VulkanoContext, window_renderer: &mut VulkanoWindowRenderer) -> Vec<Arc<Framebuffer>> {
    let window_extent = window_renderer.window().inner_size();
    let depth_buffer_view = ImageView::new_default(
        Image::new(
            context.memory_allocator().clone(),
            ImageCreateInfo {
                image_type: ImageType::Dim2d,
                format: Format::D32_SFLOAT,
                extent: [window_extent.width, window_extent.height, 1],
                usage: ImageUsage::TRANSIENT_ATTACHMENT | ImageUsage::DEPTH_STENCIL_ATTACHMENT,
                ..Default::default()
            },
            AllocationCreateInfo::default(),
        )
        .unwrap(),
    )
    .unwrap();

    window_renderer.

    swapchain_images
        .iter()
        .map(|swapchain_image| {
            let swapchain_image_view = ImageView::new_default(swapchain_image.clone()).unwrap();
            Framebuffer::new(
                render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![swindow_size_dependent_setup
}
*/

struct Game {
    rooms: Vec<Room>,
    physics: CpuSolver<f32, CommonBody<f32>>,
}

fn create_game() -> Game {
    let mut game = Game {
        rooms: Vec::with_capacity(DUNGEON_SIZE * DUNGEON_SIZE),
        physics: CpuSolver::new(
            [0.0, 50.0, 0.0],
            [0.8, 1.0, 0.8],
            [DUNGEON_SIZE, 1, DUNGEON_SIZE],
            [0.0, 0.0, 0.0],
            ROOM_SIZE,
            OutsideOfGridBoundsBehaviour::ContinueUpdating,
            Vec::with_capacity(DUNGEON_SIZE * DUNGEON_SIZE * 10), // Around 10 bodies per room seems reasonable.
        ),
    };

    let mut rng = rand::thread_rng();
    let variety_range = Uniform::from(0..10);

    for i in 0..DUNGEON_SIZE * DUNGEON_SIZE {
        game.rooms.push(generate_room(
            i,
            variety_range.sample(&mut rng),
            &mut game.physics,
        ));
    }

    game
}

#[derive(Clone)]
struct Room {
    cuboid_instances: Vec<buffer_contents::Colour3DInstance>,
}

fn generate_room(
    room_index: usize,
    variety: u8,
    physics: &mut CpuSolver<f32, CommonBody<f32>>,
) -> Room {
    let room_position = math::position_from_index_2d(room_index, DUNGEON_SIZE);
    let real_room_position = [
        room_position[0] as f32 * ROOM_SIZE[0] as f32,
        room_position[1] as f32 * ROOM_SIZE[1] as f32,
    ];

    let mut room = Room {
        cuboid_instances: Vec::with_capacity(15), // 15 instances per room?
    };

    // bottom
    room.cuboid_instances.push(Colour3DInstance::new(
        [1.0, 1.0, 1.0, 1.0],
        Matrix4::from_translation([
            real_room_position[0] + ROOM_SIZE[0] as f32 * 0.5,
            0.5,
            real_room_position[2] + ROOM_SIZE[2] as f32 * 0.5,
        ]) * Matrix4::from_scale([ROOM_SIZE[0] as f32, 1.0, ROOM_SIZE[2] as f32]),
    ));

    // Walls:

    // top
    room.cuboid_instances.push(Colour3DInstance::new(
        [1.0, 1.0, 1.0, 1.0],
        Matrix4::from_translation([
            real_room_position[0] + ROOM_SIZE[0] as f32 * 0.5,
            -(ROOM_SIZE[1] as f32) - 0.5,
            real_room_position[2] + ROOM_SIZE[2] as f32 * 0.5,
        ]) * Matrix4::from_scale([ROOM_SIZE[0] as f32, 1.0, ROOM_SIZE[2] as f32]),
    ));

    // +x
    room.cuboid_instances.push(Colour3DInstance::new(
        [1.0, 1.0, 1.0, 1.0],
        Matrix4::from_translation([
            real_room_position[0] + ROOM_SIZE[0] as f32 + 0.5,
            ROOM_SIZE[1] as f32 * -0.5,
            real_room_position[2] + ROOM_SIZE[2] as f32 * 0.5,
        ]) * Matrix4::from_scale([1.0, ROOM_SIZE[1] as f32, ROOM_SIZE[2] as f32]),
    ));

    // -x
    room.cuboid_instances.push(Colour3DInstance::new(
        [1.0, 1.0, 1.0, 1.0],
        Matrix4::from_translation([
            real_room_position[0] - 0.5,
            ROOM_SIZE[1] as f32 * -0.5,
            real_room_position[2] + ROOM_SIZE[2] as f32 * 0.5,
        ]) * Matrix4::from_scale([1.0, ROOM_SIZE[1] as f32, ROOM_SIZE[2] as f32]),
    ));

    // +z
    room.cuboid_instances.push(Colour3DInstance::new(
        [1.0, 1.0, 1.0, 1.0],
        Matrix4::from_translation([
            real_room_position[0] + ROOM_SIZE[0] as f32 * 0.5,
            ROOM_SIZE[1] as f32 * -0.5,
            real_room_position[2] + ROOM_SIZE[2] as f32 + 0.5,
        ]) * Matrix4::from_scale([ROOM_SIZE[0] as f32, ROOM_SIZE[1] as f32, 1.0]),
    ));

    // -z
    room.cuboid_instances.push(Colour3DInstance::new(
        [1.0, 1.0, 1.0, 1.0],
        Matrix4::from_translation([
            real_room_position[0] + ROOM_SIZE[0] as f32 * 0.5,
            ROOM_SIZE[1] as f32 * -0.5,
            real_room_position[2] - 0.5,
        ]) * Matrix4::from_scale([ROOM_SIZE[0] as f32, ROOM_SIZE[1] as f32, 1.0]),
    ));

    // Doors:

    // +x
    if room_position[0] != DUNGEON_SIZE-1 {
        room.cuboid_instances.push(Colour3DInstance::new(
            [1.0, 1.0, 1.0, 1.0],
            Matrix4::from_translation([
                real_room_position[0] + ROOM_SIZE[0] as f32 + 0.5,
                ROOM_SIZE[1] as f32 * -0.5,
                real_room_position[2] + ROOM_SIZE[2] as f32 * 0.5,
            ]) * Matrix4::from_scale([DOOR_WIDTH_HEIGHT_AND_THICKNESS[2], DOOR_WIDTH_HEIGHT_AND_THICKNESS[1], DOOR_WIDTH_HEIGHT_AND_THICKNESS[0]]),
        ));
    }

    // -x
    if room_position[0] != DUNGEON_SIZE-1 {
        room.cuboid_instances.push(Colour3DInstance::new(
            [1.0, 1.0, 1.0, 1.0],
            Matrix4::from_translation([
                real_room_position[0] - 0.5,
                ROOM_SIZE[1] as f32 * -0.5,
                real_room_position[2] + ROOM_SIZE[2] as f32 * 0.5,
            ]) * Matrix4::from_scale([DOOR_WIDTH_HEIGHT_AND_THICKNESS[2], DOOR_WIDTH_HEIGHT_AND_THICKNESS[1], DOOR_WIDTH_HEIGHT_AND_THICKNESS[0]]),
        ));
    }

    // +z
    if room_position[0] != DUNGEON_SIZE-1 {
        room.cuboid_instances.push(Colour3DInstance::new(
            [0.0, 0.0, 0.0, 1.0],
            Matrix4::from_translation([
                real_room_position[0] + ROOM_SIZE[0] as f32 * 0.5,
                ROOM_SIZE[1] as f32 * -0.5,
                real_room_position[2] + ROOM_SIZE[2] as f32 + 0.5,
            ]) * Matrix4::from_scale([DOOR_WIDTH_HEIGHT_AND_THICKNESS[0], DOOR_WIDTH_HEIGHT_AND_THICKNESS[1], DOOR_WIDTH_HEIGHT_AND_THICKNESS[2]]),
        ));
    }

    // -z
    if room_position[0] != DUNGEON_SIZE-1 {
        room.cuboid_instances.push(Colour3DInstance::new(
            [0.0, 0.0, 0.0, 1.0],
            Matrix4::from_translation([
                real_room_position[0] + ROOM_SIZE[0] as f32 * 0.5,
                ROOM_SIZE[1] as f32 * -0.5,
                real_room_position[2] - 0.5,
            ]) * Matrix4::from_scale([DOOR_WIDTH_HEIGHT_AND_THICKNESS[0], DOOR_WIDTH_HEIGHT_AND_THICKNESS[1], DOOR_WIDTH_HEIGHT_AND_THICKNESS[2]]),
        ));
    }

    match variety {
        _ => {
            unreachable!();
        }
    }

    room
}
