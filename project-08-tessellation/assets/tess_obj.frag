#version 410 core

layout(location = 0) out vec4 color;

in vec2 uv;
in vec3 pos_v;
in vec3 light_dir_v;

uniform mat3 model_view_normal;
uniform sampler2D normal_map;

void main() {
  vec3 norm_m = texture(normal_map, uv).xyz;
  vec3 normal_v = normalize(model_view_normal * norm_m);
  vec3 light_dir_vn = normalize(light_dir_v);

  float geom = max(dot(normal_v, light_dir_vn), 0.0);
  vec3 k_diff = vec3(0.8, 0.8, 0.8) * geom;

  vec3 view_dir_v = normalize(-pos_v);
  vec3 half_dir_v = normalize(light_dir_vn + view_dir_v);
  float specular = pow(max(dot(normal_v, half_dir_v), 0.0), 32);
  vec3 k_spec = vec3(0.5, 0.5, 0.5) * specular;

  vec3 ambient = vec3(0.1, 0.1, 0.1);

  color = vec4(k_diff + k_spec + ambient, 1.0);
}
