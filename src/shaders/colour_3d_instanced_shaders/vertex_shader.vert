#version 450

// vertex data
layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;

// instance data
layout(location = 2) in vec4 colour;
layout(location = 3) in vec4 model_to_world_0;
layout(location = 4) in vec4 model_to_world_1;
layout(location = 5) in vec4 model_to_world_2;
layout(location = 6) in vec4 model_to_world_3;

layout(location = 0) out vec3 normal_out;
layout(location = 1) out vec4 colour_out;
layout(location = 2) out vec3 fragment_position;
layout(location = 3) out vec3 camera_position;

layout(set = 0, binding = 0) uniform CameraData3D {
    vec3 position;
    mat4 camera_to_clip;
    mat4 world_to_camera;
} camera;

void main() {
    mat4 model_to_world = mat4(model_to_world_0, model_to_world_1, model_to_world_2, model_to_world_3);
    vec4 temp = model_to_world * vec4(position, 1.0);
    temp = camera.world_to_camera * temp;
    gl_Position = camera.camera_to_clip * temp;

    //normal_out = transpose(inverse(mat3(camera.world_to_camera))) * normal; // TODO: inversing matrices is expensive. Do on the cpu, store as part of the instance. Also is it really world_to_camera?
    normal_out = transpose(inverse(mat3(model_to_world))) * normal;
    colour_out = colour;
    fragment_position = vec3(model_to_world * vec4(position, 1.0));
    camera_position = camera.position;
}