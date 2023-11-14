// struct Aabb {

// }

#[derive(Clone, Copy)]
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
