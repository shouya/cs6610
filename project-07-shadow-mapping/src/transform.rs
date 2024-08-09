use glam::{EulerRot, Mat3, Mat4, Vec3};

#[derive(Debug, Clone, Copy)]
pub struct Transform {
  pub translation: Vec3,
  // euler angle xyz, in radians
  pub rotation: Vec3,
  pub scale: Vec3,
}

impl Transform {
  pub fn to_mat3(&self) -> Mat3 {
    let rot = Mat3::from_euler(
      EulerRot::XYZ,
      self.rotation.x,
      self.rotation.y,
      self.rotation.z,
    );

    let scale = Mat3::from_diagonal(self.scale);

    rot * scale
  }

  pub fn to_mat4(&self) -> Mat4 {
    let mut mat: Mat4 = Mat4::from_mat3(self.to_mat3());
    mat.w_axis = self.translation.extend(1.0);
    mat
  }
}

impl Default for Transform {
  fn default() -> Self {
    Self {
      translation: Vec3::ZERO,
      rotation: Vec3::ZERO,
      scale: Vec3::ONE,
    }
  }
}
