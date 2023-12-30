// remember when doing matrix math transformations we do translate * rotate * scale
// All rights go to cgmath, I've just slighty tweaked their stuff.

use const_soft_float::soft_f32::SoftF32;

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

// Replace with something faster when possible.
// impl Mul<Matrix4> for Matrix4 {
//     type Output = Matrix4;
//     const fn mul(self, rhs: Matrix4) -> Self::Output {
//         let lhs_array = [self.x, self.y, self.z, self.w];
//         let rhs_array = [rhs.x, rhs.y, rhs.z, rhs.w];
//         let mut final_array = [[0.0, 0.0, 0.0, 0.0],[0.0, 0.0, 0.0, 0.0],[0.0, 0.0, 0.0, 0.0],[0.0, 0.0, 0.0, 0.0]];
//         let mut counter_inner = 0;
//         let mut counter_outer = 0;

//         for i in 0..lhs_array.len() {
//             for j in 0..rhs_array[0].len() {
//                 let mut product = 0.0;

//                 for v in 0..rhs_array[i].len() {
//                     product += lhs_array[i][v] * rhs_array[v][j];
//                 }
//                 final_array[counter_outer][counter_inner] = product;
//                 counter_inner += 1;
//             }
//             counter_outer += 1;
//         }

//         Matrix4 {
//             x: final_array[0],
//             y: final_array[1],
//             z: final_array[2],
//             w: final_array[3],
//         }
//     }
// }

impl Matrix4 {
    pub const fn multiply(self, other: Matrix4) -> Matrix4 {
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
        #[cfg_attr(rustfmt, rustfmt_skip)]
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
        #[cfg_attr(rustfmt, rustfmt_skip)]
        Matrix4 {
            x: [scale[0], 0.0, 0.0, 0.0],
            y: [0.0, scale[1], 0.0, 0.0],
            z: [0.0, 0.0, scale[2], 0.0],
            w: [0.0, 0.0, 0.0, 1.0],
        }
    }

    pub const fn from_angle_x(theta: Radians) -> Matrix4 {
        let theta_sin = SoftF32(theta.0).sin().to_f32();
        let theta_cos = SoftF32(theta.0).cos().to_f32();

        #[cfg_attr(rustfmt, rustfmt_skip)]
        Matrix4 {
            x: [1.0, 0.0, 0.0, 0.0],
            y: [0.0, theta_cos, theta_sin, 0.0],
            z: [0.0, -theta_sin, theta_cos, 0.0],
            w: [0.0, 0.0, 0.0, 1.0],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Radians(pub f32);

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Degrees(pub f32);

impl Degrees {
    #[inline]
    pub const fn to_radians(&self) -> Radians {
        Radians(self.0 * std::f32::consts::PI / 180.0)
    }
}

// #[repr(C)]
// #[derive(Copy, Clone)]
// pub struct Vector4 {
//     pub x: f32,
//     pub y: f32,
//     pub z: f32,
//     pub w: f32,
// }

// impl Vector4 { // Consider removing and just using a [f32; 4] for everything instead
//     #[inline]
//     pub const fn from_float_array(float_array: [f32; 4]) -> Vector4 {
//         Vector4 {
//             x: float_array[0],
//             y: float_array[1],
//             z: float_array[2],
//             w: float_array[3],
//         }
//     }
// }
