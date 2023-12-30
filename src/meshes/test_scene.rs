// Const naming scheme should be MESH_NAME_INSTANCE_TYPE_PLURALISED
// Currently we will just use mod with .rs as the file extension for scenes

use crate::buffer_contents;
use crate::math::Degrees;
use crate::math::Matrix4;

pub const CUBE_COLOUR_3D_INSTANCES: &[buffer_contents::Colour3DInstance] = &[
    buffer_contents::Colour3DInstance::new(
        // Simple red cube.
        [1.0, 0.0, 0.0, 1.0],
        Matrix4::from_translation([0.0, -0.75, -5.0]),
    ),
    buffer_contents::Colour3DInstance::new(
        // Magenta light bulb
        [1.0, 0.0, 1.0, 1.0],
        Matrix4::from_translation([0.0, -6.5, 0.0]).multiply(Matrix4::from_scale([3.5, 0.1, 3.5])),
    ),
    buffer_contents::Colour3DInstance::new(
        // Partially hidden cube inside of the giant purple sphere
        [0.961, 0.678, 0.184, 1.0],
        Matrix4::from_translation([-20.0, -5.0, 20.0])
            .multiply(Matrix4::from_scale([7.0, 7.0, 7.0])),
    ),
    buffer_contents::Colour3DInstance::new(
        // Transparent blue rotated rectangle
        [0.0, 1.0, 1.0, 0.75],
        Matrix4::from_translation([3.5, -1.0, -2.5])
            .multiply(Matrix4::from_angle_x(Degrees(45.0).to_radians()))
            .multiply(Matrix4::from_scale([0.5, 3.5, 2.6])),
    ),
];

pub const SPHERE_COLOUR_3D_INSTANCES: &[buffer_contents::Colour3DInstance] = &[
    buffer_contents::Colour3DInstance::new(
        [0.0, 1.0, 0.0, 1.0],
        Matrix4::from_scale([10.0, 0.5, 10.0]),
    ),
    buffer_contents::Colour3DInstance::new(
        [0.525, 0.067, 0.78, 1.0],
        Matrix4::from_translation([-20.0, -5.0, 20.0]).multiply(Matrix4::from_scale([7.0, 7.0, 7.0])),
    ),
];

/*
List of wants and needs for scenes:

EW 1. Scenes should be loaded compile time rather than run time.
N 2. Scenes should be easilly editable from a scene editing menu or via modifying the file itself.
N 3. Scene files should contain instance arrays

Goal state key:
N = Not started implementation.
S = Started work on implementation.
EB = Experimental implementation that is broken.
EW = Experimental implementation that is working.
A = Goal achieved.
*/
