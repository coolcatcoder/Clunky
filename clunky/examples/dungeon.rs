use std::sync::Arc;

use clunky::{
    buffer_contents::{self, Colour3DInstance},
    lost_code::{is_pressed, FixedUpdate, FpsTracker},
    math::{self, index_from_position_2d, Matrix4, Radians},
    meshes,
    physics::physics_3d::{
        aabb::AabbCentredOrigin,
        verlet::{
            bodies::{CommonBody, ImmovableCuboid, Player},
            CpuSolver, OutsideOfGridBoundsBehaviour, Particle,
        },
    },
    rendering::draw_instanced,
    shaders::colour_3d_instanced_shaders::{self, Camera},
};
use rand::{
    distributions::{Distribution, Uniform},
    rngs::ThreadRng, Rng,
};
use vulkano::{
    buffer::{
        allocator::{SubbufferAllocator, SubbufferAllocatorCreateInfo},
        Buffer, BufferCreateInfo, BufferUsage, Subbuffer,
    },
    command_buffer::{
        allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
        RenderPassBeginInfo,
    },
    descriptor_set::{
        allocator::{DescriptorSetAllocator, StandardDescriptorSetAllocator},
        PersistentDescriptorSet, WriteDescriptorSet,
    },
    device::Device,
    format::{ClearValue, Format},
    image::{view::ImageView, Image, ImageCreateInfo, ImageType, ImageUsage},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
    pipeline::{graphics::viewport::Viewport, GraphicsPipeline, Pipeline, PipelineBindPoint},
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass},
};
use vulkano_util::{
    context::{VulkanoConfig, VulkanoContext},
    renderer::VulkanoWindowRenderer,
    window::VulkanoWindows,
};
use winit::{
    dpi::PhysicalPosition,
    event::{DeviceEvent, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Fullscreen, WindowId},
};

use vulkano::sync::GpuFuture;

const DUNGEON_SIZE: usize = 10;
const ROOM_SIZE: [usize; 3] = [25, 30, 25];
const DOOR_WIDTH_HEIGHT_AND_THICKNESS: [f32; 3] = [2.0, 3.0, 1.5];

const FIXED_DELTA_TIME: f32 = 0.04;
const MAX_SUBSTEPS: u32 = 200;

fn main() {
    let context = VulkanoContext::new(VulkanoConfig::default());
    let event_loop = EventLoop::new();
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

    let pipelines = create_pipelines(context.device(), &render_pass);
    let allocators = create_allocators(context.device(), context.memory_allocator());

    let mut viewport = Viewport {
        offset: [0.0, 0.0],
        extent: [0.0, 0.0],
        depth_range: 0.0..=1.0,
    };

    let mut fps_tracker = FpsTracker::<f64>::new();

    let fps_cap: Option<f64> = None;

    let mut game = create_game(&context.memory_allocator());

    let mut fixed_update_runner = FixedUpdate::new(FIXED_DELTA_TIME);

    event_loop.run(move |event, _, control_flow| match event {
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
            viewport.extent = window_renderer.window_size();
            game.camera.aspect_ratio = window_renderer.aspect_ratio();
        }

        Event::MainEventsCleared => {
            fixed_update_runner.update(MAX_SUBSTEPS, || fixed_update(&mut game));

            update(&mut game);

            render(
                &context,
                &mut windows_manager,
                &allocators,
                &render_pass,
                &game,
                &viewport,
                &pipelines,
            );
            fps_tracker.update();
            //println!("{}", fps_tracker.average_fps());
        }

        Event::WindowEvent {
            event: WindowEvent::KeyboardInput { input, .. },
            ..
        } => {
            on_keyboard_input(
                input,
                control_flow,
                &fps_tracker,
                &mut windows_manager.get_primary_renderer_mut().unwrap(),
                &mut game,
            );
        }

        Event::DeviceEvent {
            event: DeviceEvent::Motion { axis, value },
            ..
        } => {
            let window_renderer = windows_manager.get_primary_renderer_mut().unwrap();
            if game.paused || !window_renderer.window().has_focus() {
                return;
            }

            match axis {
                0 => game.camera.rotation[1] += value as f32 * game.mouse_sensitivity,
                1 => game.camera.rotation[0] -= value as f32 * game.mouse_sensitivity,
                _ => (),
            }

            let window_extent = window_renderer.window_size();

            window_renderer
                .window()
                .set_cursor_position(PhysicalPosition::new(
                    window_extent[0] / 2.0,
                    window_extent[1] / 2.0,
                ))
                .unwrap();
            window_renderer.window().set_cursor_visible(false);
        }

        Event::DeviceEvent {
            event: DeviceEvent::MouseWheel { delta },
            ..
        } => {}

        _ => (),
    });
}

