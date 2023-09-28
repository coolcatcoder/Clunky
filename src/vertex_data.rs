use vulkano::{buffer::BufferContents, pipeline::graphics::vertex_input::Vertex};

#[derive(BufferContents, Vertex)]
#[repr(C)]
pub struct VertexData {
    #[format(R32G32_SFLOAT)]
    pub position: [f32; 2],
}
