use glam::{Mat3, Mat4, Vec3};
use glium::Rect;

pub struct Camera {
  clear_color: [f32; 4],
  // camera
  aspect_ratio: f32,
  distance: f32,
  rotation: [f32; 2],
  perspective: bool,
  // only draw within this rectangle (window space)
  scissor: Option<Rect>,
  // cached matrices
  m_view: Mat4,
  m_proj: Mat4,
  m_view_proj: Mat4,
  mirror: bool,
}

impl Camera {
  pub fn new() -> Self {
    Self {
      clear_color: [0.0, 0.0, 0.0, 1.0],
      aspect_ratio: 1.0,
      distance: 2.0,
      rotation: [0.0, 0.0],
      scissor: None,
      perspective: true,

      m_view: Mat4::IDENTITY,
      m_proj: Mat4::IDENTITY,
      m_view_proj: Mat4::IDENTITY,
      mirror: false,
    }
  }

  pub fn from_view_projection(view: Mat4, proj: Mat4, mirror: bool) -> Self {
    let m_view_proj = proj * view;

    Self {
      // all unused
      clear_color: [0.0, 0.0, 0.0, 1.0],
      aspect_ratio: 1.0,
      distance: 0.0,
      rotation: [0.0, 0.0],
      perspective: true,
      scissor: None,

      // actually used
      m_view: view,
      m_proj: proj,
      m_view_proj,
      mirror,
    }
  }

  // sitting at a point, looking to a direction.
  // point and direction are given in world space.
  pub fn for_cubemap_face(
    point: Vec3,
    dir: Vec3,
    up: Vec3,
    mirror: bool,
  ) -> Self {
    let m_view = Mat4::look_to_rh(point, dir, up);

    let m_proj = Mat4::perspective_rh(90.0f32.to_radians(), 1.0, 0.1, 100.0);
    let m_view_proj = m_proj * m_view;

    Self {
      clear_color: [0.0, 0.0, 0.0, 1.0],
      // these fields are all dummy. they are not used in this context.
      aspect_ratio: 1.0,
      distance: 0.0,
      rotation: [0.0, 0.0],
      perspective: true,
      scissor: None,

      // only these fields can be safely used.
      m_view,
      m_proj,
      m_view_proj,
      mirror,
    }
  }

  pub fn mirror(&self) -> bool {
    self.mirror
  }

  pub fn scissor(&self) -> Option<Rect> {
    self.scissor
  }

  pub fn set_scissor(&mut self, scissor: Rect) {
    self.scissor = Some(scissor);
  }

  pub fn clear_color(&self) -> [f32; 4] {
    self.clear_color
  }

  pub fn handle_window_resize(&mut self, new_size: (u32, u32)) {
    self.aspect_ratio = new_size.0 as f32 / new_size.1 as f32;
  }

  pub fn view(&self) -> Mat4 {
    self.m_view
  }

  pub fn view_projection(&self) -> Mat4 {
    self.m_view_proj
  }

  pub fn projection(&self) -> Mat4 {
    self.m_proj
  }

  pub fn calc_view(&self) -> Mat4 {
    // default view matrix: eye at (0, 0, 2), looking at (0, 0, -1), up (0, 1, 0)
    let dir = Mat3::from_rotation_y(self.rotation[0].to_radians())
      * Mat3::from_rotation_x(-self.rotation[1].to_radians())
      * Vec3::Z;
    let eye = -dir * self.distance;
    let up = Vec3::Y;
    let origin = Vec3::ZERO;
    Mat4::look_at_rh(eye, origin, up)
  }

  pub fn calc_proj(&self) -> Mat4 {
    if self.perspective {
      Mat4::perspective_rh(90.0f32.to_radians(), self.aspect_ratio, 0.1, 100.0)
    } else {
      Mat4::from_scale(Vec3::splat(1.0 / self.distance))
        * Mat4::orthographic_rh(
          -1.0,
          1.0,
          -1.0 / self.aspect_ratio,
          1.0 / self.aspect_ratio,
          0.1,
          100.0,
        )
    }
  }

  pub fn update_view(&mut self) {
    self.m_view = self.calc_view();
    self.m_proj = self.calc_proj();
    self.m_view_proj = self.m_proj * self.m_view;
  }

  pub fn toggle_perspective(&mut self) {
    self.perspective ^= true;
  }

  pub fn add_rotation(&mut self, delta: [f32; 2]) {
    self.rotation[0] += delta[0];
    self.rotation[1] += delta[1];
  }

  pub fn add_distance(&mut self, delta: f32) {
    self.distance += delta;
    self.distance = self.distance.clamp(0.01, 100.0);
  }
}
