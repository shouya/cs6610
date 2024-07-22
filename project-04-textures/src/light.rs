use cgmath::{Matrix4, Point3, Rad, Transform};

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
      color: [1.0, 1.0, 1.0],
      distance: 1.0,
      rotation: 0.0,
    }
  }
  pub fn position_world(&self) -> [f32; 3] {
    let location = Point3::new(self.distance, self.distance, 0.0);
    Matrix4::from_angle_y(Rad(self.rotation))
      .transform_point(location)
      .into()
  }

  pub fn add_rotation(&mut self, delta: f32) {
    self.rotation += delta;
  }

  fn model(&self) -> Matrix4<f32> {
    Matrix4::from_translation(self.position_world().into())
      * Matrix4::from_scale(0.05)
  }

  pub fn color(&self) -> [f32; 3] {
    self.color
  }
}
