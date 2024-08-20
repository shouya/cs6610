use glam::{Mat3, Mat4, Quat, Vec3};

#[derive(Debug, Clone, Copy)]
pub struct Transform {
  pub translation: Vec3,
  // euler angle xyz, in radians
  pub rotation: Quat,
  pub scale: Vec3,
}

impl Transform {
  pub fn map_point(&self, p: Vec3) -> Vec3 {
    self.scale * (self.rotation * p) + self.translation
  }

  pub fn map_vec(&self, v: Vec3) -> Vec3 {
    self.scale * (self.rotation * v)
  }

  pub fn to_mat3(&self) -> Mat3 {
    let scale = Mat3::from_diagonal(self.scale);
    scale * Mat3::from_quat(self.rotation)
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
      rotation: Quat::IDENTITY,
      scale: Vec3::ONE,
    }
  }
}
