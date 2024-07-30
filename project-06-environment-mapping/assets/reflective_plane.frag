#version 330 core

layout(location = 0) out vec4 color;

in vec2 uv_t; // in texture space
in vec3 pos_v; // in view space
in vec3 n_v; // in view space

// diffuse, specular, ambient
uniform vec3 Kd, Ks, Ka;
uniform float Ns; // shininess


// unused properties:

// dissolve (transparency), index of refraction,
// transmission filter.
uniform float d, Ni, Tr;
uniform vec3 Tf, Ke; // Tf, emission
uniform uint illum;

// end unused properties

// textures
uniform sampler2D map_Kd, map_Ks, map_Ka;
uniform uint use_map_Kd, use_map_Ks, use_map_Ka;

// in view space
uniform vec3 light_pos;
uniform vec3 light_color;

// world texture
uniform sampler2D world_texture;

void main() {
  vec3 n_v = normalize(n_v);
  vec3 light_dir = normalize(light_pos - pos_v);
  vec3 view_dir = normalize(-pos_v);
  // geometry term
  float geom = max(dot(n_v, light_dir), 0.0);

  // half vector
  vec3 h = normalize(light_dir + view_dir);
  float spec = pow(max(dot(n_v, h), 0.0), Ns * 100);

  // coordinates in view port
  vec2 uv = gl_FragCoord.xy;
  vec3 Kr = texture(world_texture, uv).rgb;

  vec3 oKd = vec3(0.05);
  if (use_map_Kd == 1u) {
    oKd += texture(map_Kd, uv_t).rgb * Kd;
  } else {
    oKd += Kd;
  }

  vec3 oKs = vec3(0.001);;
  if (use_map_Ks == 1u) {
    oKs += texture(map_Ks, uv_t).rgb * Ks;
  } else {
    oKs += Ks;
  }

  vec3 oKa = vec3(0.1);;
  if (use_map_Ka == 1u) {
    oKa += texture(map_Ka, uv_t).rgb * Ka;
  } else {
    oKa += Ka;
  }

  color = vec4(light_color * (geom * oKd + spec * oKs) + oKa, 1.0);
}
