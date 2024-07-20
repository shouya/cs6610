#version 330 core

layout(location = 0) out vec4 color;

in vec2 uv_t; // in texture space
in vec3 pos_v; // in view space
in vec3 n_v; // in view space

uniform vec3 k_d;
uniform vec3 k_s;
uniform vec3 k_a;
uniform float shininess;

// in view space
uniform vec3 light_pos;
uniform vec3 light_color;
uniform float light_cone;

// modes:
// 0. default = full blinn shading
// 1. surface normal
// 2. depth
// 3. view position
// 4. ambient + diffuse
// 5. specular (blinn)
// 6. full blinn shading
// 8. full phong shading
uniform int mode;

void main() {
  if (mode == 1) {
    color = vec4(n_v, 1.0);
    return;
  }

  if (mode == 2) {
    color = vec4(gl_FragCoord.zzz / gl_FragCoord.w / 5.0, 1.0);
    return;
  }

  if (mode == 3) {
    color = vec4(pos_v, 1.0);
    return;
  }

  // critical step!
  vec3 n_v = normalize(n_v);

  vec3 light_dir = normalize(light_pos - pos_v);
  vec3 view_dir = normalize(-pos_v);
  // geometry term
  float geom = max(dot(n_v, light_dir), 0.0);

  if (mode == 3) {
    color = vec4(geom, geom, geom, 1.0);
    return;
  }

  // diffuse
  if (mode == 4) {
    color = vec4(light_color * geom * k_d + k_a, 1.0);
    return;
  }

  vec3 h = normalize(light_dir + view_dir);
  float spec = pow(max(dot(n_v, h), 0.0), shininess);
  if (mode == 5) {
    color = vec4(light_color * spec * k_s, 1.0);
    return;
  }

  if (mode == 0 || mode == 6) {
    color = vec4(light_color * (geom * k_d + spec * k_s) + k_a, 1.0);
    return;
  }

  vec3 r = reflect(-light_dir, n_v);
  float spec2 = pow(max(dot(r, view_dir), 0.0), shininess);
  if (mode == 7) {
    color = vec4(light_color * spec2 * k_s, 1.0);
    return;
  }

  if (mode == 8) {
    color = vec4(light_color * (geom * k_d + spec2 * k_s) + k_a, 1.0);
    return;
  }
}
