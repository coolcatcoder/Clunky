/*
TODO:

Island shapes.

Plant generation.

Better player movement.

Rain.

Crafting.

Island spacing, size, and how to get from one to another.

Depths have huge thick islands.
*/
use cgmath::Rad;
use rand::distributions::Bernoulli;
use rand::distributions::Distribution;
use rand::distributions::Uniform;
use rand::rngs::ThreadRng;
use rand::Rng;
use vulkano::buffer::BufferUsage;
use vulkano::pipeline::graphics::input_assembly::PrimitiveTopology;
use vulkano::pipeline::graphics::rasterization::CullMode;
use vulkano::pipeline::graphics::rasterization::FrontFace;
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
use crate::menu_rendering;
use crate::menu_rendering::BufferTypes;
use crate::menu_rendering::EditFrequency;
use crate::menu_rendering::FrequentAccessRenderBuffer;
use crate::menu_rendering::InstanceBuffer;
use crate::menu_rendering::RenderBuffer;
use crate::menu_rendering::UniformBuffer;
use crate::menu_rendering::VertexBuffer;
use crate::menus;
use crate::meshes;
use crate::physics;
use crate::random_generation;
use crate::shaders;

const STEP_HEIGHT: f32 = 0.75;

pub struct Example3DStorage {
    pub camera_3d_position: [f32; 3],
    pub camera_3d_rotation: [f32; 3],
    pub camera_3d_scale: [f32; 3],

    pub particle: physics::physics_3d::verlet::Particle<f32>,

    pub island_storage: IslandStorage,

    pub jump_held: bool,
    pub grounded: bool,

    pub altitude: Altitude,
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
        Vec<buffer_contents::Colour3DInstance>,
        Vec<buffer_contents::Colour3DInstance>,
    ) {
        let altitude_index = altitude.to_u8();

        let mut box_instances = vec![];
        let mut sphere_instances = vec![];
        self.current_aabbs = vec![];

        if altitude_index != 0 {
            if altitude_index != 1 {
                let layer = self.altitude_to_layer(Altitude::from_u8(altitude_index - 1));

                box_instances.append(&mut layer.box_instances.clone());
                sphere_instances.append(&mut layer.sphere_instances.clone());
                self.current_aabbs.append(&mut layer.aabbs.clone());
            }

            if altitude_index != 4 {
                let layer = self.altitude_to_layer(altitude);

                box_instances.append(&mut layer.box_instances.clone());
                sphere_instances.append(&mut layer.sphere_instances.clone());
                self.current_aabbs.append(&mut layer.aabbs.clone());
            }
        }

        if altitude_index != 3 && altitude_index != 4 {
            let layer = self.altitude_to_layer(Altitude::from_u8(altitude_index + 1));

            box_instances.append(&mut layer.box_instances.clone());
            sphere_instances.append(&mut layer.sphere_instances.clone());
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

        (box_instances, sphere_instances)
    }
}

pub struct Layer {
    box_instances: Vec<buffer_contents::Colour3DInstance>,
    sphere_instances: Vec<buffer_contents::Colour3DInstance>,
    aabbs: Vec<physics::physics_3d::aabb::AabbCentredOrigin<f32>>,
}

