#version 450

layout(location = 0) in vec2 position;

layout(set = 0, binding = 0) uniform Data {
    float scale;
} uniforms;

void main() {
    gl_Position = vec4(position.x * 0.5, position.y * uniforms.scale * 0.5, 0.0, 1.0);
}