#version 450

// vertex
layout(location = 0) in vec2 position;
layout(location = 1) in vec2 uv;

// instance
layout(location = 2) in vec2 uv_offset;
layout(location = 3) in vec4 colour;
layout(location = 4) in float smoothing;
layout(location = 5) in float inverse_boldness;
layout(location = 6) in vec3 model_to_world_0;
layout(location = 7) in vec3 model_to_world_1;
layout(location = 8) in vec3 model_to_world_2;

// passed to fragment shader
layout(location = 0) out vec2 uv_out;
layout(location = 1) out vec4 colour_out;
layout(location = 2) out float smoothing_out;
layout(location = 3) out float inverse_boldness_out;

layout(set = 0, binding = 0) uniform Font {
    vec2 glyph_size;
    float aspect_ratio;
} font;

void main() {
    mat3 model_to_world = mat3(model_to_world_0, model_to_world_1, model_to_world_2);
    gl_Position.xyz = model_to_world * vec3(position, 1.0);
    gl_Position.y *= font.aspect_ratio;
    gl_Position.w = 1.0;

    uv_out = uv * font.glyph_size + uv_offset;
    colour_out = colour;
    smoothing_out = smoothing;
    inverse_boldness_out = inverse_boldness;
}