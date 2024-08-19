#version 330 core

layout(location = 0) out vec4 color;

in vec2 uv_t; // in texture space
in vec3 pos_v; // in view space
in vec3 n_v; // in view space

// diffuse, specular, ambient
uniform vec3 Kd, Ks, Ka;
uniform float Ns; // shininess

// textures
uniform sampler2D map_Kd, map_Ks, map_Ka, bump_map;
uniform uint use_map_Kd, use_map_Ks, use_map_Ka;

// in view space
in vec3 light_dir_raw;
uniform float light_cone_angle;
uniform vec3 light_color;

// in light space
in vec4 shadow_pos;
uniform sampler2DShadow shadow_map;

void main() {
  vec3 n_v = normalize(n_v);
  vec3 view_dir = normalize(-pos_v);
  vec3 light_dir = normalize(light_dir_raw);
  // geometry term
  float geom = max(dot(n_v, light_dir), 0.0);

  float shadow;
  if (geom <= 0.0) {
    // fix shadow acne on the back surfaces
    shadow = 1;
  } else {
    vec3 shadow_uv = shadow_pos.xyz;
    float bias = mix(0.001, 0.0001, geom);
    shadow_uv.z -= bias;
    shadow_uv = shadow_uv.xyz / shadow_pos.w;
    shadow = texture(shadow_map, shadow_uv);
  }

  vec3 oKd;
  if (use_map_Kd == 1u) {
    oKd = texture(map_Kd, uv_t).rgb * Kd;
  } else {
    oKd = Kd;
  }

  // half vector
  vec3 h = normalize(light_dir + view_dir);
  float spec = pow(max(dot(n_v, h), 0.0), Ns);
  vec3 oKs;
  if (use_map_Ks == 1u) {
    oKs = texture(map_Ks, uv_t).rgb * Ks;
  } else {
    oKs = Ks;
  }

  vec3 oKa;
  if (use_map_Ka == 1u) {
    oKa = texture(map_Ka, uv_t).rgb * Ka;
  } else {
    oKa = Ka;
  }

  vec3 rgb = light_color * 2 * (geom * oKd + spec * oKs) * shadow + oKa * 0.3;

  color = vec4(rgb, 1.0);
}
