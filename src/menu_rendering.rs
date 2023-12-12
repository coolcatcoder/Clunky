use std::{default, sync::Arc};

use vulkano::{
    buffer::{
        self, allocator::SubbufferAllocator, Buffer, BufferContents, BufferCreateInfo, BufferUsage,
        Subbuffer,
    },
    descriptor_set::PersistentDescriptorSet,
    device::Device,
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
    pipeline::graphics::{
        input_assembly::PrimitiveTopology,
        vertex_input::{Vertex, VertexBufferDescription},
    },
    shader::ShaderModule,
    sync::HostAccessError,
    Validated, VulkanError,
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
    SimpleTest,
}

impl VertexShader {
    pub fn load(&self, device: Arc<Device>) -> Result<Arc<ShaderModule>, Validated<VulkanError>> {
        match *self {
            VertexShader::Instanced2D => crate::vertex_shader_map::load(device),
            VertexShader::SimpleTest => crate::simple_test_vertex_shader::load(device),
        }
    }
}
pub enum FragmentShader {
    Instanced2D,
    SimpleTest,
}

impl FragmentShader {
    pub fn load(&self, device: Arc<Device>) -> Result<Arc<ShaderModule>, Validated<VulkanError>> {
        match *self {
            FragmentShader::Instanced2D => crate::fragment_shader_map::load(device),
            FragmentShader::SimpleTest => crate::simple_test_fragment_shader::load(device),
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

/* Goals are ill defined. Better goals:
EW 1. Main should not have to be altered in order to add or remove draw calls. Only menus should have to be altered.
EW 2. Menus should be able to specify draw calls.
EW 3. Menus should be able to specify buffers.
EW 4. Menus should be able to specify vertex buffer types.
EW 5. Menus should be able to specify shaders.
A 6. Changing menu state should not change any other states, unless the menu requests it in its start function, or elsewhere.
EW 7. Menu should have some easy way of accessing the buffers it has requested. Perhaps have a generic that requires a tuple of vertex types? This sounds bad. Split into separate problem, see below.
EW 8. Some way to specify if a buffer should be created using a subbuffer allocator.
S 9. Images. Images need to work with all the above goals. How? Images require a pipeline. This could be tricky.
S 10. descriptor sets are nightmares. Consider having 1 optional descriptor set as part of the buffers. Perhaps containing the uniform buffers.
N 11. Have some easy way to debug everything. Perhaps consider having a debug struct full of bools, that print during different conditions, such as when a buffer updates, and stuff like that.

Goal state key:
N = Not started implementation.
S = Started work on implementation.
EB = Experimental implementation that is broken.
EW = Experimental implementation that is working.
A = Goal achieved.
*/

// TODO: too much duplication, consider removing the whole vertex and index buffer settings, and instance buffer settings. Perhaps make them their own struct, also containing the buffers. Currently we have 3 structs telling us what the buffer type is. Not good. Too much matching.
// pub struct RenderSettings {
//     pub vertex_shader: VertexShader,
//     pub fragment_shader: FragmentShader,
//     pub vertex_and_index_buffer_settings: VertexAndIndexBufferSettings,
//     pub instance_buffer_settings: Option<InstanceBufferSettings>,
//     pub depth: bool,
// }

// pub struct VertexAndIndexBufferSettings {
//     pub vertex_buffer_type: VertexBufferType,
//     pub vertex_buffer_edit_frequency: EditFrequency,
//     pub topology: PrimitiveTopology,
//     pub index_buffer_length: usize,
//     pub index_buffer_edit_frequency: EditFrequency,
// }

// pub struct InstanceBufferSettings {
//     pub instance_buffer_type: InstanceBufferType,
//     pub instance_buffer_edit_frequency: EditFrequency,
// }

pub enum VertexBufferType {
    Test(usize),
}

pub enum InstanceBufferType {
    Test(usize),
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
// pub struct RenderBuffers {
//     pub vertex_buffer: VertexBuffer,
//     pub vertex_count: usize,
//     pub update_vertex_buffer: bool,

//     pub index_buffer: Vec<u32>,
//     pub index_count: usize,
//     pub update_index_buffer: bool,

//     pub instance_buffer: Option<InstanceBuffer>,
//     pub instance_count: usize,
//     pub update_instance_buffer: bool,
// }

// pub enum VertexBuffer {
//     UvVertexBuffer(Vec<vertex_data::UvVertex>),
//     ColourVertexBuffer(Vec<vertex_data::ColourVertex>),
// }

// pub enum InstanceBuffer {
//     TestInstanceBuffer(Vec<vertex_data::TestInstance>),
// }

// Real buffers below should contain actual sub buffers. Used only by main, not by menus.

// pub struct RealRenderBuffers {
//     pub vertex_buffer: RealVertexBuffer,
//     pub vertex_count: Vec<usize>,
//     pub update_vertex_buffer: Vec<bool>, // Essentially when we get the signal to update the buffer from the menu, we then set that to false, and set this entire vec to true. Whenever we can write to the buffer we set one of them to false, for that specific buffer.

//     pub index_buffer: Vec<Subbuffer<[u32]>>,
//     pub index_count: Vec<usize>,
//     pub update_index_buffer: Vec<bool>,

//     pub instance_buffer: Option<RealInstanceBuffer>,
//     pub instance_count: Vec<usize>,
//     pub update_instance_buffer: Vec<bool>,
// }

// pub enum RealVertexBuffer {
//     UvVertexBuffer(Vec<Subbuffer<[vertex_data::UvVertex]>>),
//     ColourVertexBuffer(Vec<Subbuffer<[vertex_data::ColourVertex]>>),
// }

// pub enum RealInstanceBuffer {
//     TestInstanceBuffer(Vec<Subbuffer<[vertex_data::TestInstance]>>),
// }

pub struct FrequentAccessRenderBuffer<T>
where
    T: BufferContents + Copy,
{
    pub buffer: Vec<T>,
}

impl<T> FrequentAccessRenderBuffer<T>
where
    T: BufferContents + Copy,
{
    pub fn allocate_and_get_real_buffer(
        &self,
        buffer_allocator: &SubbufferAllocator,
    ) -> Subbuffer<[T]> {
        let real_buffer = buffer_allocator
            .allocate_slice(self.buffer.len() as u64)
            .unwrap();
        real_buffer
            .write()
            .unwrap()
            .copy_from_slice(self.buffer.as_slice());

        real_buffer
    }
}

pub struct RenderBuffer<T>
where
    T: BufferContents + Copy,
{
    pub buffer: Vec<T>,
    pub element_count: usize,
    pub update_buffer: bool,

    pub real_buffer: Vec<Subbuffer<[T]>>,
    pub real_element_count: Vec<usize>,
    pub real_update_buffer: Vec<bool>,

    pub edit_frequency: EditFrequency,
}

impl<T> RenderBuffer<T>
where
    T: BufferContents + Copy,
{
    pub fn new(
        default_element: T,
        length: usize,
        edit_frequency: EditFrequency,
        memory_allocator: Arc<StandardMemoryAllocator>,
        usage: BufferUsage,
    ) -> RenderBuffer<T> {
        let real_length = edit_frequency.to_buffer_amount();

        let mut real_buffer = vec![];

        for i in 0..real_length {
            real_buffer.push(
                Buffer::from_iter(
                    memory_allocator.clone(),
                    BufferCreateInfo {
                        usage,
                        ..Default::default()
                    },
                    AllocationCreateInfo {
                        memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                            | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                        ..Default::default()
                    },
                    vec![default_element; length],
                )
                .unwrap(),
            )
        }

        RenderBuffer {
            buffer: vec![default_element; length],
            element_count: 0,
            update_buffer: false,

            real_buffer,
            real_element_count: vec![0; real_length],
            real_update_buffer: vec![false; real_length],

            edit_frequency,
        }
    }

    pub fn update(&mut self, frame_count: usize) {
        if self.update_buffer {
            self.update_buffer = false;
            for update_buffer in &mut self.real_update_buffer {
                *update_buffer = true;
            }
        }

        let real_index = frame_count % self.edit_frequency.to_buffer_amount();

        if !self.real_update_buffer[real_index] {
            return;
        }

        println!("Updated");

        let writer = self.real_buffer[real_index].write();

        match writer {
            Ok(mut writer) => {
                writer[0..self.element_count].copy_from_slice(&self.buffer[0..self.element_count]);
                self.real_element_count[real_index] = self.element_count;
                self.real_update_buffer[real_index] = false;
            }
            Err(e) => match e {
                HostAccessError::AccessConflict(access_conflict) => {
                    println!("Failed to update buffer. {access_conflict}");
                }
                _ => panic!("couldn't write to the buffer: {e}"),
            },
        };
    }

    pub fn get_real_buffer(&self, frame_count: usize) -> &Subbuffer<[T]> {
        &self.real_buffer[frame_count % self.edit_frequency.to_buffer_amount()]
    }
}

pub enum BufferTypes<T>
where
    T: BufferContents + Copy,
{
    RenderBuffer(RenderBuffer<T>),
    FrequentAccessRenderBuffer(FrequentAccessRenderBuffer<T>),
}

impl<T> BufferTypes<T>
where
    T: BufferContents + Copy,
{
    pub fn get_cloned_buffer(&self, render_storage: &crate::RenderStorage) -> Subbuffer<[T]> {
        // TODO: bad name, consider take_buffer perhaps?
        match self {
            BufferTypes::FrequentAccessRenderBuffer(buffer) => {
                buffer.allocate_and_get_real_buffer(&render_storage.buffer_allocator)
            }
            BufferTypes::RenderBuffer(buffer) => {
                buffer.get_real_buffer(render_storage.frame_count).clone()
            }
        }
    }

    pub fn len(&self, render_storage: &crate::RenderStorage) -> usize {
        match self {
            BufferTypes::FrequentAccessRenderBuffer(buffer) => buffer.buffer.len(),
            BufferTypes::RenderBuffer(buffer) => {
                buffer.real_element_count
                    [render_storage.frame_count % buffer.edit_frequency.to_buffer_amount()]
            }
        }
    }
}

pub enum VertexBuffer {
    UvVertexBuffer(BufferTypes<vertex_data::UvVertex>),
    ColourVertexBuffer(BufferTypes<vertex_data::ColourVertex>),
}

impl VertexBuffer {
    pub fn per_vertex(&self) -> VertexBufferDescription {
        match *self {
            VertexBuffer::UvVertexBuffer(_) => vertex_data::UvVertex::per_vertex(),
            VertexBuffer::ColourVertexBuffer(_) => vertex_data::ColourVertex::per_vertex(),
        }
    }
}

pub enum InstanceBuffer {
    Test(BufferTypes<vertex_data::TestInstance>),
    ForceMultipleTest(BufferTypes<vertex_data::ForceMultipleTestInstance>),
}

pub enum UniformBuffer {
    Test(BufferTypes<crate::vertex_shader_map::Data>),
}

pub struct RenderBuffers {
    pub vertex_buffer: VertexBuffer,
    pub index_buffer: BufferTypes<u32>,
    pub instance_buffer: Option<InstanceBuffer>,
}

pub struct RenderCall {
    pub vertex_shader: VertexShader,
    pub fragment_shader: FragmentShader,
    pub topology: PrimitiveTopology,
    pub depth: bool,
}

pub struct DescriptorSetAndContainedBuffers {
    pub descriptor_set: Arc<PersistentDescriptorSet>,

    pub uniform_buffer: Option<UniformBuffer>,
}

pub struct EntireRenderData {
    // TODO: terrible name
    pub render_buffers: RenderBuffers,
    pub render_call: RenderCall,
    pub descriptor_set_and_contained_buffers: Option<DescriptorSetAndContainedBuffers>,
}
