use std::path::Path;
use std::time::Duration;

use cgmath::Matrix4;
use common::RawObj;
use glium::{backend::Facade, uniform, DrawParameters, Frame};

use crate::Camera;
use crate::Result;

use crate::mesh::{self, TriangleList};

pub struct Teapot<Mesh> {
  rotation: f32,
  rotation_speed: f32,
  mesh: Mesh,
}

impl Teapot<TriangleList> {
  pub fn new_triangle_list() -> Result<Self> {
    let path = Path::new(common::teapot_path());
    let raw_obj = RawObj::load_from(path)?;
    let mesh = TriangleList::from_raw_obj(raw_obj);
    Self::new(mesh)
  }
}

impl<Mesh> Teapot<Mesh> {
  pub fn new(mesh: Mesh) -> Result<Self> {
    Ok(Self {
      rotation: 0.0,
      rotation_speed: 1.0,
      mesh,
    })
  }

  pub fn update(&mut self, dt: Duration) {
    self.rotation += dt.as_secs_f32() * self.rotation_speed;
  }

  fn model_transform(&self) -> Matrix4<f32> {
    Matrix4::from_scale(0.05)
      * Matrix4::from_angle_y(cgmath::Rad(self.rotation))
    // the object itself is rotated 90 to the front, let's rotate it back a little.
      * Matrix4::from_angle_x(cgmath::Deg(-90.0))
  }

  pub fn upload<GPUMesh>(&self, surface: &impl Facade) -> Teapot<GPUMesh>
  where
    Mesh: mesh::MeshFormat<GPURepr = GPUMesh>,
  {
    Teapot {
      rotation: self.rotation,
      rotation_speed: self.rotation_speed,
      mesh: self.mesh.upload(surface),
    }
  }

  pub fn draw(&self, frame: &mut Frame, camera: &Camera) -> Result<()>
  where
    Mesh: mesh::GPUMeshFormat,
  {
    let vp = camera.vp();
    let mvp: [[f32; 4]; 4] = (vp * self.model_transform()).into();
    let uniforms = uniform! {
      mvp: mvp,
      clr: [1.0, 0.0, 1.0f32],
    };

    let draw_params = DrawParameters {
      point_size: Some(2.0),
      ..Default::default()
    };

    self.mesh.draw(frame, &uniforms, &draw_params);

    Ok(())
  }
}
