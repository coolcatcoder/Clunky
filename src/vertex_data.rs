use vulkano::{buffer::BufferContents, pipeline::graphics::vertex_input::Vertex};

#[derive(BufferContents, Vertex, Copy, Clone, Debug)]
#[repr(C)]
pub struct MapVertex {
    #[format(R32G32_SFLOAT)]
    pub position: [f32; 2],

    #[format(R32G32_SFLOAT)]
    pub uv: [f32; 2],
}

#[derive(BufferContents, Vertex, Copy, Clone, Debug)]
#[repr(C)]
pub struct UIVertex {
    #[format(R32G32_SFLOAT)]
    pub position: [f32; 2],

    #[format(R32G32_SFLOAT)]
    pub uv: [f32; 2],

    #[format(R32G32B32A32_SFLOAT)]
    pub colour: [f32; 4],
}
