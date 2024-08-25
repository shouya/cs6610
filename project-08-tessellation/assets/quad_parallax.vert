#version 330 core

in vec3 pos;
in vec2 uv;

out VS_OUT {
  vec2 frag_uv;
  vec3 light_dir_m;
  vec3 camera_pos_t;
  vec3 frag_pos_t;
} vs_out;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;
// from model to tangent space
uniform mat3 tbn_matrix;

uniform vec3 light_dir_or_loc;
uniform int light_type;

void main() {
  vec3 pos_w = (model * vec4(pos, 1)).xyz;
  vec3 pos_v = (view * vec4(pos_w, 1)).xyz;

  vs_out.frag_pos_t = tbn_matrix * pos;

  gl_Position = projection * vec4(pos_v, 1);

  mat4 inv_model = inverse(model);
  mat4 inv_view = inverse(view);

  vec3 light_dir_m;
  switch (light_type) {
  case 0: // directional light
    light_dir_m = normalize((inv_model * vec4(light_dir_or_loc, 0.0)).xyz);
    break;
  case 1: // spot light
    vec3 dir = normalize(light_dir_or_loc - pos_w);
    light_dir_m = normalize((inv_model * vec4(dir, 0.0)).xyz);
    break;
  }
  vs_out.light_dir_m = light_dir_m;

  vec3 camera_pos_w = (inv_view * vec4(0, 0, 0, 1)).xyz;
  vec3 camera_pos_m = (inv_model * vec4(camera_pos_w, 1)).xyz;
  vs_out.camera_pos_t = tbn_matrix * camera_pos_m;

  vs_out.frag_uv = uv;
}
