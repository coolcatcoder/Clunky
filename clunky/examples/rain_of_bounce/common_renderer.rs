use std::{collections::HashMap, sync::Arc};

use clunky::{
    math::{mul_3d_by_1d, Matrix4},
    meshes,
    physics::{
        physics_3d::{aabb::AabbCentredOrigin, bodies::ImmovableCuboid, solver::CpuSolver},
        PhysicsSimulation,
    },
    shaders::instanced_simple_lit_colour_3d::{self, Camera},
};
use vulkano::{
    buffer::{allocator::SubbufferAllocator, Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    command_buffer::{
        AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer, RenderPassBeginInfo,
    },
    descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet},
    device::Device,
    format::{ClearValue, Format},
    image::{view::ImageView, Image, ImageCreateInfo, ImageType, ImageUsage},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter},
    pipeline::{
        graphics::{
            input_assembly::{InputAssemblyState, PrimitiveTopology},
            multisample::MultisampleState,
            rasterization::{CullMode, FrontFace, RasterizationState},
            viewport::{Viewport, ViewportState},
            GraphicsPipelineCreateInfo,
        },
        DynamicState, GraphicsPipeline, Pipeline, PipelineBindPoint,
    },
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass},
    swapchain::SwapchainCreateInfo,
    sync::GpuFuture,
};
use vulkano_util::{context::VulkanoContext, window::WindowDescriptor};
use winit::{
    event_loop::{EventLoop, EventLoopWindowTarget},
    window::WindowId,
};

use crate::{body::Body, engine::{Renderer, SimpleEngine}};

#[derive(Default)]
pub struct Config {
    pub starting_windows: Vec<WindowConfig>,
}

/// Config for creating a new window.
#[derive(Clone)]
pub struct WindowConfig {
    pub camera: Camera,

    pub window_descriptor: WindowDescriptor,
    pub swapchain_create_info_modify: fn(&mut SwapchainCreateInfo),
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            camera: Default::default(),
            window_descriptor: Default::default(),
            swapchain_create_info_modify: |_| {},
        }
    }
}

pub struct WindowSpecific {
    pub camera: Camera,
    viewport: Viewport,
}

pub struct CommonRenderer {
    engine: &'static mut SimpleEngine<Self>
}

impl Renderer for CommonRenderer {
    type Physics = CpuSolver<f32, Body>;
    type Storage = Storage;

    fn new(engine: &'static mut SimpleEngine<Self>) -> Self {
        Self {
            engine,
        }
    }

    fn correct_window_size(
        self,
        window_id: WindowId,
        new_window_size: [f32; 2],
        new_aspect_ratio: f32,
    ) {
        let window_specific = self.engine.renderer_storage.window_specific.get_mut(&window_id).unwrap();
        window_specific.viewport.extent = new_window_size;
        window_specific.camera.aspect_ratio = new_aspect_ratio;
    }

