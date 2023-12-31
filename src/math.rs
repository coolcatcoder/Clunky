// remember when doing matrix math transformations we do translate * rotate * scale unless you are doing world_to_camera, in which case it won't work, and you should try the reverse.
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

impl Matrix4 {
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

    pub const fn from_angle_x(theta: Radians) -> Matrix4 {
        let theta_sin = SoftF32(theta.0).sin().to_f32();
        let theta_cos = SoftF32(theta.0).cos().to_f32();

        Matrix4 {
            x: [1.0, 0.0, 0.0, 0.0],
            y: [0.0, theta_cos, theta_sin, 0.0],
            z: [0.0, -theta_sin, theta_cos, 0.0],
            w: [0.0, 0.0, 0.0, 1.0],
        }
    }

    pub const fn from_angle_y(theta: Radians) -> Matrix4 {
        let theta_sin = SoftF32(theta.0).sin().to_f32();
        let theta_cos = SoftF32(theta.0).cos().to_f32();

        Matrix4 {
            x: [theta_cos, 0.0, -theta_sin, 0.0],
            y: [0.0, 1.0, 0.0, 0.0],
            z: [theta_sin, 0.0, theta_cos, 0.0],
            w: [0.0, 0.0, 0.0, 1.0],
        }
    }

    pub const fn from_angle_z(theta: Radians) -> Matrix4 {
        let theta_sin = SoftF32(theta.0).sin().to_f32();
        let theta_cos = SoftF32(theta.0).cos().to_f32();

        Matrix4 {
            x: [theta_cos, theta_sin, 0.0, 0.0],
            y: [-theta_sin, theta_cos, 0.0, 0.0],
            z: [0.0, 0.0, 1.0, 0.0],
            w: [0.0, 0.0, 0.0, 1.0],
        }
    }

    // cannot be const, due to assert!() not be const sadly
    pub fn from_perspective(fovy: Radians, aspect: f32, near: f32, far: f32) -> Matrix4 {
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

    pub const fn from_perspective_no_checks(
        fovy: Radians,
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

#[inline]
pub const fn cot(theta: f32) -> f32 {
    1.0 / tan(theta)
}

#[inline]
pub const fn tan(theta: f32) -> f32 {
    SoftF32(theta).sin().0 / SoftF32(theta).cos().0
}
