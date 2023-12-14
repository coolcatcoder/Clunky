use std::sync::Arc;

use vulkano::{
    buffer::{
        allocator::SubbufferAllocator, Buffer, BufferContents, BufferCreateInfo, BufferUsage,
        Subbuffer,
    },
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

use crate::buffer_contents;

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
    Colour2D,
    Uv2D,
}

impl VertexShader {
    pub fn load(&self, device: Arc<Device>) -> Result<Arc<ShaderModule>, Validated<VulkanError>> {
        match *self {
            VertexShader::Colour2D => crate::colour_2d_vertex_shader::load(device),
            VertexShader::Uv2D => crate::uv_2d_vertex_shader::load(device),
        }
    }
}
pub enum FragmentShader {
    Colour2D,
    Uv2D,
}

impl FragmentShader {
    pub fn load(&self, device: Arc<Device>) -> Result<Arc<ShaderModule>, Validated<VulkanError>> {
        match *self {
            FragmentShader::Colour2D => crate::colour_2d_fragment_shader::load(device),
            FragmentShader::Uv2D => crate::uv_2d_fragment_shader::load(device),

        }
    }
}

/* Goals are ill defined. Better goals:
A 1. Main should not have to be altered in order to add or remove draw calls. Only menus should have to be altered.
A 2. Menus should be able to specify draw calls.
A 3. Menus should be able to specify buffers.
A 4. Menus should be able to specify vertex buffer types.
A 5. Menus should be able to specify shaders.
A 6. Changing menu state should not change any other states, unless the menu requests it in its start function, or elsewhere.
A 7. Menu should have some easy way of accessing the buffers it has requested. Perhaps have a generic that requires a tuple of vertex types?
A 8. Some way to specify if a buffer should be created using a subbuffer allocator.
A 9. Images. Images need to work with all the above goals. How? Images require a pipeline. This could be tricky.
A 10. descriptor sets are nightmares. Consider having 1 optional descriptor set as part of the buffers. Perhaps containing the uniform buffers.
N 11. Have some easy way to debug everything. Perhaps consider having a debug struct full of bools, that print during different conditions, such as when a buffer updates, and stuff like that.
N 12. Having an easy way for menus to assume the buffers are of a type would be nice. Create a function called assume_render_buffer_is_of_type or perhaps a macro?

Goal state key:
N = Not started implementation.
S = Started work on implementation.
EB = Experimental implementation that is broken.
EW = Experimental implementation that is working.
A = Goal achieved.
*/

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

        for _ in 0..real_length {
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

                println!("{}", self.element_count);
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
    Uv(BufferTypes<buffer_contents::UvVertex>),
    Colour(BufferTypes<buffer_contents::ColourVertex>),
    PositionOnly(BufferTypes<buffer_contents::PositionOnlyVertex>),
}

impl VertexBuffer {
    pub fn per_vertex(&self) -> VertexBufferDescription {
        match *self {
            VertexBuffer::Uv(_) => buffer_contents::UvVertex::per_vertex(),
            VertexBuffer::Colour(_) => buffer_contents::ColourVertex::per_vertex(),
            VertexBuffer::PositionOnly(_) => buffer_contents::PositionOnlyVertex::per_vertex(),
        }
    }
}

pub enum InstanceBuffer {
    Uv(BufferTypes<buffer_contents::UvInstance>),
    Colour(BufferTypes<buffer_contents::ColourInstance>),
}

pub enum UniformBuffer {
    CameraData2D(BufferTypes<crate::colour_2d_vertex_shader::CameraData2D>),
}

pub struct RenderBuffers {
    pub vertex_buffer: VertexBuffer,
    pub index_buffer: Option<BufferTypes<u32>>,
    pub instance_buffer: Option<InstanceBuffer>,
    pub shader_accessible_buffers: Option<ShaderAccessibleBuffers>,
}

pub struct RenderCall {
    pub vertex_shader: VertexShader,
    pub fragment_shader: FragmentShader,
    pub topology: PrimitiveTopology,
    pub depth: bool,
}

pub struct ShaderAccessibleBuffers {
    pub uniform_buffer: Option<UniformBuffer>,
    pub image: Option<usize>,
}

pub struct EntireRenderData {
    // TODO: terrible name
    pub render_buffers: RenderBuffers,
    pub render_call: RenderCall,
}
