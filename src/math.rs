use cgmath::Matrix4;

// TODO: face the rotation beast
pub fn full_transformation_to_matrix(scale: [f32; 3], translation: [f32; 3]) -> Matrix4<f32> {
    Matrix4::from_nonuniform_scale(scale[0], scale[1], scale[2])
        * Matrix4::from_translation(translation.into())
}
