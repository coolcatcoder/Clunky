use crate::{
    math::{self, Direction},
    physics::physics_3d::{self, aabb::AabbCentredOrigin},
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

impl<T> CommonBody<T>
where
    T: math::Float,
{
    /// Returns the position of the common body, should it have a position. If it doesn't it returns an error.
    pub fn position(&self) -> Result<[T; 3], &'static str> {
        match self {
            CommonBody::Player(player) => Ok(player.particle.position),
            CommonBody::Cuboid(cuboid) => Ok(cuboid.particle.position),
            CommonBody::ImmovableCuboid(immovable_cuboid) => Ok(immovable_cuboid.aabb.position),
            CommonBody::None => Err("CommonBody::None does not have a position."),
        }
    }

    /// Returns the size of the common body, should it have a size (Or half size, we can just * 2.0). If it doesn't it returns an error.
    pub fn size(&self) -> Result<[T; 3], &'static str> {
        match self {
            CommonBody::Player(player) => {
                Ok(math::mul_3d_by_1d(player.half_size, T::from_f32(2.0)))
            }
            CommonBody::Cuboid(cuboid) => {
                Ok(math::mul_3d_by_1d(cuboid.half_size, T::from_f32(2.0)))
            }
            CommonBody::ImmovableCuboid(immovable_cuboid) => Ok(math::mul_3d_by_1d(
                immovable_cuboid.aabb.half_size,
                T::from_f32(2.0),
            )),
            CommonBody::None => Err("CommonBody::None does not have a half_size."),
        }
    }
    /// Returns the half size of the common body, should it have a half size (Or size, we can just * 0.5). If it doesn't it returns an error.
    pub fn half_size(&self) -> Result<[T; 3], &'static str> {
        match self {
            CommonBody::Player(player) => Ok(player.half_size),
            CommonBody::Cuboid(cuboid) => Ok(cuboid.half_size),
            CommonBody::ImmovableCuboid(immovable_cuboid) => Ok(immovable_cuboid.aabb.half_size),
            CommonBody::None => Err("CommonBody::None does not have a half_size."),
        }
    }
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
            CommonBody::None => unreachable!(),
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
                let lhs_player_aabb = AabbCentredOrigin {
                    position: lhs_player.particle.position,
                    half_size: lhs_player.half_size,
                };
                let rhs_cuboid_aabb = AabbCentredOrigin {
                    position: rhs_cuboid.particle.position,
                    half_size: rhs_cuboid.half_size,
                };
                if lhs_player_aabb.is_intersected_by_aabb(rhs_cuboid_aabb) {
                    let (collision_normal, penetration) =
                        lhs_player_aabb.get_collision_normal_and_penetration(&rhs_cuboid_aabb);
                    let collision_normal_signed_number =
                        math::direction_3d_to_signed_number_3d(collision_normal);
                    let collision_translation = math::mul_3d_by_1d(
                        collision_normal_signed_number,
                        -penetration * T::from_f32(0.5),
                    );

                    lhs_player
                        .particle
                        .apply_uniform_position_change(collision_translation);
                    rhs_cuboid
                        .particle
                        .apply_uniform_position_change(math::neg_3d(collision_translation));

                    if Direction::Positive == collision_normal[1] {
                        lhs_player.grounded = true;
                    }

                    let impulse = physics_3d::calculate_collision_impulse(
                        lhs_player.particle.calculate_velocity(delta_time),
                        T::ONE,
                        rhs_cuboid.particle.calculate_velocity(delta_time),
                        T::ONE,
                        collision_normal_signed_number,
                        T::from_f32(0.5),
                    );
                    lhs_player.particle.apply_impulse(impulse, delta_time);
                    rhs_cuboid
                        .particle
                        .apply_impulse(math::neg_3d(impulse), delta_time);
                }
            }
            (CommonBody::Player(lhs_player), CommonBody::ImmovableCuboid(rhs_immovable_cuboid)) => {
                let lhs_player_aabb = AabbCentredOrigin {
                    position: lhs_player.particle.position,
                    half_size: lhs_player.half_size,
                };
                if lhs_player_aabb.is_intersected_by_aabb(rhs_immovable_cuboid.aabb) {
                    //println!("lhs: {:?}\nrhs: {:?}",lhs_player, rhs_immovable_cuboid.aabb);

                    let (collision_normal, penetration) = lhs_player_aabb
                        .get_collision_normal_and_penetration(&rhs_immovable_cuboid.aabb);
                    let collision_normal_signed_number =
                        math::direction_3d_to_signed_number_3d(collision_normal);
                    let collision_translation =
                        math::mul_3d_by_1d(collision_normal_signed_number, -penetration);
                    //println!("normal: {:?}", collision_normal);

                    lhs_player
                        .particle
                        .apply_uniform_position_change(collision_translation);

                    if Direction::Positive == collision_normal[1] {
                        lhs_player.grounded = true;
                    }

                    // TODO: investigate stepping up onto small ledges. This could react unpredictably with collision_normal and penetration.
                    /*
                    let step_up = ((lhs_player.particle.position[1] + lhs_player.half_size[1])
                        - (rhs_immovable_cuboid.aabb.position[1]
                            - rhs_immovable_cuboid.aabb.half_size[1]))
                        < T::from_f32(0.5);
                    */

                    let impulse = physics_3d::calculate_collision_impulse_with_immovable_rhs(
                        lhs_player.particle.calculate_velocity(delta_time),
                        T::ONE,
                        collision_normal_signed_number,
                        T::from_f32(0.5),
                    );
                    //println!("impulse: {:?}", impulse);
                    lhs_player.particle.apply_impulse(impulse, delta_time);
                }
            }

            // cuboid
            (CommonBody::Cuboid(lhs_cuboid), CommonBody::Player(rhs_player)) => {
                //todo!();
                //println!("whoops");
            }
            (CommonBody::Cuboid(lhs_cuboid), CommonBody::Cuboid(rhs_cuboid)) => {
                let lhs_cuboid_aabb = AabbCentredOrigin {
                    position: lhs_cuboid.particle.position,
                    half_size: lhs_cuboid.half_size,
                };
                let rhs_cuboid_aabb = AabbCentredOrigin {
                    position: rhs_cuboid.particle.position,
                    half_size: rhs_cuboid.half_size,
                };
                if lhs_cuboid_aabb.is_intersected_by_aabb(rhs_cuboid_aabb) {
                    let (collision_normal, penetration) =
                        lhs_cuboid_aabb.get_collision_normal_and_penetration(&rhs_cuboid_aabb);
                    let collision_normal_signed_number =
                        math::direction_3d_to_signed_number_3d(collision_normal);
                    let collision_translation = math::mul_3d_by_1d(
                        collision_normal_signed_number,
                        -penetration * T::from_f32(0.5),
                    );

                    lhs_cuboid
                        .particle
                        .apply_uniform_position_change(collision_translation);
                    rhs_cuboid
                        .particle
                        .apply_uniform_position_change(math::neg_3d(collision_translation));

                    let impulse = physics_3d::calculate_collision_impulse(
                        lhs_cuboid.particle.calculate_velocity(delta_time),
                        T::ONE,
                        rhs_cuboid.particle.calculate_velocity(delta_time),
                        T::ONE,
                        collision_normal_signed_number,
                        T::from_f32(0.5),
                    );
                    lhs_cuboid.particle.apply_impulse(impulse, delta_time);
                    rhs_cuboid
                        .particle
                        .apply_impulse(math::neg_3d(impulse), delta_time);
                }
            }
            (CommonBody::Cuboid(lhs_cuboid), CommonBody::ImmovableCuboid(rhs_immovable_cuboid)) => {
                let lhs_cuboid_aabb = AabbCentredOrigin {
                    position: lhs_cuboid.particle.position,
                    half_size: lhs_cuboid.half_size,
                };
                if lhs_cuboid_aabb.is_intersected_by_aabb(rhs_immovable_cuboid.aabb) {
                    let (collision_normal, penetration) = lhs_cuboid_aabb
                        .get_collision_normal_and_penetration(&rhs_immovable_cuboid.aabb);
                    let collision_normal_signed_number =
                        math::direction_3d_to_signed_number_3d(collision_normal);
                    let collision_translation =
                        math::mul_3d_by_1d(collision_normal_signed_number, -penetration);

                    lhs_cuboid
                        .particle
                        .apply_uniform_position_change(collision_translation);

                    let impulse = physics_3d::calculate_collision_impulse_with_immovable_rhs(
                        lhs_cuboid.particle.calculate_velocity(delta_time),
                        T::ONE,
                        collision_normal_signed_number,
                        T::from_f32(0.5),
                    );
                    lhs_cuboid.particle.apply_impulse(impulse, delta_time);
                }
            }

            // immovable simple cuboid (This cannot happen, as immovable simple cuboides don't check to see if they have collided with others.)
            (CommonBody::ImmovableCuboid(_), CommonBody::Player(_)) => unreachable!(),
            (CommonBody::ImmovableCuboid(_), CommonBody::Cuboid(_)) => unreachable!(),
            (CommonBody::ImmovableCuboid(_), CommonBody::ImmovableCuboid(_)) => {
                unreachable!()
            }

            (CommonBody::None, _) => unreachable!(),
            (_, CommonBody::None) => unreachable!(),
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
