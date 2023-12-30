use gltf::{buffer::Data, mesh::util::ReadIndices, Primitive};

pub const MESH_LOADERS: &[MeshLoader] = &[
    MeshLoader {
        path: "src/meshes/ico_sphere.glb",
        primitive_and_buffers_to_arrays: |primitive, buffers, debug| {
            basic_3d_to_arrays("SPHERE", primitive, buffers, debug)
        },
    },
    MeshLoader {
        path: "src/meshes/cube.glb",
        primitive_and_buffers_to_arrays: |primitive, buffers, debug| {
            basic_3d_to_arrays("CUBE", primitive, buffers, debug)
        },
    },
];

pub struct MeshLoader {
    pub path: &'static str,
    pub primitive_and_buffers_to_arrays: fn(Primitive<'_>, Vec<Data>, &mut String) -> String,
}

fn basic_3d_to_arrays(
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
            vertex_position[1],
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
