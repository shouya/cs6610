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
  vec2 uv = gl_FragCoord.xy + vec2(2.0);
  vec3 K = texelFetch(world_texture, ivec2(uv), 0).rgb;

  vec3 oKd = vec3(0.1);
  vec3 oKa = K * 0.5;

  color = vec4(K * 0.9, 1.0);

  // color = vec4(light_color * geom * oKd + oKa, 1.0);
}