fn render(
    context: &VulkanoContext,
    windows_manager: &mut VulkanoWindows,
    allocators: &Allocators,
    render_pass: &Arc<RenderPass>,
    game: &Game,
    viewport: &Viewport,
    pipelines: &Pipelines,
) {
    let window_renderer = windows_manager.get_primary_renderer_mut().unwrap();

    let future = window_renderer.acquire().unwrap();

    let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
        &allocators.command_buffer_allocator,
        context.graphics_queue().queue_family_index(),
        CommandBufferUsage::OneTimeSubmit,
    )
    .unwrap();

    // Creating a depth buffer and a frame buffer every frame is very very bad. Not avoidable until next vulkano version.

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
        },
    )
    .unwrap();

    let camera_uniform = allocators.subbuffer_allocator.allocate_sized().unwrap();
    *camera_uniform.write().unwrap() = game.camera.to_uniform();

    command_buffer_builder
        .begin_render_pass(
            RenderPassBeginInfo {
                clear_values: vec![
                    // Sets background colour.
                    Some([0.0, 0.0, 1.0, 1.0].into()),
                    Some(ClearValue::Depth(1.0)),
                ],
                ..RenderPassBeginInfo::framebuffer(framebuffer)
            },
            Default::default(),
        )
        .unwrap()
        .set_viewport(0, [viewport.clone()].into_iter().collect())
        .unwrap()
        .bind_pipeline_graphics(pipelines.colour_pipeline.clone())
        .unwrap()
        .bind_descriptor_sets(
            PipelineBindPoint::Graphics,
            pipelines.colour_pipeline.layout().clone(),
            0,
            vec![PersistentDescriptorSet::new(
                &allocators.descriptor_set_allocator,
                pipelines
                    .colour_pipeline
                    .layout()
                    .set_layouts()
                    .get(0)
                    .unwrap()
                    .clone(),
                [WriteDescriptorSet::buffer(0, camera_uniform)],
                [],
            )
            .unwrap()],
        )
        .unwrap();

    draw_instanced(
        &mut command_buffer_builder,
        &game.cuboid_buffers.instance_buffer,
        &game.cuboid_buffers.vertex_buffer,
        &game.cuboid_buffers.index_buffer,
        &allocators.subbuffer_allocator,
    );

    command_buffer_builder
        .end_render_pass(Default::default())
        .unwrap();

    let command_buffer = command_buffer_builder.build().unwrap();

    window_renderer.present(
        future
            .then_execute(context.graphics_queue().clone(), command_buffer)
            .unwrap()
            .boxed(),
        false,
    );
}

struct Allocators {
    command_buffer_allocator: StandardCommandBufferAllocator,
    subbuffer_allocator: SubbufferAllocator,
    descriptor_set_allocator: StandardDescriptorSetAllocator,
}

