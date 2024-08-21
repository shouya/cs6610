#version 410 core

layout(location = 0) in vec3 pos;
layout(location = 1) in vec2 uv;

out VertData {
  vec3 pos;
  vec2 uv;
} to_tcs;

void main()
{
  to_tcs.pos = pos;
  to_tcs.uv = uv;
}
