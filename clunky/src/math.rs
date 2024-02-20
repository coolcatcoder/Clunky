// remember when doing matrix math transformations we do translate * rotate * scale unless you are doing world_to_camera, in which case it won't work, and you should try the reverse.
// All rights go to cgmath, I've just slighty tweaked their stuff.

// TODO:
// split functions into const and non-const where applicable
// Sort out the whole messy generic and non-generic stuff. Once const traits become stable, we must use them!

use const_soft_float::soft_f32::SoftF32;
use std::ops;
extern crate test;

pub trait Number:
    Copy
    + Clone
    + std::fmt::Debug
    + ops::Add<Output = Self>
    + ops::AddAssign
    + ops::Sub<Output = Self>
    + ops::SubAssign
    + ops::Mul<Output = Self>
    + ops::Div<Output = Self>
    + ops::Rem<Output = Self>
    + PartialOrd
    + std::fmt::Debug
    + From<u16>
{
    const ZERO: Self;
    const ONE: Self;
    fn abs(self) -> Self;
    fn to_usize(self) -> usize; // TODO: all number should be able to convert to all other numbers, but this will take a few minutes, and I'm lazy
    fn to_isize(self) -> isize;
    fn is_sign_positive(self) -> bool;
}

pub trait Float: Number + ops::Neg<Output = Self> {
    fn sqrt(self) -> Self;
    fn sin(self) -> Self;
    fn cos(self) -> Self;
    fn to_radians(self) -> Self;
    fn from_f32(value: f32) -> Self;
    fn from_f64(value: f64) -> Self;
}

impl Number for f32 {
    const ZERO: Self = 0.0;
    const ONE: Self = 1.0;
    #[inline]
    fn abs(self) -> Self {
        self.abs()
    }

    #[inline]
    fn to_usize(self) -> usize {
        self as usize
    }
    #[inline]
    fn to_isize(self) -> isize {
        self as isize
    }
    #[inline]
    fn is_sign_positive(self) -> bool {
        self.is_sign_positive()
    }
}

impl Float for f32 {
    #[inline]
    fn sqrt(self) -> Self {
        self.sqrt()
    }

    #[inline]
    fn sin(self) -> Self {
        self.sin()
    }

    #[inline]
    fn cos(self) -> Self {
        self.cos()
    }

    #[inline]
    fn to_radians(self) -> Self {
        self.to_radians()
    }

    #[inline]
    fn from_f32(value: f32) -> Self {
        value
    }

    #[inline]
    fn from_f64(value: f64) -> Self {
        value as f32
    }
}

impl Number for f64 {
    const ZERO: Self = 0.0;
    const ONE: Self = 1.0;
    #[inline]
    fn abs(self) -> Self {
        self.abs()
    }

    #[inline]
    fn to_usize(self) -> usize {
        self as usize
    }
    #[inline]
    fn to_isize(self) -> isize {
        self as isize
    }
    #[inline]
    fn is_sign_positive(self) -> bool {
        self.is_sign_positive()
    }
}

impl Float for f64 {
    #[inline]
    fn sqrt(self) -> Self {
        self.sqrt()
    }

    #[inline]
    fn sin(self) -> Self {
        self.sin()
    }

    #[inline]
    fn cos(self) -> Self {
        self.cos()
    }

    #[inline]
    fn to_radians(self) -> Self {
        self.to_radians()
    }

    #[inline]
    fn from_f32(value: f32) -> Self {
        value as f64
    }

    #[inline]
    fn from_f64(value: f64) -> Self {
        value
    }
}

impl Number for usize {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    #[inline]
    fn abs(self) -> Self {
        self
    }

    #[inline]
    fn to_usize(self) -> usize {
        self
    }

    #[inline]
    fn to_isize(self) -> isize {
        self as isize
    }

    #[inline]
    fn is_sign_positive(self) -> bool {
        true
    }
}

/// remember when doing matrix math transformations we do translate * rotate * scale unless you are doing world_to_camera, in which case it won't work, and you should try the reverse.
/// All rights go to cgmath, I've just slighty tweaked their stuff.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct Matrix4 {
    /// The first column of the matrix.
    pub x: [f32; 4],
    /// The second column of the matrix.
    pub y: [f32; 4],
    /// The third column of the matrix.
    pub z: [f32; 4],
    /// The fourth column of the matrix.
    pub w: [f32; 4],
}

