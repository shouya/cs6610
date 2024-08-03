#version 330 core

layout(location = 0) out vec4 color;

in vec2 uv_t; // in texture space
in vec3 pos_v; // in view space
in vec3 n_v; // in view space

in vec3 orig_pos; // in model space

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

// time, used for animating the ripple effect.
uniform float t;

void main() {
  vec3 n_v = normalize(n_v);
  vec3 light_dir = normalize(light_pos - pos_v);
  vec3 view_dir = normalize(-pos_v);
  // geometry term
  float geom = max(dot(n_v, light_dir), 0.0);

  // half vector
  vec3 h = normalize(light_dir + view_dir);
  float spec = pow(max(dot(n_v, h), 0.0), Ns);

  // ripple effect
  float amplitude = sin(-t * 10.0 + length(orig_pos.xy) * 30.0);
  vec2 offset = amplitude * view_dir.xy * 100.0;

  // // coordinates in view port
  vec2 uv_k = gl_FragCoord.xy + offset;
  vec3 K = texelFetch(world_texture, ivec2(uv_k), 0).rgb;

  color = vec4(K * 0.9, 1.0);
}
