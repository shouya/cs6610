use glam::{EulerRot, Mat4, Vec3};

pub struct Light {
  // note the light's color can exceed 1.0
  color: [f32; 3],

  // in world space
  distance: f32,

  // rotation around the y axis
  rotation: f32,
}

impl Light {
  pub fn new() -> Self {
    Self {
      color: [10.0, 10.0, 10.0],
      distance: 1.0,
      rotation: 0.0,
    }
  }
  pub fn position_world(&self) -> Vec3 {
    let location = Vec3::new(self.distance, self.distance, 0.0);
    let transform = Mat4::from_euler(EulerRot::XYZ, 0.0, self.rotation, 0.0);

    transform.transform_point3(location)
  }

  pub fn add_rotation(&mut self, delta: f32) {
    self.rotation += delta;
  }

  #[allow(dead_code)]
  fn model(&self) -> Mat4 {
    Mat4::from_translation(self.position_world())
      * Mat4::from_scale(Vec3::splat(0.05))
  }

  pub fn color(&self) -> [f32; 3] {
    self.color
  }
}
