use std::time::Duration;

use derive_more::From;

use common::SimpleObj;
use glam::EulerRot;
use glam::Mat3;
use glam::Mat4;
use glam::Vec3;
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

  fn model_transform(&self) -> Mat4 {
    // the object itself is rotated 90 to the front, let's rotate it back a little.
    Mat4::from_scale(Vec3::splat(0.05))
      * Mat4::from_euler(EulerRot::YXZ, self.rotation, -90f32.to_radians(), 0.0)
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
    let mv: Mat4 = camera.view() * self.model_transform();
    let mv3: Mat3 = Mat3::from_mat4(mv);
    let mv_n: Mat3 = mv3.inverse().transpose();
    let mvp: Mat4 = camera.projection() * mv;

    // in view space
    let light_pos: Vec3 =
      camera.view().transform_point3(light.position_world());

    let uniforms = uniform! {
      mvp: mvp.to_cols_array_2d(),
      mv: mv.to_cols_array_2d(),
      mv_n: mv_n.to_cols_array_2d(),
      mode: self.render_mode as u32 as i32,
      k_a: [0.1, 0.1, 0.1f32],
      k_d: [1.0, 0.0, 0.0f32],
      k_s: [1.0, 1.0, 1.0f32],
      light_pos: light_pos.to_array(),
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
