#version 450

layout(location = 0) in vec2 uv;
layout(location = 1) in vec4 colour;
layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 0) uniform Data {
    float scale;
    float camera_scale;
    vec2 camera_position;

    float brightness;
} uniforms;

layout(set = 1, binding = 1) uniform sampler2D textures;

void main() { // surely there is a way to simplify this, right???
    f_color = vec4(colour.x * uniforms.brightness / uniforms.brightness, colour.y * uniforms.brightness / uniforms.brightness, colour.z * uniforms.brightness / uniforms.brightness, texture(textures,uv).w);
}