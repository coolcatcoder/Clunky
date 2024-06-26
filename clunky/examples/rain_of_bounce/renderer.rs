use std::{collections::HashMap, sync::Arc};

use clunky::{
    math::Matrix4, meshes, physics::physics_3d::bodies::Body as BodyTrait,
    shaders::instanced_simple_lit_colour_3d,
};
use rayon::iter::{IntoParallelRefIterator, ParallelExtend, ParallelIterator};
use vulkano::{
    buffer::{
        allocator::{SubbufferAllocator, SubbufferAllocatorCreateInfo},
        Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer,
    }, command_buffer::{
        allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
        CopyBufferInfo, PrimaryAutoCommandBuffer, PrimaryCommandBufferAbstract,
        RenderPassBeginInfo,
    }, descriptor_set::{allocator::StandardDescriptorSetAllocator, PersistentDescriptorSet, WriteDescriptorSet}, device::Device, format::{ClearValue, Format}, image::{view::ImageView, Image, ImageCreateInfo, ImageType, ImageUsage}, memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator}, pipeline::{graphics::{input_assembly::{InputAssemblyState, PrimitiveTopology}, multisample::MultisampleState, rasterization::{CullMode, FrontFace, RasterizationState}, viewport::{Viewport, ViewportState}, GraphicsPipelineCreateInfo}, DynamicState, GraphicsPipeline, Pipeline, PipelineBindPoint}, render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass}, sync::GpuFuture, DeviceSize
};
use vulkano_util::{
    context::{VulkanoConfig, VulkanoContext},
    renderer::VulkanoWindowRenderer,
    window::VulkanoWindows,
};
use winit::{event_loop::EventLoop, window::WindowId};

use crate::body::Body;

const DEPTH_FORMAT: Format = Format::D32_SFLOAT;
const BACKGROUND_COLOUR: [f32; 4] = [0.0, 0.0, 1.0, 1.0];

const CUBOID_COLOUR_INSTANCES_STARTING_CAPACITY: usize = 50;
const POTENTIAL_CUBOID_COLOUR_INSTANCES_STARTING_CAPACITY: usize = 1000;

pub struct WindowSpecific {
    viewport: Viewport,
}

pub struct Allocators {
    // This could be gotten via VulkanoContext, but it is more intuitive to get it from here.
    pub memory_allocator: Arc<StandardMemoryAllocator>,
    pub command_buffer_allocator: StandardCommandBufferAllocator,
    pub subbuffer_allocator: SubbufferAllocator,
    pub descriptor_set_allocator: StandardDescriptorSetAllocator,
}

impl Allocators {
    fn new(context: &VulkanoContext) -> Self {
        Self {
            memory_allocator: context.memory_allocator().clone(),
            command_buffer_allocator: StandardCommandBufferAllocator::new(
                context.device().clone(),
                Default::default(),
            ),
            //TODO: I'm interested in whether we actually want PREFER_DEVICE | HOST_SEQUENTIAL_WRITE. May we instead want staging buffers? Uncertain. Should profile.
            subbuffer_allocator: SubbufferAllocator::new(
                context.memory_allocator().clone(),
                SubbufferAllocatorCreateInfo {
                    buffer_usage: BufferUsage::UNIFORM_BUFFER | BufferUsage::VERTEX_BUFFER,
                    memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                        | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                },
            ),
            descriptor_set_allocator: StandardDescriptorSetAllocator::new(
                context.device().clone(),
                Default::default(),
            ),
        }
    }
}

pub struct Renderer {
    context: VulkanoContext,

    allocators: Allocators,

    render_pass: Arc<RenderPass>,
    pipelines: Pipelines,

    buffers: Buffers,

    windows_manager: VulkanoWindows,
    // Consider faster hash, like ahash?
    window_specifics: HashMap<WindowId, WindowSpecific>,
}

