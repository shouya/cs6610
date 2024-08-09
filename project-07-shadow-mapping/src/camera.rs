use glam::{EulerRot, Mat3, Mat4, Vec3};

use crate::transform::Transform;

pub enum Projection {
  Orthographic,
  Perspective {
    // angle in degrees
    fov: f32,
  },
}

pub struct Camera {
  pos: Vec3,
  looking_at: Vec3,
  projection: Projection,
  aspect: f32,
}

// #[derive(Debug, Clone, Copy)]
// struct CameraCache {
//   view: Mat4,
//   projection: Mat4,
// }

impl Default for Camera {
  fn default() -> Self {
    Self {
      pos: Vec3::new(0.5, 2.0, 2.0),
      looking_at: Vec3::ZERO,
      projection: Projection::Perspective { fov: 90.0 },
      aspect: 1.0,
    }
  }
}

impl Camera {
  pub fn handle_resize(&mut self, width: f32, height: f32) {
    if height == 0.0 {
      return;
    }
    self.aspect = width / height;
  }

  pub fn view(&self) -> Mat4 {
    // TODO: cache the result
    self.compute_view_matrix()
  }

  pub fn projection(&self) -> Mat4 {
    // TODO: cache the result
    self.compute_projection_matrix()
  }

  pub fn view_projection(&self) -> Mat4 {
    // TODO: cache the result
    self.projection() * self.view()
  }

  fn compute_view_matrix(&self) -> Mat4 {
    Mat4::look_at_rh(self.pos, self.looking_at, Vec3::Y)
  }

  fn compute_projection_matrix(&self) -> Mat4 {
    match self.projection {
      Projection::Orthographic => {
        // give a fake sense of distance
        let dist = self.pos.distance(self.looking_at);
        let scale = Mat4::from_scale(Vec3::from([1.0 / dist; 3]));
        let proj = Mat4::orthographic_rh_gl(
          -self.aspect,
          self.aspect,
          -1.0,
          1.0,
          -1.0,
          1.0,
        );
        scale * proj
      }
      Projection::Perspective { fov } => {
        Mat4::perspective_rh_gl(fov.to_radians(), self.aspect, 0.1, 100.0)
      }
    }
  }

  pub fn change_distance(&mut self, delta: f32) {
    let orig_pos = self.pos;
    let looking_dir = (self.looking_at - self.pos).normalize();
    let new_pos = self.pos + looking_dir * delta * 0.003;

    // new pos moved past the looking direction
    if orig_pos.dot(new_pos) < 0.0 {
      return;
    }

    self.pos = new_pos;
  }

  pub fn rotate(&mut self, dx: f32, dy: f32) {
    let dir = (self.looking_at - self.pos).normalize();
    let dist = self.pos.distance(self.looking_at);
    let new_dir = Mat3::from_euler(EulerRot::YXZ, dy, dx, 0.0) * dir;
    let new_pos = new_dir * dist;

    self.pos = new_pos;
  }

  pub fn toggle_projection(&mut self) {
    self.projection = match self.projection {
      Projection::Orthographic => Projection::Perspective { fov: 90.0 },
      Projection::Perspective { .. } => Projection::Orthographic,
    };
  }

  // handle view update by recomputing the matrices
  pub fn update_view(&mut self) {
    // TODO: invalidate the cache
  }
}
