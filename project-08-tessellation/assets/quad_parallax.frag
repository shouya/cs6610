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

uniform float displacement_scale;
uniform float detail_level;

vec2 trace(vec3 view_dir, vec2 uv, float levels) {
  float curr_h = 1 - texture(displacement_map, uv).z;
  float prev_h = 0;

  vec2 offset = vec2(0);
  vec2 duv = view_dir.xy / view_dir.z * displacement_scale;

  float level_height = 1.0 / levels;
  float target_h = level_height;

  while (curr_h > target_h) {
    prev_h = curr_h;
    offset += level_height * duv;
    target_h += level_height;
    curr_h = 1 - texture(displacement_map, uv - offset).z;
  }

  // interpolate between the last heights
  float h1 = target_h - curr_h;
  float h2 = prev_h - (target_h - level_height);
  float r = h1 / (h1 + h2);

  if (r > 0 && r < 1) {
    offset -= level_height * duv * r;
  }

  return uv - offset;
}

// the naive parallax mapping

// vec2 trace(vec3 view_dir, vec2 uv) {
//   float h = 1 - texture(displacement_map, uv).z;
//   vec2 duv = view_dir.xy / view_dir.z * displacement_scale;
//   vec2 offset = duv * h;
//   return uv - offset;
// }

void main() {
  vec3 view_dir_t = normalize(fs_in.camera_pos_t - fs_in.frag_pos_t);
  vec2 traced_uv = trace(view_dir_t, fs_in.frag_uv, detail_level);

  if (traced_uv.x < 0 || traced_uv.x > 1 || traced_uv.y < 0 || traced_uv.y > 1) {
    discard;
  }

  vec3 light_dir_m = normalize(fs_in.light_dir_m);

  vec3 normal = normalize(texture(normal_map, traced_uv).xyz * 2 - 1);

  float geom = clamp(dot(light_dir_m, normal), 0.0, 1.0);
  vec3 color_diffuse = texture(color_texture, traced_uv).xyz * geom;

  vec3 color_ambient = vec3(0.1, 0.1, 0.1);

  color = vec4(color_diffuse + color_ambient, 1);
}
