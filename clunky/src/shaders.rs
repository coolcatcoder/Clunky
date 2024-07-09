pub mod instanced_simple_lit_colour_3d;
pub mod instanced_simple_lit_uv_3d;
pub mod instanced_text_sdf;
pub mod instanced_unlit_uv_2d_stretch;
pub mod simple_lit_colour_3d;

pub mod colour_2d_vertex_shader {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/shaders/colour_2d_shaders/vertex_shader.vert",
    }
}

pub mod colour_2d_fragment_shader {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/colour_2d_shaders/fragment_shader.frag",
    }
}

pub mod uv_2d_vertex_shader {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/shaders/uv_2d_shaders/vertex_shader.vert",
    }
}

pub mod uv_2d_fragment_shader {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/uv_2d_shaders/fragment_shader.frag",
    }
}
