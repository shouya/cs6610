#version 330 core

layout(location = 0) in vec3 pos;
layout(location = 1) in vec2 uv;
layout(location = 2) in vec3 n;

out vec2 uv_t; // in texture space
out vec3 pos_v; // in view space
out vec3 n_v; // in view space

out vec3 orig_pos; // in model space

uniform mat4 mv, mvp, v, m;
uniform mat3 mv3, mv_n; // for transforming vertex normals

uniform vec3 light_dir_or_loc;
uniform int light_type;

out vec3 light_dir_raw;

uniform mat4 shadow_transform; // map from world space to shadow texture space
out vec3 shadow_pos; // in shadow texture space

void main()
{
  orig_pos = pos;
  gl_Position = mvp * vec4(pos, 1.0);

  pos_v = (mv * vec4(pos, 1.0)).xyz;
  n_v = mv_n * n;

  uv_t = uv;

  shadow_pos = (shadow_transform * m * vec4(pos, 1.0)).xyz;
  shadow_pos.z -= 0.001;

  switch (light_type) {
  case 0: // directional light
    light_dir_raw = normalize((v * vec4(light_dir_or_loc, 0.0)).xyz);
    break;
  case 1: // point light
  case 2: // spot light
    vec3 dir = normalize(light_dir_or_loc - pos_v);
    light_dir_raw = normalize((v * vec4(dir, 0.0)).xyz);
    break;
  }
}
