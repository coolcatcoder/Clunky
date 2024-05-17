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
pub struct FixedUpdate<T: Float> {
    fixed_time_passed: T,
    pub fixed_delta_time: T,
    starting_time: Instant,
    max_substeps: MaxSubsteps,
}

/// Do you want a max amount of substeps, and if so, how should we handle going over?
pub enum MaxSubsteps {
    Infinite,
    ReturnAt(u32),
    PanicAt(u32),
    WarnAt(u32),
}

impl<T: Float> FixedUpdate<T> {
    /// Create a new FixedUpdate.
    ///
    /// fixed_delta_time should be how often you want your function to be called, in seconds.
    #[must_use]
    pub fn new(fixed_delta_time: T, max_substeps: MaxSubsteps) -> FixedUpdate<T> {
        FixedUpdate {
            fixed_time_passed: T::ZERO,
            fixed_delta_time,
            starting_time: Instant::now(),
            max_substeps,
        }
    }

    /// Every time this is called, it will see how long has passed, and call the callback the amount of times it should have been called, in that time span.
    pub fn update<F: FnMut()>(&mut self, mut callback: F) {
        let seconds_since_start = T::from_f64(self.starting_time.elapsed().as_secs_f64());
        let mut substeps = 0;

        while self.fixed_time_passed < seconds_since_start {
            callback();

            self.fixed_time_passed += self.fixed_delta_time;

            substeps += 1;

            match self.max_substeps {
                MaxSubsteps::Infinite => (),
                MaxSubsteps::ReturnAt(max) => {
                    if substeps > max {
                        return;
                    }
                }
                MaxSubsteps::PanicAt(max) => {
                    if substeps > max {
                        panic!("Too many substeps.\nSubsteps: {}\nMax: {}", substeps, max);
                    }
                }
                MaxSubsteps::WarnAt(max) => {
                    if substeps > max {
                        eprintln!("Too many substeps.\nSubsteps: {}\nMax: {}", substeps, max);
                    }
                }
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

    /// Time since previous frame.
    #[inline]
    #[must_use]
    pub fn delta_time(&self) -> T {
        self.delta_time
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
