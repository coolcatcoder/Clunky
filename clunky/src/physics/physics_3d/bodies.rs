use crate::{
    math::{self, add_3d, Direction},
    physics::physics_3d::{self, aabb::AabbCentredOrigin},
};

use super::verlet;

/// Usually you want to implement this for an enum that has varients for each of the different body types you want to use with the verlet solver.
///
/// For an example of an enum that implements this, check out [CommonBody].
pub trait Body<T>: Send + Sync + std::fmt::Debug
where
    T: math::Float,
{
    /// Updates the body
    fn update(&mut self, gravity: [T; 3], dampening: [T; 3], delta_time: T);
    /// Gets the position of the body. Panic if it doesn't have a position.
    fn position_unchecked(&self) -> [T; 3];
    /// Gets the half size of the body. Panics if it doesn't have a size.
    fn half_size_unchecked(&self) -> [T; 3];
    /// If the body is nothing. It won't even bother to place this thing in the grid.
    /// This is useful for when you don't want to disturb the indices of bodies, but still want to remove bodies.
    fn is_none(&self) -> bool;
    fn collide_with_others(&self) -> bool;
    fn collide(&mut self, other: &mut Self, other_index: usize, delta_time: T);
    fn respond_to_collision(&mut self, other: &mut Self, other_index: usize, delta_time: T);

    fn detect_collision(&self, other: &Self) -> bool;
}

/// A premade enum for you to use as the body type for the [super::solver::CpuSolver].
///Player
/// The name might change.
#[derive(Debug, Clone)]
pub enum CommonBody<T>
where
    T: math::Float,
{
    Player(verlet::bodies::Player<T>),
    Cuboid(verlet::bodies::Cuboid<T>),
    ImmovableCuboid(ImmovableCuboid<T>),
    CollisionRecorderCuboid(CollisionRecorderCuboid<T, CommonBody<T>>),
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
            CommonBody::CollisionRecorderCuboid(collision_recorder_cuboid) => {
                Ok(collision_recorder_cuboid.aabb.position)
            }
            CommonBody::None => Err("CommonBody::None does not have a position."),
        }
    }

    /// Translates the position of the common body. If it doesn't have a position, it returns an error
    #[must_use]
    pub fn translate(&mut self, translation: [T; 3]) -> Result<(), &'static str> {
        match self {
            CommonBody::Player(player) => {
                player.particle.apply_uniform_position_change(translation);
                Ok(())
            }
            CommonBody::Cuboid(cuboid) => {
                cuboid.particle.apply_uniform_position_change(translation);
                Ok(())
            }
            CommonBody::ImmovableCuboid(immovable_cuboid) => {
                immovable_cuboid.aabb.position =
                    add_3d(immovable_cuboid.aabb.position, translation);
                Ok(())
            }
            CommonBody::CollisionRecorderCuboid(collision_recorder_cuboid) => {
                collision_recorder_cuboid.aabb.position =
                    add_3d(collision_recorder_cuboid.aabb.position, translation);
                Ok(())
            }
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
            CommonBody::CollisionRecorderCuboid(collision_recorder_cuboid) => Ok(
                math::mul_3d_by_1d(collision_recorder_cuboid.aabb.half_size, T::from_f32(2.0)),
            ),
            CommonBody::None => Err("CommonBody::None does not have a half_size."),
        }
    }
    /// Returns the half size of the common body, should it have a half size (Or size, we can just * 0.5). If it doesn't it returns an error.
    pub fn half_size(&self) -> Result<[T; 3], &'static str> {
        match self {
            CommonBody::Player(player) => Ok(player.half_size),
            CommonBody::Cuboid(cuboid) => Ok(cuboid.half_size),
            CommonBody::ImmovableCuboid(immovable_cuboid) => Ok(immovable_cuboid.aabb.half_size),
            CommonBody::CollisionRecorderCuboid(collision_recorder_cuboid) => {
                Ok(collision_recorder_cuboid.aabb.half_size)
            }
            CommonBody::None => Err("CommonBody::None does not have a half_size."),
        }
    }
}

