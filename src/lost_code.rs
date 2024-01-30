// If we don't know where some code belongs, we chuck it in here temporarily.

use std::time::Instant;

use winit::event::ElementState;

pub fn wrap(value: f32, start: f32, limit: f32) -> f32 {
    start + (value - start) % (limit - start)
}

pub fn is_pressed(state: ElementState) -> bool {
    match state {
        ElementState::Pressed => true,
        ElementState::Released => false,
    }
}

/// Calls a function every fixed_delta_time seconds.
pub struct FixedUpdate {
    fixed_time_passed: f32,
    fixed_delta_time: f32,
    starting_time: Instant,
}

impl FixedUpdate {
    /// Create a new FixedUpdate.
    ///
    /// fixed_delta_time should be how often you want your function to be called, in seconds.
    #[must_use]
    pub fn new(fixed_delta_time: f32) -> FixedUpdate {
        FixedUpdate {
            fixed_time_passed: 0.0,
            fixed_delta_time,
            starting_time: Instant::now(),
        }
    }

    /// Call this every frame, and it will call closure every fixed_delta_time seconds.
    ///
    /// If there are more substeps than max_substeps it will println!() to let you know, but will not panic!(). This may change.
    pub fn update<F>(&mut self, max_substeps: u32, mut closure: F) where F: FnMut() {
        let seconds_since_start = self.starting_time.elapsed().as_secs_f32();
        let mut substeps = 0;

        while self.fixed_time_passed < seconds_since_start {
            closure();

            self.fixed_time_passed += self.fixed_delta_time;

            substeps += 1;

            if substeps > max_substeps {
                println!(
                    "Too many substeps per frame. Entered performance sinkhole. Substeps: {}",
                    substeps
                )
            }
        }
    }
}
