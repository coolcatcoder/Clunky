use std::sync::Arc;

/*
TODO:
All "todo"s should say these 4 things: difficulty, time, value, dependencies

More plants and biomes. | Easy |  Medium length | 3: Adds more variety, and detail to the world. | None |

Island shapes. | Hard | Unknown length | 4: Makes it feel more like a world of floating islands. Increases immersion. | Collision detection being better. |

Better player movement. | Hard | Very quick or very long. | 2: Movement is used the entire game, so having it feel good is important. | None. |

Rain. | Not enough research. | Not enough research. | 5: I want a dynamic weather system to be a core mechanic. | None. |

Distance fog. | Easy. | Quick. | 1: Slight immersion increase. | Shader reorganisation. |

Crafting. | Hard. | Extremely long. | 7: Crafting will be an essential mechanic. | Text rendering. Materials. |

Island spacing, size, and how to get from one to another.

Depths have huge thick islands.
*/
use rand::distributions::Bernoulli;
use rand::distributions::Distribution;
use rand::distributions::Uniform;
use rand::rngs::ThreadRng;
use rand::Rng;
use vulkano::buffer::allocator::SubbufferAllocator;
use vulkano::buffer::Buffer;
use vulkano::buffer::BufferCreateInfo;
use vulkano::buffer::BufferUsage;
use vulkano::buffer::Subbuffer;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::PrimaryAutoCommandBuffer;
use vulkano::descriptor_set::PersistentDescriptorSet;
use vulkano::descriptor_set::WriteDescriptorSet;
use vulkano::device::Device;
use vulkano::device::DeviceOwned;
use vulkano::memory::allocator::AllocationCreateInfo;
use vulkano::memory::allocator::MemoryTypeFilter;
use vulkano::pipeline::graphics::color_blend::AttachmentBlend;
use vulkano::pipeline::graphics::color_blend::ColorBlendAttachmentState;
use vulkano::pipeline::graphics::color_blend::ColorBlendState;
use vulkano::pipeline::graphics::depth_stencil::CompareOp;
use vulkano::pipeline::graphics::depth_stencil::DepthState;
use vulkano::pipeline::graphics::depth_stencil::DepthStencilState;
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::input_assembly::PrimitiveTopology;
use vulkano::pipeline::graphics::multisample::MultisampleState;
use vulkano::pipeline::graphics::rasterization::CullMode;
use vulkano::pipeline::graphics::rasterization::FrontFace;
use vulkano::pipeline::graphics::rasterization::RasterizationState;
use vulkano::pipeline::graphics::vertex_input::Vertex;
use vulkano::pipeline::graphics::vertex_input::VertexDefinition;
use vulkano::pipeline::graphics::viewport::Scissor;
use vulkano::pipeline::graphics::viewport::Viewport;
use vulkano::pipeline::graphics::viewport::ViewportState;
use vulkano::pipeline::graphics::GraphicsPipelineCreateInfo;
use vulkano::pipeline::layout::PipelineDescriptorSetLayoutCreateInfo;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::Pipeline;
use vulkano::pipeline::PipelineBindPoint;
use vulkano::pipeline::PipelineLayout;
use vulkano::pipeline::PipelineShaderStageCreateInfo;
use vulkano::render_pass::RenderPass;
use vulkano::render_pass::Subpass;
use winit::dpi::PhysicalPosition;
use winit::event::DeviceEvent;
use winit::event::Event;
use winit::event::KeyboardInput;
use winit::event::VirtualKeyCode;
use winit::event::WindowEvent;
use winit::window::Fullscreen;

use crate::buffer_contents;
use crate::lost_code::is_pressed;
use crate::math;
use crate::menus;
use crate::meshes;
use crate::physics;
use crate::random_generation;
use crate::shaders;

const STEP_HEIGHT: f32 = 0.75;

pub struct OtherExample3DStorage {
    pub camera_3d_position: [f32; 3],
    pub camera_3d_rotation: [f32; 3],
    pub camera_3d_scale: [f32; 3],

    pub particle: physics::physics_3d::verlet::Particle<f32>,

    pub island_storage: IslandStorage,

    pub jump_held: bool,
    pub grounded: bool,

    pub altitude: Altitude,

    box_vertex_buffer: Subbuffer<[buffer_contents::Basic3DVertex]>,
    box_index_buffer: Subbuffer<[u32]>,
    box_instance_buffer: Vec<buffer_contents::Colour3DInstance>,

