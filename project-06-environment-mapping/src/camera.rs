use cgmath::{Deg, Matrix3, Matrix4, Point3, SquareMatrix as _, Vector3};

pub struct Camera {
  clear_color: [f32; 4],
  // camera
  aspect_ratio: f32,
  distance: f32,
  rotation: [f32; 2],
  perspective: bool,
  // cached matrices
  m_view: Matrix4<f32>,
  m_proj: Matrix4<f32>,
  m_view_proj: Matrix4<f32>,
}

impl Camera {
  pub fn new() -> Self {
    Self {
      clear_color: [0.0, 0.0, 0.0, 1.0],
      aspect_ratio: 1.0,
      distance: 2.0,
      rotation: [0.0, 0.0],
      perspective: true,

      m_view: Matrix4::identity(),
      m_proj: Matrix4::identity(),
      m_view_proj: Matrix4::identity(),
    }
  }

  // sitting at a point, looking to a direction.
  // point and direction are given in world space.
  pub fn for_cubemap_face(
    point: Point3<f32>,
    dir: [i8; 3],
    up: [i8; 3],
  ) -> Self {
    let up = Vector3::new(up[0] as f32, up[1] as f32, up[2] as f32);
    let dir = Vector3::new(dir[0] as f32, dir[1] as f32, dir[2] as f32);

    let m_view = Matrix4::look_to_rh(point, dir, up);

    let m_proj = cgmath::perspective(cgmath::Deg(90.0), 1.0, 0.1, 100.0);
    let m_view_proj = m_proj * m_view;

    Self {
      clear_color: [0.0, 0.0, 0.0, 1.0],
      // these fields are all dummy. they are not used in this context.
      aspect_ratio: 1.0,
      distance: 0.0,
      rotation: [0.0, 0.0],
      perspective: true,

      // only these fields can be safely used.
      m_view,
      m_proj,
      m_view_proj,
    }
  }

  pub fn clear_color(&self) -> [f32; 4] {
    self.clear_color
  }

  pub fn handle_window_resize(&mut self, new_size: (u32, u32)) {
    self.aspect_ratio = new_size.0 as f32 / new_size.1 as f32;
  }

  pub fn view(&self) -> Matrix4<f32> {
    self.m_view
  }

  pub fn view_projection(&self) -> Matrix4<f32> {
    self.m_view_proj
  }

  pub fn projection(&self) -> Matrix4<f32> {
    self.m_proj
  }

  pub fn calc_view(&self) -> Matrix4<f32> {
    // default view matrix: eye at (0, 0, 2), looking at (0, 0, -1), up (0, 1, 0)
    let dir = Matrix3::from_angle_y(Deg(self.rotation[0]))
      * Matrix3::from_angle_x(-Deg(self.rotation[1]))
      * Vector3::new(0.0, 0.0, 1.0);
    let eye = Point3::new(0.0, 0.0, 0.0) + -dir * self.distance;
    let up = Vector3::new(0.0, 1.0, 0.0);
    let origin = Point3::new(0.0, 0.0, 0.0);
    Matrix4::look_at_rh(eye, origin, up)
  }

  pub fn calc_proj(&self) -> Matrix4<f32> {
    if self.perspective {
      cgmath::perspective(cgmath::Deg(90.0), self.aspect_ratio, 0.1, 100.0)
    } else {
      cgmath::Matrix4::from_scale(1.0 / self.distance)
        * cgmath::ortho(
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
