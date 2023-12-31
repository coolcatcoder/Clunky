#version 450

layout(location = 0) in vec2 uv;
layout(location = 0) out vec4 f_color;

layout(set = 1, binding = 0) uniform sampler texture_sampler;
layout(set = 1, binding = 1) uniform texture2D image;

void main() {
    f_color = texture(sampler2D(image, texture_sampler), uv);
}