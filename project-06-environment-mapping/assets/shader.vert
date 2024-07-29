#version 330 core

layout(location = 0) in vec3 pos;
layout(location = 1) in vec2 uv;
layout(location = 2) in vec3 n;

out vec2 uv_t; // in texture space
out vec3 pos_v; // in view space
out vec3 n_v; // in view space

uniform mat4 mv, mvp;
uniform mat3 mv3, mv_n; // for transforming vertex normals

uniform sampler2D map_bump;
uniform uint use_map_bump;

void main()
{
  vec3 pos = pos;
  if (use_map_bump == 1u) {
    pos += n * texture(map_bump, uv).x;
  }

  gl_Position = mvp * vec4(pos, 1.0);

  pos_v = (mv * vec4(pos, 1.0)).xyz;
  n_v = mv_n * n;

  uv_t = uv;
}
