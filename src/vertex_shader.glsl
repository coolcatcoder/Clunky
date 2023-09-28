#version 450

layout(location = 0) in vec2 position;

layout(set = 0, binding = 0) uniform Data {
    float scale;
} uniforms;

void main() {
    gl_Position = vec4(position * uniforms.scale, 0.0, 1.0);
}