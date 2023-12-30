use crate::buffer_contents;

include!(concat!(env!("OUT_DIR"), "/gltf_meshes.rs"));

// include entire scenes here to get instance arrays
//include!("scenes/test_scene.clunky_scene");
pub mod test_scene;

pub const DEBUG_VIEWER: bool = false;
