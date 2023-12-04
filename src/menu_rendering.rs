use std::sync::Arc;

use vulkano::{
    device::Device, pipeline::graphics::input_assembly::PrimitiveTopology, shader::ShaderModule,
    Validated, VulkanError, buffer::Subbuffer,
};

use crate::vertex_data;

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

/* Goals are ill defined. Better goals:
S 1. Main should not have to be altered in order to add or remove draw calls. Only menus should have to be altered.
S 2. Menus should be able to specify draw calls.
S 3. Menus should be able to specify buffers.
S 4. Menus should be able to specify vertex buffer types.
EW 5. Menus should be able to specify shaders.
A 6. Changing menu state should not change any other states, unless the menu requests it in its start function.
S 7. Menu should have some easy way of accessing the buffers it has requested. Perhaps have a generic that requires a tuple of vertex types? This sounds bad. Split into separate problem, see below.

Goal state key:
N = Not started implementation.
S = Started work on implementation.
EB = Experimental implementation that is broken.
EW = Experimental implementation that is working.
A = Goal achieved.
*/

pub struct ExperimentalRenderCall {
    pub vertex_shader: VertexShader,
    pub fragment_shader: FragmentShader,
    pub vertex_and_index_buffer_settings: ExperimentalVertexAndIndexBufferSettings,
    pub instance_buffer_settings: Option<ExperimentalInstanceBufferSettings>,
    pub depth: bool,
}

pub struct ExperimentalVertexAndIndexBufferSettings {
    pub vertex_buffer_type: VertexBufferType,
    pub vertex_buffer_edit_frequency: EditFrequency,
    pub topology: PrimitiveTopology,
    pub index_buffer_edit_frequency: EditFrequency,
}

pub struct ExperimentalInstanceBufferSettings {
    pub instance_buffer_type: InstanceBufferType,
    pub instance_buffer_edit_frequency: EditFrequency,
}

pub enum VertexBufferType {
    Test,
}

pub enum InstanceBufferType {
    Test,
}

/* problem 7 deconstruction:

we want a struct of an unknown amount of arrays, each of an unknown type. So perhaps a tuple, but how to construct.
Generics might be possible somehow. Perhaps macros?

Perhaps have each menu be its own struct? We can use traits to constrain it heavily perhaps?
Then they can just specify in a trait, what type they want? All menu using functions would need to be generics then...
I don't know.

Easier solution, have a struct of all possible buffers, stored in render_storage. Code is nastier, but it might not work...
We would somehow have to work out what the max amount of buffers is, by checking every menu.
Cause draw calls are decided by the menus.. ahhh

check out https://users.rust-lang.org/t/vector-with-generic-types-heterogeneous-container/46644
I really like the idea of using enums containing types. Not exactly certain how yet. But I'll work it out!
*/


// We can store a Vec of this in render storage, and manipulated via functions run by the menus, should we want to update the buffers.
pub struct RenderBufferContainer {
    pub vertex_buffer: VertexBuffer,
    pub index_buffer: Vec<u32>,
    pub instance_buffer: Option<InstanceBuffer>,
}

pub enum VertexBuffer {
    UvVertexBuffer(Vec<vertex_data::UvVertex>),
}

pub enum InstanceBuffer {
    TestInstanceBuffer(Vec<vertex_data::TestInstance>),
}

// Real buffers below should contain actual sub buffers. Used only by main, not by menus.

pub struct RealRenderBufferContainer {
    pub vertex_buffer: RealVertexBuffer,
    pub index_buffer: Vec<Subbuffer<u32>>,
    pub instance_buffer: Option<RealInstanceBuffer>,
}

pub enum RealVertexBuffer {
    UvVertexBuffer(Vec<Subbuffer<vertex_data::UvVertex>>),
}

pub enum RealInstanceBuffer {
    TestInstanceBuffer(Vec<Subbuffer<vertex_data::TestInstance>>),
}