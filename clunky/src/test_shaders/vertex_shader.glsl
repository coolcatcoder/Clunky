#version 450

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 uv;

layout(location = 2) in vec3 position_offset;
layout(location = 3) in vec2 scale;
layout(location = 4) in vec2 uv_centre;

layout(location = 0) out vec2 uv_out;

layout(set = 0, binding = 0) uniform Data {
    float scale;
    float camera_scale;
    vec2 camera_position;

    float brightness;
} uniforms;

void main() {
    //gl_Position = vec4((position.x - uniforms.camera_position.x) * uniforms.camera_scale, (position.y - uniforms.camera_position.y) * uniforms.camera_scale * uniforms.scale, 0.0, 1.0);
    gl_Position = vec4((position.x * scale.x + position_offset.x - uniforms.camera_position.x) * uniforms.camera_scale * uniforms.scale, (position.y * scale.y + position_offset.y - uniforms.camera_position.y) * uniforms.camera_scale, position_offset.z, 1.0);
    uv_out = uv + uv_centre;
}