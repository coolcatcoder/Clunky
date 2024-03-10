use gltf::{buffer::Data, mesh::util::ReadIndices, Document, Primitive};

pub const LOADERS: &[Loader] = &[
    Loader {
        path: "src/meshes/ico_sphere.glb",
        gltf_and_buffers_to_constants: |gltf, buffers, debug| {
            let mesh = gltf.meshes().next().unwrap();
            let primitive = mesh.primitives().next().unwrap();
            basic_3d_mesh_to_arrays("SPHERE", primitive, buffers, debug)
        },
    },
    Loader {
        path: "src/meshes/cube.glb",
        gltf_and_buffers_to_constants: |gltf, buffers, debug| {
            let mesh = gltf.meshes().next().unwrap();
            let primitive = mesh.primitives().next().unwrap();
            basic_3d_mesh_to_arrays("CUBE", primitive, buffers, debug)
        },
    },
    Loader {
        path: "src/meshes/moon_wax_tree.glb",
        gltf_and_buffers_to_constants: |gltf, buffers, debug| {
            let mesh = gltf.meshes().next().unwrap();
            let primitive = mesh.primitives().next().unwrap();
            uv_3d_mesh_to_arrays("MOON_WAX_TREE", primitive, buffers, debug)
        },
    },
    Loader {
        path: "src/meshes/simple_grass.glb",
        gltf_and_buffers_to_constants: |gltf, buffers, debug| {
            let mesh = gltf.meshes().next().unwrap();
            let primitive = mesh.primitives().next().unwrap();
            basic_3d_mesh_to_arrays("SIMPLE_GRASS", primitive, buffers, debug)
        },
    },
    Loader {
        path: "src/meshes/fnaf_scene.glb",
        gltf_and_buffers_to_constants: |gltf, _buffers, debug| {
            format!(
                "{}\n{}",
                basic_3d_scene_to_arrays(
                    &gltf,
                    debug,
                    "FNAF_SCENE",
                    &[("Icosphere", "SPHERE"), ("Cube", "CUBE"),]
                ),
                basic_3d_scene_to_a_2d_aabb_array(&gltf, debug, "FNAF_SCENE", "wall")
            )
        },
    },
    Loader {
        path: "src/meshes/rooms/test.glb",
        gltf_and_buffers_to_constants: |gltf, _buffers, debug| {
            format!(
                "{}\n{}",
                dungeon_3d_scene_to_arrays(
                    &gltf,
                    debug,
                    "ROOM_TEST",
                    &[("Icosphere", "SPHERE"), ("Cube", "CUBE"),]
                ),
                dungeon_3d_scene_to_a_3d_aabb_array(&gltf, debug, "ROOM_TEST", "solid")
            )
        },
    },
];

pub struct Loader {
    pub path: &'static str,
    pub gltf_and_buffers_to_constants: fn(Document, Vec<Data>, &mut String) -> String,
}

fn basic_3d_mesh_to_arrays(
    prefix: &'static str,
    primitive: Primitive<'_>,
    buffers: Vec<Data>,
    debug: &mut String,
) -> String {
    let mut vertices = format!(
        "pub const {}_VERTICES: &[buffer_contents::Basic3DVertex] = &[",
        prefix
    );
    let mut indices = format!("pub const {}_INDICES: &[u32] = &[", prefix);

    let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

    let Some(position_iterator) = reader.read_positions() else {
        panic!()
    };
    debug.push_str(&format!("vertex amount: {}\n", position_iterator.len()));

    let Some(normal_iterator) = reader.read_normals() else {
        panic!()
    };

    let Some(ReadIndices::U16(index_iterator)) = reader.read_indices() else {
        println!("{:?}", reader.read_indices());
        panic!()
    };
    debug.push_str(&format!("index amount: {}\n", index_iterator.len()));

    for (vertex_position, normal) in position_iterator.zip(normal_iterator) {
        vertices.push_str(&format!(
            "
            buffer_contents::Basic3DVertex {{
                position: [{0:?}, {1:?}, {2:?}],
                normal: [{3:?}, {4:?}, {5:?}]
            }},\n
            ",
            vertex_position[0],
            -vertex_position[1],
            vertex_position[2],
            normal[0],
            normal[1],
            normal[2]
        ));
    }

    for index in index_iterator {
        indices.push_str(&format!("{},", index));
    }

    debug.push_str(&format!("vertices: {vertices}\n"));
    debug.push_str(&format!("indices: {indices}\n"));
    debug.push_str(&format!("Mode: {:?}\n", primitive.mode()));
    format!("{0}];\n{1}];\n", vertices, indices)
}