pub fn get_starting_storage() -> Example3DStorage {
    Example3DStorage {
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
    _scale: [f32; 3],
    overall_island_type: SkyMiddleIslandTypes,
) {
    match overall_island_type {
        SkyMiddleIslandTypes::TallForest => {
            if rng.gen() {
                return;
            }

            let trunk_thickness = rng.gen_range(1.0..3.0);
            let tree_scale = [trunk_thickness, rng.gen_range(30.0..50.0), trunk_thickness];
            let tree_position = [position[0], position[1] - tree_scale[1] * 0.5, position[2]];

            layer
                .box_instances
                .push(buffer_contents::Colour3DInstance::new(
                    [1.0, 0.0, 0.0, 1.0],
                    math::Matrix4::from_translation(tree_position)
                        .multiply(math::Matrix4::from_scale(tree_scale)),
                ));
        }

        SkyMiddleIslandTypes::SmallForest => {
            if rng.gen() {
                return;
            }

            let trunk_thickness = rng.gen_range(0.5..1.5);
            let tree_scale = [trunk_thickness, rng.gen_range(10.0..30.0), trunk_thickness];
            let tree_position = [position[0], position[1] - tree_scale[1] * 0.5, position[2]];

            layer
                .box_instances
                .push(buffer_contents::Colour3DInstance::new(
                    [1.0, 0.0, 0.0, 1.0],
                    math::Matrix4::from_translation(tree_position)
                        .multiply(math::Matrix4::from_scale(tree_scale)),
                ));
        }

        SkyMiddleIslandTypes::Plains => {}
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
        let instances = user_storage
            .example_3d_storage
            .island_storage
            .update_altitude_and_get_instances(user_storage.example_3d_storage.altitude);

        let uniform_buffer = shaders::colour_3d_instanced_shaders::vertex_shader::CameraData3D {
            position: user_storage.example_3d_storage.camera_3d_position.into(),

            ambient_strength: 0.3,
            specular_strength: 0.5.into(),
            light_colour: [0.5, 0.5, 0.5].into(),
            light_position: [0.0, -1250.0, 0.0].into(),

            camera_to_clip: math::Matrix4::IDENTITY_AS_2D_ARRAY,
            world_to_camera: math::Matrix4::IDENTITY_AS_2D_ARRAY,
        };

        render_storage.entire_render_datas = vec![
            menu_rendering::EntireRenderData {
                render_buffers: menu_rendering::RenderBuffers {
                    vertex_buffer: menu_rendering::VertexBuffer::Basic3D(
                        menu_rendering::BufferTypes::RenderBuffer(
                            menu_rendering::RenderBuffer::new(
                                buffer_contents::Basic3DVertex {
                                    position: [0.0, 0.0, 0.0],
                                    normal: [0.0, 0.0, 0.0],
                                },
                                meshes::CUBE_VERTICES.len(),
                                menu_rendering::EditFrequency::Rarely,
                                render_storage.memory_allocator.clone(),
                                BufferUsage::VERTEX_BUFFER,
                            ),
                        ),
                    ),
                    index_buffer: Some(BufferTypes::RenderBuffer(RenderBuffer::new(
                        0,
                        meshes::CUBE_INDICES.len(),
                        EditFrequency::Rarely,
                        render_storage.memory_allocator.clone(),
                        BufferUsage::INDEX_BUFFER,
                    ))),
                    instance_buffer: Some(InstanceBuffer::Colour3D(
                        BufferTypes::FrequentAccessRenderBuffer(FrequentAccessRenderBuffer {
                            buffer: instances.0,
                        }),
                    )),
                    shader_accessible_buffers: Some(menu_rendering::ShaderAccessibleBuffers {
                        uniform_buffer: Some(menu_rendering::UniformBuffer::CameraData3D(
                            menu_rendering::BufferTypes::FrequentAccessRenderBuffer(
                                menu_rendering::FrequentAccessRenderBuffer {
                                    buffer: vec![uniform_buffer],
                                },
                            ),
                        )),
                        image: None,
                    }),
                },
                settings: menu_rendering::Settings {
                    vertex_shader: menu_rendering::VertexShader::Colour3DInstanced,
                    fragment_shader: menu_rendering::FragmentShader::Colour3DInstanced,
                    topology: PrimitiveTopology::TriangleList,
                    depth: true,
                    cull_mode: CullMode::Back,
                    front_face: FrontFace::Clockwise,
                },
            },
            menu_rendering::EntireRenderData {
                render_buffers: menu_rendering::RenderBuffers {
                    vertex_buffer: menu_rendering::VertexBuffer::Basic3D(
                        menu_rendering::BufferTypes::RenderBuffer(
                            menu_rendering::RenderBuffer::new(
                                buffer_contents::Basic3DVertex {
                                    position: [0.0, 0.0, 0.0],
                                    normal: [0.0, 0.0, 0.0],
                                },
                                meshes::SPHERE_VERTICES.len(),
                                menu_rendering::EditFrequency::Rarely,
                                render_storage.memory_allocator.clone(),
                                BufferUsage::VERTEX_BUFFER,
                            ),
                        ),
                    ),
                    index_buffer: Some(BufferTypes::RenderBuffer(RenderBuffer::new(
                        0,
                        meshes::SPHERE_INDICES.len(),
                        EditFrequency::Rarely,
                        render_storage.memory_allocator.clone(),
                        BufferUsage::INDEX_BUFFER,
                    ))),
                    instance_buffer: Some(InstanceBuffer::Colour3D(
                        BufferTypes::FrequentAccessRenderBuffer(FrequentAccessRenderBuffer {
                            //buffer: Vec::from(meshes::test_scene::SPHERE_COLOUR_3D_INSTANCES),
                            buffer: instances.1,
                        }),
                    )),
                    shader_accessible_buffers: Some(menu_rendering::ShaderAccessibleBuffers {
                        uniform_buffer: Some(menu_rendering::UniformBuffer::CameraData3D(
                            menu_rendering::BufferTypes::FrequentAccessRenderBuffer(
                                menu_rendering::FrequentAccessRenderBuffer {
                                    buffer: vec![uniform_buffer],
                                },
                            ),
                        )),
                        image: None,
                    }),
                },
                settings: menu_rendering::Settings {
                    vertex_shader: menu_rendering::VertexShader::Colour3DInstanced,
                    fragment_shader: menu_rendering::FragmentShader::Colour3DInstanced,
                    topology: PrimitiveTopology::TriangleList,
                    depth: true,
                    cull_mode: CullMode::Back,
                    front_face: FrontFace::Clockwise,
                },
            },
        ];

        let entire_render_data_0 = &mut render_storage.entire_render_datas[0];

        // TODO: create macro for assuming a buffer is of a type

        let VertexBuffer::Basic3D(vertex_buffer) =
            &mut entire_render_data_0.render_buffers.vertex_buffer
        else {
            panic!()
        };
        let BufferTypes::RenderBuffer(vertex_buffer) = vertex_buffer else {
            panic!()
        };

        vertex_buffer.buffer.copy_from_slice(meshes::CUBE_VERTICES);

        vertex_buffer.element_count = meshes::CUBE_VERTICES.len();
        vertex_buffer.update_buffer = true;

        let Some(BufferTypes::RenderBuffer(index_buffer)) =
            &mut entire_render_data_0.render_buffers.index_buffer
        else {
            panic!()
        };

        index_buffer.buffer.copy_from_slice(meshes::CUBE_INDICES);

        index_buffer.element_count = meshes::CUBE_INDICES.len();
        index_buffer.update_buffer = true;

        let entire_render_data_1 = &mut render_storage.entire_render_datas[1];

        let VertexBuffer::Basic3D(vertex_buffer) =
            &mut entire_render_data_1.render_buffers.vertex_buffer
        else {
            panic!()
        };
        let BufferTypes::RenderBuffer(vertex_buffer) = vertex_buffer else {
            panic!()
        };

        vertex_buffer
            .buffer
            .copy_from_slice(meshes::SPHERE_VERTICES);

        vertex_buffer.element_count = meshes::SPHERE_VERTICES.len();
        vertex_buffer.update_buffer = true;

        let Some(BufferTypes::RenderBuffer(index_buffer)) =
            &mut entire_render_data_1.render_buffers.index_buffer
        else {
            panic!()
        };

        index_buffer.buffer.copy_from_slice(meshes::SPHERE_INDICES);

        index_buffer.element_count = meshes::SPHERE_INDICES.len();
        index_buffer.update_buffer = true;

        render_storage.force_run_window_dependent_setup = true;
    },
    update: |_user_storage, _render_storage, _delta_time, _average_fps| {},
    fixed_update: menus::FixedUpdate {
        delta_time: 0.04,
        max_substeps: 150,
        function: |user_storage, render_storage| {
            //user_storage.camera_3d_rotation[2] += 10.0 *MENU.fixed_update.0;
            //user_storage.camera_3d_rotation[2] %= 360.0;

            //println!("{}", user_storage.example_3d_storage.try_jump);

            // let (entire_render_data_boxes, entire_render_data_spheres) = unsafe {
            //     (&mut *(render_storage.entire_render_datas[0] as *mut _),
            //     &mut *(render_storage.entire_render_datas[1] as *mut _))
            // };

            let entire_render_data_boxes = &mut render_storage.entire_render_datas[0];

            let Some(uniform_buffer) = &mut entire_render_data_boxes
                .render_buffers
                .shader_accessible_buffers
            else {
                panic!()
            };
            let Some(uniform_buffer) = &mut uniform_buffer.uniform_buffer else {
                panic!()
            };
            let UniformBuffer::CameraData3D(uniform_buffer) = uniform_buffer else {
                panic!()
            };
            let BufferTypes::FrequentAccessRenderBuffer(uniform_buffer) = uniform_buffer else {
                panic!()
            };

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

            if user_storage.example_3d_storage.jump_held || user_storage.example_3d_storage.grounded
            {
                speed *= 0.5;
            }

            let real_motion = (motion.0 * speed, motion.1 * speed);

            let y_rotation_cos =
                (user_storage.example_3d_storage.camera_3d_rotation[1].to_radians()).cos();
            let y_rotation_sin =
                (user_storage.example_3d_storage.camera_3d_rotation[1].to_radians()).sin();

            let real_motion = (
                real_motion.0 * y_rotation_cos - real_motion.1 * y_rotation_sin,
                real_motion.1 * y_rotation_cos + real_motion.0 * y_rotation_sin,
            );

            user_storage.example_3d_storage.particle.accelerate([
                real_motion.0,
                50.0,
                real_motion.1,
            ]);

            let mut displacement = user_storage
                .example_3d_storage
                .particle
                .calculate_displacement();

            let horizontal_dampening = if user_storage.example_3d_storage.grounded {
                0.8
            } else {
                0.95
            };

            displacement[0] = displacement[0] * horizontal_dampening;
            displacement[1] = displacement[1] * 0.98;
            displacement[2] = displacement[2] * horizontal_dampening;

            user_storage
                .example_3d_storage
                .particle
                .update(MENU.fixed_update.delta_time, displacement);

            let previous_player_aabb = physics::physics_3d::aabb::AabbCentredOrigin {
                position: user_storage.example_3d_storage.particle.previous_position,
                half_size: [0.5, 1.0, 0.5],
            }; // don't know if this needs to be mut, it might? Test and find out. TODO

            let mut player_aabb = physics::physics_3d::aabb::AabbCentredOrigin {
                position: user_storage.example_3d_storage.particle.position,
                half_size: [0.5, 1.0, 0.5],
            }; // this is mutable so that future aabbs don't detect false collision, incase a previous collision moved it out of the way of a future collision aswell. This might be wrong. TODO

            user_storage.example_3d_storage.grounded = false;

            for aabb in &user_storage.example_3d_storage.island_storage.current_aabbs {
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
                            // Would this need to be previous aabb if it was mut? TODO
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
                            user_storage.example_3d_storage.grounded = true;
                        }
                    }
                }
            }

            if user_storage.example_3d_storage.jump_held {
                if user_storage.example_3d_storage.grounded {
                    user_storage
                        .example_3d_storage
                        .particle
                        .accelerate([0.0, -1000.0, 0.0]);
                } else {
                    if user_storage.wasd_held.0
                        || user_storage.wasd_held.1
                        || user_storage.wasd_held.2
                        || user_storage.wasd_held.3
                    {
                        user_storage
                            .example_3d_storage
                            .particle
                            .accelerate([0.0, -50.0, 0.0]);
                    } else {
                        user_storage
                            .example_3d_storage
                            .particle
                            .accelerate([0.0, -300.0, 0.0]);
                    }
                }
            }

            user_storage.example_3d_storage.particle.position = player_aabb.position;

            let mut instances = None;

            let altitude =
                Altitude::get_altitude(user_storage.example_3d_storage.particle.position[1]);

            if altitude != user_storage.example_3d_storage.altitude {
                user_storage.example_3d_storage.altitude = altitude;

                let Some(instance_buffer) =
                    &mut entire_render_data_boxes.render_buffers.instance_buffer
                else {
                    panic!()
                };
                let InstanceBuffer::Colour3D(instance_buffer) = instance_buffer else {
                    panic!()
                };
                let BufferTypes::FrequentAccessRenderBuffer(instance_buffer) = instance_buffer
                else {
                    panic!()
                };

                let temp_instances = user_storage
                    .example_3d_storage
                    .island_storage
                    .update_altitude_and_get_instances(altitude);

                instance_buffer.buffer = temp_instances.0;

                instances = Some(temp_instances.1);
            }

            user_storage.example_3d_storage.camera_3d_position =
                user_storage.example_3d_storage.particle.position;

            user_storage.example_3d_storage.camera_3d_position[1] -= 1.0;

            uniform_buffer.buffer[0].position =
                user_storage.example_3d_storage.camera_3d_position.into();

            // uniform_buffer.buffer[0].world_to_camera =
            //     (Matrix4::from_angle_x(Deg(user_storage.camera_3d_rotation[0]))
            //         * Matrix4::from_angle_y(Deg(user_storage.camera_3d_rotation[1]))
            //         * Matrix4::from_translation(user_storage.camera_3d_position.into()))
            //     .into();

            uniform_buffer.buffer[0].world_to_camera =
                math::Matrix4::from_scale(user_storage.example_3d_storage.camera_3d_scale)
                    .multiply(math::Matrix4::from_angle_x(
                        math::Degrees(user_storage.example_3d_storage.camera_3d_rotation[0])
                            .to_radians(),
                    ))
                    .multiply(math::Matrix4::from_angle_y(
                        math::Degrees(user_storage.example_3d_storage.camera_3d_rotation[1])
                            .to_radians(),
                    ))
                    .multiply(math::Matrix4::from_angle_z(
                        math::Degrees(user_storage.example_3d_storage.camera_3d_rotation[2])
                            .to_radians(),
                    ))
                    .multiply(math::Matrix4::from_translation([
                        -user_storage.example_3d_storage.camera_3d_position[0],
                        -user_storage.example_3d_storage.camera_3d_position[1],
                        -user_storage.example_3d_storage.camera_3d_position[2],
                    ]))
                    .as_2d_array();

            uniform_buffer.buffer[0].camera_to_clip = cgmath::perspective(
                Rad(std::f32::consts::FRAC_PI_2),
                render_storage.other_aspect_ratio,
                0.01,
                1000.0,
            )
            .into();

            let temp_uniform_copy = uniform_buffer.buffer[0]; // This is temp because having 2 uniform buffers in this case is not requires, and probably costs us quite a bit of performance and memory. Fix asap with reusable buffers or give up and have good performance only available to those who want to do everything manually.

            let entire_render_data_1 = &mut render_storage.entire_render_datas[1];

            let Some(uniform_buffer) = &mut entire_render_data_1
                .render_buffers
                .shader_accessible_buffers
            else {
                panic!()
            };
            let Some(uniform_buffer) = &mut uniform_buffer.uniform_buffer else {
                panic!()
            };
            let UniformBuffer::CameraData3D(uniform_buffer) = uniform_buffer else {
                panic!()
            };
            let BufferTypes::FrequentAccessRenderBuffer(uniform_buffer) = uniform_buffer else {
                panic!()
            };

            uniform_buffer.buffer[0] = temp_uniform_copy;

            if let Some(instances) = instances {
                let Some(instance_buffer) =
                    &mut entire_render_data_1.render_buffers.instance_buffer
                else {
                    panic!()
                };
                let InstanceBuffer::Colour3D(instance_buffer) = instance_buffer else {
                    panic!()
                };
                let BufferTypes::FrequentAccessRenderBuffer(instance_buffer) = instance_buffer
                else {
                    panic!()
                };

                instance_buffer.buffer = instances
            }
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
                    user_storage.example_3d_storage.camera_3d_rotation[1] +=
                        value as f32 * user_storage.sensitivity
                }
                1 => {
                    user_storage.example_3d_storage.camera_3d_rotation[0] -=
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
    create_pipelines: |_extent, _render_pass, _user_storage, _render_storage| vec![],
    on_draw: |_user_storage, _render_storage, _sprites, _sampler, _pipelines, _builder| {},
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
                user_storage.example_3d_storage.jump_held = is_pressed(input.state)
            }
            _ => (),
        }
    }
}
