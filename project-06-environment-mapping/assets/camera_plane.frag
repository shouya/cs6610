#version 330 core

layout(location = 0) out vec4 color;

uniform samplerCube env_map;

in vec3 world_pos;

void main() {
  // normalization not needed.
  color = texture(env_map, world_pos);
}