fn uv_3d_mesh_to_arrays(
    prefix: &'static str,
    primitive: Primitive<'_>,
    buffers: Vec<Data>,
    debug: &mut String,
) -> String {
    let mut vertices = format!(
        "pub const {}_VERTICES: &[buffer_contents::Uv3DVertex] = &[",
        prefix
    );
    let mut indices = format!("pub const {}_INDICES: &[u32] = &[", prefix);

    let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

    let Some(position_iterator) = reader.read_positions() else {
        panic!()
    };
    debug.push_str(&format!("vertex amount: {}\n", position_iterator.len()));

    let Some(normal_iterator) = reader.read_normals() else {
        panic!()
    };

    let Some(uv_iterator) = reader.read_tex_coords(0) else {
        panic!()
    };

    let Some(ReadIndices::U16(index_iterator)) = reader.read_indices() else {
        panic!()
    };
    debug.push_str(&format!("index amount: {}\n", index_iterator.len()));

    for (vertex_position, (normal, uv)) in
        position_iterator.zip(normal_iterator.zip(uv_iterator.into_f32()))
    {
        vertices.push_str(&format!(
            "
            buffer_contents::Uv3DVertex {{
                position: [{0:?}, {1:?}, {2:?}],
                normal: [{3:?}, {4:?}, {5:?}],
                uv: [{6:?}, {7:?}]
            }},\n
            ",
            vertex_position[0],
            -vertex_position[1],
            vertex_position[2],
            normal[0],
            normal[1],
            normal[2],
            uv[0],
            uv[1],
        ));
    }

    for index in index_iterator {
        indices.push_str(&format!("{},", index));
    }

    debug.push_str(&format!("vertices: {vertices}\n"));
    debug.push_str(&format!("indices: {indices}\n"));
    debug.push_str(&format!("Mode: {:?}\n", primitive.mode()));
    format!("{0}];\n{1}];\n", vertices, indices)
}

fn basic_3d_scene_to_arrays(
    gltf: &Document,
    debug: &mut String,
    scene_prefix: &'static str,
    node_names_and_corresponding_infixes: &[(&'static str, &'static str)],
) -> String {
    //let mut spheres = String::from("pub const ISLAND_TEST_SCENE_SPHERE_COLOUR_3D_INSTANCES: &[buffer_contents::Colour3DInstance] = &[");
    let mut constant_arrays = vec![];

    for strings in node_names_and_corresponding_infixes {
        constant_arrays.push(format!(
            "pub const {}_{}_COLOUR_3D_INSTANCES: &[buffer_contents::Colour3DInstance] = &[",
            scene_prefix, strings.1
        ));
    }

    for node in gltf.nodes() {
        if let Some(node_name) = node.name() {
            let transform_matrix = node.transform().matrix();
            let colour = {
                let Some(temp_colour) = node.extras() else {
                    panic!()
                };

                let temp_colour = temp_colour.get();

                let temp_colour = temp_colour.get(10..temp_colour.len() - 1).unwrap();

                temp_colour
            };

            for strings_index in 0..node_names_and_corresponding_infixes.len() {
                let strings = node_names_and_corresponding_infixes[strings_index];
                if node_name.contains(strings.0) {
                    constant_arrays[strings_index].push_str(&format!(
                        "
                    buffer_contents::Colour3DInstance::new(
                        {},
                        math::Matrix4::from_angle_x_const(math::Degrees(180.0).to_radians())
                        .multiply(math::Matrix4::from_angle_y_const(math::Degrees(180.0).to_radians()))
                        .multiply(math::Matrix4 {{
                            x: {:?},
                            y: {:?},
                            z: {:?},
                            w: {:?},
                        }}),
                    ),
                    ",
                        colour,
                        transform_matrix[0],
                        transform_matrix[1],
                        transform_matrix[2],
                        transform_matrix[3]
                    ));
                }
            }
        }
    }

    let mut combined_arrays = String::from("");

    for mut constant_array in constant_arrays {
        constant_array.push_str("];");

        combined_arrays.push_str(&constant_array);
    }

    debug.push_str(&format!("{}\n", combined_arrays));
    combined_arrays
}

fn basic_3d_scene_to_a_2d_aabb_array(
    gltf: &Document,
    debug: &mut String,
    scene_prefix: &'static str,
    name_of_node: &'static str,
) -> String {
    let mut aabbs = format!(
        "pub const {}_AABBS: &[physics::physics_2d::aabb::AabbCentredOrigin<f32>] = &[",
        scene_prefix,
    );

    for node in gltf.nodes() {
        if let Some(node_name) = node.name() {
            let transform_decomposed = node.transform().decomposed();

            if node_name.contains(name_of_node) {
                aabbs.push_str(&format!(
                    "
                physics::physics_2d::aabb::AabbCentredOrigin::<f32>{{
                    position: [{:?}, -{:?}],
                    half_size: [{:?}, {:?}],
                }},
                ",
                    transform_decomposed.0[0],
                    transform_decomposed.0[2],
                    transform_decomposed.2[0] / 2.0,
                    transform_decomposed.2[2] / 2.0,
                ));
            }
        }
    }

    aabbs.push_str("];");

    debug.push_str(&format!("{}\n", aabbs));
    aabbs
}

