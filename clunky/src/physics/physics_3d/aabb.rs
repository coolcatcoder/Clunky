// TODO: Rename this to axis_aligned_shapes.rs perhaps? I want to have more than just aabb in here.

use crate::math::{Direction, Number, SignedNumber};
extern crate test;

pub struct AabbTopLeftOrigin<T>
where
    T: Number,
{
    pub position: [T; 3],
    pub size: [T; 3],
}

impl<T> AabbTopLeftOrigin<T>
where
    T: Number,
{
    pub fn is_intersected_by_point(&self, point: [T; 3]) -> bool {
        point[0] < self.position[0] + self.size[0]
            && point[0] > self.position[0]
            && point[1] < self.position[1] + self.size[1]
            && point[1] > self.position[1]
            && point[2] < self.position[2] + self.size[2]
            && point[2] > self.position[2]
    }

    pub fn is_intersected_by_aabb(&self, aabb: AabbTopLeftOrigin<T>) -> bool {
        self.position[0] < aabb.position[0] + aabb.size[0]
            && self.position[0] + self.size[0] > aabb.position[0]
            && self.position[1] < aabb.position[1] + aabb.size[1]
            && self.position[1] + self.size[1] > aabb.position[1]
            && self.position[2] < aabb.position[2] + aabb.size[2]
            && self.position[2] + self.size[2] > aabb.position[2]
    }
}

#[derive(Copy, Clone, Debug)]
pub struct AabbCentredOrigin<T>
where
    T: Number,
{
    pub position: [T; 3],
    pub half_size: [T; 3],
}

impl<T> AabbCentredOrigin<T>
where
    T: SignedNumber, // Need to split up properly. See 2d for more info.
{
    pub fn is_intersected_by_point(&self, point: [T; 3]) -> bool {
        if (self.position[0] - point[0]).abs() > self.half_size[0] {
            return false;
        }
        if (self.position[1] - point[1]).abs() > self.half_size[1] {
            return false;
        }
        if (self.position[2] - point[2]).abs() > self.half_size[2] {
            return false;
        }
        true
    }

    pub fn is_intersected_by_aabb(&self, aabb: AabbCentredOrigin<T>) -> bool {
        if (self.position[0] - aabb.position[0]).abs() > self.half_size[0] + aabb.half_size[0] {
            return false;
        }
        if (self.position[1] - aabb.position[1]).abs() > self.half_size[1] + aabb.half_size[1] {
            return false;
        }
        if (self.position[2] - aabb.position[2]).abs() > self.half_size[2] + aabb.half_size[2] {
            return false;
        }
        true
    }

    /// See [AabbCentredOrigin::get_collision_axis_with_direction] for more info. Both are garbage with documentation.
    pub fn get_collision_axis(&self, other: AabbCentredOrigin<T>) -> [bool; 3] {
        // Run this on previous position instead, so you can see what axis wasn't intersecting before the collision.
        [
            (self.position[0] - other.position[0]).abs() > self.half_size[0] + other.half_size[0],
            (self.position[1] - other.position[1]).abs() > self.half_size[1] + other.half_size[1],
            (self.position[2] - other.position[2]).abs() > self.half_size[2] + other.half_size[2],
        ]
    }

    /// Works out on what axis and in what direction a collision occured.
    ///
    /// This function has only been tested by calling it on a non-moving aabb, while previous_other was the previous aabb of a moving aabb.
    pub fn get_collision_axis_with_direction(
        &self,
        previous_other: AabbCentredOrigin<T>,
    ) -> [Direction; 3] {
        // Run this on previous position instead, so you can see what axis wasn't intersecting before the collision.
        let mut collisions = [Direction::None; 3];

        let x_difference = self.position[0] - previous_other.position[0];
        if x_difference.abs() > self.half_size[0] + previous_other.half_size[0] {
            if x_difference.is_sign_positive() {
                collisions[0] = Direction::Positive;
            } else {
                collisions[0] = Direction::Negative;
            }
        }

        let y_difference = self.position[1] - previous_other.position[1];
        if y_difference.abs() > self.half_size[1] + previous_other.half_size[1] {
            if y_difference.is_sign_positive() {
                collisions[1] = Direction::Positive;
            } else {
                collisions[1] = Direction::Negative;
            }
        }

        let z_difference = self.position[2] - previous_other.position[2];
        if z_difference.abs() > self.half_size[2] + previous_other.half_size[2] {
            if z_difference.is_sign_positive() {
                collisions[2] = Direction::Positive;
            } else {
                collisions[2] = Direction::Negative;
            }
        }

        collisions
    }

    /// This won't work. Good luck.
    pub fn get_collision_normal_and_penetration(
        &self,
        other: &AabbCentredOrigin<T>,
    ) -> ([Direction; 3], T) {
        let mut normal = [Direction::None; 3];
        let mut smallest_penetration = T::MAX;
        let mut temp;

        temp = (self.position[0] + self.half_size[0]) - (other.position[0] - other.half_size[0]);
        //println!("+x penetration: {:?}", temp);
        if temp < smallest_penetration {
            normal = [Direction::Positive, Direction::None, Direction::None];
            smallest_penetration = temp;
        }

        temp = (self.position[1] + self.half_size[1]) - (other.position[1] - other.half_size[1]);
        //println!("+y penetration: {:?}", temp);
        if temp < smallest_penetration {
            normal = [Direction::None, Direction::Positive, Direction::None];
            smallest_penetration = temp;
        }

        temp = (self.position[2] + self.half_size[2]) - (other.position[2] - other.half_size[2]);
        //println!("+z penetration: {:?}", temp);
        if temp < smallest_penetration {
            normal = [Direction::None, Direction::None, Direction::Positive];
            smallest_penetration = temp;
        }

        temp = ((self.position[0] - self.half_size[0]) - (other.position[0] + other.half_size[0]))
            .abs();
        //println!("-x penetration: {:?}", temp);
        if temp < smallest_penetration {
            normal = [Direction::Negative, Direction::None, Direction::None];
            smallest_penetration = temp;
        }

        temp = ((self.position[1] - self.half_size[1]) - (other.position[1] + other.half_size[1]))
            .abs();
        //println!("-y penetration: {:?}", temp);
        if temp < smallest_penetration {
            normal = [Direction::None, Direction::Negative, Direction::None];
            smallest_penetration = temp;
        }

        temp = ((self.position[2] - self.half_size[2]) - (other.position[2] + other.half_size[2]))
            .abs();
        //println!("-z penetration: {:?}", temp);
        if temp < smallest_penetration {
            normal = [Direction::None, Direction::None, Direction::Negative];
            smallest_penetration = temp;
        }

        (normal, smallest_penetration)
    }
}

