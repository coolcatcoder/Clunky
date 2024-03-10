#![feature(const_fn_floating_point_arithmetic)] // Required for math for now.
#![feature(test)]
#![feature(vec_push_within_capacity)]
#![doc = include_str!("../../README.md")]
//#![warn(missing_docs)] // Uncomment this when you want to do some documenting. Otherwise leave commented.

pub mod buffer_contents;

pub mod lost_code;

pub mod math;

#[allow(clippy::excessive_precision)]
#[allow(clippy::double_neg)]
pub mod meshes;

pub mod physics;

pub mod random_generation;

pub mod rendering;

pub mod shaders;

mod vertex_shader_map {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/map_shaders/vertex_shader.glsl",
    }
}

mod fragment_shader_map {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/map_shaders/fragment_shader.glsl",
    }
}

mod vertex_shader_ui {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/ui_shaders/vertex_shader.glsl",
    }
}

mod fragment_shader_ui {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/ui_shaders/fragment_shader.frag",
    }
}

mod vertex_shader_test {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/test_shaders/vertex_shader.glsl",
    }
}

mod fragment_shader_test {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/test_shaders/fragment_shader.glsl",
    }
}
