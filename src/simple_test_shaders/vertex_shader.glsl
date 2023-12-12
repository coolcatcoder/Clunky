#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec4 colour;

layout(location = 0) out vec4 colour_out;

void main() {
    gl_Position = vec4(position.x, position.y, position.z, 1.0);
    colour_out = colour;
}