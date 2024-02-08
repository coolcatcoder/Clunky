#version 450

// vertex data
layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 uv;

// instance data
layout(location = 3) in vec2 uv_offset;
layout(location = 4) in vec4 model_to_world_0;
layout(location = 5) in vec4 model_to_world_1;
layout(location = 6) in vec4 model_to_world_2;
layout(location = 7) in vec4 model_to_world_3;

layout(location = 0) out vec3 normal_out;
layout(location = 1) out vec2 uv_out;
layout(location = 2) out vec3 fragment_position;
layout(location = 3) out vec3 camera_position;

layout(location = 4) out float ambient_strength;
layout(location = 5) out float specular_strength;

layout(location = 6) out vec3 light_colour;
layout(location = 7) out vec3 light_position;

layout(set = 0, binding = 0) uniform CameraData3D {
    vec3 position;
    
    float ambient_strength;
    float specular_strength;
    vec3 light_colour;
    vec3 light_position;

    mat4 camera_to_clip;
    mat4 world_to_camera;
} camera;

void main() {
    mat4 model_to_world = mat4(model_to_world_0, model_to_world_1, model_to_world_2, model_to_world_3);
    vec4 temp = model_to_world * vec4(position, 1.0);
    temp = camera.world_to_camera * temp;
    gl_Position = camera.camera_to_clip * temp;

    normal_out = transpose(inverse(mat3(model_to_world))) * normal; // TODO: inversing matrices is expensive. Do on the cpu, store as part of the instance.
    uv_out = uv + uv_offset;
    fragment_position = vec3(model_to_world * vec4(position, 1.0));
    camera_position = camera.position;

    ambient_strength = camera.ambient_strength;
    specular_strength = camera.specular_strength;
    light_colour = camera.light_colour;
    light_position = camera.light_position;
}