impl Renderer {
    pub fn new() -> (Self, EventLoop<()>) {
        let event_loop = EventLoop::new();

        let context = VulkanoContext::new(VulkanoConfig::default());

        let mut windows_manager = VulkanoWindows::default();

        let render_pass = vulkano::single_pass_renderpass!(
            context.device().clone(),
            attachments: {
                color: {
                    format: windows_manager.get_primary_renderer().unwrap().swapchain_format(),
                    samples: 1,
                    load_op: Clear,
                    store_op: Store,
                },
                depth: {
                    format: DEPTH_FORMAT,
                    samples: 1,
                    load_op: Clear,
                    store_op: Store,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {depth},
            },
        )
        .unwrap();

        let allocators = Allocators::new(&context);

        let buffers = Buffers::new(&allocators, &context);

        let pipelines = Pipelines::new(context.device(), &render_pass);

        let renderer = Self {
            context,

            allocators,

            render_pass,
            pipelines,

            buffers,

            windows_manager,
            window_specifics: Default::default(),
        };

        (renderer, event_loop)
    }

    pub fn correct_window_size(&mut self, window_id: WindowId, new_viewport_extent: [f32; 2]) {
        let window_specific = self.window_specifics.get_mut(&window_id).unwrap();

        window_specific.viewport.extent = new_viewport_extent;
    }

    pub fn render(&mut self) {
        for (window_id, window_specific) in &mut self.window_specifics {
            let window_renderer = self.windows_manager.get_renderer_mut(*window_id).unwrap();

            let future = window_renderer.acquire().unwrap();

            let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
                &self.allocators.command_buffer_allocator,
                self.context.graphics_queue().queue_family_index(),
                CommandBufferUsage::OneTimeSubmit,
            )
            .unwrap();

            //TODO: Creating a depth buffer and a frame buffer every frame for every window is very very bad. Not avoidable until next vulkano version.

            let depth_buffer_view = ImageView::new_default(
                Image::new(
                    self.context.memory_allocator().clone(),
                    ImageCreateInfo {
                        image_type: ImageType::Dim2d,
                        format: Format::D32_SFLOAT,
                        extent: window_renderer.swapchain_image_view().image().extent(),
                        usage: ImageUsage::TRANSIENT_ATTACHMENT
                            | ImageUsage::DEPTH_STENCIL_ATTACHMENT,
                        ..Default::default()
                    },
                    AllocationCreateInfo::default(),
                )
                .unwrap(),
            )
            .unwrap();

            let framebuffer = Framebuffer::new(
                self.render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![window_renderer.swapchain_image_view(), depth_buffer_view],
                    ..Default::default()
                },
            )
            .unwrap();

            command_buffer_builder
                .begin_render_pass(
                    RenderPassBeginInfo {
                        clear_values: vec![
                            // Sets background colour.
                            Some(ClearValue::Float(BACKGROUND_COLOUR)),
                            Some(ClearValue::Depth(1.0)),
                        ],
                        ..RenderPassBeginInfo::framebuffer(framebuffer)
                    },
                    Default::default(),
                )
                .unwrap()
                .set_viewport(0, [window_specific.viewport.clone()].into_iter().collect())
                .unwrap();
        }
    }
}

struct Buffers {
    cuboid_colour_vertices_and_indices:
        DeviceVerticesAndIndices<instanced_simple_lit_colour_3d::Vertex, u8>,
    cuboid_colour_instances: Vec<instanced_simple_lit_colour_3d::Instance>,
    potential_cuboid_colour_instances: Vec<PotentialCuboidColourInstance>,
    // This is only valid during rendering.
    cuboid_colour_drain_start_index: usize,
}

impl Buffers {
    fn new(allocators: &Allocators, context: &VulkanoContext) -> Self {
        let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
            &allocators.command_buffer_allocator,
            context.graphics_queue().queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        let cuboid_colour_vertices =
            instanced_simple_lit_colour_3d::Vertex::get_array_from_gltf(meshes::CUBE_GLTF, 0);
        let cuboid_colour_indices = meshes::get_indices_from_gltf(meshes::CUBE_GLTF, 0);

        let buffers = Self {
            cuboid_colour_vertices_and_indices: DeviceVerticesAndIndices::new(
                cuboid_colour_vertices,
                cuboid_colour_indices,
                &allocators.memory_allocator,
                &mut command_buffer_builder,
            ),
            cuboid_colour_instances: Vec::with_capacity(CUBOID_COLOUR_INSTANCES_STARTING_CAPACITY),
            potential_cuboid_colour_instances: Vec::with_capacity(
                POTENTIAL_CUBOID_COLOUR_INSTANCES_STARTING_CAPACITY,
            ),
            cuboid_colour_drain_start_index: 0,
        };

        command_buffer_builder
            .build()
            .unwrap()
            .execute(context.graphics_queue().clone())
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap()
            .wait(None)
            .unwrap();

        buffers
    }

    /// Turns all potential instances into real instances.
    fn before_rendering(&mut self, bodies: &[Body]) {
        self.cuboid_colour_drain_start_index = self.cuboid_colour_instances.len();

        self.cuboid_colour_instances.par_extend(
            self.potential_cuboid_colour_instances
                .par_iter()
                .filter_map(|potential_cuboid_colour_instance| {
                    potential_cuboid_colour_instance.to_instance(bodies)
                }),
        );
    }

    /// Removes all the realised potential instances from the instances.
    fn after_rendering(&mut self) {
        self.cuboid_colour_instances
            .drain(self.cuboid_colour_drain_start_index..);
        debug_assert!(self.cuboid_colour_drain_start_index == self.cuboid_colour_instances.len());
    }
}

#[derive(Clone)]
pub enum PotentialCuboidColourInstance {
    None,
    // This is so expensive, due to the large instance size, that if I need this, I should probably just have another enum with only this.
    //Instance(instanced_simple_lit_colour_3d::Instance),
    PhysicsWithColour { body_index: usize, colour: [f32; 4] },
}

