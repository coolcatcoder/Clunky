#version 450

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 uv;
layout(location = 0) out vec2 uv_out;

layout(set = 0, binding = 0) uniform Data {
    float scale;
} uniforms;

void main() {
    gl_Position = vec4(position.x * 0.5, position.y * uniforms.scale * 0.5, 0.0, 1.0);
    uv_out = uv;
}