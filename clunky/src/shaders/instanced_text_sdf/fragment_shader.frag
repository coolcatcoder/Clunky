#version 450

layout(location = 0) in vec2 uv;
layout(location = 1) in vec4 colour;
layout(location = 2) in float smoothing;
layout(location = 3) in float inverse_boldness;

layout(set = 1, binding = 0) uniform sampler texture_sampler;
layout(set = 1, binding = 1) uniform texture2D image;

layout(location = 0) out vec4 f_color;

void main() {
    float distance = texture(sampler2D(image, texture_sampler), uv).x;

    float alpha = smoothstep(inverse_boldness - smoothing, inverse_boldness + smoothing, distance);

    f_color = vec4(colour.xyz, alpha * colour.w);

    // Debug uv
    //f_color = vec4(uv[0], 0.0, uv[1], 1.0);
}