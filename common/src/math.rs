use cgmath::{EuclideanSpace as _, Matrix3, Matrix4, Point3, Vector3};

pub fn mat4_to_3(mat: Matrix4<f32>) -> Matrix3<f32> {
  let [[m00, m01, m02, _], [m10, m11, m12, _], [m20, m21, m22, _], [_, _, _, _]] =
    mat.into();
  Matrix3::from([[m00, m01, m02], [m10, m11, m12], [m20, m21, m22]])
}

pub fn reflect4x4(point: Point3<f32>, normal: Vector3<f32>) -> Matrix4<f32> {
  let [x, y, z] = normal.into();
  let d = -point.dot(normal);
  let mat = [
    [1.0 - 2.0 * x * x, -2.0 * x * y, -2.0 * x * z, 0.0],
    [-2.0 * x * y, 1.0 - 2.0 * y * y, -2.0 * y * z, 0.0],
    [-2.0 * x * z, -2.0 * y * z, 1.0 - 2.0 * z * z, 0.0],
    [2.0 * x * d, 2.0 * y * d, 2.0 * z * d, 1.0],
  ];
  mat.into()
}
