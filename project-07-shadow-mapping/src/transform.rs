use cgmath::{Euler, Matrix3, Matrix4, Transform3, Vector3};

pub struct Transform {
  pub translation: [f32; 3],
  // euler angle xyz, in degree
  pub rotation: [f32; 3],
  pub scale: [f32; 3],
}

impl Transform {
  pub fn to_mat3(&self) -> Matrix3<f32> {
    let mut rot: Matrix3<_> = Euler::new(
      cgmath::Deg(self.rotation[0]),
      cgmath::Deg(self.rotation[1]),
      cgmath::Deg(self.rotation[2]),
    )
    .into();

    rot.x[0] *= self.scale[0];
    rot.x[1] *= self.scale[0];
    rot.x[2] *= self.scale[0];
    rot
  }

  pub fn to_mat4(&self) -> Matrix4<f32> {
    let mut mat: Matrix4<_> = self.to_mat3().into();
    mat.w = self.translation().extend(1.0);
    mat
  }

  pub fn translation(&self) -> Vector3<f32> {
    Vector3::new(
      self.translation[0],
      self.translation[1],
      self.translation[2],
    )
  }
}
