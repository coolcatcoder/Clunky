#version 450

layout(location = 0) in vec2 uv;
layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 0) uniform Data {
    float scale;
    float camera_scale;
    vec2 camera_position;

    float brightness;
} uniforms;

layout(set = 1, binding = 1) uniform sampler2D textures;

void main() {
    f_color = texture(textures,uv) * vec4(uniforms.brightness, uniforms.brightness, uniforms.brightness, 1.0);
}