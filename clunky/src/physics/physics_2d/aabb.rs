use crate::math::{Number, SignedNumber};
extern crate test;

pub struct AabbTopLeftOrigin<T>
where
    T: Number,
{
    pub position: [T; 2],
    pub size: [T; 2],
}

impl<T> AabbTopLeftOrigin<T>
where
    T: Number,
{
    pub fn is_intersected_by_point(&self, point: [T; 2]) -> bool {
        point[0] < self.position[0] + self.size[0]
            && point[0] > self.position[0]
            && point[1] < self.position[1] + self.size[1]
            && point[1] > self.position[1]
    }

    pub fn is_intersected_by_aabb(&self, aabb: AabbTopLeftOrigin<T>) -> bool {
        self.position[0] < aabb.position[0] + aabb.size[0]
            && self.position[0] + self.size[0] > aabb.position[0]
            && self.position[1] < aabb.position[1] + aabb.size[1]
            && self.position[1] + self.size[1] > aabb.position[1]
    }
}

#[derive(Copy, Clone)]
pub struct AabbCentredOrigin<T>
where
    T: Number,
{
    pub position: [T; 2],
    pub half_size: [T; 2],
}

impl<T> AabbCentredOrigin<T>
where
    T: SignedNumber, // I need to split these up into signed and unsigned versions, that make use of abs sometimes.
{
    pub fn is_intersected_by_point(&self, point: [T; 2]) -> bool {
        if (self.position[0] - point[0]).abs() > self.half_size[0] {
            return false;
        }
        if (self.position[1] - point[1]).abs() > self.half_size[1] {
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
        true
    }
}

pub struct AabbMinMax<T>
where
    T: Number,
{
    pub min: [T; 2],
    pub max: [T; 2],
}

impl<T> AabbMinMax<T>
where
    T: Number,
{
    pub fn is_intersected_by_point(&self, point: [T; 2]) -> bool {
        point[0] >= self.min[0]
            && point[0] <= self.max[0]
            && point[1] >= self.min[1]
            && point[1] <= self.max[1]
    }

    pub fn is_intersected_by_aabb(&self, aabb: AabbMinMax<T>) -> bool {
        self.min[0] <= aabb.max[0]
            && self.max[0] >= aabb.min[0]
            && self.min[1] <= aabb.max[1]
            && self.max[1] >= aabb.min[1]
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
                position: [4.0954, 7.823],
                size: [2.0, 3.0],
            });

            return aabb.is_intersected_by_point([5.0, 6.3]);
        })
    }

    #[bench]
    fn bench_aabb_top_left_origin_is_intersected_by_aabb(b: &mut Bencher) {
        b.iter(|| {
            let aabb1 = test::black_box(AabbTopLeftOrigin {
                position: [3.0, 2.0],
                size: [10.0, 5.0],
            });

            let aabb2 = test::black_box(AabbTopLeftOrigin {
                position: [3.0, 2.0],
                size: [10.0, 5.0],
            });

            return aabb1.is_intersected_by_aabb(aabb2);
        })
    }

    #[bench]
    fn bench_aabb_centred_origin_is_intersected_by_point(b: &mut Bencher) {
        b.iter(|| {
            let aabb = test::black_box(AabbCentredOrigin {
                position: [4.0954, 7.823],
                half_size: [2.0, 3.0],
            });

            return aabb.is_intersected_by_point([5.0, 6.3]);
        })
    }

    #[bench]
    fn bench_aabb_centred_origin_is_intersected_by_aabb(b: &mut Bencher) {
        b.iter(|| {
            let aabb1 = test::black_box(AabbCentredOrigin {
                position: [3.0, 2.0],
                half_size: [10.0, 5.0],
            });

            let aabb2 = test::black_box(AabbCentredOrigin {
                position: [3.0, 2.0],
                half_size: [10.0, 5.0],
            });

            return aabb1.is_intersected_by_aabb(aabb2);
        })
    }

    #[bench]
    fn bench_aabb_min_max_is_intersected_by_point(b: &mut Bencher) {
        b.iter(|| {
            let aabb = test::black_box(AabbMinMax {
                min: [3.0, 2.0],
                max: [10.0, 5.0],
            });

            return aabb.is_intersected_by_point([5.0, 6.3]);
        })
    }

    #[bench]
    fn bench_aabb_min_max_is_intersected_by_aabb(b: &mut Bencher) {
        b.iter(|| {
            let aabb1 = test::black_box(AabbMinMax {
                min: [3.0, 2.0],
                max: [10.0, 5.0],
            });

            let aabb2 = test::black_box(AabbMinMax {
                min: [3.0, 2.0],
                max: [10.0, 5.0],
            });

            return aabb1.is_intersected_by_aabb(aabb2);
        })
    }
}
