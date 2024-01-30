pub mod colour_3d_instanced_vertex_shader {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/shaders/colour_3d_instanced_shaders/vertex_shader.vert",
    }
}

pub mod colour_3d_instanced_fragment_shader {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/colour_3d_instanced_shaders/fragment_shader.frag",
    }
}

pub mod uv_3d_instanced_vertex_shader {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/shaders/uv_3d_instanced_shaders/vertex_shader.vert",
    }
}

pub mod uv_3d_instanced_fragment_shader {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/uv_3d_instanced_shaders/fragment_shader.frag",
    }
}

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
