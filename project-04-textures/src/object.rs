use std::any::Any;
use std::path::{Path, PathBuf};

use cgmath::{Matrix as _, Matrix3, Matrix4, SquareMatrix as _, Transform};
use common::{asset_path, teapot_path};
use glium::{uniform, DrawParameters, Program};

use crate::light::Light;
use crate::mesh::{GPUMesh, Mesh};
use crate::{Camera, Result};

pub struct Teapot;

impl Teapot {
  pub fn load(facade: &impl glium::backend::Facade) -> Result<GPUObject> {
    const SHADER_PATH: &str =
      concat!(env!("CARGO_MANIFEST_DIR"), "/assets/shader");
    let mut object = GPUObject::load(&teapot_path(), &SHADER_PATH, facade)?;

    object.model = Matrix4::from_scale(0.05)
      * Matrix4::from_angle_y(cgmath::Rad(0.0))
    // the object itself is rotated 90 to the front, let's rotate it back a little.
      * Matrix4::from_angle_x(cgmath::Deg(-90.0));

    Ok(object)
  }
}

pub struct Yoda;

impl Yoda {
  pub fn load(facade: &impl glium::backend::Facade) -> Result<GPUObject> {
    const SHADER_PATH: &str =
      concat!(env!("CARGO_MANIFEST_DIR"), "/assets/shader");
    let yoda_model = asset_path("yoda/yoda.obj");
    let mut object = GPUObject::load(&yoda_model, &SHADER_PATH, facade)?;

    object.model = Matrix4::from_translation([0.0, -0.3, 0.0].into())
      * Matrix4::from_scale(0.0003)
      * Matrix4::from_angle_y(cgmath::Deg(180.0))
      * Matrix4::from_angle_x(cgmath::Deg(-90.0));

    Ok(object)
  }
}

pub struct GPUObject {
  shader_path: PathBuf,
  program: Program,
  mesh: GPUMesh,
  model: Matrix4<f32>,
}

impl GPUObject {
  pub fn reload_shader(
    &mut self,
    facade: &impl glium::backend::Facade,
  ) -> Result<()> {
    let shader_path = &self.shader_path;

    match load_program(shader_path, facade) {
      Ok(program) => self.program = program,
      Err(e) => {
        eprintln!("Failed to reload shader: {}", e);
      }
    }

    Ok(())
  }

  pub fn load(
    obj_path: &impl AsRef<Path>,
    shader_path: &impl AsRef<Path>,
    facade: &impl glium::backend::Facade,
  ) -> Result<Self> {
    let mesh = Mesh::load(obj_path)?;
    let mesh = mesh.upload(facade)?;
    let program = load_program(shader_path.as_ref(), facade)?;
    let model = Matrix4::identity();

    Ok(Self {
      shader_path: shader_path.as_ref().to_path_buf(),
      program,
      mesh,
      model,
    })
  }

  pub fn update(&mut self, _dt: &std::time::Duration) {}

  pub fn model(&self) -> Matrix4<f32> {
    self.model
  }

  pub fn draw(&self, frame: &mut glium::Frame, camera: &Camera, light: &Light) {
    let mv: Matrix4<f32> = camera.view() * self.model();
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
      mvp: Into::<[[f32; 4]; 4]>::into(mvp),
      mv: Into::<[[f32; 4]; 4]>::into(mv),
      mv_n: Into::<[[f32; 3]; 3]>::into(mv_n),
      light_pos: light_pos,
      light_color: light.color(),
    };

    let draw_params = DrawParameters {
      depth: glium::Depth {
        test: glium::draw_parameters::DepthTest::IfLess,
        write: true,
        ..Default::default()
      },
      ..Default::default()
    };
    let program = &self.program;

    self.mesh.draw(frame, program, &uniforms, &draw_params);
  }
}

fn load_program(
  path: &Path,
  facade: &impl glium::backend::Facade,
) -> Result<Program> {
  let vert_path = path.with_extension("vert");
  let frag_path = path.with_extension("frag");

  let vert_src = std::fs::read_to_string(&vert_path)?;
  let frag_src = std::fs::read_to_string(&frag_path)?;

  Ok(Program::from_source(facade, &vert_src, &frag_src, None)?)
}
