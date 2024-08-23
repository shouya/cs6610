#version 410 core

layout(location = 0) out vec4 color;

in vec2 uv;
in vec3 pos_v;
in vec3 pos_w;
in vec3 norm_v;
in vec3 light_dir_v;
in vec3 shadow_pos;

uniform sampler2D normal_map;

// shadow map (depth comparison enabled)
uniform sampler2DShadow shadow_map;
// from world space to shadow map space
// uniform mat4 shadow_transform;

uniform mat3 model_view_normal;

void main() {
  vec3 norm_m = normalize(texture(normal_map, uv).xyz * 2 - 1); // [0,1] -> [-1,1]
  vec3 normal_v = normalize(model_view_normal * norm_m);
  vec3 light_dir_vn = normalize(light_dir_v);

  float geom = max(dot(normal_v, light_dir_vn), 0.0);
  vec3 k_diff = vec3(0.5,0.5,0.5) * geom;

  vec3 view_dir_v = normalize(-pos_v);
  vec3 half_dir_v = normalize(light_dir_vn + view_dir_v);
  float specular = pow(max(dot(normal_v, half_dir_v), 0.0), 32);
  vec3 k_spec = vec3(1,1,1) * specular;

  vec3 ambient = vec3(0.1, 0.1, 0.1);

  color = vec4(shadow_pos, 1.0);
  vec3 shadow_pos_biased = shadow_pos;
  shadow_pos_biased.z -= mix(0.0005, 0.1, 1.0 - geom);
  float shadow = texture(shadow_map, shadow_pos_biased);
  if (shadow_pos_biased.z >= 1.0) {
    shadow = 1;
  }

  color = vec4(2.0 * (k_diff + k_spec) * shadow + ambient, 1.0);
}
