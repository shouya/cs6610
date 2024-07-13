#version 330 core

layout(location = 0) out vec4 color;
uniform vec3 clr;

void main() {
  color = vec4(clr, 1.0);
}
