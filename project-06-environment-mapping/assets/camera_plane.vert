#version 330 core

layout(location = 0) in vec2 clip_pos;

uniform mat4 view_proj_inv;
uniform samplerCube env_map;

out vec3 world_pos;

void main()
{
  vec4 world_pos4 = view_proj_inv * vec4(clip_pos.xy, 1.0, 1.0);
  world_pos = world_pos4.xyz / world_pos4.w;

  // set depth to be 1.0 (far plane)
  gl_Position = vec4(clip_pos.xy, 1.0, 1.0);
}
