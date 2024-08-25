#version 330 core

layout(location = 0) out vec4 color;

in vec2 frag_uv;

in VS_OUT {
  vec2 frag_uv;
  vec3 light_dir_m;
  vec3 camera_pos_t;
  vec3 frag_pos_t;
} fs_in;

uniform sampler2D displacement_map;
uniform sampler2D normal_map;
uniform sampler2D color_texture;

uniform mat3 tbn_matrix;

const int MAX_LEVEL = 5;
const float LEVEL_HEIGHT = 1.0 / MAX_LEVEL;

const float BUMP_SCALE = 0.05;

vec2 trace(vec3 view_dir, vec2 uv) {
  float prev_h = 0;

  vec2 offset = vec2(0);
  vec2 duv = view_dir.xy / view_dir.z * BUMP_SCALE;

  for (int i = 0; i < MAX_LEVEL; i++) {
    // h: trough depth, 0 is the surface, 1 is the deepest
    float h = texture(displacement_map, uv - offset).z;
    // we can handle trough at most this height
    float max_h = (i + 1) * LEVEL_HEIGHT;
    // if the trough is too shallow, we stop
    if (h <= max_h) {
      offset += (h - i * LEVEL_HEIGHT) * duv;
      break;
    }
    offset += LEVEL_HEIGHT * duv;
    prev_h = h;
  }

  return uv - offset;
}

// the naive parallax mapping

// vec2 trace(vec3 view_dir, vec2 uv) {
//   float h = texture(displacement_map, uv).z;
//   vec2 duv = view_dir.xy / view_dir.z * BUMP_SCALE;
//   vec2 offset = duv * h;
//   return uv - offset;
// }

void main() {
  vec3 view_dir_t = normalize(fs_in.camera_pos_t - fs_in.frag_pos_t);
  vec2 traced_uv = trace(view_dir_t, fs_in.frag_uv);

  if (traced_uv.x < 0 || traced_uv.x > 1 || traced_uv.y < 0 || traced_uv.y > 1) {
    discard;
  }

  vec3 light_dir_m = normalize(fs_in.light_dir_m);

  vec3 normal_t = normalize(texture(normal_map, traced_uv).xyz * 2 - 1);
  vec3 normal_m = normalize(tbn_matrix * normal_t);

  float geom = clamp(dot(light_dir_m, normal_m), 0.0, 1.0);
  vec3 color_diffuse = texture(color_texture, traced_uv).xyz * geom;

  vec3 color_ambient = vec3(0.1, 0.1, 0.1);

  color = vec4(color_diffuse + color_ambient, 1);
}