impl PotentialCuboidColourInstance {
    #[inline]
    fn to_instance(&self, bodies: &[Body]) -> Option<instanced_simple_lit_colour_3d::Instance> {
        match self {
            Self::PhysicsWithColour { body_index, colour } => {
                let body = &bodies[*body_index];
                Some(instanced_simple_lit_colour_3d::Instance::new(
                    *colour,
                    Matrix4::from_translation(body.position_unchecked()),
                ))
            }
            Self::None => None,
        }
    }
}

/// Not accessible to the host. These will be on the device only.
///
/// Because it copies from a staging buffer to a device local buffer, you will need to use a command buffer before first use.
struct DeviceVerticesAndIndices<V, I> {
    vertices: Subbuffer<[V]>,
    indices: Subbuffer<[I]>,
}

impl<V, I> DeviceVerticesAndIndices<V, I> {
    fn new<VI, II>(
        vertices: VI,
        indices: II,
        memory_allocator: &Arc<StandardMemoryAllocator>,
        command_buffer_builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
    ) -> Self
    where
        V: BufferContents,
        I: BufferContents,
        VI: IntoIterator<Item = V>,
        VI::IntoIter: ExactSizeIterator,
        II: IntoIterator<Item = I>,
        II::IntoIter: ExactSizeIterator,
    {
        // Go to Vulkano's Buffer documentation. I basically copied the staging example.

        // VERTICES
        let vertices_iter = vertices.into_iter();
        let vertices_len = vertices_iter.len();

        let vertices_staging = Buffer::from_iter(
            memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::TRANSFER_SRC,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            vertices_iter,
        )
        .unwrap();

        let vertices_device = Buffer::new_slice::<V>(
            memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER | BufferUsage::TRANSFER_DST,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
                ..Default::default()
            },
            vertices_len as DeviceSize,
        )
        .unwrap();

        command_buffer_builder
            .copy_buffer(CopyBufferInfo::buffers(
                vertices_staging,
                vertices_device.clone(),
            ))
            .unwrap();

        // INDICES
        let indices_iter = indices.into_iter();
        let indices_len = indices_iter.len();

        let indices_staging = Buffer::from_iter(
            memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::TRANSFER_SRC,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            indices_iter,
        )
        .unwrap();

        let indices_device = Buffer::new_slice::<I>(
            memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::INDEX_BUFFER | BufferUsage::TRANSFER_DST,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
                ..Default::default()
            },
            indices_len as DeviceSize,
        )
        .unwrap();

        command_buffer_builder
            .copy_buffer(CopyBufferInfo::buffers(
                indices_staging,
                indices_device.clone(),
            ))
            .unwrap();

        Self {
            indices: indices_device,
            vertices: vertices_device,
        }
    }
}

struct Pipelines {
    instanced_simple_lit_colour_3d: Arc<GraphicsPipeline>,
}

impl Pipelines {
    fn new(device: &Arc<Device>, render_pass: &Arc<RenderPass>) -> Self {
        let subpass = Subpass::from(render_pass.clone(), 0).unwrap();

        let instanced_simple_lit_colour_3d = GraphicsPipeline::new(
            device.clone(),
            None,
            GraphicsPipelineCreateInfo {
                viewport_state: Some(ViewportState::default()),
                rasterization_state: Some(RasterizationState {
                    cull_mode: CullMode::Back,
                    front_face: FrontFace::CounterClockwise,
                    ..Default::default()
                }),
                input_assembly_state: Some(InputAssemblyState {
                    topology: PrimitiveTopology::TriangleList,
                    ..Default::default()
                }),
                multisample_state: Some(MultisampleState::default()),
                dynamic_state: [DynamicState::Viewport].into_iter().collect(),
                ..instanced_simple_lit_colour_3d::graphics_pipeline_create_info(
                    device.clone(),
                    subpass.clone(),
                )
            },
        )
        .unwrap();

        Self {
            instanced_simple_lit_colour_3d,
        }
    }

    fn bind_instanced_simple_lit_colour_3d(&self, command_buffer_builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, allocators: &Allocators, camera_uniform: Subbuffer<instanced_simple_lit_colour_3d::CameraUniform>) {
        command_buffer_builder
                .bind_pipeline_graphics(
                    self
                        .instanced_simple_lit_colour_3d
                        .clone(),
                )
                .unwrap()
                .bind_descriptor_sets(
                    PipelineBindPoint::Graphics,
                    self
                        .instanced_simple_lit_colour_3d
                        .layout()
                        .clone(),
                    0,
                    //TODO: Shouldn't this be persistent, as the name implies? Why is this being created every frame?
                    PersistentDescriptorSet::new(
                        &allocators.descriptor_set_allocator,
                        self
                            .instanced_simple_lit_colour_3d
                            .layout()
                            .set_layouts()
                            [0]
                            .clone(),
                        [WriteDescriptorSet::buffer(0, camera_uniform)],
                        [],
                    )
                    .unwrap(),
                )
                .unwrap();
    }
}
