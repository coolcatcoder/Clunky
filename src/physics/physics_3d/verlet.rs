use crate::math;

#[derive(Debug)]
pub struct Particle<T>
where
    T: math::Number,
{
    pub position: [T; 3],
    pub previous_position: [T; 3],
    pub acceleration: [T; 3],
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
}

// Solvers are always way too set in stone for my liking currently.
// pub struct Solver<T>
// where
//     T: math::Number,
// {
//     pub gravity: [T; 3],
//     pub particles: Vec<Particle<T>>,
// }

// impl<T> Solver<T>
// where
//     T: math::Number,
// {
//     pub fn update(&mut self, delta_time: T) {
//         for particle in &mut self.particles {
//             particle.accelerate(self.gravity);
//             particle.update(delta_time);
//         }
//     }
// }
