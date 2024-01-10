use crate::buffer_contents;
use crate::math;
use crate::physics;

include!(concat!(env!("OUT_DIR"), "/loaded_from_gltf.rs"));

// include entire scenes here to get instance arrays
//include!("scenes/test_scene.clunky_scene");
pub mod test_scene;

pub const DEBUG_VIEWER: bool = false;
