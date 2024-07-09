#version 450

// vertex data
layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec4 colour;

layout(location = 0) out vec3 normal_out;
layout(location = 1) out vec4 colour_out;
layout(location = 2) out vec3 fragment_position;
layout(location = 3) out vec3 camera_position;

layout(location = 4) out float ambient_strength;
layout(location = 5) out float specular_strength;

layout(location = 6) out vec3 light_colour;
layout(location = 7) out vec3 light_position;

layout(set = 0, binding = 0) uniform Camera {
    vec3 position;
    
    float ambient_strength;
    float specular_strength;
    vec3 light_colour;
    vec3 light_position;

    mat4 camera_to_clip;
    mat4 world_to_camera;
} camera;

void main() {
    vec4 temp = vec4(position, 1.0);
    temp = camera.world_to_camera * temp;
    gl_Position = camera.camera_to_clip * temp;

    normal_out = normal;
    colour_out = colour;
    fragment_position = position;
    camera_position = camera.position;

    ambient_strength = camera.ambient_strength;
    specular_strength = camera.specular_strength;
    light_colour = camera.light_colour;
    light_position = camera.light_position;
}