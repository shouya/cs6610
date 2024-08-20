#version 330 core

layout(location = 0) out vec4 color;

in vec2 frag_uv;
uniform sampler2D shadow_map;

void main() {
  vec3 color3 = texture(shadow_map, frag_uv).xxx;
  color = vec4(color3, 1.0);
}
