#version 450

layout(location = 0) in vec2 uv;

layout(set = 0, binding = 0) uniform sampler texture_sampler;
layout(set = 0, binding = 1) uniform texture2D image;

layout(location = 0) out vec4 f_color;

void main() {
    f_color = texture(sampler2D(image, texture_sampler), uv);

    // Debug uv
    //f_color = vec4(uv[0], 0.0, uv[1], 1.0);
}