use std::path::{Path, PathBuf};

use common::{asset_path, teapot_path};
use glam::{EulerRot, Mat3, Mat4, Vec3};
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

    object.model = Mat4::from_scale(Vec3::splat(0.05))
      * Mat4::from_euler(glam::EulerRot::YXZ, 0.0, -90f32.to_radians(), 0.0);

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

    object.model = Mat4::from_translation([0.0, -0.3, 0.0].into())
      * Mat4::from_scale(Vec3::splat(0.0003))
      * Mat4::from_euler(
        EulerRot::YXZ,
        180f32.to_radians(),
        -90f32.to_radians(),
        0.0,
      );

    Ok(object)
  }
}

pub struct GPUObject {
  shader_path: PathBuf,
  program: Program,
  mesh: GPUMesh,
  model: Mat4,
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
    let model = Mat4::IDENTITY;

    Ok(Self {
      shader_path: shader_path.as_ref().to_path_buf(),
      program,
      mesh,
      model,
    })
  }

  pub fn update(&mut self, _dt: &std::time::Duration) {}

  pub fn model(&self) -> Mat4 {
    self.model
  }

  pub fn draw(&self, frame: &mut glium::Frame, camera: &Camera, light: &Light) {
    let mv: Mat4 = camera.view() * self.model();
    let mvp: Mat4 = camera.projection() * mv;

    let mv3: Mat3 = Mat3::from_mat4(mv);
    let mv_n: Mat3 = mv3.inverse().transpose();

    // in view space
    let light_pos: Vec3 =
      camera.view().transform_point3(light.position_world());

    let uniforms = uniform! {
      mvp: mvp.to_cols_array_2d(),
      mv: mv.to_cols_array_2d(),
      mv_n: mv_n.to_cols_array_2d(),
      light_pos: light_pos.to_array(),
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

  let vert_src = std::fs::read_to_string(vert_path)?;
  let frag_src = std::fs::read_to_string(frag_path)?;

  Ok(Program::from_source(facade, &vert_src, &frag_src, None)?)
}
