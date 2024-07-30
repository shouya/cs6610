use std::time::Duration;

use cgmath::Matrix as _;
use cgmath::Matrix3;
use cgmath::SquareMatrix as _;
use cgmath::Transform;
use derive_more::From;

use cgmath::Matrix4;
use common::SimpleObj;
use glium::{backend::Facade, uniform, DrawParameters, Frame};

use crate::mesh::TriangleIndex;
use crate::mesh::TriangleIndexGPU;
use crate::mesh::TriangleListGPU;
use crate::mesh::TriangleStrip;
use crate::mesh::TriangleStripGPU;
use crate::Camera;
use crate::Light;
use crate::Result;

use crate::mesh::{self, TriangleList};

#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub enum RenderMode {
  Default = 0,
  SurfaceNormal = 1,
  Depth = 2,
  ViewPosition = 3,
  Diffuse = 4,
  SpecularBlinn = 5,
  FullBlinn = 6,
  SpecularPhong = 7,
  FullPhong = 8,
  Plain = 9,
}

impl RenderMode {
  pub fn from_key(key: impl AsRef<str>) -> Option<Self> {
    match key.as_ref() {
      "0" => Some(Self::Default),
      "1" => Some(Self::SurfaceNormal),
      "2" => Some(Self::Depth),
      "3" => Some(Self::ViewPosition),
      "4" => Some(Self::Diffuse),
      "5" => Some(Self::SpecularBlinn),
      "6" => Some(Self::FullBlinn),
      "7" => Some(Self::SpecularPhong),
      "8" => Some(Self::FullPhong),
      "9" => Some(Self::Plain),
      _ => None,
    }
  }
}

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

  pub fn draw(
    &self,
    frame: &mut Frame,
    camera: &Camera,
    light: &Light,
  ) -> Result<()> {
    match self {
      Self::TrigList(teapot) => teapot.draw(frame, camera, light),
      Self::TrigIndex(teapot) => teapot.draw(frame, camera, light),
      Self::TriangleStrip(teapot) => teapot.draw(frame, camera, light),
    }
  }

  pub fn set_render_mode(&mut self, render_mode: RenderMode) {
    match self {
      Self::TrigList(teapot) => teapot.set_render_mode(render_mode),
      Self::TrigIndex(teapot) => teapot.set_render_mode(render_mode),
      Self::TriangleStrip(teapot) => teapot.set_render_mode(render_mode),
    }
  }
}

pub struct Teapot<Mesh> {
  rotation: f32,
  rotation_speed: f32,
  mesh: Mesh,
  render_mode: RenderMode,
}

impl Teapot<TriangleList> {
  pub fn new_triangle_list() -> Result<Self> {
    let simple_obj = SimpleObj::load_from(&common::teapot_path())?;
    let mesh = TriangleList::from_simple_obj(simple_obj);
    Self::new(mesh)
  }
}

impl Teapot<TriangleIndex> {
  pub fn new_triangle_index() -> Result<Self> {
    let simple_obj = SimpleObj::load_from(&common::teapot_path())?;
    let mesh = TriangleIndex::from_simple_obj(simple_obj);
    Self::new(mesh)
  }

  pub fn to_strips(&self) -> Result<Teapot<TriangleStrip>> {
    let mesh = TriangleStrip::from(&self.mesh);
    Ok(Teapot {
      rotation: self.rotation,
      rotation_speed: self.rotation_speed,
      render_mode: self.render_mode,
      mesh,
    })
  }
}

impl<Mesh> Teapot<Mesh> {
  pub fn new(mesh: Mesh) -> Result<Self> {
    Ok(Self {
      rotation: 0.0,
      rotation_speed: 1.0,
      render_mode: RenderMode::Default,
      mesh,
    })
  }

  pub fn update(&mut self, dt: Duration) {
    self.rotation += dt.as_secs_f32() * self.rotation_speed;
  }

  pub fn set_render_mode(&mut self, render_mode: RenderMode) {
    println!("render mode: {:?}", render_mode);
    self.render_mode = render_mode;
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
      render_mode: self.render_mode,
      mesh: self.mesh.upload(surface),
    }
  }

  pub fn draw(
    &self,
    frame: &mut Frame,
    camera: &Camera,
    light: &Light,
  ) -> Result<()>
  where
    Mesh: mesh::GPUMeshFormat,
  {
    let mv: Matrix4<f32> = camera.view() * self.model_transform();
    let mv3: Matrix3<f32> = Matrix3 {
      x: mv.x.truncate(),
      y: mv.y.truncate(),
      z: mv.z.truncate(),
    };
    let mv_n: Matrix3<f32> = mv3.invert().unwrap().transpose();
    let mvp: Matrix4<f32> = camera.projection() * mv;

    // in view space
    let light_pos: [f32; 3] = camera
      .view()
      .transform_point(light.position_world().into())
      .into();

    let uniforms = uniform! {
      mvp: <Matrix4<f32> as Into<[[f32; 4]; 4]>>::into(mvp),
      mv: <Matrix4<f32> as Into<[[f32; 4]; 4]>>::into(mv),
      mv_n: <Matrix3<f32> as Into<[[f32; 3]; 3]>>::into(mv_n),
      mode: self.render_mode as u32 as i32,
      k_a: [0.1, 0.1, 0.1f32],
      k_d: [1.0, 0.0, 0.0f32],
      k_s: [1.0, 1.0, 1.0f32],
      light_pos: light_pos,
      light_color: light.color(),
      shininess: 100.0f32,
    };

    // by default the depth buffer is not used.
    let draw_params = DrawParameters {
      depth: glium::Depth {
        test: glium::draw_parameters::DepthTest::IfLess,
        write: true,
        ..Default::default()
      },
      backface_culling:
        glium::draw_parameters::BackfaceCullingMode::CullClockwise,
      ..Default::default()
    };

    self.mesh.draw(frame, &uniforms, &draw_params);

    Ok(())
  }
}
