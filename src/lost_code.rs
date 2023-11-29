// If we don't know where some code belongs, we chuck it in here temporarily.

#![allow(dead_code)] // Dead code is allowed, but occasionally comment this out so you can clear away functions which will likely never be used.

use winit::event::ElementState;

fn wrap(value: f32, start: f32, limit: f32) -> f32 {
    start + (value - start) % (limit - start)
}

pub fn is_pressed(state: ElementState) -> bool {
    match state {
        ElementState::Pressed => true,
        ElementState::Released => false,
    }
}