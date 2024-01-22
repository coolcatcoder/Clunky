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
        rasterization::{CullMode, FrontFace},
        vertex_input::{Vertex, VertexBufferDescription},
    },
    shader::ShaderModule,
    sync::HostAccessError,
    Validated, VulkanError,
};

use crate::buffer_contents;

// Experimental draw call container

pub trait DrawCallContainer {
    // TODO
}

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
    Colour3DInstanced,
}

impl VertexShader {
    pub fn load(&self, device: Arc<Device>) -> Result<Arc<ShaderModule>, Validated<VulkanError>> {
        match *self {
            VertexShader::Colour2D => crate::colour_2d_vertex_shader::load(device),
            VertexShader::Uv2D => crate::uv_2d_vertex_shader::load(device),
            VertexShader::Colour3DInstanced => {
                crate::colour_3d_instanced_vertex_shader::load(device)
            }
        }
    }
}
pub enum FragmentShader {
    Colour2D,
    Uv2D,
    Colour3DInstanced,
}

impl FragmentShader {
    pub fn load(&self, device: Arc<Device>) -> Result<Arc<ShaderModule>, Validated<VulkanError>> {
        match *self {
            FragmentShader::Colour2D => crate::colour_2d_fragment_shader::load(device),
            FragmentShader::Uv2D => crate::uv_2d_fragment_shader::load(device),
            FragmentShader::Colour3DInstanced => {
                crate::colour_3d_instanced_fragment_shader::load(device)
            }
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
N 13. Buffer reuse across multiple entire render datas should be allowed, somehow. Avoid recursion by having a single match statement which panics upon recursion.

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
//TODO bad name
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
    //ReuseBuffer(ReusableBuffers, usize) // This name is confusing. Essentially the usize is an index into render_storage.entire_render_datas, and the ReusableBuffers lets you know what buffer you are trying to reuse.
    // TODO: Consider placing the reusable buffers as a generic that is part of the enum or something.
}

// pub enum ReusableBuffers {
//     UniformBuffer,
//     VertexBuffer,
//     IndexBuffer,
//     InstanceBuffer,
// }

// TODO: asap get this working please. This is probably the most advanced thing I've ever done!
// macro_rules! reusable_buffers_generic_caller {
//     ($reusable_buffers:expr, $render_buffers:expr, $function:expr) => {
//         match $reusable_buffers {
//             crate::menu_rendering::ReusableBuffers::UniformBuffer => {
//                 let Some(shader_accessible_buffers) = $render_buffers.shader_accessible_buffers else {
//                     panic!()
//                 };
//                 let Some(uniform_buffer) = shader_accessible_buffers.uniform_buffer else {
//                     panic!()
//                 };

//                 $function(uniform_buffer)
//             },
//             crate::menu_rendering::ReusableBuffers::VertexBuffer => {
//                 let Some(shader_accessible_buffers) = $render_buffers.shader_accessible_buffers else {
//                     panic!()
//                 };
//                 let Some(uniform_buffer) = shader_accessible_buffers.uniform_buffer else {
//                     panic!()
//                 };

//                 $function(uniform_buffer)
//             },
//             crate::menu_rendering::ReusableBuffers::IndexBuffer => {
//                 let Some(shader_accessible_buffers) = $render_buffers.shader_accessible_buffers else {
//                     panic!()
//                 };
//                 let Some(uniform_buffer) = shader_accessible_buffers.uniform_buffer else {
//                     panic!()
//                 };

//                 $function(uniform_buffer)
//             },
//             crate::menu_rendering::ReusableBuffers::InstanceBuffer => {
//                 let Some(shader_accessible_buffers) = $render_buffers.shader_accessible_buffers else {
//                     panic!()
//                 };
//                 let Some(uniform_buffer) = shader_accessible_buffers.uniform_buffer else {
//                     panic!()
//                 };

//                 $function(uniform_buffer)
//             },
//         }
//     };
// }
// pub(crate) use reusable_buffers_generic_caller;

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

    pub fn length(&self, render_storage: &crate::RenderStorage) -> usize {
        match self {
            BufferTypes::FrequentAccessRenderBuffer(buffer) => buffer.buffer.len(),
            BufferTypes::RenderBuffer(buffer) => {
                buffer.real_element_count
                    [render_storage.frame_count % buffer.edit_frequency.to_buffer_amount()]
            }
        }
    }
}

// This function shouldn't exist right? What the hell?????
pub fn buffer_types_length<T>(
    buffer_type: &BufferTypes<T>,
    render_storage: &crate::RenderStorage,
) -> usize
where
    T: BufferContents + Copy,
{
    match buffer_type {
        BufferTypes::FrequentAccessRenderBuffer(buffer) => buffer.buffer.len(),
        BufferTypes::RenderBuffer(buffer) => {
            buffer.real_element_count
                [render_storage.frame_count % buffer.edit_frequency.to_buffer_amount()]
        }
    }
}

pub enum VertexBuffer {
    Uv(BufferTypes<buffer_contents::UvVertex>),
    Colour(BufferTypes<buffer_contents::ColourVertex>),
    PositionOnly(BufferTypes<buffer_contents::PositionOnlyVertex>),
    Basic3D(BufferTypes<buffer_contents::Basic3DVertex>),
}

impl VertexBuffer {
    pub fn per_vertex(&self) -> VertexBufferDescription {
        match *self {
            VertexBuffer::Uv(_) => buffer_contents::UvVertex::per_vertex(),
            VertexBuffer::Colour(_) => buffer_contents::ColourVertex::per_vertex(),
            VertexBuffer::PositionOnly(_) => buffer_contents::PositionOnlyVertex::per_vertex(),
            VertexBuffer::Basic3D(_) => buffer_contents::Basic3DVertex::per_vertex(),
        }
    }
}

// TODO: horrific name
macro_rules! vertex_buffer_generic_caller {
    ($vertex_buffer:expr, $function:expr) => {
        match $vertex_buffer {
            menu_rendering::VertexBuffer::Uv(buffer) => $function(buffer),
            menu_rendering::VertexBuffer::Colour(buffer) => $function(buffer),
            menu_rendering::VertexBuffer::PositionOnly(buffer) => $function(buffer),
            menu_rendering::VertexBuffer::Basic3D(buffer) => $function(buffer),
        }
    };
}
pub(crate) use vertex_buffer_generic_caller;

pub enum InstanceBuffer {
    Uv(BufferTypes<buffer_contents::UvInstance>),
    Colour(BufferTypes<buffer_contents::ColourInstance>),
    Colour3D(BufferTypes<buffer_contents::Colour3DInstance>),
}

impl InstanceBuffer {
    pub fn per_instance(&self) -> VertexBufferDescription {
        match *self {
            InstanceBuffer::Uv(_) => buffer_contents::UvInstance::per_instance(),
            InstanceBuffer::Colour(_) => buffer_contents::ColourInstance::per_instance(),
            InstanceBuffer::Colour3D(_) => buffer_contents::Colour3DInstance::per_instance(),
        }
    }
}

macro_rules! instance_buffer_generic_caller {
    ($instance_buffer:expr, $function:expr) => {
        match $instance_buffer {
            menu_rendering::InstanceBuffer::Uv(buffer) => $function(buffer),
            menu_rendering::InstanceBuffer::Colour(buffer) => $function(buffer),
            menu_rendering::InstanceBuffer::Colour3D(buffer) => $function(buffer),
        }
    };
}
pub(crate) use instance_buffer_generic_caller;

pub enum UniformBuffer {
    CameraData2D(BufferTypes<crate::colour_2d_vertex_shader::CameraData2D>),
    CameraData3D(BufferTypes<crate::colour_3d_instanced_vertex_shader::CameraData3D>),
}

pub struct RenderBuffers {
    pub vertex_buffer: VertexBuffer,
    pub index_buffer: Option<BufferTypes<u32>>,
    pub instance_buffer: Option<InstanceBuffer>,
    pub shader_accessible_buffers: Option<ShaderAccessibleBuffers>,
}

pub struct Settings {
    pub vertex_shader: VertexShader,
    pub fragment_shader: FragmentShader,
    pub topology: PrimitiveTopology,
    pub depth: bool,
    pub cull_mode: CullMode,
    pub front_face: FrontFace,
}

pub struct ShaderAccessibleBuffers {
    pub uniform_buffer: Option<UniformBuffer>,
    pub image: Option<usize>,
}

pub struct EntireRenderData {
    // TODO: terrible name
    pub render_buffers: RenderBuffers,
    pub settings: Settings,
}
