use crate::{
    math,
    physics::physics_3d::aabb::{AabbCentredOrigin, CollisionEnum},
};

use super::Particle;

/// Usually you want to implement this for an enum that has varients for each of the different body types you want to use with the verlet solver.
///
/// For an example of an enum that implements this, check out [CommonBody].
pub trait Body<T>
where
    T: math::Float,
{
    fn update_and_get_position(
        &mut self,
        gravity: [T; 3],
        dampening: [T; 3],
        delta_time: T,
    ) -> [T; 3];
    /// If the body is nothing. It won't even bother to place this thing in the grid.
    /// This is useful for when you don't want to disturb the indices of bodies, but still want to remove bodies.
    fn is_none(&self) -> bool;
    fn collide_with_others(&self) -> bool;
    fn collide(&mut self, other: &mut Self, other_index: usize, delta_time: T);
}

/// A premade enum for you to use as the body type for the verlet solver.
///
/// The name might change.
pub enum CommonBody<T>
where
    T: math::Float,
{
    Player(Player<T>),
    Cuboid(Cuboid<T>),
    ImmovableCuboid(ImmovableCuboid<T>),
    None,
}

impl<T> Body<T> for CommonBody<T>
where
    T: math::Float,
{
    fn update_and_get_position(
        &mut self,
        gravity: [T; 3],
        dampening: [T; 3],
        delta_time: T,
    ) -> [T; 3] {
        match self {
            CommonBody::Player(player) => player.update_and_get_position(gravity, delta_time),
            CommonBody::Cuboid(simple_cuboid) => {
                simple_cuboid.update_and_get_position(gravity, dampening, delta_time)
            }
            CommonBody::ImmovableCuboid(immovable_cuboid) => {
                immovable_cuboid.update_and_get_position(gravity, dampening, delta_time)
            }
            CommonBody::None => unreachable!()
        }
    }

    fn is_none(&self) -> bool {
        match self {
            CommonBody::Player(_) => false,
            CommonBody::Cuboid(_) => false,
            CommonBody::ImmovableCuboid(_) => false,
            CommonBody::None => true,
        }
    }

    // terribly named, but very useful. TODO: Replace with proper lhs and rhs bitwise values for checking how objects can interact perhaps.
    fn collide_with_others(&self) -> bool {
        match self {
            CommonBody::Player(_) => true,
            CommonBody::Cuboid(_) => true,
            CommonBody::ImmovableCuboid(_) => false,
            CommonBody::None => unreachable!(),
        }
    }

    // This function is insanely expensive, no clue why.
    #[inline]
    fn collide(&mut self, other: &mut CommonBody<T>, other_index: usize, delta_time: T) {
        match (self, other) {
            // player
            (CommonBody::Player(lhs_player), CommonBody::Player(rhs_player)) => {
                todo!();
            }
            (CommonBody::Player(lhs_player), CommonBody::Cuboid(rhs_cuboid)) => {
                todo!();
            }
            (CommonBody::Player(lhs_player), CommonBody::ImmovableCuboid(rhs_immovable_cuboid)) => {
                let lhs_player_aabb = AabbCentredOrigin {
                    position: lhs_player.particle.position,
                    half_size: lhs_player.half_size,
                };
                if lhs_player_aabb.is_intersected_by_aabb(rhs_immovable_cuboid.aabb) {
                    //println!("lhs: {:?}\nrhs: {:?}",lhs_player, rhs_immovable_cuboid.aabb);
                    let previous_player_aabb = AabbCentredOrigin {
                        position: lhs_player.particle.previous_position,
                        half_size: lhs_player.half_size,
                    };
                    let previous_collision_direction = rhs_immovable_cuboid
                        .aabb
                        .get_collision_axis_with_direction(previous_player_aabb);
                    //println!("direction: {:?}", previous_collision_direction);

                    // TODO: investigate stepping up onto small ledges.
                    let step_up = ((lhs_player.particle.position[1] + lhs_player.half_size[1])
                        - (rhs_immovable_cuboid.aabb.position[1]
                            - rhs_immovable_cuboid.aabb.half_size[1]))
                        < T::from_f32(0.5);

                    if CollisionEnum::Positive == previous_collision_direction[0] && !step_up {
                        lhs_player.particle.position[0] = rhs_immovable_cuboid.aabb.position[0]
                            - rhs_immovable_cuboid.aabb.half_size[0]
                            - lhs_player.half_size[0]
                            - T::from_f32(0.01);
                    } else if CollisionEnum::Negative == previous_collision_direction[0] && !step_up
                    {
                        lhs_player.particle.position[0] = rhs_immovable_cuboid.aabb.position[0]
                            + rhs_immovable_cuboid.aabb.half_size[0]
                            + lhs_player.half_size[0]
                            + T::from_f32(0.01);
                    }

                    if CollisionEnum::Positive == previous_collision_direction[1]
                        || (step_up && CollisionEnum::None == previous_collision_direction[1])
                    {
                        lhs_player.particle.position[1] = rhs_immovable_cuboid.aabb.position[1]
                            - rhs_immovable_cuboid.aabb.half_size[1]
                            - lhs_player.half_size[1]
                            - T::from_f32(0.01);

                        //println!("Landed!");
                        lhs_player.grounded = true;
                    } else if CollisionEnum::Negative == previous_collision_direction[1] {
                        lhs_player.particle.position[1] = rhs_immovable_cuboid.aabb.position[1]
                            + rhs_immovable_cuboid.aabb.half_size[1]
                            + lhs_player.half_size[1]
                            + T::from_f32(0.01);
                    }

                    if CollisionEnum::Positive == previous_collision_direction[2] && !step_up {
                        lhs_player.particle.position[2] = rhs_immovable_cuboid.aabb.position[2]
                            - rhs_immovable_cuboid.aabb.half_size[2]
                            - lhs_player.half_size[2]
                            - T::from_f32(0.01);
                    } else if CollisionEnum::Negative == previous_collision_direction[2] && !step_up
                    {
                        lhs_player.particle.position[2] = rhs_immovable_cuboid.aabb.position[2]
                            + rhs_immovable_cuboid.aabb.half_size[2]
                            + lhs_player.half_size[2]
                            + T::from_f32(0.01);
                    }
                }
            }

            // cuboid
            (CommonBody::Cuboid(lhs_cuboid), CommonBody::Player(rhs_player)) => {
                todo!();
            }
            (CommonBody::Cuboid(lhs_cuboid), CommonBody::Cuboid(rhs_cuboid)) => {
                // TODO: Mess with ref mut and mut and all that, and you will gain and lose 70x performance, somehow.
                // if lhs_cuboid
                //     .aabb
                //     .is_intersected_by_aabb(rhs_cuboid.aabb)
                // {
                //     lhs_cuboid.particle.position = lhs_cuboid.particle.previous_position; // these lines in particular are slow
                //     rhs_cuboid.particle.position = rhs_cuboid.particle.previous_position;
                // }
                todo!();
            }
            (CommonBody::Cuboid(lhs_cuboid), CommonBody::ImmovableCuboid(rhs_immovable_cuboid)) => {
                todo!()
                // if lhs_cuboid
                //     .aabb
                //     .is_intersected_by_aabb(rhs_immovable_cuboid.aabb)
                // {
                //     let collision_axis = rhs_immovable_cuboid
                //         .aabb
                //         .get_collision_axis_with_direction(AabbCentredOrigin {
                //             position: lhs_cuboid.particle.previous_position,
                //             half_size: lhs_cuboid.aabb.half_size,
                //         });

                //     //println!("collision axis: {:?}", collision_axis);

                //     lhs_cuboid.particle.position = lhs_cuboid.particle.previous_position;

                //     //println!("Collision!");
                // }
            }

            // immovable simple cuboid (This cannot happen, as immovable simple cuboides don't check to see if they have collided with others.)
            (CommonBody::ImmovableCuboid(_), CommonBody::Player(_)) => unreachable!(),
            (CommonBody::ImmovableCuboid(_), CommonBody::Cuboid(_)) => unreachable!(),
            (CommonBody::ImmovableCuboid(_), CommonBody::ImmovableCuboid(_)) => {
                unreachable!()
            }

            (CommonBody::None,_) => unreachable!(),
            (_,CommonBody::None) => unreachable!(),
        }
    }
}

