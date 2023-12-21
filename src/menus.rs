use crate::events;
use std::sync::Arc;
use vulkano::command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer};
use vulkano::pipeline::GraphicsPipeline;
use winit::event::Event;

mod example_1;
mod example_3d;
mod image_example;

pub const STARTING_MENU: Menu = Menu::Example1; // TODO: Very worried! When I set this to example 1 and then transition to example 3d, then example 3d works. If I go to example 3d directly using this const, then it doesn't work. WHAT THE HELL

pub const PNG_BYTES_LIST: [&[u8]; 1] = [include_bytes!("sprite_sheet.png").as_slice()];

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Menu {
    Example1,
    ImageExample,
    Example3D,
}

impl Menu {
    pub const fn get_data(&self) -> Data {
        match *self {
            Menu::Example1 => example_1::MENU,
            Menu::Example3D => example_3d::MENU,
            Menu::ImageExample => image_example::MENU,
        }
    }
}

pub struct Data {
    pub start: fn(&mut events::UserStorage, &mut crate::RenderStorage),
    pub update: fn(&mut events::UserStorage, &mut crate::RenderStorage, f32, f32),
    pub fixed_update: (f32, fn(&mut events::UserStorage, &mut crate::RenderStorage)),
    pub handle_events: fn(&mut events::UserStorage, &mut crate::RenderStorage, Event<'_, ()>),
    pub create_pipelines:
        fn(&mut events::UserStorage, &mut crate::RenderStorage) -> Vec<Arc<GraphicsPipeline>>,
    pub on_draw: fn(
        &mut events::UserStorage,
        &mut crate::RenderStorage,
        &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
    ),
    pub end: fn(&mut events::UserStorage, &mut crate::RenderStorage),
}
