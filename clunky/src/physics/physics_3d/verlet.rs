use crate::math;

extern crate test;

pub mod bodies;

#[derive(Debug, Clone, Copy)]
pub struct Particle<T>
where
    T: math::Number,
{
    pub position: [T; 3],
    pub previous_position: [T; 3],
    pub acceleration: [T; 3],
}

// I am interested in an experimental particle that uses integers.
struct Experiment {
    position: [u16; 3],
    // Perhaps this could be even smaller, and just have a lower precision, which I think would be fine.
    previous_position: [u16; 3],

    acceleration: [u16; 3],
}

impl<T> Particle<T>
where
    T: math::Number,
{
    #[inline]
    pub fn from_position(position: [T; 3]) -> Particle<T> {
        Particle {
            position,
            previous_position: position,
            acceleration: [T::ZERO; 3],
        }
    }

    #[inline] // Add must use attributes.
    pub fn calculate_displacement(&self) -> [T; 3] {
        math::sub_3d(self.position, self.previous_position)
    }

    /// Calculates the velocity using the formula displacement / time.
    #[inline]
    #[must_use]
    pub fn calculate_velocity(&self, delta_time: T) -> [T; 3] {
        math::div_3d_by_1d(self.calculate_displacement(), delta_time)
    }

    pub fn update(&mut self, delta_time: T, displacement: [T; 3]) {
        self.previous_position = self.position;

        let acceleration = math::mul_3d_by_1d(self.acceleration, delta_time * delta_time);

        self.position = math::add_3d(math::add_3d(self.position, displacement), acceleration);

        self.acceleration = [T::ZERO; 3];
    }

    #[inline]
    pub fn accelerate(&mut self, acceleration: [T; 3]) {
        self.acceleration[0] += acceleration[0];
        self.acceleration[1] += acceleration[1];
        self.acceleration[2] += acceleration[2];
    }

    /// Applies an impulse to the verlet particle.
    pub fn apply_impulse(&mut self, impulse: [T; 3], delta_time: T) {
        self.previous_position = math::sub_3d(
            self.previous_position,
            math::mul_3d_by_1d(impulse, delta_time),
        );
    }

    /// Moves both position and previous_position.
    /// This avoids accidental velocity and displacement changes.
    pub fn apply_uniform_position_change(&mut self, translation: [T; 3]) {
        self.position = math::add_3d(self.position, translation);
        self.previous_position = math::add_3d(self.previous_position, translation);
    }
}
