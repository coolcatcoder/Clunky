use std::{collections::HashMap, sync::Arc};

use clunky::shaders::colour_3d_instanced_shaders::Camera;
use vulkano::{
    command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage}, format::Format, image::{view::ImageView, Image, ImageCreateInfo, ImageType, ImageUsage}, memory::allocator::AllocationCreateInfo, pipeline::graphics::viewport::Viewport, render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass}, swapchain::SwapchainCreateInfo
};
use vulkano_util::window::WindowDescriptor;
use winit::{event_loop::EventLoop, window::WindowId};

use crate::engine::{AccessibleToRenderer, Renderer};

#[derive(Default)]
pub struct Config {
    pub starting_windows: Vec<WindowConfig>,
}

// Consider manually impl-ing Default
#[derive(Clone)]
pub struct WindowConfig {
    pub camera: Camera,

    pub window_descriptor: WindowDescriptor,
    pub swapchain_create_info_modify: fn(&mut SwapchainCreateInfo),
}

pub struct WindowSpecific {
    pub camera: Camera,
    viewport: Viewport,
}

/// Contains a few basic draw calls. Useful for prototypes.
/// Mainly designed for copying and editing, to fit your needs.
pub struct CommonRenderer {
    window_specific: HashMap<WindowId, WindowSpecific>,
    
    render_pass: Arc<RenderPass>,
}

impl CommonRenderer {
    pub fn new(config: Config, accessible_to_renderer: &mut AccessibleToRenderer, event_loop: &EventLoop<()>,) -> Self {
        let render_pass = vulkano::single_pass_renderpass!(
            accessible_to_renderer.context.device().clone(),
            attachments: {
                color: {
                    format: accessible_to_renderer.windows_manager.get_primary_renderer().unwrap().swapchain_format(),
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

        let mut window_specific = HashMap::new();

        for window_config in config.starting_windows {
            let id = accessible_to_renderer.windows_manager.create_window(event_loop, &accessible_to_renderer.context, &window_config.window_descriptor, window_config.swapchain_create_info_modify);

            window_specific.insert(id, WindowSpecific {
                camera: window_config.camera,
                viewport: Viewport {
                    offset: [0.0, 0.0],
                    extent: [0.0, 0.0],
                    depth_range: 0.0..=1.0,
                },
            });
        }

        Self {
            window_specific,

            render_pass,
        }
    }
}

impl Renderer for CommonRenderer {
    fn render(&mut self, accessible_to_renderer: &mut AccessibleToRenderer) {
        let window_renderer = accessible_to_renderer
            .windows_manager
            .get_primary_renderer_mut()
            .unwrap();

        let _future = window_renderer.acquire().unwrap();

        let _command_buffer_builder = AutoCommandBufferBuilder::primary(
            &accessible_to_renderer.allocators.command_buffer_allocator,
            accessible_to_renderer
                .context
                .graphics_queue()
                .queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        //TODO: Creating a depth buffer and a frame buffer every frame is very very bad. Not avoidable until next vulkano version.

        let depth_buffer_view = ImageView::new_default(
            Image::new(
                accessible_to_renderer.context.memory_allocator().clone(),
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

        let _framebuffer = Framebuffer::new(
            self.render_pass.clone(),
            FramebufferCreateInfo {
                attachments: vec![window_renderer.swapchain_image_view(), depth_buffer_view],
                ..Default::default()
            },
        )
        .unwrap();
    }
}