    sphere_vertex_buffer: Subbuffer<[buffer_contents::Basic3DVertex]>,
    sphere_index_buffer: Subbuffer<[u32]>,
    sphere_instance_buffer: Vec<buffer_contents::Colour3DInstance>,

    moon_wax_tree_vertex_buffer: Subbuffer<[buffer_contents::Uv3DVertex]>,
    moon_wax_tree_index_buffer: Subbuffer<[u32]>,
    moon_wax_tree_instance_buffer: Vec<buffer_contents::Uv3DInstance>,

    simple_grass_vertex_buffer: Subbuffer<[buffer_contents::Basic3DVertex]>,
    simple_grass_index_buffer: Subbuffer<[u32]>,
    simple_grass_instance_buffer: Vec<buffer_contents::Colour3DInstance>,

    uniform_buffer: shaders::colour_3d_instanced_vertex_shader::CameraData3D,
    //box_pipeline: Arc<GraphicsPipeline>,
    //sphere_pipeline: Arc<GraphicsPipeline>, TODO: I just can't work out how to do this. I can't init this, because I don't have the stuff until later. I could do Option<> but that is messy and is terrible.
    verlet_solver: physics::physics_3d::verlet::CpuSolver<f32, physics::physics_3d::verlet::bodies::CommonBody<f32>>,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Altitude {
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

pub struct IslandStorage {
    sky_top: Layer,
    sky_middle: Layer,
    sky_bottom: Layer,

    current_aabbs: Vec<physics::physics_3d::aabb::AabbCentredOrigin<f32>>,
}

impl IslandStorage {
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
        &mut self,
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

        self.current_aabbs = vec![];

        if altitude_index != 0 {
            if altitude_index != 1 {
                let layer = self.altitude_to_layer(Altitude::from_u8(altitude_index - 1));

                box_instances.append(&mut layer.box_instances.clone());
                sphere_instances.append(&mut layer.sphere_instances.clone());

                moon_wax_tree_instances.append(&mut layer.moon_wax_tree_instances.clone());
                simple_grass_instances.append(&mut layer.simple_grass_instances.clone());

                self.current_aabbs.append(&mut layer.aabbs.clone());
            }

            if altitude_index != 4 {
                let layer = self.altitude_to_layer(altitude);

                box_instances.append(&mut layer.box_instances.clone());
                sphere_instances.append(&mut layer.sphere_instances.clone());

                moon_wax_tree_instances.append(&mut layer.moon_wax_tree_instances.clone());
                simple_grass_instances.append(&mut layer.simple_grass_instances.clone());

                self.current_aabbs.append(&mut layer.aabbs.clone());
            }
        }

        if altitude_index != 3 && altitude_index != 4 {
            let layer = self.altitude_to_layer(Altitude::from_u8(altitude_index + 1));

            box_instances.append(&mut layer.box_instances.clone());
            sphere_instances.append(&mut layer.sphere_instances.clone());

            moon_wax_tree_instances.append(&mut layer.moon_wax_tree_instances.clone());
            simple_grass_instances.append(&mut layer.simple_grass_instances.clone());

            self.current_aabbs.append(&mut layer.aabbs.clone());
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

pub struct Layer {
    box_instances: Vec<buffer_contents::Colour3DInstance>,
    sphere_instances: Vec<buffer_contents::Colour3DInstance>,

    moon_wax_tree_instances: Vec<buffer_contents::Uv3DInstance>,

    simple_grass_instances: Vec<buffer_contents::Colour3DInstance>,

    aabbs: Vec<physics::physics_3d::aabb::AabbCentredOrigin<f32>>,
}

pub fn get_starting_storage(render_storage: &mut crate::RenderStorage) -> OtherExample3DStorage {
    OtherExample3DStorage {
        camera_3d_position: [0.0, 0.0, 0.0],
        camera_3d_rotation: [0.0, 0.0, 0.0],
        camera_3d_scale: [1.0, 1.0, 1.0],

        particle: physics::physics_3d::verlet::Particle::from_position([0.0, -1050.0, 0.0]),

        island_storage: IslandStorage {
            sky_top: temp_gen_islands(-1000.0..-750.0, [1.0, 0.0, 0.0, 1.0]),
            sky_middle: generate_islands_circle_technique(
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
            ),
            sky_bottom: generate_islands_circle_technique(
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
            ),

            current_aabbs: vec![],
        },

        jump_held: false,
        grounded: false,

        altitude: Altitude::AboveAll,

        box_vertex_buffer: Buffer::from_iter(
            render_storage.memory_allocator.clone(),
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
        box_index_buffer: Buffer::from_iter(
            render_storage.memory_allocator.clone(),
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
        box_instance_buffer: vec![],

        sphere_vertex_buffer: Buffer::from_iter(
            render_storage.memory_allocator.clone(),
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
        sphere_index_buffer: Buffer::from_iter(
            render_storage.memory_allocator.clone(),
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
        sphere_instance_buffer: vec![],

        moon_wax_tree_vertex_buffer: Buffer::from_iter(
            render_storage.memory_allocator.clone(),
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
        moon_wax_tree_index_buffer: Buffer::from_iter(
            render_storage.memory_allocator.clone(),
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
        moon_wax_tree_instance_buffer: vec![],

        simple_grass_vertex_buffer: Buffer::from_iter(
            render_storage.memory_allocator.clone(),
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
        simple_grass_index_buffer: Buffer::from_iter(
            render_storage.memory_allocator.clone(),
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
        simple_grass_instance_buffer: vec![],

        uniform_buffer: shaders::colour_3d_instanced_vertex_shader::CameraData3D {
            position: [0.0, 0.0, 0.0],

            ambient_strength: 0.3,
            specular_strength: 0.5.into(),
            light_colour: [0.5, 0.5, 0.5].into(),
            light_position: [0.0, 1250.0, 0.0].into(), // Y is up here, I don't know. I don't understand.

            camera_to_clip: math::Matrix4::IDENTITY_AS_2D_ARRAY,
            world_to_camera: math::Matrix4::IDENTITY_AS_2D_ARRAY,
        },

        verlet_solver: physics::physics_3d::verlet::CpuSolver::new(
            [0.0, 50.0, 0.0],
            [0.8, 1.0, 0.8],
            [2, 15, 2],
            [-1000.0, -2000.0, -1000.0],
            [1000, 300, 1000],
            physics::physics_3d::verlet::OutsideOfGridBoundsBehaviour::ContinueUpdating,
            vec![physics::physics_3d::verlet::bodies::CommonBody::Cuboid(
                physics::physics_3d::verlet::bodies::Cuboid {
                    particle: physics::physics_3d::verlet::Particle::from_position([
                        0.0, -1050.0, 0.0,
                    ]),
                    half_size: [0.5, 1.0, 0.5],
                },
            )],
        ),
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum TempIslandTiles {
    Rock,
    Cliff,
    Nothing,
}

impl TempIslandTiles {
    fn to_colour(&self) -> [f32; 4] {
        match self {
            TempIslandTiles::Rock => [0.5, 0.5, 0.5, 1.0],
            TempIslandTiles::Cliff => [0.25, 0.25, 0.25, 1.0],
            _ => panic!(),
        }
    }
}

impl random_generation::wave_function_collapse::Cell for TempIslandTiles {}

fn get_possibilities(
    _cells: &Vec<
        random_generation::wave_function_collapse::CellStateStorePossibilities<TempIslandTiles>,
    >,
    cell_index: usize,
) -> Vec<TempIslandTiles> {
    let _cell_position = math::position_from_index_2d(cell_index, 100);

    // if cell_position[0] != 0 {
    //     if let random_generation::wave_function_collapse::CellStateStorePossibilities::Decided(island_tile) = cells[cell_index-1] {
    //         if island_tile == TempIslandTiles::Cliff {
    //             cliff_nearby = true;
    //         }
    //     }
    // }

    vec![TempIslandTiles::Rock, TempIslandTiles::Cliff]
}

fn pick_possibility(
    _cells: &Vec<
        random_generation::wave_function_collapse::CellStateStorePossibilities<TempIslandTiles>,
    >,
    possibilities: &Vec<TempIslandTiles>,
    _cell_index: usize,
) -> TempIslandTiles {
    // let mut actual_possibilities = Vec::with_capacity(possibilities.len());
    // let mut rng = rand::thread_rng();

    // for possibility in possibilities {
    //     if possibility == TempIslandTiles::Nothing {
    //         return TempIslandTiles::Nothing
    //     }

    //     if possibility == TempIslandTiles::Cliff && rng {
    //         return TempIslandTiles::Cliff
    //     }

    //     actual_possibilities.push(possibility);
    // }
    // actual_possibilities[rng.gen_range(0..actual_possibilities.len())]

    possibilities[rand::thread_rng().gen_range(0..possibilities.len())]
}

fn temp_gen_islands(_vertical_range: std::ops::Range<f32>, _debug_colour: [f32; 4]) -> Layer {
    let offset = [-50.0, -900.0, -50.0];

    let grid_size = [100, 100];

    let mut layer = Layer {
        box_instances: vec![],
        sphere_instances: vec![],

        moon_wax_tree_instances: vec![],

        simple_grass_instances: vec![],

        aabbs: vec![],
    };

    let test_island =
        random_generation::wave_function_collapse::generate_2d_assumes_only_4_nearest_tiles_matter_and_starting_position_is_not_on_edge(
            grid_size,
            [5, 5],
            vec![
                TempIslandTiles::Rock,
                TempIslandTiles::Cliff,
                TempIslandTiles::Nothing,
            ],
            get_possibilities,
            pick_possibility,
        );

    for cell_index in 0..test_island.len() {
        let position_x_z = math::position_from_index_2d(cell_index, grid_size[0]);
        let position = [
            offset[0] + position_x_z[0] as f32,
            offset[1],
            offset[2] + position_x_z[1] as f32,
        ];

        if let TempIslandTiles::Nothing = test_island[cell_index] {
            continue;
        } else {
            layer
                .box_instances
                .push(buffer_contents::Colour3DInstance::new(
                    test_island[cell_index].to_colour(),
                    math::Matrix4::from_translation(position),
                ));
            layer
                .aabbs
                .push(physics::physics_3d::aabb::AabbCentredOrigin {
                    position,
                    half_size: [0.5, 0.5, 0.5],
                });
        }
    }

    layer
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
) -> Layer {
    let mut layer = Layer {
        box_instances: vec![],
        sphere_instances: vec![],

        moon_wax_tree_instances: vec![],
        simple_grass_instances: vec![],

        aabbs: vec![],
    };

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
                .sphere_instances
                .push(buffer_contents::Colour3DInstance::new(
                    colour,
                    math::Matrix4::from_translation(previous_position)
                        .multiply(math::Matrix4::from_scale(previous_scale)),
                ));
            layer
                .aabbs
                .push(physics::physics_3d::aabb::AabbCentredOrigin {
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

    layer
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

pub const MENU: menus::Data = menus::Data {
    start: |user_storage, render_storage| {
        let (box_instances, sphere_instances, moon_wax_tree_instances, simple_grass_instances) =
            user_storage
                .other_example_3d_storage
                .island_storage
                .update_altitude_and_get_instances(user_storage.other_example_3d_storage.altitude);

        user_storage.other_example_3d_storage.box_instance_buffer = box_instances;
        user_storage.other_example_3d_storage.sphere_instance_buffer = sphere_instances;
        user_storage
            .other_example_3d_storage
            .simple_grass_instance_buffer = simple_grass_instances;

        user_storage
            .other_example_3d_storage
            .moon_wax_tree_instance_buffer = moon_wax_tree_instances;

        render_storage.force_run_window_dependent_setup = true;

        for aabb in &user_storage
            .other_example_3d_storage
            .island_storage
            .sky_top
            .aabbs
        {
            user_storage
                .other_example_3d_storage
                .verlet_solver
                .bodies
                .push(physics::physics_3d::verlet::bodies::CommonBody::ImmovableCuboid(
                    physics::physics_3d::verlet::bodies::ImmovableCuboid { aabb: *aabb },
                ));
        }
    },
    update: |_user_storage, _render_storage, _delta_time, _average_fps| {},
    fixed_update: menus::FixedUpdate {
        delta_time: 0.04,
        max_substeps: 150,
        function: |user_storage, render_storage| {
            //user_storage.camera_3d_rotation[2] += 10.0 *MENU.fixed_update.0;
            //user_storage.camera_3d_rotation[2] %= 360.0;

            //println!("{}", user_storage.other_example_3d_storage.try_jump);

            let motion = match user_storage.wasd_held {
                (true, false, false, false) => (0.0, -1.0),
                (false, false, true, false) => (0.0, 1.0),
                (false, false, false, true) => (1.0, 0.0),
                (false, true, false, false) => (-1.0, 0.0),

                (true, true, false, false) => (-0.7, -0.7),
                (true, false, false, true) => (0.7, -0.7),

                (false, true, true, false) => (-0.7, 0.7),
                (false, false, true, true) => (0.7, 0.7),

                _ => (0.0, 0.0),
            };

            let mut speed = match user_storage.sprinting {
                true => 100.0,
                false => 50.0,
            };

            if user_storage.other_example_3d_storage.jump_held
                || user_storage.other_example_3d_storage.grounded
            {
                speed *= 0.5;
            }

            let real_motion = (motion.0 * speed, motion.1 * speed);

            let y_rotation_cos =
                (user_storage.other_example_3d_storage.camera_3d_rotation[1].to_radians()).cos();
            let y_rotation_sin =
                (user_storage.other_example_3d_storage.camera_3d_rotation[1].to_radians()).sin();

            let real_motion = (
                real_motion.0 * y_rotation_cos - real_motion.1 * y_rotation_sin,
                real_motion.1 * y_rotation_cos + real_motion.0 * y_rotation_sin,
            );

            user_storage.other_example_3d_storage.particle.accelerate([
                real_motion.0,
                50.0,
                real_motion.1,
            ]);

            let mut displacement = user_storage
                .other_example_3d_storage
                .particle
                .calculate_displacement();

            let horizontal_dampening = if user_storage.other_example_3d_storage.grounded {
                0.8
            } else {
                0.95
            };

            displacement[0] = displacement[0] * horizontal_dampening;
            displacement[1] = displacement[1] * 0.98;
            displacement[2] = displacement[2] * horizontal_dampening;

            user_storage
                .other_example_3d_storage
                .particle
                .update(MENU.fixed_update.delta_time, displacement);

            user_storage
                .other_example_3d_storage
                .verlet_solver
                .update(MENU.fixed_update.delta_time);

            let previous_player_aabb = physics::physics_3d::aabb::AabbCentredOrigin {
                position: user_storage
                    .other_example_3d_storage
                    .particle
                    .previous_position,
                half_size: [0.5, 1.0, 0.5],
            }; // don't know if this needs to be mut, it might? Test and find out. TODO:

            let mut player_aabb = physics::physics_3d::aabb::AabbCentredOrigin {
                position: user_storage.other_example_3d_storage.particle.position,
                half_size: [0.5, 1.0, 0.5],
            }; // this is mutable so that future aabbs don't detect false collision, incase a previous collision moved it out of the way of a future collision aswell. This might be wrong. TODO:

            user_storage.other_example_3d_storage.grounded = false;

            for aabb in &user_storage
                .other_example_3d_storage
                .island_storage
                .current_aabbs
            {
                if aabb.is_intersected_by_aabb(player_aabb) {
                    let previous_collision_axis = aabb.get_collision_axis(previous_player_aabb);

                    let mut step = false;

                    if previous_collision_axis[0] {
                        if ((player_aabb.position[1] + player_aabb.half_size[1])
                            - (aabb.position[1] - aabb.half_size[1]))
                            .abs()
                            <= STEP_HEIGHT
                        {
                            println!("step x");
                            step = true;
                            player_aabb.position[1] = aabb.position[1] - aabb.half_size[1];
                        } else {
                            player_aabb.position[0] =
                                previous_player_aabb.position[0] - player_aabb.half_size[1] - 0.1;
                            // Would this need to be previous aabb if it was mut? TODO:
                        }
                    }

                    if previous_collision_axis[2] {
                        if ((player_aabb.position[1] + player_aabb.half_size[1])
                            - (aabb.position[1] - aabb.half_size[1]))
                            .abs()
                            <= STEP_HEIGHT
                        {
                            println!("step z");
                            step = true;
                            player_aabb.position[1] = aabb.position[1]
                                - aabb.half_size[1]
                                - player_aabb.half_size[1]
                                - 0.1;
                        } else {
                            player_aabb.position[2] = previous_player_aabb.position[2];
                        }
                    }

                    if previous_collision_axis[1] && !step {
                        player_aabb.position[1] = previous_player_aabb.position[1];

                        if previous_player_aabb.position[1] + previous_player_aabb.half_size[1]
                            <= aabb.position[1] - aabb.half_size[1]
                        {
                            user_storage.other_example_3d_storage.grounded = true;
                        }
                    }
                }
            }

            if user_storage.other_example_3d_storage.jump_held {
                if user_storage.other_example_3d_storage.grounded {
                    user_storage
                        .other_example_3d_storage
                        .particle
                        .accelerate([0.0, -1000.0, 0.0]);
                } else {
                    if user_storage.wasd_held.0
                        || user_storage.wasd_held.1
                        || user_storage.wasd_held.2
                        || user_storage.wasd_held.3
                    {
                        user_storage
                            .other_example_3d_storage
                            .particle
                            .accelerate([0.0, -50.0, 0.0]);
                    } else {
                        user_storage
                            .other_example_3d_storage
                            .particle
                            .accelerate([0.0, -300.0, 0.0]);
                    }
                }
            }

            user_storage.other_example_3d_storage.particle.position = player_aabb.position;

            let altitude =
                Altitude::get_altitude(user_storage.other_example_3d_storage.particle.position[1]);

            if altitude != user_storage.other_example_3d_storage.altitude {
                user_storage.other_example_3d_storage.altitude = altitude;

                let (
                    box_instances,
                    sphere_instances,
                    moon_wax_tree_instances,
                    simple_grass_instances,
                ) = user_storage
                    .other_example_3d_storage
                    .island_storage
                    .update_altitude_and_get_instances(
                        user_storage.other_example_3d_storage.altitude,
                    );

                user_storage.other_example_3d_storage.box_instance_buffer = box_instances;
                user_storage.other_example_3d_storage.sphere_instance_buffer = sphere_instances;
                user_storage
                    .other_example_3d_storage
                    .simple_grass_instance_buffer = simple_grass_instances;

                user_storage
                    .other_example_3d_storage
                    .moon_wax_tree_instance_buffer = moon_wax_tree_instances;
            }

            user_storage.other_example_3d_storage.camera_3d_position =
                user_storage.other_example_3d_storage.particle.position;

            // if let physics::physics_3d::verlet::VerletBody::SimpleBox(simple_box) = &user_storage.other_example_3d_storage.verlet_solver.verlet_bodies[0] {
            //     user_storage.other_example_3d_storage.camera_3d_position = simple_box.particle.position;
            // } else {
            //     panic!();
            // }

            user_storage.other_example_3d_storage.camera_3d_position[1] -= 1.0;

            user_storage
                .other_example_3d_storage
                .uniform_buffer
                .position = user_storage
                .other_example_3d_storage
                .camera_3d_position
                .into();

            user_storage
                .other_example_3d_storage
                .uniform_buffer
                .world_to_camera =
                math::Matrix4::from_scale(user_storage.other_example_3d_storage.camera_3d_scale)
                    .multiply(math::Matrix4::from_angle_x(
                        math::Degrees(user_storage.other_example_3d_storage.camera_3d_rotation[0])
                            .to_radians(),
                    ))
                    .multiply(math::Matrix4::from_angle_y(
                        math::Degrees(user_storage.other_example_3d_storage.camera_3d_rotation[1])
                            .to_radians(),
                    ))
                    .multiply(math::Matrix4::from_angle_z(
                        math::Degrees(user_storage.other_example_3d_storage.camera_3d_rotation[2])
                            .to_radians(),
                    ))
                    .multiply(math::Matrix4::from_translation([
                        -user_storage.other_example_3d_storage.camera_3d_position[0],
                        -user_storage.other_example_3d_storage.camera_3d_position[1],
                        -user_storage.other_example_3d_storage.camera_3d_position[2],
                    ]))
                    .as_2d_array();

            user_storage
                .other_example_3d_storage
                .uniform_buffer
                .camera_to_clip = math::Matrix4::from_perspective(
                // use my perspecitve function intead.
                math::Radians(std::f32::consts::FRAC_PI_2),
                render_storage.other_aspect_ratio,
                0.01,
                1000.0,
            )
            .as_2d_array();
        },
    },
    handle_events: |user_storage, render_storage, event| match event {
        Event::WindowEvent {
            event: WindowEvent::KeyboardInput { input, .. },
            ..
        } => {
            on_keyboard_input(user_storage, render_storage, input);
        }
        Event::DeviceEvent {
            event: DeviceEvent::Motion { axis, value },
            ..
        } => {
            if !render_storage.window.has_focus() {
                return;
            }

            match axis {
                0 => {
                    user_storage.other_example_3d_storage.camera_3d_rotation[1] +=
                        value as f32 * user_storage.sensitivity
                }
                1 => {
                    user_storage.other_example_3d_storage.camera_3d_rotation[0] -=
                        value as f32 * user_storage.sensitivity
                }
                _ => (),
            }

            let window_size = render_storage.window.inner_size();
            render_storage
                .window
                .set_cursor_position(PhysicalPosition::new(
                    window_size.width / 2,
                    window_size.height / 2,
                ))
                .unwrap();
            render_storage.window.set_cursor_visible(false);
        }
        _ => {}
    },
    create_pipelines: |extent, render_pass, _user_storage, render_storage| {
        let device = render_storage.memory_allocator.device();
        vec![
            get_colour_pipeline(device.clone(), extent, render_pass.clone()),
            get_uv_pipeline(device.clone(), extent, render_pass),
        ]
    },
    on_draw: |user_storage, render_storage, sprites, sampler, pipelines, builder| {
        let uniform_buffer = render_storage.buffer_allocator.allocate_sized().unwrap();
        *uniform_buffer.write().unwrap() = user_storage.other_example_3d_storage.uniform_buffer;

        builder
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                pipelines[0].layout().clone(),
                0,
                vec![PersistentDescriptorSet::new(
                    &render_storage.descriptor_set_allocator,
                    pipelines[0].layout().set_layouts().get(0).unwrap().clone(),
                    [WriteDescriptorSet::buffer(0, uniform_buffer.clone())],
                    [],
                )
                .unwrap()],
            )
            .unwrap();

        draw_spheres(pipelines[0].clone(), builder, user_storage, render_storage);
        draw_boxes(pipelines[0].clone(), builder, user_storage, render_storage);

        draw_colour(
            pipelines[0].clone(),
            builder,
            &user_storage
                .other_example_3d_storage
                .simple_grass_instance_buffer,
            &user_storage
                .other_example_3d_storage
                .simple_grass_vertex_buffer,
            &user_storage
                .other_example_3d_storage
                .simple_grass_index_buffer,
            &render_storage.buffer_allocator,
        );

        builder
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                pipelines[1].layout().clone(),
                0,
                vec![
                    PersistentDescriptorSet::new(
                        &render_storage.descriptor_set_allocator,
                        pipelines[1].layout().set_layouts().get(0).unwrap().clone(),
                        [WriteDescriptorSet::buffer(0, uniform_buffer.clone())],
                        [],
                    )
                    .unwrap(),
                    PersistentDescriptorSet::new(
                        &render_storage.descriptor_set_allocator,
                        pipelines[1].layout().set_layouts().get(1).unwrap().clone(),
                        [
                            WriteDescriptorSet::sampler(0, sampler.clone()),
                            WriteDescriptorSet::image_view(1, sprites[1].clone()),
                        ],
                        [],
                    )
                    .unwrap(),
                ],
            )
            .unwrap();

        draw_uv(
            pipelines[1].clone(),
            builder,
            &user_storage
                .other_example_3d_storage
                .moon_wax_tree_instance_buffer,
            &user_storage
                .other_example_3d_storage
                .moon_wax_tree_vertex_buffer,
            &user_storage
                .other_example_3d_storage
                .moon_wax_tree_index_buffer,
            &render_storage.buffer_allocator,
        );

        // You may note that we do not call end_render_pass(), this is because main.rs does this for us.
    },
    end: |_user_storage, _render_storage| {},
};

fn on_keyboard_input(
    user_storage: &mut menus::UserStorage,
    render_storage: &mut crate::RenderStorage,
    input: KeyboardInput,
) {
    if let Some(key_code) = input.virtual_keycode {
        match key_code {
            VirtualKeyCode::W => user_storage.wasd_held.0 = is_pressed(input.state),
            VirtualKeyCode::A => user_storage.wasd_held.1 = is_pressed(input.state),
            VirtualKeyCode::S => user_storage.wasd_held.2 = is_pressed(input.state),
            VirtualKeyCode::D => user_storage.wasd_held.3 = is_pressed(input.state),
            VirtualKeyCode::Up => user_storage.zoom_held.0 = is_pressed(input.state),
            VirtualKeyCode::Down => user_storage.zoom_held.1 = is_pressed(input.state),

            VirtualKeyCode::Backslash => {
                if is_pressed(input.state) {
                    if let None = render_storage.window.fullscreen() {
                        render_storage
                            .window
                            .set_fullscreen(Some(Fullscreen::Borderless(None)));
                    } else {
                        render_storage.window.set_fullscreen(None);
                    }
                }
            }

            VirtualKeyCode::F => {
                if is_pressed(input.state) {
                    user_storage.sprinting = !user_storage.sprinting;
                }
            }

            VirtualKeyCode::Space => {
                user_storage.other_example_3d_storage.jump_held = is_pressed(input.state)
            }
            _ => (),
        }
    }
}

fn get_colour_pipeline(
    device: Arc<Device>,
    extent: [u32; 3],
    render_pass: Arc<RenderPass>,
) -> Arc<GraphicsPipeline> {
    let vertex_shader_entrance = shaders::colour_3d_instanced_vertex_shader::load(device.clone())
        .unwrap()
        .entry_point("main")
        .unwrap();
    let fragment_shader_entrance =
        shaders::colour_3d_instanced_fragment_shader::load(device.clone())
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

    let subpass = Subpass::from(render_pass, 0).unwrap();
    GraphicsPipeline::new(
        device,
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
    device: Arc<Device>,
    extent: [u32; 3],
    render_pass: Arc<RenderPass>,
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

    let subpass = Subpass::from(render_pass, 0).unwrap();
    GraphicsPipeline::new(
        device,
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
                front_face: FrontFace::CounterClockwise, // Negating the y of vertices may flip this.
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

fn draw_boxes(
    pipeline: Arc<GraphicsPipeline>,
    builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
    user_storage: &mut menus::UserStorage,
    render_storage: &mut crate::RenderStorage,
) {
    let instance_buffer = render_storage
        .buffer_allocator
        .allocate_slice(
            user_storage
                .other_example_3d_storage
                .box_instance_buffer
                .len() as u64,
        )
        .unwrap();
    instance_buffer
        .write()
        .unwrap()
        .copy_from_slice(&user_storage.other_example_3d_storage.box_instance_buffer);

    builder
        .bind_pipeline_graphics(pipeline)
        .unwrap()
        .bind_vertex_buffers(
            0,
            (
                user_storage
                    .other_example_3d_storage
                    .box_vertex_buffer
                    .clone(),
                instance_buffer,
            ),
        )
        .unwrap()
        .bind_index_buffer(
            user_storage
                .other_example_3d_storage
                .box_index_buffer
                .clone(),
        )
        .unwrap()
        .draw_indexed(
            user_storage.other_example_3d_storage.box_index_buffer.len() as u32,
            user_storage
                .other_example_3d_storage
                .box_instance_buffer
                .len() as u32,
            0,
            0,
            0,
        )
        .unwrap();
}

fn draw_spheres(
    pipeline: Arc<GraphicsPipeline>,
    builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
    user_storage: &mut menus::UserStorage,
    render_storage: &mut crate::RenderStorage,
) {
    let instance_buffer = render_storage
        .buffer_allocator
        .allocate_slice(
            user_storage
                .other_example_3d_storage
                .sphere_instance_buffer
                .len() as u64,
        )
        .unwrap();
    instance_buffer
        .write()
        .unwrap()
        .copy_from_slice(&user_storage.other_example_3d_storage.sphere_instance_buffer);

    builder
        .bind_pipeline_graphics(pipeline)
        .unwrap()
        .bind_vertex_buffers(
            0,
            (
                user_storage
                    .other_example_3d_storage
                    .sphere_vertex_buffer
                    .clone(),
                instance_buffer,
            ),
        )
        .unwrap()
        .bind_index_buffer(
            user_storage
                .other_example_3d_storage
                .sphere_index_buffer
                .clone(),
        )
        .unwrap()
        .draw_indexed(
            user_storage
                .other_example_3d_storage
                .sphere_index_buffer
                .len() as u32,
            user_storage
                .other_example_3d_storage
                .sphere_instance_buffer
                .len() as u32,
            0,
            0,
            0,
        )
        .unwrap();
}

fn draw_colour(
    pipeline: Arc<GraphicsPipeline>,
    builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
    instances: &Vec<buffer_contents::Colour3DInstance>,
    vertices: &Subbuffer<[buffer_contents::Basic3DVertex]>,
    indices: &Subbuffer<[u32]>,
    buffer_allocator: &SubbufferAllocator,
) {
    let instance_buffer = buffer_allocator
        .allocate_slice(instances.len() as u64)
        .unwrap();
    instance_buffer.write().unwrap().copy_from_slice(instances);

    builder
        .bind_pipeline_graphics(pipeline)
        .unwrap()
        .bind_vertex_buffers(0, (vertices.clone(), instance_buffer))
        .unwrap()
        .bind_index_buffer(indices.clone())
        .unwrap()
        .draw_indexed(indices.len() as u32, instances.len() as u32, 0, 0, 0)
        .unwrap();
}

fn draw_uv(
    pipeline: Arc<GraphicsPipeline>,
    builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
    instances: &Vec<buffer_contents::Uv3DInstance>,
    vertices: &Subbuffer<[buffer_contents::Uv3DVertex]>,
    indices: &Subbuffer<[u32]>,
    buffer_allocator: &SubbufferAllocator,
) {
    let instance_buffer = buffer_allocator
        .allocate_slice(instances.len() as u64)
        .unwrap();
    instance_buffer.write().unwrap().copy_from_slice(instances);

    builder
        .bind_pipeline_graphics(pipeline)
        .unwrap()
        .bind_vertex_buffers(0, (vertices.clone(), instance_buffer))
        .unwrap()
        .bind_index_buffer(indices.clone())
        .unwrap()
        .draw_indexed(indices.len() as u32, instances.len() as u32, 0, 0, 0)
        .unwrap();
}
