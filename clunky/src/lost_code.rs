// If we don't know where some code belongs, we chuck it in here temporarily.

use std::time::Instant;

use winit::event::ElementState;

use crate::math::Float;

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
    pub fn update<F>(&mut self, max_substeps: u32, mut closure: F)
    where
        F: FnMut(),
    {
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

/// Keeps track of how many frames are shown per second.
pub struct FpsTracker<T: Float> {
    delta_time_sum: T,
    average_fps: T,
    frame_count: u16, // If you have over 60 thousand fps, then you have a problem. Try implementing some inefficient single threaded cpu fluid physics to slow things down a bit.
    delta_time: T,
    time_since_previous_frame: Instant,
}

impl<T: Float> FpsTracker<T> {
    /// Creates a new fps tracker!
    #[must_use]
    pub fn new() -> FpsTracker<T> {
        FpsTracker {
            delta_time_sum: T::ZERO,
            average_fps: T::ZERO,
            frame_count: 0,
            delta_time: T::ZERO,
            time_since_previous_frame: Instant::now(),
        }
    }
    /// Gets fps averaged over the last second-ish.
    /// We can't stop at the end of a second exactly, hence why it is slightly averaged.
    #[inline]
    #[must_use]
    pub fn average_fps(&self) -> T {
        self.average_fps
    }

    /// Call this when a frame has passed.
    /// Updates delta_time.
    /// If over a second has passed, it sets average_fps.
    pub fn update(&mut self) {
        // Bit messy, but it works.
        self.delta_time = T::from_f64(self.time_since_previous_frame.elapsed().as_secs_f64());
        self.delta_time_sum += self.delta_time;
        self.frame_count += 1;
        self.time_since_previous_frame = Instant::now();

        if self.delta_time_sum > T::ONE {
            self.average_fps = T::from(self.frame_count) / self.delta_time_sum;
            self.frame_count = 0;
            self.delta_time_sum = T::ZERO;
        }
    }
}

impl<T: Float> Default for FpsTracker<T> {
    fn default() -> Self {
        Self::new()
    }
}
