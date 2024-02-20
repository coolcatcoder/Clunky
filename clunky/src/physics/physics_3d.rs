use crate::math::{self, Float};

pub mod aabb;
pub mod verlet;

/// Calculates the final velocities when 2 rigid bodies collide elastically.
/// Cannot handle an infinite mass rhs. Even if it did, it would just return the negative of the lhs velocity.
/// This assumes the collision happened with no intersection between the bodies. If there is intersection then it will likely provide incorrect velocities.
#[must_use]
pub fn calculate_velocities_during_elastic_collision<T: Float>(
    lhs_mass: T,
    lhs_velocity: [T; 3],
    rhs_mass: T,
    rhs_velocity: [T; 3],
) -> ([T; 3], [T; 3]) {
    let total_momentum_before = math::add_3d(
        math::mul_3d_by_1d(lhs_velocity, lhs_mass),
        math::mul_3d_by_1d(rhs_velocity, rhs_mass),
    );
    let relative_velocity_before = math::sub_3d(rhs_velocity, lhs_velocity);

    // Before the mangling: 2.0 * (lhs_mass * rhs_mass) / (lhs_mass + rhs_mass) * dot(relative_velocity_before, total_momentum_before) / dot(total_momentum_before, total_momentum_before)
    let impulse = T::from_f32(2.0) * (lhs_mass * rhs_mass) / (lhs_mass + rhs_mass)
        * math::dot(relative_velocity_before, total_momentum_before)
        / math::dot(total_momentum_before, total_momentum_before);

    // Before the mangling: lhs_velocity + impulse / lhs_mass * total_momentum_before. (Same for rhs.)
    (
        math::add_3d(
            lhs_velocity,
            math::mul_3d_by_1d(total_momentum_before, impulse / lhs_mass),
        ),
        math::add_3d(
            rhs_velocity,
            math::mul_3d_by_1d(total_momentum_before, impulse / rhs_mass),
        ),
    )
}

/// Calculates the final velocities when 2 rigid bodies collide elastically.
/// Gravity is the constant downward acceleration. I have no idea how to set this correctly currently. Try 9.8?
/// Cannot handle an infinite mass rhs. Uncertain what to use instead.
/// This assumes the collision happened with no intersection between the bodies. If there is intersection then it will likely provide incorrect velocities.
/// This doesn't work. It was generated with chatgpt, as a starting point, because finding niche physics functions is surprisingly hard. I'm continuing to modify and fix this function,but it may never work.
#[must_use]
pub fn calculate_velocities_during_elastic_collision_with_friction_and_restitution<T: Float>(
    lhs_mass: T,
    lhs_velocity: [T; 3],

    rhs_mass: T,
    rhs_velocity: [T; 3],

    gravity: T,
    friction: T,
    restitution: T,
) -> ([T; 3], [T; 3]) {
    let total_momentum_before = math::add_3d(
        math::mul_3d_by_1d(lhs_velocity, lhs_mass),
        math::mul_3d_by_1d(rhs_velocity, rhs_mass),
    );
    let relative_velocity_before = math::sub_3d(rhs_velocity, lhs_velocity);

    // Before the mangling: (lhs_restitution + rhs_restitution) * (lhs_mass * rhs_mass) / (lhs_mass + rhs_mass) * dot(relative_velocity_before, total_momentum_before) / dot(total_momentum_before, total_momentum_before)
    let impulse = (T::from_f32(1.0) + restitution) * (lhs_mass * rhs_mass) / (lhs_mass + rhs_mass)
        * math::dot(relative_velocity_before, total_momentum_before)
        / math::dot(total_momentum_before, total_momentum_before);

    // Before the mangling: lhs_velocity + impulse / lhs_mass * total_momentum_before. (Same for rhs.)
    let lhs_simple_final_velocity = math::add_3d(
        lhs_velocity,
        math::mul_3d_by_1d(total_momentum_before, impulse / lhs_mass),
    );
    let rhs_simple_final_velocity = math::sub_3d(
        rhs_velocity,
        math::mul_3d_by_1d(total_momentum_before, impulse / rhs_mass),
    );

    let lhs_friction_force = friction * lhs_mass * gravity;
    let lhs_friction_direction = math::neg_3d(math::normalise_3d(lhs_velocity));
    let lhs_friction_impulse = math::mul_3d_by_1d(lhs_friction_direction, lhs_friction_force);

    let rhs_friction_force = friction * rhs_mass * gravity;
    let rhs_friction_direction = math::neg_3d(math::normalise_3d(rhs_velocity));
    let rhs_friction_impulse = math::mul_3d_by_1d(rhs_friction_direction, rhs_friction_force);

    (
        math::add_3d(
            lhs_simple_final_velocity,
            math::div_3d_by_1d(lhs_friction_impulse, lhs_mass),
        ),
        math::add_3d(
            rhs_simple_final_velocity,
            math::div_3d_by_1d(rhs_friction_impulse, rhs_mass),
        ),
    )
}
