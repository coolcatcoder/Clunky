use std::sync::Arc;

use vulkano::{
    device::Device, pipeline::graphics::input_assembly::PrimitiveTopology, shader::ShaderModule,
    Validated, VulkanError,
};

pub enum EditFrequency {
    Rarely,
    Often,
    VeryOften,
}

impl EditFrequency {
    pub const fn to_buffer_amount(&self) -> usize {
        match *self {
            EditFrequency::Rarely => 1,
            EditFrequency::Often => 2,
            EditFrequency::VeryOften => 3,
        }
    }
}

pub enum VertexShader {
    Instanced2D,
}

impl VertexShader {
    pub fn load(&self, device: Arc<Device>) -> Result<Arc<ShaderModule>, Validated<VulkanError>> {
        match *self {
            VertexShader::Instanced2D => crate::vertex_shader_map::load(device),
        }
    }
}
pub enum FragmentShader {
    Instanced2D,
}

impl FragmentShader {
    pub fn load(&self, device: Arc<Device>) -> Result<Arc<ShaderModule>, Validated<VulkanError>> {
        match *self {
            FragmentShader::Instanced2D => crate::fragment_shader_map::load(device),
        }
    }
}

/* Unsolved problems:

How will uniform buffers work? Perhaps have a function field in the struct that gives access to render and user stuff?

How on earth are regular buffers even going to work?

We need someway to force recreation of pipelines from user code, so switching menus can force recreation

How will multiple pipelines work? What if something needs multiple vertex definitions, and therefore multiple draw calls? How can this be handled? Temp solution: Only allow 2 vertex buffers. One with colours, and one with uv.

Perhaps instead of having arrays of 2, we should instead have an option<> of an array of the whole struct. Like [Option<MenuRenderSettings>; 2] or something.

Ui handling??
*/
pub struct RenderSettings {
    pub uv_vertex_and_index_buffer_settings: Option<VertexAndIndexBufferSettings>,
    pub colour_vertex_and_index_buffer_settings: Option<VertexAndIndexBufferSettings>,
}

pub struct VertexAndIndexBufferSettings {
    pub edit_frequency: EditFrequency,
    pub instance_edit_frequency: Option<EditFrequency>,

    pub vertex_shader: VertexShader,
    pub fragment_shader: FragmentShader,

    pub topology: PrimitiveTopology,

    pub depth: bool,
}
