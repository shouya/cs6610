#version 330 core

layout(location = 0) out vec4 color;
uniform vec3 clr;

in vec2 out_uv; // in texture space
in vec3 out_pos; // in view space
in vec3 out_n; // in view space

void main() {
  // color = vec4(clr, 1.0);
  color = vec4(out_n, 1.0);
}
