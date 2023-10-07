#version 450

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 uv;
layout(location = 0) out vec2 uv_out;

layout(set = 0, binding = 0) uniform Data {
    float scale;
    float camera_scale;
    vec2 camera_position;

    float brightness;
} uniforms;

void main() {
    //gl_Position = vec4((position.x - uniforms.camera_position.x) * uniforms.camera_scale, (position.y - uniforms.camera_position.y) * uniforms.camera_scale * uniforms.scale, 0.0, 1.0);
    gl_Position = vec4((position.x - uniforms.camera_position.x) * uniforms.camera_scale * uniforms.scale, (position.y - uniforms.camera_position.y) * uniforms.camera_scale, 0.0, 1.0);
    uv_out = uv;
}