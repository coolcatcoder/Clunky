use crate::{math, physics::physics_3d::aabb::AabbCentredOrigin};

use super::Particle;

/// Usually you want to implement this for an enum that has varients for each of the different body types you want to use with the verlet solver.
/// 
/// For an example of an enum that implements this, check out [CommonBody].
pub trait Body<T> where T: math::Float {
    fn update_and_get_position(&mut self, gravity: [T; 3], dampening: [T; 3], delta_time: T) -> [T; 3];
    fn collide_with_others(&self) -> bool;
    fn collide(&mut self, other: &mut Self, other_index: usize);
}

/// A premade enum for you to use as the body type for the verlet solver.
/// 
/// The name might change.
pub enum CommonBody<T>
where
    T: math::Float,
{
    Player(Player<T>),
    Box(Box<T>),
    ImmovableBox(ImmovableBox<T>),
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
            CommonBody::Player(player) => {
                player.update_and_get_position(gravity, delta_time)
            }
            CommonBody::Box(simple_box) => {
                simple_box.update_and_get_position(gravity, dampening, delta_time)
            }
            CommonBody::ImmovableBox(immovable_box) => {
                immovable_box.update_and_get_position(gravity, dampening, delta_time)
            }
        }
    }

    // terribly named, but very useful. TODO: Replace with proper lhs and rhs bitwise values for checking how objects can interact perhaps.
    fn collide_with_others(&self) -> bool {
        match self {
            CommonBody::Player(_) => true,
            CommonBody::Box(_) => true,
            CommonBody::ImmovableBox(_) => false,
        }
    }

    // This function is insanely expensive, no clue why.
    #[inline]
    fn collide(&mut self, other: &mut CommonBody<T>, other_index: usize) {
        match (self, other) {
            // player
            (CommonBody::Player(lhs_player), CommonBody::Player(rhs_player)) => {
                todo!();
            }
            (CommonBody::Player(lhs_player), CommonBody::Box(rhs_box)) => {
                todo!();
            }
            (CommonBody::Player(lhs_player), CommonBody::ImmovableBox(rhs_immovable_box)) => {
                if lhs_player.aabb.is_intersected_by_aabb(rhs_immovable_box.aabb) {
                    let previous_player_aabb = AabbCentredOrigin {
                        position: lhs_player.particle.previous_position,
                        half_size: lhs_player.aabb.half_size,
                    };
                    let previous_collision_axis = rhs_immovable_box.aabb.get_collision_axis(previous_player_aabb);

                    if previous_collision_axis[0] {
                        lhs_player.particle.position[0] = lhs_player.particle.previous_position[0]; // TODO: Investigate stepping up onto small ledges.
                    }

                    if previous_collision_axis[1] {
                        lhs_player.particle.position[1] = lhs_player.particle.previous_position[1];

                        if previous_player_aabb.position[1] + previous_player_aabb.half_size[1]
                            <= rhs_immovable_box.aabb.position[1] - rhs_immovable_box.aabb.half_size[1]
                        {
                            lhs_player.grounded = true;
                        }
                    }

                    if previous_collision_axis[2] {
                        lhs_player.particle.position[2] = lhs_player.particle.previous_position[2];
                    }
                }
            }

            // box which has to be called simple_box sometimes due to naming conflicts...
            (CommonBody::Box(lhs_box), CommonBody::Player(rhs_player)) => {
                todo!();
            }
            (CommonBody::Box(lhs_box), CommonBody::Box(rhs_box)) => {
                // TODO: Mess with ref mut and mut and all that, and you will gain and lose 70x performance, somehow.
                // if lhs_simple_box
                //     .aabb
                //     .is_intersected_by_aabb(rhs_simple_box.aabb)
                // {
                //     lhs_simple_box.particle.position = lhs_simple_box.particle.previous_position; // these lines in particular are slow
                //     rhs_simple_box.particle.position = rhs_simple_box.particle.previous_position;
                // }
                todo!();
            }
            (
                CommonBody::Box(lhs_simple_box),
                CommonBody::ImmovableBox(rhs_immovable_simple_box),
            ) => {
                if lhs_simple_box
                    .aabb
                    .is_intersected_by_aabb(rhs_immovable_simple_box.aabb)
                {
                    let collision_axis = rhs_immovable_simple_box
                        .aabb
                        .get_collision_axis_with_direction(AabbCentredOrigin {
                            position: lhs_simple_box.particle.previous_position,
                            half_size: lhs_simple_box.aabb.half_size,
                        });

                    //println!("collision axis: {:?}", collision_axis);

                    lhs_simple_box.particle.position = lhs_simple_box.particle.previous_position;

                    //println!("Collision!");
                }
            }

            // immovable simple box (This cannot happen, as immovable simple boxes don't check to see if they have collided with others.)
            (CommonBody::ImmovableBox(_), CommonBody::Player(_)) => unreachable!(),
            (CommonBody::ImmovableBox(_), CommonBody::Box(_)) => unreachable!(),
            (CommonBody::ImmovableBox(_), CommonBody::ImmovableBox(_)) => {
                unreachable!()
            }
        }
    }
}

pub struct Player<T>
where
    T: math::Float,
{
    pub particle: Particle<T>,
    pub aabb: crate::physics::physics_3d::aabb::AabbCentredOrigin<T>,
    pub dampening: [T; 3],
    pub grounded: bool,
}

impl<T> Player<T>
where
    T: math::Float,
{
    pub fn update_and_get_position(
        &mut self,
        gravity: [T; 3],
        delta_time: T,
    ) -> [T; 3] {
        self.particle.accelerate(gravity);
        self.particle.update(
            delta_time,
            math::mul_3d(self.particle.calculate_displacement(), self.dampening),
        );

        self.aabb.position = self.particle.position;
        self.grounded = false;

        self.particle.position
    }
}

//#[derive(Clone, Copy)]
pub struct Box<T>
where
    T: math::Float,
{
    pub particle: Particle<T>, // Do we even want to store particles with collision data?
    pub aabb: crate::physics::physics_3d::aabb::AabbCentredOrigin<T>,
}

impl<T> Box<T>
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

        self.aabb.position = self.particle.position;

        self.particle.position
    }
}

pub struct ImmovableBox<T>
where
    T: math::Float,
{
    pub aabb: crate::physics::physics_3d::aabb::AabbCentredOrigin<T>,
}

impl<T> ImmovableBox<T>
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

// TODO: List of common shaps I want to include here. But first, a naming scheme. No rotation should be by default. Axis aligned should be the default. At least 1 particle should be the default. As such "box" should refer to an axis aligned box with a single particle that can't rotate.
// List: Box, ImmovableBox, Sphere, ImmovableSphere, Player, Cylinder, ImmovableCylinder