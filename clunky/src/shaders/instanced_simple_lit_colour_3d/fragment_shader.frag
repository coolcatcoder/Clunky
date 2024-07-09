#version 450

layout(location = 0) in vec3 normal;
layout(location = 1) in vec4 colour;
layout(location = 2) in vec3 fragment_position;
layout(location = 3) in vec3 camera_position;

layout(location = 4) in float ambient_strength;
layout(location = 5) in float specular_strength;

layout(location = 6) in vec3 light_colour;
layout(location = 7) in vec3 light_position;

layout(location = 0) out vec4 f_color;

//const float AMBIENT_STRENGTH = 0.1;
//const float SPECULAR_STRENGTH = 0.5;

//const vec3 LIGHT_COLOUR = vec3(1.0, 1.0, 1.0);
//const vec3 LIGHT_POSITION = vec3(0.0, 0.0, 0.0);

void main() {
    // ambient:
    vec3 ambient = ambient_strength * light_colour;

    // diffuse:
    vec3 normal = normalize(normal);
    vec3 light_direction = normalize(light_position - fragment_position);
    vec3 diffuse = max(dot(normal, light_direction), 0.0) * light_colour;
    
    // specular
    vec3 view_direction = normalize(camera_position - fragment_position);
    vec3 reflect_direction = reflect(-light_direction, normal);
    vec3 specular = specular_strength * pow(max(dot(view_direction, reflect_direction), 0.0), 32) * light_colour;

    f_color = vec4((ambient + diffuse + specular) * colour.xyz, colour.w);
}