impl Matrix4 {
    pub const IDENTITY: Matrix4 = Matrix4 {
        x: [1.0, 0.0, 0.0, 0.0],
        y: [0.0, 1.0, 0.0, 0.0],
        z: [0.0, 0.0, 1.0, 0.0],
        w: [0.0, 0.0, 0.0, 1.0],
    };

    pub const IDENTITY_AS_2D_ARRAY: [[f32; 4]; 4] = Matrix4::IDENTITY.as_2d_array();

    pub const fn as_2d_array(self) -> [[f32; 4]; 4] {
        [self.x, self.y, self.z, self.w]
    }

    pub const fn multiply(self, other: Matrix4) -> Matrix4 {
        // Could this be simd oneday?
        // Should this be inlined?
        let mut result = Matrix4 {
            x: [0.0; 4],
            y: [0.0; 4],
            z: [0.0; 4],
            w: [0.0; 4],
        };

        result.x[0] = self.x[0] * other.x[0]
            + self.y[0] * other.x[1]
            + self.z[0] * other.x[2]
            + self.w[0] * other.x[3];
        result.x[1] = self.x[1] * other.x[0]
            + self.y[1] * other.x[1]
            + self.z[1] * other.x[2]
            + self.w[1] * other.x[3];
        result.x[2] = self.x[2] * other.x[0]
            + self.y[2] * other.x[1]
            + self.z[2] * other.x[2]
            + self.w[2] * other.x[3];
        result.x[3] = self.x[3] * other.x[0]
            + self.y[3] * other.x[1]
            + self.z[3] * other.x[2]
            + self.w[3] * other.x[3];

        result.y[0] = self.x[0] * other.y[0]
            + self.y[0] * other.y[1]
            + self.z[0] * other.y[2]
            + self.w[0] * other.y[3];
        result.y[1] = self.x[1] * other.y[0]
            + self.y[1] * other.y[1]
            + self.z[1] * other.y[2]
            + self.w[1] * other.y[3];
        result.y[2] = self.x[2] * other.y[0]
            + self.y[2] * other.y[1]
            + self.z[2] * other.y[2]
            + self.w[2] * other.y[3];
        result.y[3] = self.x[3] * other.y[0]
            + self.y[3] * other.y[1]
            + self.z[3] * other.y[2]
            + self.w[3] * other.y[3];

        result.z[0] = self.x[0] * other.z[0]
            + self.y[0] * other.z[1]
            + self.z[0] * other.z[2]
            + self.w[0] * other.z[3];
        result.z[1] = self.x[1] * other.z[0]
            + self.y[1] * other.z[1]
            + self.z[1] * other.z[2]
            + self.w[1] * other.z[3];
        result.z[2] = self.x[2] * other.z[0]
            + self.y[2] * other.z[1]
            + self.z[2] * other.z[2]
            + self.w[2] * other.z[3];
        result.z[3] = self.x[3] * other.z[0]
            + self.y[3] * other.z[1]
            + self.z[3] * other.z[2]
            + self.w[3] * other.z[3];

        result.w[0] = self.x[0] * other.w[0]
            + self.y[0] * other.w[1]
            + self.z[0] * other.w[2]
            + self.w[0] * other.w[3];
        result.w[1] = self.x[1] * other.w[0]
            + self.y[1] * other.w[1]
            + self.z[1] * other.w[2]
            + self.w[1] * other.w[3];
        result.w[2] = self.x[2] * other.w[0]
            + self.y[2] * other.w[1]
            + self.z[2] * other.w[2]
            + self.w[2] * other.w[3];
        result.w[3] = self.x[3] * other.w[0]
            + self.y[3] * other.w[1]
            + self.z[3] * other.w[2]
            + self.w[3] * other.w[3];

        result
    }

