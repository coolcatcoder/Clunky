use crate::math::Float;

pub mod physics_2d;
pub mod physics_3d;

pub trait PhysicsSimulation<F: Float> {
    fn update(&mut self, delta_time: F);
}
