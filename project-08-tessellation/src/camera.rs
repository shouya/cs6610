use std::cell::RefCell;

use common::CameraLike;
use glam::{EulerRot, Mat3, Mat4, Quat, Vec3};

pub enum Projection {
  Orthographic,
  Perspective {
    // angle in degrees
    fov: f32,
  },
  Custom(Mat4),
}

pub struct Camera {
  pos: Vec3,
  looking_at: Vec3,
  projection: Projection,
  aspect: f32,
  cache: RefCell<Option<CameraCache>>,
}

#[derive(Debug, Clone, Copy)]
struct CameraCache {
  view: Mat4,
  projection: Mat4,
  view_projection: Mat4,
}

impl Default for Camera {
  fn default() -> Self {
    Self {
      pos: Vec3::new(0.5, 1.0, 1.0),
      looking_at: Vec3::ZERO,
      projection: Projection::Perspective { fov: 90.0 },
      aspect: 1.0,
      cache: RefCell::new(None),
    }
  }
}

impl Camera {
  pub fn new(
    pos: Vec3,
    looking_at: Vec3,
    projection: Projection,
    aspect: f32,
  ) -> Self {
    Self {
      pos,
      looking_at,
      projection,
      aspect,
      cache: RefCell::new(None),
    }
  }

  pub fn handle_resize(&mut self, width: f32, height: f32) {
    if height == 0.0 {
      return;
    }
    self.aspect = width / height;
  }

  pub fn view(&self) -> Mat4 {
    self
      .cache
      .borrow_mut()
      .get_or_insert_with(|| self.compute_cache())
      .view
  }

  pub fn projection(&self) -> Mat4 {
    self
      .cache
      .borrow_mut()
      .get_or_insert_with(|| self.compute_cache())
      .projection
  }

  pub fn view_projection(&self) -> Mat4 {
    self
      .cache
      .borrow_mut()
      .get_or_insert_with(|| self.compute_cache())
      .view_projection
  }

  fn compute_cache(&self) -> CameraCache {
    let view = self.compute_view_matrix();
    let projection = self.compute_projection_matrix();
    let view_projection = projection * view;
    CameraCache {
      view,
      projection,
      view_projection,
    }
  }

  // the bounding box of the view frustum in world space
  pub fn world_bounding_box(&self) -> (Vec3, Vec3) {
    let inverse_vp = self.view_projection().inverse();
    let corners = [
      Vec3::new(-1.0, -1.0, -1.0),
      Vec3::new(1.0, -1.0, -1.0),
      Vec3::new(-1.0, 1.0, -1.0),
      Vec3::new(1.0, 1.0, -1.0),
      Vec3::new(-1.0, -1.0, 1.0),
      Vec3::new(1.0, -1.0, 1.0),
      Vec3::new(-1.0, 1.0, 1.0),
      Vec3::new(1.0, 1.0, 1.0),
    ];

    let mut min = Vec3::splat(f32::INFINITY);
    let mut max = Vec3::splat(f32::NEG_INFINITY);

    for corner in &corners {
      let corner = inverse_vp * corner.extend(1.0);
      min = min.min(corner.truncate());
      max = max.max(corner.truncate());
    }

    (min, max)
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
          -10.0,
          10.0,
        );
        scale * proj
      }
      Projection::Perspective { fov } => {
        Mat4::perspective_rh_gl(fov.to_radians(), self.aspect, 0.1, 100.0)
      }
      Projection::Custom(mat) => mat,
    }
  }

  pub fn change_distance(&mut self, delta: f32) {
    let looking_dir = (self.looking_at - self.pos).normalize();
    let new_pos = self.pos + looking_dir * delta * 0.003;

    // new pos moved past the looking direction
    if self.pos.dot(new_pos) < 0.0 {
      return;
    }

    self.pos = new_pos;
  }

  pub fn rotate(&mut self, dx: f32, dy: f32) {
    let dir = (self.pos - self.looking_at).normalize();
    let dist = self.pos.distance(self.looking_at);

    let horz_axis = Vec3::Y.cross(dir).normalize();

    let rot_vert = Mat3::from_quat(Quat::from_axis_angle(horz_axis, dy * 0.01));
    let rot_horz = Mat3::from_euler(EulerRot::XYZ, 0.0, -dx * 0.01, 0.0);

    let new_dir = rot_horz * rot_vert * dir;
    let new_pos = self.looking_at + new_dir * dist;

    self.pos = new_pos;
  }

  pub fn toggle_projection(&mut self) {
    self.projection = match self.projection {
      Projection::Orthographic => Projection::Perspective { fov: 90.0 },
      Projection::Perspective { .. } => Projection::Orthographic,
      Projection::Custom(_) => unreachable!(),
    };
  }

  // handle view update by recomputing the matrices
  pub fn update_view(&mut self) {
    self.cache.borrow_mut().take();
  }
}

impl CameraLike for Camera {
  fn view(&self) -> [[f32; 4]; 4] {
    self.view().to_cols_array_2d()
  }

  fn projection(&self) -> [[f32; 4]; 4] {
    self.projection().to_cols_array_2d()
  }
}
