use vulkano::{buffer::BufferContents, pipeline::graphics::vertex_input::Vertex};

#[derive(BufferContents, Vertex, Copy, Clone, Debug)]
#[repr(C)]
pub struct UvInstance {
    #[format(R32G32B32_SFLOAT)]
    pub position_offset: [f32; 3],

    #[format(R32G32_SFLOAT)]
    pub scale: [f32; 2],

    #[format(R32G32_SFLOAT)]
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
