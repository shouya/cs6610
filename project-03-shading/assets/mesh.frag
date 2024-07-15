#version 330 core

layout(location = 0) out vec4 color;
uniform vec3 clr;

in vec3 uv; // in texture space
in vec3 pos_v; // in view space
in vec3 n; // in view space

void main() {
  color = vec4(clr, 1.0);
}