#[derive(Debug)]
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
    pub fn update_and_get_position(&mut self, gravity: [T; 3], delta_time: T) -> [T; 3] {
        self.particle.accelerate(gravity);
        self.particle.update(
            delta_time,
            math::mul_3d(self.particle.calculate_displacement(), self.dampening),
        );

        self.grounded = false;

        self.particle.position
    }
}

//#[derive(Clone, Copy)]
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
    pub fn update_and_get_position(
        &mut self,
        gravity: [T; 3],
        dampening: [T; 3],
        delta_time: T,
    ) -> [T; 3] {
        self.particle.accelerate(gravity);
        self.particle.update(
            delta_time,
            math::mul_3d(self.particle.calculate_displacement(), dampening),
        );

        self.particle.position
    }
}

pub struct ImmovableCuboid<T>
where
    T: math::Float,
{
    pub aabb: AabbCentredOrigin<T>,
}

impl<T> ImmovableCuboid<T>
where
    T: math::Float,
{
    pub fn update_and_get_position(
        &mut self,
        _gravity: [T; 3],
        _dampening: [T; 3],
        _delta_time: T,
    ) -> [T; 3] {
        self.aabb.position
    }
}

// TODO: List of common shaps I want to include here. But first, a naming scheme. No rotation should be by default. Axis aligned should be the default. At least 1 particle should be the default. As such "cuboid" should refer to an axis aligned cuboid with a single particle that can't rotate.
// List: Cuboid, ImmovableCuboid, Sphere, ImmovableSphere, Player, Cylinder, ImmovableCylinder
