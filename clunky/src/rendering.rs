use std::sync::Arc;

use vulkano::{
    buffer::{
        allocator::SubbufferAllocator, Buffer, BufferContents, BufferCreateInfo, BufferUsage,
        Subbuffer,
    },
    command_buffer::{
        allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
        CopyBufferToImageInfo, PrimaryAutoCommandBuffer,
    },
    device::{
        physical::PhysicalDeviceType, Device, DeviceCreateInfo, DeviceExtensions, Queue,
        QueueCreateInfo, QueueFlags,
    },
    format::Format,
    image::{view::ImageView, Image, ImageCreateInfo, ImageType, ImageUsage},
    instance::{Instance, InstanceCreateFlags, InstanceCreateInfo},
    memory::allocator::{
        AllocationCreateInfo, FreeListAllocator, GenericMemoryAllocator, MemoryTypeFilter,
        StandardMemoryAllocator,
    },
    pipeline::GraphicsPipeline,
    swapchain::{Surface, Swapchain, SwapchainCreateInfo},
    DeviceSize, VulkanLibrary,
};
use winit::{
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

/// Creates an instance and an event loop.
#[must_use]
pub fn create_instance_and_event_loop() -> (Arc<Instance>, EventLoop<()>) {
    let library = VulkanLibrary::new().unwrap();
    let event_loop = EventLoop::new();
    let required_extensions = Surface::required_extensions(&event_loop);

    let instance = Instance::new(
        library,
        InstanceCreateInfo {
            enabled_extensions: required_extensions,
            flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
            ..InstanceCreateInfo::application_from_cargo_toml()
        },
    )
    .unwrap();

    (instance, event_loop)
}

/// Creates a window, and a surface from that window.
#[must_use]
pub fn create_window_and_surface(
    instance: &Arc<Instance>,
    event_loop: &EventLoop<()>,
) -> (Arc<Window>, Arc<Surface>) {
    let window = Arc::new(WindowBuilder::new().build(event_loop).unwrap());
    let surface = Surface::from_window(instance.clone(), window.clone()).unwrap();

    (window, surface)
}

/// Picks the most capable physical device that supports all the queue flags and device extensions, in this order:
/// discrete gpu, integrated gpu, virtual gpu, cpu, other.
///
/// The queue flags are basically what you want the device to be able to do. For example if you wanted the ability to render frames, you would use QueueFlags::GRAPHICS.
///
/// The device extensions should be the extensions you want the device to support. For example if you wanted to render frames, you would set khr_swapchain to true.
#[must_use]
pub fn create_device_and_queue(
    device_extensions: DeviceExtensions,
    queue_flags: QueueFlags,
    surface: &Arc<Surface>,
    instance: &Arc<Instance>,
) -> (Arc<Device>, Arc<Queue>) {
    let (physical_device, queue_family_index) = instance
        .enumerate_physical_devices()
        .unwrap()
        .filter(|p| p.supported_extensions().contains(&device_extensions))
        .filter_map(|p| {
            p.queue_family_properties()
                .iter()
                .enumerate()
                .position(|(i, q)| {
                    q.queue_flags.intersects(queue_flags)
                        && p.surface_support(i as u32, surface).unwrap_or(false)
                })
                .map(|i| (p, i as u32))
        })
        .min_by_key(|(p, _)| match p.properties().device_type {
            PhysicalDeviceType::DiscreteGpu => 0,
            PhysicalDeviceType::IntegratedGpu => 1,
            PhysicalDeviceType::VirtualGpu => 2,
            PhysicalDeviceType::Cpu => 3,
            PhysicalDeviceType::Other => 4,
            _ => 5,
        })
        .unwrap();

    let (device, mut queues) = Device::new(
        physical_device,
        DeviceCreateInfo {
            enabled_extensions: device_extensions,
            queue_create_infos: vec![QueueCreateInfo {
                queue_family_index,
                ..Default::default()
            }],
            ..Default::default()
        },
    )
    .unwrap();

    let queue = queues.next().unwrap();

    (device, queue)
}

/// Creates a swapchain and its images.
#[must_use]
pub fn create_swapchain(
    device: &Arc<Device>,
    surface: &Arc<Surface>,
    window: &Arc<Window>,
) -> (Arc<Swapchain>, Vec<Arc<Image>>) {
    let surface_capabilities = device
        .physical_device()
        .surface_capabilities(&surface, Default::default())
        .unwrap();
    let image_format = device
        .physical_device()
        .surface_formats(&surface, Default::default())
        .unwrap()[0]
        .0;

    Swapchain::new(
        device.clone(),
        surface.clone(),
        SwapchainCreateInfo {
            min_image_count: surface_capabilities.min_image_count,
            image_format,
            image_extent: window.inner_size().into(),
            image_usage: ImageUsage::COLOR_ATTACHMENT,
            composite_alpha: surface_capabilities
                .supported_composite_alpha
                .into_iter()
                .next()
                .unwrap(),
            ..Default::default()
        },
    )
    .unwrap()
}

/// Does most of the initiation needed for general rendering.
///
/// Returns (event_loop, window, surface, device, queue, swapchain, swapchain_images).
///
/// This function needs a better name.
///
/// Picks the most capable physical device that supports all the queue flags and device extensions, in this order:
/// discrete gpu, integrated gpu, virtual gpu, cpu, other.
///
/// The queue flags are basically what you want the device to be able to do. For example if you wanted the ability to render frames, you would use QueueFlags::GRAPHICS.
///
/// The device extensions should be the extensions you want the device to support. For example if you wanted to render frames, you would set khr_swapchain to true.
#[must_use]
pub fn initiate_general(
    queue_flags: QueueFlags,
    device_extensions: DeviceExtensions,
) -> (
    EventLoop<()>,
    Arc<Window>,
    Arc<Surface>,
    Arc<Device>,
    Arc<Queue>,
    Arc<Swapchain>,
    Vec<Arc<Image>>,
) {
    let (instance, event_loop) = create_instance_and_event_loop();

    let (window, surface) = create_window_and_surface(&instance, &event_loop);

    let (device, queue) =
        create_device_and_queue(device_extensions, queue_flags, &surface, &instance);

    let (swapchain, swapchain_images) = create_swapchain(&device, &surface, &window);

    (
        event_loop,
        window,
        surface,
        device,
        queue,
        swapchain,
        swapchain_images,
    )
}

/// Does most of the initiation needed for making a basic game.
///
/// Compared to [initiate_general] this has less customization, and is designed to make initialisation for games extra easy. For specific effects and optimizations you may still wish to use [initiate_general] or other initiation functions.
///
/// Returns (event_loop, window, surface, device, queue, swapchain, swapchain_images, memory_allocator, command_buffer_allocator).
///
/// This function needs a better name.
///
/// Picks the most capable physical device that supports all the queue flags and device extensions, in this order:
/// discrete gpu, integrated gpu, virtual gpu, cpu, other.
///
/// The queue flags are basically what you want the device to be able to do. For example if you wanted the ability to render frames, you would use QueueFlags::GRAPHICS.
///
/// The device extensions should be the extensions you want the device to support. For example if you wanted to render frames, you would set khr_swapchain to true.
#[must_use]
pub fn initate_game() {
    todo!()
}

/// Loads image bytes for later use in shaders.
///
/// Don't forget to execute the command_buffer_builder!
///
/// To convert an image into bytes you can use include_bytes!().
#[must_use]
pub fn load_images<
    'a,
    T: IntoIterator<IntoIter = impl ExactSizeIterator<Item = ImageBytes<'a>>>,
>(
    list_of_image_bytes: T,
    memory_allocator: &Arc<GenericMemoryAllocator<FreeListAllocator>>,
    command_buffer_builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
) -> Vec<Arc<ImageView>> {
    let image_bytes_iterator = list_of_image_bytes.into_iter();
    let mut image_views = Vec::with_capacity(image_bytes_iterator.len());

    for image_bytes in image_bytes_iterator {
        let (upload_buffer, extent) = match image_bytes {
            ImageBytes::Png(png_bytes) => {
                let decoder = png::Decoder::new(png_bytes);
                let mut reader = decoder.read_info().unwrap();
                let info = reader.info();
                let extent = [info.width, info.height, 1];

                let upload_buffer = Buffer::new_slice(
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
                    (info.width * info.height * 4) as DeviceSize,
                )
                .unwrap();

                reader
                    .next_frame(&mut upload_buffer.write().unwrap())
                    .unwrap();

                (upload_buffer, extent)
            }
        };

        let image = Image::new(
            memory_allocator.clone(),
            ImageCreateInfo {
                image_type: ImageType::Dim2d,
                format: Format::R8G8B8A8_SRGB,
                extent,
                usage: ImageUsage::TRANSFER_DST | ImageUsage::SAMPLED,
                ..Default::default()
            },
            AllocationCreateInfo::default(),
        )
        .unwrap();

        command_buffer_builder
            .copy_buffer_to_image(CopyBufferToImageInfo::buffer_image(
                upload_buffer,
                image.clone(),
            ))
            .unwrap();

        image_views.push(ImageView::new_default(image).unwrap());
    }

    image_views
}

pub enum ImageBytes<'a> {
    Png(&'a [u8]),
}

/// Binds the buffers, and then calls .draw_indexed on the command_buffer_builder, it is still up to you to start and end render passes, and suchlike.
///
/// This does make some assumptions. It assumes that your instance buffer is going to be done using the subbuffer allocator, and that your vertex and index buffers are just regular subbuffers.
///
/// The name may change.
pub fn draw_instanced<V, I>(
    pipeline: Arc<GraphicsPipeline>,
    command_buffer_builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
    instances: &Vec<I>,
    vertices: &Subbuffer<[V]>,
    indices: &Subbuffer<[u32]>,
    subbuffer_allocator: &SubbufferAllocator,
) where
    V: BufferContents,
    I: BufferContents + Copy,
{
    let instance_buffer = subbuffer_allocator
        .allocate_slice(instances.len() as u64)
        .unwrap();
    instance_buffer.write().unwrap().copy_from_slice(instances);

    command_buffer_builder
        .bind_pipeline_graphics(pipeline)
        .unwrap()
        .bind_vertex_buffers(0, (vertices.clone(), instance_buffer))
        .unwrap()
        .bind_index_buffer(indices.clone())
        .unwrap()
        .draw_indexed(indices.len() as u32, instances.len() as u32, 0, 0, 0)
        .unwrap();
}