    #[must_use = "Method constructs a new matrix."]
    #[inline]
    pub const fn from_translation(translation: [f32; 3]) -> Matrix4 {
        Matrix4 {
            x: [1.0, 0.0, 0.0, 0.0],
            y: [0.0, 1.0, 0.0, 0.0],
            z: [0.0, 0.0, 1.0, 0.0],
            w: [translation[0], translation[1], translation[2], 1.0],
        }
    }

    #[must_use = "Method constructs a new matrix."]
    #[inline]
    pub const fn from_scale(scale: [f32; 3]) -> Matrix4 {
        Matrix4 {
            x: [scale[0], 0.0, 0.0, 0.0],
            y: [0.0, scale[1], 0.0, 0.0],
            z: [0.0, 0.0, scale[2], 0.0],
            w: [0.0, 0.0, 0.0, 1.0],
        }
    }

    pub const fn from_angle_x_const(theta: Radians<f32>) -> Matrix4 {
        let theta_sin = SoftF32(theta.0).sin().to_f32();
        let theta_cos = SoftF32(theta.0).cos().to_f32();

        Matrix4 {
            x: [1.0, 0.0, 0.0, 0.0],
            y: [0.0, theta_cos, theta_sin, 0.0],
            z: [0.0, -theta_sin, theta_cos, 0.0],
            w: [0.0, 0.0, 0.0, 1.0],
        }
    }

    pub fn from_angle_x(theta: Radians<f32>) -> Matrix4 {
        let theta_sin = theta.0.sin();
        let theta_cos = theta.0.cos();

        Matrix4 {
            x: [1.0, 0.0, 0.0, 0.0],
            y: [0.0, theta_cos, theta_sin, 0.0],
            z: [0.0, -theta_sin, theta_cos, 0.0],
            w: [0.0, 0.0, 0.0, 1.0],
        }
    }

    pub const fn from_angle_y_const(theta: Radians<f32>) -> Matrix4 {
        let theta_sin = SoftF32(theta.0).sin().to_f32();
        let theta_cos = SoftF32(theta.0).cos().to_f32();

        Matrix4 {
            x: [theta_cos, 0.0, -theta_sin, 0.0],
            y: [0.0, 1.0, 0.0, 0.0],
            z: [theta_sin, 0.0, theta_cos, 0.0],
            w: [0.0, 0.0, 0.0, 1.0],
        }
    }

    pub fn from_angle_y(theta: Radians<f32>) -> Matrix4 {
        let theta_sin = theta.0.sin();
        let theta_cos = theta.0.cos();

        Matrix4 {
            x: [theta_cos, 0.0, -theta_sin, 0.0],
            y: [0.0, 1.0, 0.0, 0.0],
            z: [theta_sin, 0.0, theta_cos, 0.0],
            w: [0.0, 0.0, 0.0, 1.0],
        }
    }

    pub const fn from_angle_z_const(theta: Radians<f32>) -> Matrix4 {
        let theta_sin = SoftF32(theta.0).sin().to_f32();
        let theta_cos = SoftF32(theta.0).cos().to_f32();

        Matrix4 {
            x: [theta_cos, theta_sin, 0.0, 0.0],
            y: [-theta_sin, theta_cos, 0.0, 0.0],
            z: [0.0, 0.0, 1.0, 0.0],
            w: [0.0, 0.0, 0.0, 1.0],
        }
    }

    pub fn from_angle_z(theta: Radians<f32>) -> Matrix4 {
        let theta_sin = theta.0.sin();
        let theta_cos = theta.0.cos();

        Matrix4 {
            x: [theta_cos, theta_sin, 0.0, 0.0],
            y: [-theta_sin, theta_cos, 0.0, 0.0],
            z: [0.0, 0.0, 1.0, 0.0],
            w: [0.0, 0.0, 0.0, 1.0],
        }
    }

