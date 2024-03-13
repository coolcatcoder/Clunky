use crate::math;

use super::Particle;

#[derive(Debug, Clone)]
pub struct Player<T>
where
    T: math::Float,
{
    pub particle: Particle<T>,
    pub mass: T,
    pub friction: T,
    pub restitution: T,
    pub half_size: [T; 3],
    pub dampening: [T; 3],
    pub grounded: bool,
}

impl<T> Player<T>
where
    T: math::Float,
{
    pub fn update(&mut self, gravity: [T; 3], delta_time: T) {
        self.particle.accelerate(gravity);
        self.particle.update(
            delta_time,
            math::mul_3d(self.particle.calculate_displacement(), self.dampening),
        );

        self.grounded = false;
    }
}

#[derive(Debug, Clone)]
pub struct Cuboid<T>
where
    T: math::Float,
{
    pub particle: Particle<T>, // Do we even want to store particles with collision data?
    pub half_size: [T; 3],
}

impl<T> Cuboid<T>
where
    T: math::Float,
{
    pub fn update(&mut self, gravity: [T; 3], dampening: [T; 3], delta_time: T) {
        self.particle.accelerate(gravity);
        self.particle.update(
            delta_time,
            math::mul_3d(self.particle.calculate_displacement(), dampening),
        );
    }
}

// TODO: List of common shaps I want to include here. But first, a naming scheme. No rotation should be by default. Axis aligned should be the default. At least 1 particle should be the default. As such "cuboid" should refer to an axis aligned cuboid with a single particle that can't rotate.
// List: Cuboid, ImmovableCuboid, Sphere, ImmovableSphere, Player, Cylinder, ImmovableCylinder
