use vulkano::{
    command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage},
    format::Format,
    image::{view::ImageView, Image, ImageCreateInfo, ImageType, ImageUsage},
    memory::allocator::AllocationCreateInfo,
    render_pass::{Framebuffer, FramebufferCreateInfo},
};

use crate::engine::{AccessibleToRenderer, Renderer};

/// Contains a few basic draw calls. Useful for prototypes.
/// Mainly designed for copying and editing, to fit your needs.
pub struct CommonRenderer {}

impl Renderer for CommonRenderer {
    fn new(accessible_while_drawing: &mut AccessibleToRenderer) -> Self {
        Self {}
    }

    fn render(&mut self, accessible_while_drawing: &mut AccessibleToRenderer) {
        let window_renderer = accessible_while_drawing
            .windows_manager
            .get_primary_renderer_mut()
            .unwrap();

        let future = window_renderer.acquire().unwrap();

        let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
            &accessible_while_drawing.allocators.command_buffer_allocator,
            accessible_while_drawing
                .context
                .graphics_queue()
                .queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        //TODO: Creating a depth buffer and a frame buffer every frame is very very bad. Not avoidable until next vulkano version.

        let depth_buffer_view = ImageView::new_default(
            Image::new(
                accessible_while_drawing.context.memory_allocator().clone(),
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
            accessible_while_drawing.render_pass.clone(),
            FramebufferCreateInfo {
                attachments: vec![window_renderer.swapchain_image_view(), depth_buffer_view],
                ..Default::default()
            },
        )
        .unwrap();
    }
}
