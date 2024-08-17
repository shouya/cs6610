#version 330 core

layout(location = 0) out vec4 color;

in vec2 uv_t; // in texture space
in vec3 pos_v; // in view space
in vec3 n_v; // in view space

// diffuse, specular, ambient
uniform vec3 Kd, Ks, Ka;
uniform float Ns; // shininess

// textures
uniform sampler2D map_Kd, map_Ks, map_Ka;
uniform uint use_map_Kd, use_map_Ks, use_map_Ka;

// in view space
in vec3 light_dir_raw;
uniform float light_cone_angle;
uniform vec3 light_color;

// in light space
in vec3 shadow_pos;
uniform sampler2DShadow shadow_map;
uniform sampler2D shadow_map_debug;

void main() {
  vec3 n_v = normalize(n_v);
  vec3 view_dir = normalize(-pos_v);
  vec3 light_dir = normalize(light_dir_raw);
  // geometry term
  float geom = max(dot(n_v, light_dir), 0.0);

  // half vector
  vec3 h = normalize(light_dir + view_dir);
  float spec = pow(max(dot(n_v, h), 0.0), Ns);

  vec3 oKd;
  if (use_map_Kd == 1u) {
    oKd = texture(map_Kd, uv_t).rgb * Kd;
  } else {
    oKd = Kd;
  }

  vec3 oKs;
  if (use_map_Ks == 1u) {
    oKs = texture(map_Ks, uv_t).rgb * Ks;
  } else {
    oKs = Ks;
  }

  // debugging:
  // float shadow_value = texture(shadow_map_debug, uv_t).r;
  // color = vec4(vec3(shadow_value), 1.0);
  // return;

  float shadow = texture(shadow_map, shadow_pos);

  vec3 oKa;
  if (use_map_Ka == 1u) {
    oKa = texture(map_Ka, uv_t).rgb * Ka;
  } else {
    oKa = Ka;
  }

  vec3 rgb = light_color * (geom * oKd + spec * oKs) + oKa;

  color = vec4(rgb * shadow, 1.0);
}
