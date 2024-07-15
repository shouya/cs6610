use std::{mem::size_of, path::Path, time::Duration};

use cgmath::{Matrix4, Vector3};
use common::RawObj;
use glium::Surface as _;
use glium::{
  backend::Facade, program::SourceCode, uniform, DrawParameters, Frame,
  Program, VertexBuffer,
};

use crate::Result;
use crate::{Camera, SHADER_PATH};

#[allow(non_upper_case_globals)]
const VF_F32x3: glium::vertex::VertexFormat = &[(
  // attribute name
  std::borrow::Cow::Borrowed("pos"),
  // byte offset
  0,
  // this field was undocumented, maybe stride?
  0,
  // attribute type (F32F32F32)
  glium::vertex::AttributeType::F32F32F32,
  // does it need to be normalized?
  false,
)];

pub struct Teapot {
  rotation: f32,
  rotation_speed: f32,
  model_vbo: VertexBuffer<[f32; 3]>,
  program: Program,
  center: Vector3<f32>,
}

impl Teapot {
  pub fn recompile_shader<F: Facade>(&mut self, context: &F) -> Result<()> {
    let shaders_path = Path::new(SHADER_PATH);
    let vert_shader_path = shaders_path.with_extension("vert");
    let frag_shader_path = shaders_path.with_extension("frag");

    self.program = Program::new(
      context,
      SourceCode {
        vertex_shader: &std::fs::read_to_string(vert_shader_path)?,
        fragment_shader: &std::fs::read_to_string(frag_shader_path)?,
        tessellation_control_shader: None,
        tessellation_evaluation_shader: None,
        geometry_shader: None,
      },
    )?;

    Ok(())
  }

  pub fn load_file<F: Facade>(
    context: &F,
    model_path: &Path,
    shaders_path: &Path,
  ) -> Result<Self> {
    let model = RawObj::load_from(model_path)?;
    let vert_shader_path = shaders_path.with_extension("vert");
    let frag_shader_path = shaders_path.with_extension("frag");

    let source_code = SourceCode {
      vertex_shader: &std::fs::read_to_string(vert_shader_path)?,
      fragment_shader: &std::fs::read_to_string(frag_shader_path)?,
      tessellation_control_shader: None,
      tessellation_evaluation_shader: None,
      geometry_shader: None,
    };
    let program = Program::new(context, source_code)?;
    Self::new(context, &model, program)
  }

  fn new<F: Facade>(
    context: &F,
    model: &RawObj,
    program: Program,
  ) -> Result<Self> {
    eprintln!("Loaded model with {} vertices", model.v.len());
    let model_vbo = unsafe {
      VertexBuffer::new_raw(context, &model.v, VF_F32x3, size_of::<[f32; 3]>())?
    };
    let center = Vector3::from(model.center());

    Ok(Self {
      rotation: 0.0,
      rotation_speed: 1.0,
      model_vbo,
      program,
      center,
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
      * Matrix4::from_translation(-self.center)
  }

  pub fn draw(&self, frame: &mut Frame, camera: &Camera) -> Result<()> {
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

    frame.draw(
      &self.model_vbo,
      glium::index::NoIndices(glium::index::PrimitiveType::Points),
      &self.program,
      &uniforms,
      &draw_params,
    )?;

    Ok(())
  }
}
