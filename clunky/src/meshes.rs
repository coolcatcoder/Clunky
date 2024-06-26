use crate::buffer_contents;
use crate::math;
use crate::physics;

include!(concat!(env!("OUT_DIR"), "/loaded_from_gltf.rs"));

// include entire scenes here to get instance arrays
//include!("scenes/test_scene.clunky_scene");
pub mod test_scene;

pub const DEBUG_VIEWER: bool = false;

// NEW:

pub const CUBE_GLTF: &[u8] = include_bytes!("./meshes/Box.glb");

/// Gets indices from gltf and converts them to whatever format you want, as long as it implements From<u32>.
pub fn get_indices_from_gltf<T: TryFrom<u32>>(gltf: &[u8], mesh_index: usize) -> Vec<T>
where
    <T as TryFrom<u32>>::Error: std::fmt::Debug,
{
    let (gltf, buffers, _) = gltf::import_slice(gltf).unwrap();

    let mesh = gltf.meshes().nth(mesh_index).unwrap();
    let primitive = mesh.primitives().next().unwrap();

    let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

    let mut indices = vec![];

    for index in reader.read_indices().unwrap().into_u32() {
        indices.push(index.try_into().unwrap())
    }

    indices
}
