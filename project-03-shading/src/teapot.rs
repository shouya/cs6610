use std::path::Path;
use std::time::Duration;

use cgmath::Matrix as _;
use cgmath::Matrix3;
use cgmath::Point3;
use cgmath::Transform;
use derive_more::From;

use cgmath::Matrix4;
use common::RawObj;
use glium::{backend::Facade, uniform, DrawParameters, Frame};

use crate::mesh::TriangleIndex;
use crate::mesh::TriangleIndexGPU;
use crate::mesh::TriangleListGPU;
use crate::mesh::TriangleStrip;
use crate::mesh::TriangleStripGPU;
use crate::Camera;
use crate::Result;

use crate::mesh::{self, TriangleList};

#[derive(From)]
pub enum TeapotKind {
  TrigList(Teapot<TriangleListGPU>),
  TrigIndex(Teapot<TriangleIndexGPU>),
  TriangleStrip(Teapot<TriangleStripGPU>),
}

impl TeapotKind {
  pub fn update(&mut self, dt: Duration) {
    match self {
      Self::TrigList(teapot) => teapot.update(dt),
      Self::TrigIndex(teapot) => teapot.update(dt),
      Self::TriangleStrip(teapot) => teapot.update(dt),
    }
  }

  pub fn draw(&self, frame: &mut Frame, camera: &Camera) -> Result<()> {
    match self {
      Self::TrigList(teapot) => teapot.draw(frame, camera),
      Self::TrigIndex(teapot) => teapot.draw(frame, camera),
      Self::TriangleStrip(teapot) => teapot.draw(frame, camera),
    }
  }
}

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

impl Teapot<TriangleIndex> {
  pub fn new_triangle_index() -> Result<Self> {
    let path = Path::new(common::teapot_path());
    let raw_obj = RawObj::load_from(path)?;
    let mesh = TriangleIndex::from_raw_obj(raw_obj);
    Self::new(mesh)
  }

  pub fn to_strips(&self) -> Result<Teapot<TriangleStrip>> {
    let mesh = TriangleStrip::from(&self.mesh);
    Ok(Teapot {
      rotation: self.rotation,
      rotation_speed: self.rotation_speed,
      mesh,
    })
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
    let mv: Matrix4<f32> = camera.view() * self.model_transform();
    let mv3: Matrix3<f32> = Matrix3 {
      x: mv.x.truncate(),
      y: mv.y.truncate(),
      z: mv.z.truncate(),
    };
    let mv_n: Matrix3<f32> =
      <Matrix3<f32> as Transform<Point3<f32>>>::inverse_transform(&mv3)
        .unwrap()
        .transpose();
    let mvp: Matrix4<f32> = camera.projection() * mv;

    let uniforms = uniform! {
      mvp: <Matrix4<f32> as Into<[[f32; 4]; 4]>>::into(mvp),
      mv: <Matrix4<f32> as Into<[[f32; 4]; 4]>>::into(mv),
      mv_n: <Matrix3<f32> as Into<[[f32; 3]; 3]>>::into(mv_n),
      clr: [1.0, 0.0, 1.0f32],
    };

    // by default the depth buffer is not used.
    let draw_params = DrawParameters {
      depth: glium::Depth {
        test: glium::draw_parameters::DepthTest::IfLess,
        write: true,
        ..Default::default()
      },
      ..Default::default()
    };

    self.mesh.draw(frame, &uniforms, &draw_params);

    Ok(())
  }
}
