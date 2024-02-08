use std::sync::Arc;
use vulkano::command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer};
use vulkano::image::sampler::Sampler;
use vulkano::image::view::ImageView;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::render_pass::RenderPass;
use winit::event::Event;

pub mod example_1;
pub mod example_3d;
pub mod image_example;
pub mod islands;

pub const STARTING_MENU: Menu = Menu::Islands;

pub const PNG_BYTES_LIST: [&[u8]; 2] = [
    include_bytes!("sprite_sheet.png").as_slice(),
    include_bytes!("../src/sprites/moon_wax_tree.png"),
];

//pub const MAX_SUBSTEPS: u32 = 150;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Menu {
    Example1,
    ImageExample,
    Example3D,
    Islands,
}

impl Menu {
    pub const fn get_data(&self) -> Data {
        match *self {
            Menu::Example1 => example_1::MENU,
            Menu::Example3D => example_3d::MENU,
            Menu::ImageExample => image_example::MENU,
            Menu::Islands => islands::MENU,
        }
    }
}

pub fn start(render_storage: &mut crate::RenderStorage) -> UserStorage {
    let mut user_storage = UserStorage {
        wasd_held: (false, false, false, false),
        zoom_held: (false, false),
        //camera_3d_position: [4.0, 2.0, 4.25],
        //camera_3d_rotation: [0.0, 0.0, 90.0], // Set z to 90.0 to have the best time ever!
        //camera_3d_scale: [0.5, 2.3, 1.0], // Fun: [0.5, 2.3, 1.0]
        //camera_3d_rotation: [0.0, 0.0, 0.0],
        //camera_3d_scale: [1.0, 1.0, 1.0],
        example_3d_storage: example_3d::get_starting_storage(),
        other_example_3d_storage: islands::get_starting_storage(render_storage),
        sensitivity: 0.25,
        sprinting: true,
    };

    (STARTING_MENU.get_data().start)(&mut user_storage, render_storage);

    user_storage
}

pub struct UserStorage {
    // This is for the user's stuff. The event loop should not touch this.
    pub wasd_held: (bool, bool, bool, bool),
    pub zoom_held: (bool, bool),

    pub example_3d_storage: example_3d::Example3DStorage,
    pub other_example_3d_storage: islands::OtherExample3DStorage,

    pub sensitivity: f32,

    pub sprinting: bool,
}

pub struct Data {
    pub start: fn(&mut UserStorage, &mut crate::RenderStorage),
    pub update: fn(&mut UserStorage, &mut crate::RenderStorage, f32, f32),
    pub fixed_update: FixedUpdate,
    pub handle_events: fn(&mut UserStorage, &mut crate::RenderStorage, Event<'_, ()>),
    pub create_pipelines: fn(
        [u32; 3],
        Arc<RenderPass>,
        &mut UserStorage,
        &mut crate::RenderStorage,
    ) -> Vec<Arc<GraphicsPipeline>>,
    pub on_draw: fn(
        &mut UserStorage,
        &mut crate::RenderStorage,
        &Vec<Arc<ImageView>>,
        &Arc<Sampler>,
        &Vec<Arc<GraphicsPipeline>>,
        &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
    ),
    pub end: fn(&mut UserStorage, &mut crate::RenderStorage),
}

pub struct FixedUpdate {
    pub delta_time: f32,
    pub max_substeps: u32,
    pub function: fn(&mut UserStorage, &mut crate::RenderStorage),
}
