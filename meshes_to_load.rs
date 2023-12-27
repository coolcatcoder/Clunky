pub const MESH_LOADERS: &[MeshLoader] = &[
    MeshLoader {
        path: "src/meshes/test_cube.glb",
        prefix: "CUBE",
    }
];

pub struct MeshLoader {
    pub path: &'static str,

    pub prefix: &'static str,
}