use std::sync::{mpsc::channel, Arc};

use clunky::{
    buffer_contents::{self, Colour3DInstance},
    math::Matrix4,
    physics::physics_3d::{
        bodies::{Body, CommonBody},
        solver::CpuSolver,
    },
    rendering::draw_instanced,
    shaders::colour_3d_instanced_shaders::Camera,
};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use vulkano::{
    buffer::{allocator::SubbufferAllocator, Subbuffer},
    command_buffer::{
        allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
        RenderPassBeginInfo,
    },
    descriptor_set::{
        allocator::StandardDescriptorSetAllocator, PersistentDescriptorSet, WriteDescriptorSet,
    },
    format::{ClearValue, Format},
    image::{view::ImageView, Image, ImageCreateInfo, ImageType, ImageUsage},
    memory::allocator::AllocationCreateInfo,
    pipeline::{graphics::viewport::Viewport, GraphicsPipeline, Pipeline, PipelineBindPoint},
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass},
    sync::GpuFuture,
};
use vulkano_util::{context::VulkanoContext, renderer::VulkanoWindowRenderer};

/*
pub struct interesting {
    //pub camera: Camera, Don't know how to do yet, perhaps have it be a generic, for example: T::Camera with T impling 3d shader type or something.

    //physics: CpuSolver<f32, CommonBody<f32>>, Need a solver trait perhaps, and then have this be generic.
    //objects_to_render: Vec<gltf::RenderObject>,
    buffers: Buffers,
}
*/

/// This is designed for this game, and this game only. I'm hopeful I'll find a way to make it generic oneday.
pub struct SimpleEngine {
    pub camera: Camera,

    pub physics: CpuSolver<f32, CommonBody<f32>>,
    pub render_objects: Vec<RenderObject>,

    buffers: Buffers,
    allocators: Allocators,
}

impl SimpleEngine {
    pub fn update_and_render(&mut self, window_renderer: &mut VulkanoWindowRenderer) {
        let (cuboid_sender, cuboid_receiver) = channel();

        self.render_objects
            .par_iter()
            .for_each(|render_object| match render_object {
                RenderObject::None => (),
                RenderObject::Cuboid { body_index, colour } => {
                    let body = &self.physics.bodies[*body_index];
                    cuboid_sender
                        .send(Colour3DInstance::new(
                            *colour,
                            Matrix4::from_translation(body.position_unchecked())
                                * Matrix4::from_scale(body.size().unwrap()),
                        ))
                        .unwrap();
                }
                RenderObject::CuboidNoPhysics(instance) => {
                    cuboid_sender.send(*instance).unwrap();
                }
            });

        drop(cuboid_sender);

        self.buffers.cuboid.instance_buffer.extend(cuboid_receiver);
    }

    fn render_to_window(
        &self,
        window_renderer: &mut VulkanoWindowRenderer,
        context: &VulkanoContext,
        render_pass: &Arc<RenderPass>,
        viewport: &Viewport,
        pipelines: &Pipelines,
    ) {
        let future = window_renderer.acquire().unwrap();

        let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
            &self.allocators.command_buffer_allocator,
            context.graphics_queue().queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        // Creating a depth buffer and a frame buffer every frame is very very bad. Not avoidable until next vulkano version.

        let depth_buffer_view = ImageView::new_default(
            Image::new(
                context.memory_allocator().clone(),
                ImageCreateInfo {
                    image_type: ImageType::Dim2d,
                    format: Format::D32_SFLOAT,
                    extent: window_renderer.swapchain_image_view().image().extent(),
                    usage: ImageUsage::TRANSIENT_ATTACHMENT | ImageUsage::DEPTH_STENCIL_ATTACHMENT,
                    ..Default::default()
                },
                AllocationCreateInfo::default(),
            )
            .unwrap(),
        )
        .unwrap();

        let framebuffer = Framebuffer::new(
            render_pass.clone(),
            FramebufferCreateInfo {
                attachments: vec![window_renderer.swapchain_image_view(), depth_buffer_view],
                ..Default::default()
            },
        )
        .unwrap();

        let camera_uniform = self
            .allocators
            .subbuffer_allocator
            .allocate_sized()
            .unwrap();
        *camera_uniform.write().unwrap() = self.camera.to_uniform();

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
            .set_viewport(0, [viewport.clone()].into_iter().collect())
            .unwrap()
            .bind_pipeline_graphics(pipelines.colour_pipeline.clone())
            .unwrap()
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                pipelines.colour_pipeline.layout().clone(),
                0,
                vec![PersistentDescriptorSet::new(
                    &self.allocators.descriptor_set_allocator,
                    pipelines
                        .colour_pipeline
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

        // Yuck. Make this a simple function that takes only a command buffer builder, and a subbuffer_allocator, or perhaps the other way round. Impl is your friend
        draw_instanced(
            &mut command_buffer_builder,
            &self.buffers.cuboid.instance_buffer,
            &self.buffers.cuboid.vertex_buffer,
            &self.buffers.cuboid.index_buffer,
            &self.allocators.subbuffer_allocator,
        );

        command_buffer_builder
            .end_render_pass(Default::default())
            .unwrap();

        let command_buffer = command_buffer_builder.build().unwrap();

        window_renderer.present(
            future
                .then_execute(context.graphics_queue().clone(), command_buffer)
                .unwrap()
                .boxed(),
            false,
        );
    }
}

pub enum RenderObject {
    None,
    Cuboid { body_index: usize, colour: [f32; 4] },
    CuboidNoPhysics(Colour3DInstance),
}

struct Buffers {
    cuboid: ColourBuffers,
    spheroid: ColourBuffers,
}

struct ColourBuffers {
    vertex_buffer: Subbuffer<[buffer_contents::Basic3DVertex]>,
    index_buffer: Subbuffer<[u32]>,
    instance_buffer: Vec<buffer_contents::Colour3DInstance>,
}

struct Allocators {
    command_buffer_allocator: StandardCommandBufferAllocator,
    subbuffer_allocator: SubbufferAllocator,
    descriptor_set_allocator: StandardDescriptorSetAllocator,
}

struct Pipelines {
    colour_pipeline: Arc<GraphicsPipeline>,
}
