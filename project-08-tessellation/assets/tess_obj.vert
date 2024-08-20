#version 410 core

layout(location = 0) in vec3 pos;
layout(location = 1) in vec2 uv;

out vec2 texCoord;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

// in world space
uniform vec3 light_dir_or_loc;
uniform int light_type;

// in world space
out vec3 light_dir_raw;

// in shadow space
// in vec4 shadow_pos;
// uniform sampler2DShadow shadow_map;

void main()
{
  vec4 pos_w = model * vec4(pos, 1.0);
  gl_Position = projection * view * pos_w;
  texCoord = uv;

  switch (light_type) {
  case 0: // directional light
    light_dir_raw = normalize((view * vec4(light_dir_or_loc, 0.0)).xyz);
    break;
  case 1: // spot light
    vec3 dir = normalize(light_dir_or_loc - pos_w.xyz);
    light_dir_raw = normalize((view * vec4(dir, 0.0)).xyz);
    break;
  }
}