    fn render(self) {
        for (id, window_specific) in &mut self.engine.renderer_storage.window_specific {
            let window_renderer = self.engine
                .windows_manager
                .get_renderer_mut(*id)
                .unwrap();

            let future = window_renderer.acquire().unwrap();

            let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
                &self.engine.allocators.command_buffer_allocator,
                self.engine
                    .context
                    .graphics_queue()
                    .queue_family_index(),
                CommandBufferUsage::OneTimeSubmit,
            )
            .unwrap();

            //TODO: Creating a depth buffer and a frame buffer every frame is very very bad. Not avoidable until next vulkano version.

            let depth_buffer_view = ImageView::new_default(
                Image::new(
                    self.engine.context.memory_allocator().clone(),
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
                self.engine.renderer_storage.render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![window_renderer.swapchain_image_view(), depth_buffer_view],
                    ..Default::default()
                },
            )
            .unwrap();

            let camera_uniform = self.engine
                .allocators
                .subbuffer_allocator
                .allocate_sized()
                .unwrap();
            *camera_uniform.write().unwrap() = window_specific.camera.to_uniform();

            command_buffer_builder
                .begin_render_pass(
                    RenderPassBeginInfo {
                        clear_values: vec![
                            // Sets background colour.
                            Some([0.0, 0.0, 1.0, 1.0].into()),
                            Some(ClearValue::Depth(1.0)),
                        ],
                        ..RenderPassBeginInfo::framebuffer(framebuffer)
                    },
                    Default::default(),
                )
                .unwrap()
                .set_viewport(0, [window_specific.viewport.clone()].into_iter().collect())
                .unwrap();

            // COLOUR_3D_INSTANCED
            command_buffer_builder
                .bind_pipeline_graphics(self.engine.renderer_storage.pipelines.colour_3d_instanced.clone())
                .unwrap()
                .bind_descriptor_sets(
                    PipelineBindPoint::Graphics,
                    self.engine.renderer_storage.pipelines.colour_3d_instanced.layout().clone(),
                    0,
                    vec![PersistentDescriptorSet::new(
                        &self.engine.allocators.descriptor_set_allocator,
                        self.engine.renderer_storage.pipelines
                            .colour_3d_instanced
                            .layout()
                            .set_layouts()
                            .get(0)
                            .unwrap()
                            .clone(),
                        [WriteDescriptorSet::buffer(0, camera_uniform)],
                        [],
                    )
                    .unwrap()],
                )
                .unwrap();

            let mut easy_draw = EasyDraw {
                command_buffer_builder: &mut command_buffer_builder,
                subbuffer_allocator: &self.engine.allocators.subbuffer_allocator,
            };
            self.engine.renderer_storage.buffers.cuboid_colour.draw(&mut easy_draw);

            command_buffer_builder
                .end_render_pass(Default::default())
                .unwrap();

            let command_buffer = command_buffer_builder.build().unwrap();

            window_renderer.present(
                future
                    .then_execute(
                        self.engine.context.graphics_queue().clone(),
                        command_buffer,
                    )
                    .unwrap()
                    .boxed(),
                false,
            )
        }
    }
}

/// Contains a few basic draw calls. Useful for prototypes.
/// Mainly designed for copying and editing, to fit your needs.
pub struct Storage {
    window_specific: HashMap<WindowId, WindowSpecific>,

    render_pass: Arc<RenderPass>,

    pipelines: Pipelines,
    buffers: Buffers,
}