fn dungeon_3d_scene_to_arrays(
    gltf: &Document,
    debug: &mut String,
    scene_prefix: &'static str,
    node_names_and_corresponding_infixes: &[(&'static str, &'static str)],
) -> String {
    let mut constant_arrays = vec![];

    for strings in node_names_and_corresponding_infixes {
        constant_arrays.push(format!(
            "pub const {}_{}_COLOUR_3D_INSTANCES: &[buffer_contents::Colour3DInstance] = &[",
            scene_prefix, strings.1
        ));
    }

    for node in gltf.nodes() {
        if let Some(node_name) = node.name() {
            let transform_decomposed = node.transform().decomposed();
            let colour = {
                if let Some(temp_colour) = node.extras() {
                    let temp_colour = temp_colour.get();

                    let temp_colour = temp_colour.get(10..temp_colour.len() - 1).unwrap();

                    temp_colour
                } else {
                    "[1.0,1.0,1.0,1.0]"
                }
            };

            for strings_index in 0..node_names_and_corresponding_infixes.len() {
                let strings = node_names_and_corresponding_infixes[strings_index];
                if node_name.contains(strings.0) {
                    constant_arrays[strings_index].push_str(&format!(
                        "
                    buffer_contents::Colour3DInstance::new(
                        {},
                        math::Matrix4::from_translation([{:?},-{:?},-{:?}]).multiply(math::Matrix4::from_scale([{:?},{:?},{:?}])),
                    ),
                    ",
                        colour,
                        transform_decomposed.0[0], transform_decomposed.0[1], transform_decomposed.0[2], transform_decomposed.2[0], transform_decomposed.2[1], transform_decomposed.2[2],
                    ));
                }
            }
        }
    }

    let mut combined_arrays = String::from("");

    for mut constant_array in constant_arrays {
        constant_array.push_str("];");

        combined_arrays.push_str(&constant_array);
    }

    debug.push_str(&format!("{}\n", combined_arrays));
    combined_arrays
}

fn dungeon_3d_scene_to_a_3d_aabb_array(
    gltf: &Document,
    debug: &mut String,
    scene_prefix: &'static str,
    name_of_node: &'static str,
) -> String {
    let mut aabbs = format!(
        "pub const {}_AABBS: &[physics::physics_3d::aabb::AabbCentredOrigin<f32>] = &[",
        scene_prefix,
    );

    for node in gltf.nodes() {
        if let Some(node_name) = node.name() {
            let transform_decomposed = node.transform().decomposed();

            if node_name.contains(name_of_node) {
                aabbs.push_str(&format!(
                    "
                physics::physics_3d::aabb::AabbCentredOrigin::<f32>{{
                    position: [{:?}, -{:?}, -{:?}],
                    half_size: [{:?}, {:?}, {:?}],
                }},
                ",
                    transform_decomposed.0[0],
                    transform_decomposed.0[1],
                    transform_decomposed.0[2],
                    transform_decomposed.2[0] / 2.0,
                    transform_decomposed.2[1] / 2.0,
                    transform_decomposed.2[2] / 2.0,
                ));
            }
        }
    }

    aabbs.push_str("];");

    debug.push_str(&format!("{}\n", aabbs));
    aabbs
}

fn dungeon_3d_scene_to_common_body_arrays(
    gltf: &Document,
    debug: &mut String,
    scene_prefix: &'static str,
) -> String {
    const BASE_TYPE: &str = "physics::physics_3d::bodies::CommonBody::<f32>";
    const VERLET_BODY_MOD: &str = "physics::physics_3d::verlet::bodies";

    let mut bodies = format!(
        "pub const {}_BODIES: &[{}] = &[",
        scene_prefix,
        BASE_TYPE
    );

    for node in gltf.nodes() {
        if let Some(node_name) = node.name() {
            let transform_decomposed = node.transform().decomposed();

            match node_name {
                "cuboid" => {
                    bodies.push_str(&format!(
                        "
                    {BASE_TYPE}::Cuboid({VERLET_BODY_MOD}::Cuboid{{
                        position: [{:?}, -{:?}, -{:?}],
                        half_size: [{:?}, {:?}, {:?}],
                    }}),
                    ",
                        transform_decomposed.0[0],
                        transform_decomposed.0[1],
                        transform_decomposed.0[2],
                        transform_decomposed.2[0] / 2.0,
                        transform_decomposed.2[1] / 2.0,
                        transform_decomposed.2[2] / 2.0,
                    ));
                }
                _ => ()
            }
        }
    }

    bodies.push_str("];");

    debug.push_str(&format!("{}\n", bodies));
    bodies
}