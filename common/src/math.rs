use glam::{Mat4, Vec3};

pub fn reflect4x4(point: Vec3, normal: Vec3) -> Mat4 {
  let [x, y, z] = normal.into();
  let d = -point.dot(normal);

  glam::mat4(
    glam::vec4(1.0 - 2.0 * x * x, -2.0 * x * y, -2.0 * x * z, 0.0),
    glam::vec4(-2.0 * x * y, 1.0 - 2.0 * y * y, -2.0 * y * z, 0.0),
    glam::vec4(-2.0 * x * z, -2.0 * y * z, 1.0 - 2.0 * z * z, 0.0),
    glam::vec4(2.0 * x * d, 2.0 * y * d, 2.0 * z * d, 1.0),
  )
}