impl CommonRenderer {
    pub fn init(
        self,
        config: Config,
    ) {
        let event_loop = self.engine.temporary_event_loop_storage.as_ref().unwrap();
        let mut window_specific = HashMap::new();

        for window_config in config.starting_windows {
            let id = self.engine.windows_manager.create_window(
                event_loop,
                &self.engine.context,
                &window_config.window_descriptor,
                window_config.swapchain_create_info_modify,
            );

            window_specific.insert(
                id,
                WindowSpecific {
                    camera: window_config.camera,
                    viewport: Viewport {
                        offset: [0.0, 0.0],
                        extent: [1.0, 1.0],
                        depth_range: 0.0..=1.0,
                    },
                },
            );
        }

        let render_pass = vulkano::single_pass_renderpass!(
            self.engine.context.device().clone(),
            attachments: {
                color: {
                    format: self.engine.windows_manager.get_primary_renderer().unwrap().swapchain_format(),
                    samples: 1,
                    load_op: Clear,
                    store_op: Store,
                },
                depth: {
                    format: Format::D32_SFLOAT,
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

        let pipelines = Pipelines::new(self.engine.context.device(), &render_pass);

        self.engine.renderer_storage = Storage {
            window_specific,

            render_pass,

            pipelines,
            buffers: Buffers::new(self.engine),
        };
    }

    /// Creates a new window!
    pub fn create_window(
        &mut self,
        config: WindowConfig,
        event_loop: &EventLoopWindowTarget<()>,
    ) -> WindowId {
        let id = self.engine.windows_manager.create_window(
            event_loop,
            &self.engine.context,
            &config.window_descriptor,
            config.swapchain_create_info_modify,
        );

        self.engine.renderer_storage.window_specific.insert(
            id,
            WindowSpecific {
                camera: config.camera,
                viewport: Viewport {
                    offset: [0.0, 0.0],
                    extent: [1.0, 1.0],
                    depth_range: 0.0..=1.0,
                },
            },
        );

        id
    }

    // Removes a window!
    pub fn remove_window(
        self,
        id: WindowId,
    ) {
        self.engine.renderer_storage.window_specific.remove(&id);
        self.engine.windows_manager.remove_renderer(id);
    }

    pub fn get_window_specific(&mut self, window_id: WindowId) -> Option<&mut WindowSpecific> {
        self.engine.renderer_storage.window_specific.get_mut(&window_id)
    }

    pub fn add_cuboid_colour(&mut self, instance: instanced_simple_lit_colour_3d::Instance) {
        self.engine.renderer_storage.buffers.cuboid_colour.instances.push(instance);
    }

    pub fn add_cuboid_colour_from_aabb(&mut self, aabb: AabbCentredOrigin<f32>, colour: [f32; 4]) {
        self.engine.renderer_storage.buffers
            .cuboid_colour
            .instances
            .push(instanced_simple_lit_colour_3d::Instance::new(
                colour,
                Matrix4::from_translation(aabb.position)
                    * Matrix4::from_scale(mul_3d_by_1d(aabb.half_size, 2.0)),
            ));
    }
}

struct Pipelines {
    colour_3d_instanced: Arc<GraphicsPipeline>,
}

impl Pipelines {
    fn new(device: &Arc<Device>, render_pass: &Arc<RenderPass>) -> Self {
        let subpass = Subpass::from(render_pass.clone(), 0).unwrap();

        let colour_3d_instanced = GraphicsPipeline::new(
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
            colour_3d_instanced,
        }
    }
}

struct Buffers {
    cuboid_colour: Colour3DInstancedBuffers,
    //custom_colour_instanced: Vec<&'static Colour3DInstancedBuffers>,
}

impl Buffers {
    fn new(engine: &mut SimpleEngine<CommonRenderer>) -> Self {
        Buffers {
            cuboid_colour: Colour3DInstancedBuffers::new(meshes::CUBE_GLTF, &mut engine.context),
            //custom_colour_instanced: vec![],
        }
    }
}

struct Colour3DInstancedBuffers {
    vertices: Subbuffer<[instanced_simple_lit_colour_3d::Vertex]>,
    instances: Vec<instanced_simple_lit_colour_3d::Instance>,
    indices: Subbuffer<[u32]>,
}

impl Colour3DInstancedBuffers {
    fn new(gltf: &[u8], context: &mut VulkanoContext) -> Self {
        Self {
            // TODO: use a staging buffer, and keep vertices and indices in device memory only, with no host sequential write?
            vertices: Buffer::from_iter(
                context.memory_allocator().clone(),
                BufferCreateInfo {
                    usage: BufferUsage::VERTEX_BUFFER,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                        | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                },
                instanced_simple_lit_colour_3d::Vertex::get_array_from_gltf(gltf, 0),
            )
            .unwrap(),
            instances: vec![],
            indices: Buffer::from_iter(
                context.memory_allocator().clone(),
                BufferCreateInfo {
                    usage: BufferUsage::INDEX_BUFFER,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                        | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                },
                meshes::get_indices_from_gltf(gltf, 0),
            )
            .unwrap(),
        }
    }
}

impl CanEasyDraw for Colour3DInstancedBuffers {
    fn draw(&self, easy_draw: &mut EasyDraw) {
        if self.instances.len() == 0 {
            return;
        }

        let instance_buffer = easy_draw
            .subbuffer_allocator
            .allocate_slice(self.instances.len() as u64)
            .unwrap();
        instance_buffer
            .write()
            .unwrap()
            .copy_from_slice(&self.instances);

        easy_draw
            .command_buffer_builder
            .bind_vertex_buffers(0, (self.vertices.clone(), instance_buffer))
            .unwrap()
            .bind_index_buffer(self.indices.clone())
            .unwrap()
            .draw_indexed(
                self.indices.len() as u32,
                self.instances.len() as u32,
                0,
                0,
                0,
            )
            .unwrap();
    }
}

trait CanEasyDraw {
    fn draw(&self, easy_draw: &mut EasyDraw);
}

struct EasyDraw<'a> {
    command_buffer_builder: &'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
    subbuffer_allocator: &'a SubbufferAllocator,
}
