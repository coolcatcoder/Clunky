use crate::math;
use cgmath::Matrix4;
use vulkano::{buffer::BufferContents, pipeline::graphics::vertex_input::Vertex};

#[derive(BufferContents, Vertex, Copy, Clone, Debug)]
#[repr(C)]
/// A poorly named 2d instance, that renders using uv coordinates in a texture.
///
/// TODO: Look into whether using matrix math to premultiply values is better here.
pub struct UvInstance {
    #[format(R32G32B32_SFLOAT)]
    /// The offset that will be added to the vertices' positions, with [2] being depth.
    /// Perhaps should be renamed to "offset".
    pub position_offset: [f32; 3],

    #[format(R32G32_SFLOAT)]
    /// The scale applied to the vertices' positions.
    pub scale: [f32; 2],

    #[format(R32G32_SFLOAT)]
    /// The uv coordinates in the texture that should be considered the centre, in some way.
    /// It is not fully known what was meant by this, but it should be fine to use this for just general uv shenanigans.
    /// This whole instance type is going to disappear soon.
    pub uv_centre: [f32; 2],
}

#[derive(BufferContents, Vertex, Copy, Clone, Debug)]
#[repr(C)]
pub struct ColourInstance {
    #[format(R32G32B32_SFLOAT)]
    pub position_offset: [f32; 3],

    #[format(R32G32_SFLOAT)]
    pub scale: [f32; 2],

    #[format(R32G32_SFLOAT)]
    pub colour: [f32; 4],
}

#[derive(BufferContents, Vertex, Copy, Clone, Debug)]
#[repr(C)]
pub struct Colour3DInstance {
    #[format(R32G32B32A32_SFLOAT)]
    pub colour: [f32; 4],

    #[format(R32G32B32A32_SFLOAT)]
    pub model_to_world_0: [f32; 4],

    #[format(R32G32B32A32_SFLOAT)]
    pub model_to_world_1: [f32; 4],

    #[format(R32G32B32A32_SFLOAT)]
    pub model_to_world_2: [f32; 4],

    #[format(R32G32B32A32_SFLOAT)]
    pub model_to_world_3: [f32; 4],
}

impl Colour3DInstance {
    pub fn clone_matrix(&self) -> math::Matrix4 {
        math::Matrix4 {
            x: self.model_to_world_0,
            y: self.model_to_world_1,
            z: self.model_to_world_2,
            w: self.model_to_world_3,
        }
    }
    pub const fn new_with_cgmath_matrix(
        colour: [f32; 4],
        model_to_world: Matrix4<f32>,
    ) -> Colour3DInstance {
        Colour3DInstance {
            colour,
            model_to_world_0: [
                model_to_world.x.x,
                model_to_world.x.y,
                model_to_world.x.z,
                model_to_world.x.w,
            ],
            model_to_world_1: [
                model_to_world.y.x,
                model_to_world.y.y,
                model_to_world.y.z,
                model_to_world.y.w,
            ],
            model_to_world_2: [
                model_to_world.z.x,
                model_to_world.z.y,
                model_to_world.z.z,
                model_to_world.z.w,
            ],
            model_to_world_3: [
                model_to_world.w.x,
                model_to_world.w.y,
                model_to_world.w.z,
                model_to_world.w.w,
            ],
        }
    }

    pub const fn new(colour: [f32; 4], model_to_world: math::Matrix4) -> Colour3DInstance {
        Colour3DInstance {
            colour,
            model_to_world_0: model_to_world.x,
            model_to_world_1: model_to_world.y,
            model_to_world_2: model_to_world.z,
            model_to_world_3: model_to_world.w,
        }
    }
}

#[derive(BufferContents, Vertex, Copy, Clone, Debug)]
#[repr(C)]
pub struct Uv3DInstance {
    #[format(R32G32_SFLOAT)]
    pub uv_offset: [f32; 2],

    #[format(R32G32B32A32_SFLOAT)]
    pub model_to_world_0: [f32; 4],

    #[format(R32G32B32A32_SFLOAT)]
    pub model_to_world_1: [f32; 4],

    #[format(R32G32B32A32_SFLOAT)]
    pub model_to_world_2: [f32; 4],

    #[format(R32G32B32A32_SFLOAT)]
    pub model_to_world_3: [f32; 4],
}

impl Uv3DInstance {
    pub const fn new(uv_offset: [f32; 2], model_to_world: math::Matrix4) -> Uv3DInstance {
        Uv3DInstance {
            uv_offset,
            model_to_world_0: model_to_world.x,
            model_to_world_1: model_to_world.y,
            model_to_world_2: model_to_world.z,
            model_to_world_3: model_to_world.w,
        }
    }
}

#[derive(BufferContents, Vertex, Copy, Clone, Debug)]
#[repr(C)]
pub struct UvVertex {
    #[format(R32G32B32_SFLOAT)]
    pub position: [f32; 3],

    #[format(R32G32_SFLOAT)]
    pub uv: [f32; 2],
}

#[derive(BufferContents, Vertex, Copy, Clone, Debug)]
#[repr(C)]
pub struct ColourVertex {
    #[format(R32G32B32_SFLOAT)]
    pub position: [f32; 3],

    #[format(R32G32B32A32_SFLOAT)]
    pub colour: [f32; 4],
}

#[derive(BufferContents, Vertex, Copy, Clone, Debug)]
#[repr(C)]
pub struct PositionOnlyVertex {
    #[format(R32G32B32_SFLOAT)]
    pub position: [f32; 3],
}

/// A basic 3d vertex.
/// Absolutely horrible name! And even worse documentation!
#[derive(BufferContents, Vertex, Copy, Clone, Debug)]
#[repr(C)]
pub struct Basic3DVertex {
    #[format(R32G32B32_SFLOAT)]
    pub position: [f32; 3],

    #[format(R32G32B32_SFLOAT)]
    pub normal: [f32; 3],
}

#[derive(BufferContents, Vertex, Copy, Clone, Debug)]
#[repr(C)]
pub struct Uv3DVertex {
    #[format(R32G32B32_SFLOAT)]
    pub position: [f32; 3],

    #[format(R32G32B32_SFLOAT)]
    pub normal: [f32; 3],

    #[format(R32G32_SFLOAT)]
    pub uv: [f32; 2],
}