    // cannot be const, due to assert!() not be const sadly
    pub fn from_perspective(fovy: Radians<f32>, aspect: f32, near: f32, far: f32) -> Matrix4 {
        assert!(
            fovy.0 > 0.0,
            "The vertical field of view cannot be below zero, found: {:?}",
            fovy.0
        );
        assert!(
            fovy.0 < Degrees(180.0).to_radians().0,
            "The vertical field of view cannot be greater than a half turn, found: {:?}",
            fovy.0
        );
        assert!(
            aspect.abs() != 0.0,
            "The absolute aspect ratio cannot be zero, found: {:?}",
            aspect.abs()
        );
        assert!(
            near > 0.0,
            "The near plane distance cannot be below zero, found: {:?}",
            near
        );
        assert!(
            far > 0.0,
            "The far plane distance cannot be below zero, found: {:?}",
            far
        );
        assert!(
            far != near,
            "The far plane and near plane are too close, found: far: {:?}, near: {:?}",
            far,
            near
        );

        Matrix4::from_perspective_no_checks(fovy, aspect, near, far)
    }

    pub fn from_perspective_no_checks(
        fovy: Radians<f32>,
        aspect: f32,
        near: f32,
        far: f32,
    ) -> Matrix4 {
        let f = cot(fovy.0 / 2.0);

        Matrix4 {
            x: [f / aspect, 0.0, 0.0, 0.0],
            y: [0.0, f, 0.0, 0.0],
            z: [0.0, 0.0, (far + near) / (near - far), -1.0],
            w: [0.0, 0.0, (2.0 * far * near) / (near - far), 0.0],
        }
    }
}

