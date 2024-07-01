#version 450

// vertex
layout(location = 0) in vec2 position;
layout(location = 1) in vec2 uv;

// instance
layout(location = 2) in vec2 uv_offset;
layout(location = 3) in float depth;
layout(location = 4) in vec3 model_to_world_0;
layout(location = 5) in vec3 model_to_world_1;
layout(location = 6) in vec3 model_to_world_2;

// passed to fragment shader
layout(location = 0) out vec2 uv_out;

void main() {
    mat3 model_to_world = mat3(model_to_world_0, model_to_world_1, model_to_world_2);
    gl_Position.xyz = model_to_world * vec3(position, 1.0);
    gl_Position.z = depth;

    uv_out = uv + uv_offset;
}