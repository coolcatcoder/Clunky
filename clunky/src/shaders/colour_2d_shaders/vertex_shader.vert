#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec4 colour;

layout(set = 0, binding = 0) uniform CameraData2D {
    float aspect_ratio;
    float scale;
    vec2 position;
} camera;

layout(location = 0) out vec4 colour_out;

void main() {
    gl_Position = vec4((position.x - camera.position.x) * camera.scale * camera.aspect_ratio, (position.y - camera.position.y) * camera.scale, position.z, 1.0);
    colour_out = colour;
}