impl<T> Body<T> for CommonBody<T>
where
    T: math::Float,
{
    fn update(&mut self, gravity: [T; 3], dampening: [T; 3], delta_time: T) {
        match self {
            CommonBody::Player(player) => player.update(gravity, delta_time),
            CommonBody::Cuboid(simple_cuboid) => {
                simple_cuboid.update(gravity, dampening, delta_time)
            }
            CommonBody::ImmovableCuboid(immovable_cuboid) => {
                immovable_cuboid.update(gravity, dampening, delta_time)
            }
            CommonBody::CollisionRecorderCuboid(_) => (),
            CommonBody::None => unreachable!(),
        }
    }

    fn position_unchecked(&self) -> [T; 3] {
        match self {
            CommonBody::Player(player) => player.particle.position,
            CommonBody::Cuboid(cuboid) => cuboid.particle.position,
            CommonBody::ImmovableCuboid(immovable_cuboid) => immovable_cuboid.aabb.position,
            CommonBody::CollisionRecorderCuboid(collision_recorder_cuboid) => {
                collision_recorder_cuboid.aabb.position
            }
            CommonBody::None => unreachable!(),
        }
    }

    fn half_size_unchecked(&self) -> [T; 3] {
        match self {
            CommonBody::Player(player) => player.half_size,
            CommonBody::Cuboid(cuboid) => cuboid.half_size,
            CommonBody::ImmovableCuboid(immovable_cuboid) => immovable_cuboid.aabb.half_size,
            CommonBody::CollisionRecorderCuboid(collision_recorder_cuboid) => {
                collision_recorder_cuboid.aabb.half_size
            }
            CommonBody::None => unreachable!(),
        }
    }

    fn is_none(&self) -> bool {
        match self {
            CommonBody::None => true,
            _ => false,
        }
    }

    // terribly named, but very useful.
    fn collide_with_others(&self) -> bool {
        match self {
            CommonBody::Player(_) => true,
            CommonBody::Cuboid(_) => true,
            CommonBody::ImmovableCuboid(_) => false,
            CommonBody::CollisionRecorderCuboid(_) => false,
            CommonBody::None => unreachable!(),
        }
    }

    #[inline]
    fn collide(&mut self, other: &mut CommonBody<T>, _other_index: usize, delta_time: T) {
        let colliding_bodies = (self, other);
        match colliding_bodies {
            // player
            (CommonBody::Player(_lhs_player), CommonBody::Player(_rhs_player)) => {
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
            (
                CommonBody::Player(lhs_player),
                CommonBody::CollisionRecorderCuboid(rhs_collision_recorder_cuboid),
            ) => {
                let lhs_player_aabb = AabbCentredOrigin {
                    position: lhs_player.particle.position,
                    half_size: lhs_player.half_size,
                };
                if lhs_player_aabb.is_intersected_by_aabb(rhs_collision_recorder_cuboid.aabb) {
                    if (rhs_collision_recorder_cuboid.save_collision)(colliding_bodies.1) {
                        todo!();
                        //rhs_collision_recorder_cuboid.stored_collider_index =
                    }
                }
            }

            // cuboid
            (CommonBody::Cuboid(_lhs_cuboid), CommonBody::Player(_rhs_player)) => {
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
            (
                CommonBody::Cuboid(lhs_cuboid),
                CommonBody::CollisionRecorderCuboid(rhs_collision_recorder_cuboid),
            ) => {
                let lhs_cuboid_aabb = AabbCentredOrigin {
                    position: lhs_cuboid.particle.position,
                    half_size: lhs_cuboid.half_size,
                };
                if lhs_cuboid_aabb.is_intersected_by_aabb(rhs_collision_recorder_cuboid.aabb) {
                    todo!();
                    //(rhs_collision_recorder_cuboid.on_collision)(colliding_bodies.1);
                }
            }

            // immovable simple cuboid (This cannot happen, as immovable simple cuboides don't check to see if they have collided with others.)
            (CommonBody::ImmovableCuboid(_), _) => unreachable!(),

            (CommonBody::CollisionRecorderCuboid(_), _) => unreachable!(),

            (CommonBody::None, _) => unreachable!(),
            (_, CommonBody::None) => unreachable!(),
        }
    }

    #[inline]
    fn respond_to_collision(
        &mut self,
        other: &mut CommonBody<T>,
        _other_index: usize,
        delta_time: T,
    ) {
        let colliding_bodies = (self, other);
        match colliding_bodies {
            // player
            (CommonBody::Player(_lhs_player), CommonBody::Player(_rhs_player)) => {
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
            (CommonBody::Player(lhs_player), CommonBody::ImmovableCuboid(rhs_immovable_cuboid)) => {
                let lhs_player_aabb = AabbCentredOrigin {
                    position: lhs_player.particle.position,
                    half_size: lhs_player.half_size,
                };
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
            (
                CommonBody::Player(_),
                CommonBody::CollisionRecorderCuboid(_rhs_collision_recorder_cuboid),
            ) => {
                todo!();
                //(rhs_collision_recorder_cuboid.on_collision)(colliding_bodies.1);
            }

            // cuboid
            (CommonBody::Cuboid(_lhs_cuboid), CommonBody::Player(_rhs_player)) => {
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
            (CommonBody::Cuboid(lhs_cuboid), CommonBody::ImmovableCuboid(rhs_immovable_cuboid)) => {
                let lhs_cuboid_aabb = AabbCentredOrigin {
                    position: lhs_cuboid.particle.position,
                    half_size: lhs_cuboid.half_size,
                };
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
            (
                CommonBody::Cuboid(_),
                CommonBody::CollisionRecorderCuboid(_rhs_collision_recorder_cuboid),
            ) => {
                todo!();
                //(rhs_collision_recorder_cuboid.on_collision)(colliding_bodies.1);
            }

            // immovable simple cuboid (This cannot happen, as immovable simple cuboides don't check to see if they have collided with others.)
            (CommonBody::ImmovableCuboid(_), _) => unreachable!(),

            (CommonBody::CollisionRecorderCuboid(_), _) => unreachable!(),

            (CommonBody::None, _) => unreachable!(),
            (_, CommonBody::None) => unreachable!(),
        }
    }

    #[inline]
    fn detect_collision(&self, other: &CommonBody<T>) -> bool {
        let colliding_bodies = (self, other);
        match colliding_bodies {
            // player
            (CommonBody::Player(_lhs_player), CommonBody::Player(_rhs_player)) => {
                todo!()
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
                lhs_player_aabb.is_intersected_by_aabb(rhs_cuboid_aabb)
            }
            (CommonBody::Player(lhs_player), CommonBody::ImmovableCuboid(rhs_immovable_cuboid)) => {
                let lhs_player_aabb = AabbCentredOrigin {
                    position: lhs_player.particle.position,
                    half_size: lhs_player.half_size,
                };
                lhs_player_aabb.is_intersected_by_aabb(rhs_immovable_cuboid.aabb)
            }
            (
                CommonBody::Player(lhs_player),
                CommonBody::CollisionRecorderCuboid(rhs_collision_recorder_cuboid),
            ) => {
                let lhs_player_aabb = AabbCentredOrigin {
                    position: lhs_player.particle.position,
                    half_size: lhs_player.half_size,
                };
                lhs_player_aabb.is_intersected_by_aabb(rhs_collision_recorder_cuboid.aabb)
            }

            // cuboid
            (CommonBody::Cuboid(lhs_cuboid), CommonBody::Player(rhs_player)) => {
                let lhs_cuboid_aabb = AabbCentredOrigin {
                    position: lhs_cuboid.particle.position,
                    half_size: lhs_cuboid.half_size,
                };
                let rhs_player_aabb = AabbCentredOrigin {
                    position: rhs_player.particle.position,
                    half_size: rhs_player.half_size,
                };
                lhs_cuboid_aabb.is_intersected_by_aabb(rhs_player_aabb)
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
                lhs_cuboid_aabb.is_intersected_by_aabb(rhs_cuboid_aabb)
            }
            (CommonBody::Cuboid(lhs_cuboid), CommonBody::ImmovableCuboid(rhs_immovable_cuboid)) => {
                let lhs_cuboid_aabb = AabbCentredOrigin {
                    position: lhs_cuboid.particle.position,
                    half_size: lhs_cuboid.half_size,
                };
                lhs_cuboid_aabb.is_intersected_by_aabb(rhs_immovable_cuboid.aabb)
            }
            (
                CommonBody::Cuboid(lhs_cuboid),
                CommonBody::CollisionRecorderCuboid(rhs_collision_recorder_cuboid),
            ) => {
                let lhs_cuboid_aabb = AabbCentredOrigin {
                    position: lhs_cuboid.particle.position,
                    half_size: lhs_cuboid.half_size,
                };
                lhs_cuboid_aabb.is_intersected_by_aabb(rhs_collision_recorder_cuboid.aabb)
            }

            // immovable simple cuboid (This cannot happen, as immovable simple cuboides don't check to see if they have collided with others.)
            (CommonBody::ImmovableCuboid(_), _) => unreachable!(),

            (CommonBody::CollisionRecorderCuboid(_), _) => unreachable!(),

            (CommonBody::None, _) => unreachable!(),
            (_, CommonBody::None) => unreachable!(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CollisionRecorderCuboid<T, B>
where
    T: math::Float,
    B: Body<T>,
{
    pub aabb: AabbCentredOrigin<T>,
    pub save_collision: fn(&mut B) -> bool,
    pub stored_collider_index: Option<usize>,
    // TODO: multiple collisions behaviour?
}

#[derive(Debug, Clone)]
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
    #[inline]
    pub fn update(&mut self, _gravity: [T; 3], _dampening: [T; 3], _delta_time: T) {}
}

// TODO: List of common shaps I want to include here. But first, a naming scheme. No rotation should be by default. Axis aligned should be the default. At least 1 particle should be the default. As such "cuboid" should refer to an axis aligned cuboid with a single particle that can't rotate.
// List: Cuboid, ImmovableCuboid, Sphere, ImmovableSphere, Player, Cylinder, ImmovableCylinder
