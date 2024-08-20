#version 410 core

layout(location = 0) out vec4 color;

in vec2 texCoord;

void main() {
  color = vec4(0.5, 0.5, 1.0, 1.0);
}
