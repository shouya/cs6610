use cgmath::{Matrix3, Matrix4};

pub fn mat4_to_3(mat: Matrix4<f32>) -> Matrix3<f32> {
  let [[m00, m01, m02, _], [m10, m11, m12, _], [m20, m21, m22, _], [_, _, _, _]] =
    mat.into();
  Matrix3::from([[m00, m01, m02], [m10, m11, m12], [m20, m21, m22]])
}
