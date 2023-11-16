#[derive(Debug, Clone, Copy)]
pub struct Aabb {
    pub size: (f64, f64),

    pub position: (f64, f64),
}

pub fn point_intersects_aabb(aabb: Aabb, point_position: (f64, f64)) -> bool {
    point_position.0 < aabb.position.0 + aabb.size.0
        && point_position.0 > aabb.position.0
        && point_position.1 < aabb.position.1 + aabb.size.1
        && point_position.1 > aabb.position.1
}

#[derive(Debug, Clone, Copy)]
pub struct AabbCentred {
    pub position: (f32, f32),
    pub size: (f32, f32), // Perhaps make this half size?
}

pub fn detect_collision_aabb_centred(aabb_1: AabbCentred, aabb_2: AabbCentred) -> bool {
    if (aabb_1.position.0 - aabb_2.position.0).abs() > aabb_1.size.0 * 0.5 + aabb_2.size.0 * 0.5 {
        return false;
    }
    if (aabb_1.position.1 - aabb_2.position.1).abs() > aabb_1.size.1 * 0.5 + aabb_2.size.1 * 0.5 {
        return false;
    }
    true
}

pub fn point_intersects_aabb_centred(aabb: AabbCentred, point_position: (f32, f32)) -> bool {
    if (aabb.position.0 - point_position.0).abs() > aabb.size.0 * 0.5 {
        return false;
    }
    if (aabb.position.1 - point_position.1).abs() > aabb.size.1 * 0.5 {
        return false;
    }
    true
}
