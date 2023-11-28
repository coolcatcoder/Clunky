use vulkano::{buffer::BufferContents, pipeline::graphics::vertex_input::Vertex};

// TODO: Rename this file. It is more than just vertex_data (which doesn't even exist currently).

#[derive(BufferContents, Vertex, Copy, Clone, Debug)]
#[repr(C)]
pub struct MapVertex {
    #[format(R32G32B32_SFLOAT)]
    pub position: [f32; 3],

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

#[derive(BufferContents, Vertex, Copy, Clone, Debug)]
#[repr(C)]
pub struct TestInstance { // TODO: work out how this will work with marching squares
    #[format(R32G32B32_SFLOAT)]
    pub position: [f32; 3],

    #[format(R32G32_SFLOAT)]
    pub scale: [f32; 2],

    #[format(R32G32_SFLOAT)]
    pub uv: [f32; 2],
}