fn create_allocators(
    device: &Arc<Device>,
    memory_allocator: &Arc<StandardMemoryAllocator>,
) -> Allocators {
    Allocators {
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
    mouse_sensitivity: f32,
    wasd_held: [bool; 4],
    jump_held: bool,
    sprinting: bool,
    paused: bool,
    camera: Camera,
    rooms: Vec<Room>,
    physics: CpuSolver<f32, CommonBody<f32>>,

    cuboid_buffers: ColourBuffers,
}

impl Game {
    fn player(&mut self) -> &mut Player<f32> {
        let CommonBody::Player(ref mut player) = self.physics.bodies[0] else {
            unreachable!("The player will always be index 0 of bodies.")
        };
        player
    }
}

fn create_game(memory_allocator: &Arc<StandardMemoryAllocator>) -> Game {
    let mut game = Game {
        mouse_sensitivity: 1.0,
        wasd_held: [false; 4],
        jump_held: false,
        sprinting: true,
        paused: false,
        camera: Camera {
            position: [2.0, -4.0, 2.0],
            rotation: [0.0; 3],

            ambient_strength: 0.3,
            specular_strength: 0.5,
            light_colour: [0.5; 3],
            light_position: [0.0, -10.0, 0.0],

            near_distance: 0.01,
            far_distance: 50.0,
            aspect_ratio: 0.0,
            fov_y: Radians(std::f32::consts::FRAC_PI_2),
        },
        rooms: vec![Room::Empty; DUNGEON_SIZE * DUNGEON_SIZE],
        physics: CpuSolver::new(
            [0.0, 50.0, 0.0],
            [0.8, 1.0, 0.8],
            [DUNGEON_SIZE + 2, 1, DUNGEON_SIZE + 2],
            [-1.0, -(ROOM_SIZE[1] as f32) - 1.0, -1.0],
            [ROOM_SIZE[0], ROOM_SIZE[1] + 2, ROOM_SIZE[2]],
            OutsideOfGridBoundsBehaviour::ContinueUpdating,
            Vec::with_capacity(DUNGEON_SIZE * DUNGEON_SIZE * 10), // Around 10 bodies per room seems reasonable.
        ),
        cuboid_buffers: ColourBuffers {
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
        },
    };

    game.physics.bodies.push(CommonBody::Player(Player {
        particle: Particle::from_position([2.0, -2.0, 2.0]),
        mass: 30.0,
        friction: 5.0,
        restitution: 0.5,
        half_size: [0.5, 1.0, 0.5],
        dampening: [0.0, 0.0, 0.0],
        grounded: false,
    }));

    let mut rng = rand::thread_rng();
    let linked_variety_range = Uniform::from(0..100);
    let variety_range = Uniform::from(0..100);
    let position_range = Uniform::from(0..DUNGEON_SIZE);

    //let mut marked_rooms = Vec::with_capacity(30); // Might not need marker typed rooms at all.
    //let mut linked_rooms = Vec::with_capacity(30); // 30 linked rooms at a time seems reasonable. IMPORTANT linked rooms shouldn't contain anything. They basically shouldn't exist. The root room is the one that contains everything!

    // Generate linked rooms.
    for _ in 0..rng.gen_range(3..10) {
        let variety = linked_variety_range.sample(&mut rng);
        match variety {
            0..=50 => {
                let position = [position_range.sample(&mut rng), position_range.sample(&mut rng)];
                println!("position: {:?}", position);
                if enough_space_for_room(&game, position, [3,3]) {
                    link_area_to_room(&mut game, position, [3,3]);
                    let mut room = BasicRoom { cuboid_instances: Vec::with_capacity(15*9) };
                    for x in position[0]..(position[0]+3) {
                        for y in position[1]..(position[1]+3) {
                            let real_room_position = [
                                x as f32 * ROOM_SIZE[0] as f32,
                                y as f32 * ROOM_SIZE[2] as f32,
                            ];
                            let i = index_from_position_2d([x,y], DUNGEON_SIZE);
                            room.create_walls_and_floor([x,y], real_room_position, &mut game.physics);
                        }
                    }
                    for instance in &mut room.cuboid_instances {
                        instance.colour = [0.0, 1.0, 0.0, 1.0];
                    }
                    game.rooms[index_from_position_2d(position, DUNGEON_SIZE)] = Room::BasicRoom(room);
                    println!("worked?")
                }
            }
            _ => (),
        }
    }

    // It seems a great waste to spend time generating a room, then just throwing it away cause it gets linked later... Multiple passes?
    for i in 0..DUNGEON_SIZE * DUNGEON_SIZE {
        if let Room::Empty = game.rooms[i] {
            game.rooms[i] = generate_room(
                i,
                variety_range.sample(&mut rng),
                &mut game.physics,
                //&mut marked_rooms,
                //&mut linked_rooms,
            );
        }
    }

    /*
    // Todo, remove.
    for linked_room in linked_rooms {
        game.rooms[linked_room.1] = Room::LinkedRoom(LinkedRoom{real_room:linked_room.0});
    }
    */

    game
}

#[derive(Clone)]
enum Room {
    BasicRoom(BasicRoom),
    LinkedRoom(LinkedRoom),
    Marker, // No room should have this after generation is finished. This is for stuff like linked rooms, that need to link afterwards.
    Empty, // No room should have this after generation is finished. This is the only room type that is safe to replace.
}

impl Room {
    fn get_instances(&self, rooms: &Vec<Room>) -> Vec<Colour3DInstance> {
        match self {
            Room::BasicRoom(room) => room.cuboid_instances.clone(),
            Room::LinkedRoom(room) => rooms[room.real_room].get_instances(rooms),
            Room::Marker | Room::Empty => unreachable!("This room type should not exist after generation is finished."),
        }
    }
}

// bad name. Perhaps relativity or something?
enum RelativeOrConstantF32 {
    Relative(f32),
    Constant(f32),
}

#[derive(Clone)]
struct BasicRoom {
    cuboid_instances: Vec<buffer_contents::Colour3DInstance>,
}

impl BasicRoom {
    fn create_wall(
        &mut self,
        physics: &mut CpuSolver<f32, CommonBody<f32>>,
        room_position: [usize; 2],
        room_size: [usize; 2],
        // Position is the actual f32 position relative to room position and room size. The room is always [1.0,1.0,1.0,] sized no matter what.
        position: [f32; 2],
        // This has a problem. Outside wall thickness is always 1. Perhaps enum so we can have relative and constant size?
        // RelativeOrConstant
        size: [RelativeOrConstantF32; 2],
    ) {
        todo!()
    }

    fn create_walls_and_floor(
        &mut self,
        room_position: [usize; 2],
        real_room_position: [f32; 2],
        //position: [usize; 2],
        //size: [usize; 2],
        //real_position: [f32; 2],
        physics: &mut CpuSolver<f32, CommonBody<f32>>,
    ) {
        // Walls:

        // bottom
        let mut temp_position = [
            real_room_position[0] + ROOM_SIZE[0] as f32 * 0.5,
            0.5,
            real_room_position[1] + ROOM_SIZE[2] as f32 * 0.5,
        ];
        let mut temp_scale = [ROOM_SIZE[0] as f32, 1.0, ROOM_SIZE[2] as f32];
        self.cuboid_instances.push(Colour3DInstance::new(
            [1.0, 1.0, 1.0, 1.0],
            Matrix4::from_translation(temp_position) * Matrix4::from_scale(temp_scale),
        ));
        physics
            .bodies
            .push(CommonBody::ImmovableCuboid(ImmovableCuboid {
                aabb: AabbCentredOrigin {
                    position: temp_position,
                    half_size: math::mul_3d_by_1d(temp_scale, 0.5),
                },
            }));

        // top
        self.cuboid_instances.push(Colour3DInstance::new(
            [1.0, 0.0, 1.0, 1.0],
            Matrix4::from_translation([
                real_room_position[0] + ROOM_SIZE[0] as f32 * 0.5,
                -(ROOM_SIZE[1] as f32) - 0.5,
                real_room_position[1] + ROOM_SIZE[2] as f32 * 0.5,
            ]) * Matrix4::from_scale([ROOM_SIZE[0] as f32, 1.0, ROOM_SIZE[2] as f32]),
        ));

        // +x
        let positive_x_instance_count = if room_position[0] == DUNGEON_SIZE - 1 {
            temp_position = [
                real_room_position[0] + ROOM_SIZE[0] as f32,
                ROOM_SIZE[1] as f32 * -0.5,
                real_room_position[1] + ROOM_SIZE[2] as f32 * 0.5,
            ];
            temp_scale = [1.0, ROOM_SIZE[1] as f32, ROOM_SIZE[2] as f32];
            self.cuboid_instances.push(Colour3DInstance::new(
                [1.0, 1.0, 1.0, 1.0],
                Matrix4::from_translation(temp_position) * Matrix4::from_scale(temp_scale),
            ));
            physics
                .bodies
                .push(CommonBody::ImmovableCuboid(ImmovableCuboid {
                    aabb: AabbCentredOrigin {
                        position: temp_position,
                        half_size: math::mul_3d_by_1d(temp_scale, 0.5),
                    },
                }));
            1
        } else {
            // left
            temp_position = [
                real_room_position[0] + ROOM_SIZE[0] as f32,
                ROOM_SIZE[1] as f32 * -0.5,
                real_room_position[1] + ROOM_SIZE[2] as f32 * 0.25
                    - DOOR_WIDTH_HEIGHT_AND_THICKNESS[0] * 0.25,
            ];
            temp_scale = [
                1.0,
                ROOM_SIZE[1] as f32,
                ROOM_SIZE[2] as f32 * 0.5 - DOOR_WIDTH_HEIGHT_AND_THICKNESS[0] * 0.5,
            ];
            self.cuboid_instances.push(Colour3DInstance::new(
                [1.0, 0.0, 0.0, 1.0],
                Matrix4::from_translation(temp_position) * Matrix4::from_scale(temp_scale),
            ));
            physics
                .bodies
                .push(CommonBody::ImmovableCuboid(ImmovableCuboid {
                    aabb: AabbCentredOrigin {
                        position: temp_position,
                        half_size: math::mul_3d_by_1d(temp_scale, 0.5),
                    },
                }));

            // right
            temp_position = [
                real_room_position[0] + ROOM_SIZE[0] as f32,
                ROOM_SIZE[1] as f32 * -0.5,
                real_room_position[1]
                    + ROOM_SIZE[2] as f32 * 0.75
                    + DOOR_WIDTH_HEIGHT_AND_THICKNESS[0] * 0.25,
            ];
            temp_scale = [
                1.0,
                ROOM_SIZE[1] as f32,
                ROOM_SIZE[2] as f32 * 0.5 - DOOR_WIDTH_HEIGHT_AND_THICKNESS[0] * 0.5,
            ];
            self.cuboid_instances.push(Colour3DInstance::new(
                [0.0, 1.0, 0.0, 1.0],
                Matrix4::from_translation(temp_position) * Matrix4::from_scale(temp_scale),
            ));
            physics
                .bodies
                .push(CommonBody::ImmovableCuboid(ImmovableCuboid {
                    aabb: AabbCentredOrigin {
                        position: temp_position,
                        half_size: math::mul_3d_by_1d(temp_scale, 0.5),
                    },
                }));

            // top
            temp_position = [
                real_room_position[0] + ROOM_SIZE[0] as f32,
                ROOM_SIZE[1] as f32 * -1.0
                    + (ROOM_SIZE[1] as f32 - DOOR_WIDTH_HEIGHT_AND_THICKNESS[1]) * 0.5,
                real_room_position[1] + ROOM_SIZE[2] as f32 * 0.5,
            ];
            temp_scale = [
                1.0,
                ROOM_SIZE[1] as f32 - DOOR_WIDTH_HEIGHT_AND_THICKNESS[1],
                DOOR_WIDTH_HEIGHT_AND_THICKNESS[0],
            ];
            self.cuboid_instances.push(Colour3DInstance::new(
                [0.0, 1.0, 1.0, 1.0],
                Matrix4::from_translation(temp_position) * Matrix4::from_scale(temp_scale),
            ));
            physics
                .bodies
                .push(CommonBody::ImmovableCuboid(ImmovableCuboid {
                    aabb: AabbCentredOrigin {
                        position: temp_position,
                        half_size: math::mul_3d_by_1d(temp_scale, 0.5),
                    },
                }));
            2
        };

        // -x (No aabbs usually because +x will supply.)
        let negative_x_instance_count = if room_position[0] == 0 {
            temp_position = [
                real_room_position[0],
                ROOM_SIZE[1] as f32 * -0.5,
                real_room_position[1] + ROOM_SIZE[2] as f32 * 0.5,
            ];
            temp_scale = [1.0, ROOM_SIZE[1] as f32, ROOM_SIZE[2] as f32];
            self.cuboid_instances.push(Colour3DInstance::new(
                [1.0, 1.0, 1.0, 1.0],
                Matrix4::from_translation(temp_position) * Matrix4::from_scale(temp_scale),
            ));
            physics
                .bodies
                .push(CommonBody::ImmovableCuboid(ImmovableCuboid {
                    aabb: AabbCentredOrigin {
                        position: temp_position,
                        half_size: math::mul_3d_by_1d(temp_scale, 0.5),
                    },
                }));
            1
        } else {
            // right
            self.cuboid_instances.push(Colour3DInstance::new(
                [1.0, 0.0, 0.0, 1.0],
                Matrix4::from_translation([
                    real_room_position[0],
                    ROOM_SIZE[1] as f32 * -0.5,
                    real_room_position[1] + ROOM_SIZE[2] as f32 * 0.25
                        - DOOR_WIDTH_HEIGHT_AND_THICKNESS[0] * 0.25,
                ]) * Matrix4::from_scale([
                    1.0,
                    ROOM_SIZE[1] as f32,
                    ROOM_SIZE[2] as f32 * 0.5 - DOOR_WIDTH_HEIGHT_AND_THICKNESS[0] * 0.5,
                ]),
            ));

            // left
            self.cuboid_instances.push(Colour3DInstance::new(
                [0.0, 1.0, 0.0, 1.0],
                Matrix4::from_translation([
                    real_room_position[0],
                    ROOM_SIZE[1] as f32 * -0.5,
                    real_room_position[1]
                        + ROOM_SIZE[2] as f32 * 0.75
                        + DOOR_WIDTH_HEIGHT_AND_THICKNESS[0] * 0.25,
                ]) * Matrix4::from_scale([
                    1.0,
                    ROOM_SIZE[1] as f32,
                    ROOM_SIZE[2] as f32 * 0.5 - DOOR_WIDTH_HEIGHT_AND_THICKNESS[0] * 0.5,
                ]),
            ));

            // top
            self.cuboid_instances.push(Colour3DInstance::new(
                [0.0, 1.0, 1.0, 1.0],
                Matrix4::from_translation([
                    real_room_position[0],
                    ROOM_SIZE[1] as f32 * -1.0
                        + (ROOM_SIZE[1] as f32 - DOOR_WIDTH_HEIGHT_AND_THICKNESS[1]) * 0.5,
                    real_room_position[1] + ROOM_SIZE[2] as f32 * 0.5,
                ]) * Matrix4::from_scale([
                    1.0,
                    ROOM_SIZE[1] as f32 - DOOR_WIDTH_HEIGHT_AND_THICKNESS[1],
                    DOOR_WIDTH_HEIGHT_AND_THICKNESS[0],
                ]),
            ));
            2
        };

        // +z
        let positive_z_instance_count = if room_position[1] == DUNGEON_SIZE - 1 {
            temp_position = [
                real_room_position[0] + ROOM_SIZE[0] as f32 * 0.5,
                ROOM_SIZE[1] as f32 * -0.5,
                real_room_position[1] + ROOM_SIZE[2] as f32,
            ];
            temp_scale = [ROOM_SIZE[0] as f32, ROOM_SIZE[1] as f32, 1.0];
            self.cuboid_instances.push(Colour3DInstance::new(
                [1.0, 1.0, 1.0, 1.0],
                Matrix4::from_translation(temp_position) * Matrix4::from_scale(temp_scale),
            ));
            physics
                .bodies
                .push(CommonBody::ImmovableCuboid(ImmovableCuboid {
                    aabb: AabbCentredOrigin {
                        position: temp_position,
                        half_size: math::mul_3d_by_1d(temp_scale, 0.5),
                    },
                }));
            1
        } else {
            // right
            temp_position = [
                real_room_position[0] + ROOM_SIZE[0] as f32 * 0.25
                    - DOOR_WIDTH_HEIGHT_AND_THICKNESS[0] * 0.25,
                ROOM_SIZE[1] as f32 * -0.5,
                real_room_position[1] + ROOM_SIZE[2] as f32,
            ];
            temp_scale = [
                ROOM_SIZE[0] as f32 * 0.5 - DOOR_WIDTH_HEIGHT_AND_THICKNESS[0] * 0.5,
                ROOM_SIZE[1] as f32,
                1.0,
            ];
            self.cuboid_instances.push(Colour3DInstance::new(
                [1.0, 0.0, 0.0, 1.0],
                Matrix4::from_translation(temp_position) * Matrix4::from_scale(temp_scale),
            ));
            physics
                .bodies
                .push(CommonBody::ImmovableCuboid(ImmovableCuboid {
                    aabb: AabbCentredOrigin {
                        position: temp_position,
                        half_size: math::mul_3d_by_1d(temp_scale, 0.5),
                    },
                }));

            // left
            temp_position = [
                real_room_position[0]
                    + ROOM_SIZE[0] as f32 * 0.75
                    + DOOR_WIDTH_HEIGHT_AND_THICKNESS[0] * 0.25,
                ROOM_SIZE[1] as f32 * -0.5,
                real_room_position[1] + ROOM_SIZE[2] as f32,
            ];
            temp_scale = [
                ROOM_SIZE[0] as f32 * 0.5 - DOOR_WIDTH_HEIGHT_AND_THICKNESS[0] * 0.5,
                ROOM_SIZE[1] as f32,
                1.0,
            ];
            self.cuboid_instances.push(Colour3DInstance::new(
                [0.0, 1.0, 0.0, 1.0],
                Matrix4::from_translation(temp_position) * Matrix4::from_scale(temp_scale),
            ));
            physics
                .bodies
                .push(CommonBody::ImmovableCuboid(ImmovableCuboid {
                    aabb: AabbCentredOrigin {
                        position: temp_position,
                        half_size: math::mul_3d_by_1d(temp_scale, 0.5),
                    },
                }));

            // top
            temp_position = [
                real_room_position[0] + ROOM_SIZE[0] as f32 * 0.5,
                ROOM_SIZE[1] as f32 * -1.0
                    + (ROOM_SIZE[1] as f32 - DOOR_WIDTH_HEIGHT_AND_THICKNESS[1]) * 0.5,
                real_room_position[1] + ROOM_SIZE[2] as f32,
            ];
            temp_scale = [
                DOOR_WIDTH_HEIGHT_AND_THICKNESS[0],
                ROOM_SIZE[1] as f32 - DOOR_WIDTH_HEIGHT_AND_THICKNESS[1],
                1.0,
            ];
            self.cuboid_instances.push(Colour3DInstance::new(
                [0.0, 1.0, 1.0, 1.0],
                Matrix4::from_translation(temp_position) * Matrix4::from_scale(temp_scale),
            ));
            physics
                .bodies
                .push(CommonBody::ImmovableCuboid(ImmovableCuboid {
                    aabb: AabbCentredOrigin {
                        position: temp_position,
                        half_size: math::mul_3d_by_1d(temp_scale, 0.5),
                    },
                }));
            2
        };

        // -z
        let negative_z_instance_count = if room_position[1] == 0 {
            temp_position = [
                real_room_position[0] + ROOM_SIZE[0] as f32 * 0.5,
                ROOM_SIZE[1] as f32 * -0.5,
                real_room_position[1],
            ];
            temp_scale = [ROOM_SIZE[0] as f32, ROOM_SIZE[1] as f32, 1.0];
            self.cuboid_instances.push(Colour3DInstance::new(
                [1.0, 1.0, 0.0, 1.0],
                Matrix4::from_translation(temp_position) * Matrix4::from_scale(temp_scale),
            ));
            physics
                .bodies
                .push(CommonBody::ImmovableCuboid(ImmovableCuboid {
                    aabb: AabbCentredOrigin {
                        position: temp_position,
                        half_size: math::mul_3d_by_1d(temp_scale, 0.5),
                    },
                }));
            1
        } else {
            // right
            self.cuboid_instances.push(Colour3DInstance::new(
                [1.0, 0.0, 0.0, 1.0],
                Matrix4::from_translation([
                    real_room_position[0] + ROOM_SIZE[0] as f32 * 0.25
                        - DOOR_WIDTH_HEIGHT_AND_THICKNESS[0] * 0.25,
                    ROOM_SIZE[1] as f32 * -0.5,
                    real_room_position[1],
                ]) * Matrix4::from_scale([
                    ROOM_SIZE[0] as f32 * 0.5 - DOOR_WIDTH_HEIGHT_AND_THICKNESS[0] * 0.5,
                    ROOM_SIZE[1] as f32,
                    1.0,
                ]),
            ));

            // left
            self.cuboid_instances.push(Colour3DInstance::new(
                [0.0, 1.0, 0.0, 1.0],
                Matrix4::from_translation([
                    real_room_position[0]
                        + ROOM_SIZE[0] as f32 * 0.75
                        + DOOR_WIDTH_HEIGHT_AND_THICKNESS[0] * 0.25,
                    ROOM_SIZE[1] as f32 * -0.5,
                    real_room_position[1],
                ]) * Matrix4::from_scale([
                    ROOM_SIZE[0] as f32 * 0.5 - DOOR_WIDTH_HEIGHT_AND_THICKNESS[0] * 0.5,
                    ROOM_SIZE[1] as f32,
                    1.0,
                ]),
            ));

            // top
            self.cuboid_instances.push(Colour3DInstance::new(
                [0.0, 1.0, 1.0, 1.0],
                Matrix4::from_translation([
                    real_room_position[0] + ROOM_SIZE[0] as f32 * 0.5,
                    ROOM_SIZE[1] as f32 * -1.0
                        + (ROOM_SIZE[1] as f32 - DOOR_WIDTH_HEIGHT_AND_THICKNESS[1]) * 0.5,
                    real_room_position[1],
                ]) * Matrix4::from_scale([
                    DOOR_WIDTH_HEIGHT_AND_THICKNESS[0],
                    ROOM_SIZE[1] as f32 - DOOR_WIDTH_HEIGHT_AND_THICKNESS[1],
                    1.0,
                ]),
            ));
            2
        };
    }
}

#[derive(Clone)]
struct LinkedRoom {
    real_room: usize,
}

fn generate_room(
    room_index: usize,
    variety: u8,
    physics: &mut CpuSolver<f32, CommonBody<f32>>,
    //marked_rooms: &mut Vec<usize>,
    //linked_rooms: &mut Vec<(usize, usize)>,
) -> Room {
    /*
    for marked_room in &mut *marked_rooms {
        if *marked_room == room_index {
            return Room::Marker;
        }
    }
    */

    let room_position = math::position_from_index_2d(room_index, DUNGEON_SIZE);
    let real_room_position = [
        room_position[0] as f32 * ROOM_SIZE[0] as f32,
        room_position[1] as f32 * ROOM_SIZE[2] as f32,
    ];

    match variety {
        0..=25 => {
            let mut room = BasicRoom {
                cuboid_instances: Vec::with_capacity(15),
            };
            room.create_walls_and_floor(room_position, real_room_position, physics);
            for instance in &mut room.cuboid_instances {
                instance.colour = [0.0, 0.0, 1.0, 1.0];
            }
            Room::BasicRoom(room)
        }
        /*
        26..=40 => {
            //if room_position[0]
            marked_rooms.push(room_index+1);
            linked_rooms.push((room_index, room_index + 1));
            let mut room = BasicRoom {
                cuboid_instances: Vec::with_capacity(15),
            };
            room.create_walls_and_floor(room_position, real_room_position, physics);

            let room_position = math::position_from_index_2d(room_index + 1, DUNGEON_SIZE);
            let real_room_position = [
                room_position[0] as f32 * ROOM_SIZE[0] as f32,
                room_position[1] as f32 * ROOM_SIZE[2] as f32,
            ];
            room.create_walls_and_floor(room_position, real_room_position, physics);

            for instance in &mut room.cuboid_instances {
                instance.colour = [0.0, 1.0, 0.0, 1.0];
            }
            Room::BasicRoom(room)
        }
        */
        _ => {
            let mut room = BasicRoom {
                cuboid_instances: Vec::with_capacity(15),
            };
            room.create_walls_and_floor(room_position, real_room_position, physics);
            Room::BasicRoom(room)
        }
    }
}

// Position is corner.
fn enough_space_for_room(game: &Game, position: [usize; 2], size: [usize; 2]) -> bool {
    if position[0] + size[0] > DUNGEON_SIZE || position[1] + size[1] > DUNGEON_SIZE {
        return false;
    }

    for x in position[0]..(position[0]+size[0]) {
        for y in position[1]..(position[1]+size[1]) {
            println!("(x,y): {:?}", (x,y));
            if let Room::Empty = game.rooms[math::index_from_position_2d([x,y], DUNGEON_SIZE)] {

            } else {
                return false;
            }
        }
    }

    true
}

fn link_area_to_room(game: &mut Game, root: [usize; 2], size: [usize; 2]) {
    let root_index = index_from_position_2d(root, DUNGEON_SIZE);

    for x in root[0]..(root[0]+size[0]) {
        for y in root[1]..(root[1]+size[1]) {
            let i = math::index_from_position_2d([x,y], DUNGEON_SIZE);
            if let Room::Empty = game.rooms[i] {
                game.rooms[i] = Room::LinkedRoom(LinkedRoom { real_room: root_index });
            } else {
                unreachable!("Empty rooms are the only replaceable rooms.");
            }
        }
    }
}

struct ColourBuffers {
    vertex_buffer: Subbuffer<[buffer_contents::Basic3DVertex]>,
    index_buffer: Subbuffer<[u32]>,
    instance_buffer: Vec<buffer_contents::Colour3DInstance>,
}

fn fixed_update(game: &mut Game) {
    let motion = match game.wasd_held {
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

    /*
    let speed = match (sprinting, *jump_held, player.grounded) {
        (false, true, true) | (false, false, true) | (false, true, false) => 25.0,
        (true, true, true) | (true, false, true) | (false, false, false) | (true, true, false) => {
            50.0
        }
        (true, false, false) => 100.0,
    };
    */

    let speed = match game.sprinting {
        true => 50.0,
        false => 25.0,
    };

    let real_motion = (motion.0 * speed, motion.1 * speed);

    let y_rotation_cos = game.camera.rotation[1].to_radians().cos();
    let y_rotation_sin = game.camera.rotation[1].to_radians().sin();

    let real_motion = (
        real_motion.0 * y_rotation_cos - real_motion.1 * y_rotation_sin,
        real_motion.1 * y_rotation_cos + real_motion.0 * y_rotation_sin,
    );

    game.player()
        .particle
        .accelerate([real_motion.0, 0.0, real_motion.1]);

    let horizontal_dampening = if game.player().grounded { 0.8 } else { 0.95 }; // grounded originally 0.8

    game.player().dampening = [horizontal_dampening, 1.0, horizontal_dampening]; // y 0.98 originally

    game.physics.update(FIXED_DELTA_TIME);

    if game.jump_held {
        if game.player().grounded {
            game.player().particle.accelerate([0.0, -500.0, 0.0]);
        }
    }

    game.player().particle.position[0] = game.player().particle.position[0]
        .clamp(0.0, DUNGEON_SIZE as f32 * ROOM_SIZE[0] as f32 - 0.01);
    game.player().particle.position[2] = game.player().particle.position[2]
        .clamp(0.0, DUNGEON_SIZE as f32 * ROOM_SIZE[2] as f32 - 0.01);
    game.camera.position = math::add_3d(game.player().particle.position, [0.0, -1.0, 0.0]);

    game.camera.light_position[0] = game.camera.position[0];
    game.camera.light_position[2] = game.camera.position[2];
}

fn update(game: &mut Game) {
    let current_room_position = [
        game.camera.position[0] as usize / ROOM_SIZE[0],
        game.camera.position[2] as usize / ROOM_SIZE[2],
    ];

    game.cuboid_buffers.instance_buffer = game.rooms
        [math::index_from_position_2d(current_room_position, DUNGEON_SIZE)]
    .get_instances(&game.rooms);
}

fn on_keyboard_input(
    input: KeyboardInput,
    control_flow: &mut ControlFlow,
    fps_tracker: &FpsTracker<f64>,
    window_renderer: &mut VulkanoWindowRenderer,
    game: &mut Game,
) {
    if let Some(key_code) = input.virtual_keycode {
        match key_code {
            VirtualKeyCode::W => game.wasd_held[0] = is_pressed(input.state),
            VirtualKeyCode::A => game.wasd_held[1] = is_pressed(input.state),
            VirtualKeyCode::S => game.wasd_held[2] = is_pressed(input.state),
            VirtualKeyCode::D => game.wasd_held[3] = is_pressed(input.state),

            VirtualKeyCode::Backslash => {
                if is_pressed(input.state) {
                    if let None = window_renderer.window().fullscreen() {
                        window_renderer
                            .window()
                            .set_fullscreen(Some(Fullscreen::Borderless(None)));
                    } else {
                        window_renderer.window().set_fullscreen(None);
                    }
                }
            }

            VirtualKeyCode::F => {
                if is_pressed(input.state) {
                    game.sprinting = !game.sprinting;
                }
            }

            VirtualKeyCode::Space => game.jump_held = is_pressed(input.state),

            VirtualKeyCode::Delete => {
                if is_pressed(input.state) {
                    *control_flow = ControlFlow::Exit;
                }
            }

            VirtualKeyCode::X => {
                if is_pressed(input.state) {
                    println!("{}", fps_tracker.average_fps());
                }
            }

            VirtualKeyCode::Escape => {
                if is_pressed(input.state) {
                    game.paused = !game.paused;
                }
            }
            _ => (),
        }
    }
}
