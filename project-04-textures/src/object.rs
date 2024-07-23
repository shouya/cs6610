use std::path::Path;

use cgmath::{Matrix as _, Matrix3, Matrix4, SquareMatrix as _, Transform};
use glium::{uniform, DrawParameters, Program};

use crate::light::Light;
use crate::mesh::{GPUMesh, Mesh};
use crate::{Camera, Result};

pub struct GPUObject {
  program: Program,
  mesh: GPUMesh,
  model: Matrix4<f32>,
}

impl GPUObject {
  pub fn load(
    obj_path: &impl AsRef<Path>,
    shader_path: &impl AsRef<Path>,
    facade: &impl glium::backend::Facade,
  ) -> Result<Self> {
    let mesh = Mesh::load(obj_path)?;
    let mesh = mesh.upload(facade)?;

    let vert_path = shader_path.as_ref().with_extension("vert");
    let frag_path = shader_path.as_ref().with_extension("frag");

    let program = Program::from_source(
      facade,
      &std::fs::read_to_string(vert_path)?,
      &std::fs::read_to_string(frag_path)?,
      None,
    )?;

    let model = Matrix4::identity();

    Ok(Self {
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