pub struct AabbMinMax<T>
where
    T: Number,
{
    pub min: [T; 3],
    pub max: [T; 3],
}

impl<T> AabbMinMax<T>
where
    T: Number,
{
    pub fn is_intersected_by_point(&self, point: [T; 3]) -> bool {
        point[0] >= self.min[0]
            && point[0] <= self.max[0]
            && point[1] >= self.min[1]
            && point[1] <= self.max[1]
            && point[2] >= self.min[2]
            && point[2] <= self.max[2]
    }

    pub fn is_intersected_by_aabb(&self, aabb: AabbMinMax<T>) -> bool {
        self.min[0] <= aabb.max[0]
            && self.max[0] >= aabb.min[0]
            && self.min[1] <= aabb.max[1]
            && self.max[1] >= aabb.min[1]
            && self.min[2] <= aabb.max[2]
            && self.max[2] >= aabb.min[2]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[bench]
    fn bench_aabb_top_left_origin_is_intersected_by_point(b: &mut Bencher) {
        b.iter(|| {
            let aabb = test::black_box(AabbTopLeftOrigin {
                position: [4.0954, 7.823, 2.2389],
                size: [2.0, 3.0, 8.78],
            });

            return aabb.is_intersected_by_point([5.0, 6.3, 2.1]);
        })
    }

    #[bench]
    fn bench_aabb_top_left_origin_is_intersected_by_aabb(b: &mut Bencher) {
        b.iter(|| {
            let aabb1 = test::black_box(AabbTopLeftOrigin {
                position: [3.0, 2.0, 2.2389],
                size: [10.0, 5.0, 8.78],
            });

            let aabb2 = test::black_box(AabbTopLeftOrigin {
                position: [3.0, 2.0, 3.2389],
                size: [10.0, 5.0, 2.8],
            });

            return aabb1.is_intersected_by_aabb(aabb2);
        })
    }

    #[bench]
    fn bench_aabb_centred_origin_is_intersected_by_point(b: &mut Bencher) {
        b.iter(|| {
            let aabb = test::black_box(AabbCentredOrigin {
                position: [4.0954, 7.823, 2.2389],
                half_size: [2.0, 3.0, 5.5],
            });

            return aabb.is_intersected_by_point([5.0, 6.3, 8.88]);
        })
    }

    #[bench]
    fn bench_aabb_centred_origin_is_intersected_by_aabb(b: &mut Bencher) {
        b.iter(|| {
            let aabb1 = test::black_box(AabbCentredOrigin {
                position: [3.0, 2.0, 2.2389],
                half_size: [10.0, 5.0, 7.1],
            });

            let aabb2 = test::black_box(AabbCentredOrigin {
                position: [3.0, 2.0, 23.1],
                half_size: [10.0, 5.0, 2.2389],
            });

            return aabb1.is_intersected_by_aabb(aabb2);
        })
    }

    #[bench]
    fn bench_aabb_min_max_is_intersected_by_point(b: &mut Bencher) {
        b.iter(|| {
            let aabb = test::black_box(AabbMinMax {
                min: [3.0, 2.0, 3.33],
                max: [10.0, 5.0, 4.01],
            });

            return aabb.is_intersected_by_point([5.0, 6.3, 2.2389]);
        })
    }

    #[bench]
    fn bench_aabb_min_max_is_intersected_by_aabb(b: &mut Bencher) {
        b.iter(|| {
            let aabb1 = test::black_box(AabbMinMax {
                min: [3.0, 2.0, 4.5],
                max: [10.0, 5.0, 30.0],
            });

            let aabb2 = test::black_box(AabbMinMax {
                min: [3.0, 2.0, 5.8],
                max: [10.0, 5.0, 22.22],
            });

            return aabb1.is_intersected_by_aabb(aabb2);
        })
    }
}