impl ops::Mul for Matrix4 {
    type Output = Matrix4;
    fn mul(self, rhs: Self) -> Self::Output {
        self.multiply(rhs)
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Radians<T: Float>(pub T);

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Degrees<T: Float>(pub T);

impl<T: Float> Degrees<T> {
    #[inline]
    pub fn to_radians_test(&self) -> Radians<T> {
        Radians(self.0.to_radians())
    }
}

impl Degrees<f32> {
    #[inline]
    pub const fn to_radians(&self) -> Radians<f32> {
        Radians(self.0 * std::f32::consts::PI / 180.0)
    }
}

#[inline]
pub fn cot(theta: f32) -> f32 {
    1.0 / theta.tan()
}

#[inline]
pub const fn tan_const(theta: f32) -> f32 {
    SoftF32(theta).sin().0 / SoftF32(theta).cos().0
}

#[inline]
pub fn get_squared_magnitude_3d<T: Number>(vector: [T; 3]) -> T {
    vector[0] * vector[0] + vector[1] * vector[1] + vector[2] * vector[2]
}

#[inline]
pub fn get_magnitude_3d<T: Float>(vector: [T; 3]) -> T {
    get_squared_magnitude_3d(vector).sqrt()
}

/// Normalises a 3d number.
/// If the magnitude is 0, it will return 0 and not NaN.
#[inline]
#[must_use]
pub fn normalise_3d<T: Float>(vector: [T; 3]) -> [T; 3] {
    let magnitude = get_magnitude_3d(vector);

    // We can't let this function return NaN.
    if magnitude == T::ZERO {
        return [T::ZERO; 3];
    }

    [
        vector[0] / magnitude,
        vector[1] / magnitude,
        vector[2] / magnitude,
    ]
}

/// Multiply a 2d number by another 2d number.
/// \[lhs\[0] * rhs\[0], lhs\[1] * rhs\[1],]
#[inline]
#[must_use]
pub fn mul_2d<T: Number>(lhs: [T; 2], rhs: [T; 2]) -> [T; 2] {
    [lhs[0] * rhs[0], lhs[1] * rhs[1]]
}

/// Multiply a 2d number by a 1d number.
/// \[lhs\[0] * rhs, lhs\[1] * rhs,]
#[inline]
#[must_use]
pub fn mul_2d_by_1d<T: Number>(lhs: [T; 2], rhs: T) -> [T; 2] {
    [lhs[0] * rhs, lhs[1] * rhs]
}

#[inline]
pub fn add_3d<T: Number>(lhs: [T; 3], rhs: [T; 3]) -> [T; 3] {
    [lhs[0] + rhs[0], lhs[1] + rhs[1], lhs[2] + rhs[2]]
}

#[inline]
pub fn sub_3d<T: Number>(lhs: [T; 3], rhs: [T; 3]) -> [T; 3] {
    [lhs[0] - rhs[0], lhs[1] - rhs[1], lhs[2] - rhs[2]]
}

/// Multiply a 3d number by another 3d number.
/// \[lhs\[0] * rhs\[0], lhs\[1] * rhs\[1], lhs\[2] * rhs\[2],]
#[inline]
pub fn mul_3d<T: Number>(lhs: [T; 3], rhs: [T; 3]) -> [T; 3] {
    [lhs[0] * rhs[0], lhs[1] * rhs[1], lhs[2] * rhs[2]]
}

#[inline]
pub fn div_3d<T: Number>(lhs: [T; 3], rhs: [T; 3]) -> [T; 3] {
    [lhs[0] / rhs[0], lhs[1] / rhs[1], lhs[2] / rhs[2]]
}

/// Takes a 3d number and returns the 3d number with each of the axis' values being negative what they were previously.
#[inline]
#[must_use]
pub fn neg_3d<T: Number + ops::Neg<Output = T>>(value: [T; 3]) -> [T; 3] {
    [-value[0], -value[1], -value[2]]
}

/// Adds the 1d number to each of the 3d number's axis.
/// Name may change.
#[inline]
#[must_use]
pub fn add_3d_with_1d<T: Number>(lhs: [T; 3], rhs: T) -> [T; 3] {
    [lhs[0] + rhs, lhs[1] + rhs, lhs[2] + rhs]
}

/// Multiplies each axis of a 3d number by a 1d number.
#[inline]
#[must_use]
pub fn mul_3d_by_1d<T: Number>(lhs: [T; 3], rhs: T) -> [T; 3] {
    [lhs[0] * rhs, lhs[1] * rhs, lhs[2] * rhs]
}

/// Divides each axis of a 3d number by a 1d number.
#[inline]
#[must_use]
pub fn div_3d_by_1d<T: Number>(lhs: [T; 3], rhs: T) -> [T; 3] {
    [lhs[0] / rhs, lhs[1] / rhs, lhs[2] / rhs]
}

#[inline]
pub fn index_from_position_2d<T: Number>(position: [T; 2], width: T) -> T {
    position[1] * width + position[0]
}

#[inline]
pub fn position_from_index_2d<T: Number>(index: T, width: T) -> [T; 2] {
    [index % width, index / width]
}

#[inline]
pub fn index_from_position_3d<T: Number>(position: [T; 3], width: T, height: T) -> T {
    position[2] * width * height + position[1] * width + position[0]
}

#[inline]
pub fn position_from_index_3d<T: Number>(index: T, width: T, height: T) -> [T; 3] {
    let remaining = index % (width * height);
    [
        remaining % width,
        remaining / width,
        index / (width * height),
    ]
}

#[inline]
pub fn remap<T: Number>(value: T, original_range: ops::Range<T>, new_range: ops::Range<T>) -> T {
    new_range.start
        + (value - original_range.start) * (new_range.end - new_range.start)
            / (original_range.end - original_range.start)
}

pub fn rotate_2d<T: Float>(position: [T; 2], theta: T) -> [T; 2] {
    // TODO: Make clear this is radians.
    let theta_cos = theta.cos();
    let theta_sin = theta.sin();
    [
        position[0] * theta_cos - position[1] * theta_sin,
        position[1] * theta_cos + position[0] * theta_sin,
    ]
}

/// Calculates the dot product of 2 3d numbers.
pub fn dot<T: Number>(lhs: [T; 3], rhs: [T; 3]) -> T {
    lhs[0] * rhs[0] + lhs[1] * rhs[1] + lhs[2] * rhs[2]
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[bench]
    fn bench_from_angle_x(b: &mut Bencher) {
        b.iter(|| return Matrix4::from_angle_x(Degrees(test::black_box(90.0)).to_radians()))
    }

    #[bench]
    fn bench_from_angle_x_const(b: &mut Bencher) {
        b.iter(|| return Matrix4::from_angle_x_const(Degrees(test::black_box(90.0)).to_radians()))
    }

    #[bench]
    fn bench_to_radians(b: &mut Bencher) {
        b.iter(|| return test::black_box(Degrees(90.0)).to_radians())
    }

    #[bench]
    fn bench_to_radians_test(b: &mut Bencher) {
        b.iter(|| return test::black_box(Degrees(90.0)).to_radians_test())
    }
}
