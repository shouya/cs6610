use std::mem::size_of;

use cgmath::{Deg, Euler, Matrix4};
use glium::{
  backend::Facade, program::SourceCode, uniform, DrawParameters, Frame,
  Program, Surface as _, VertexBuffer,
};

pub struct Axis {
  model_vbo: VertexBuffer<[f32; 3]>,
  program: Program,
}

const VERT_SHADER: &str = r#"
#version 330 core

layout(location = 0) in vec3 pos;
uniform mat4 mvp;

void main()
{
    gl_Position = mvp * vec4(pos, 1.0);
}
"#;

const FRAG_SHADER: &str = r#"
#version 330 core

layout(location = 0) out vec4 color;
uniform vec3 clr;

void main() {
  color = vec4(clr, 1.0);
}
"#;

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

pub type Error = anyhow::Error;
pub type Result<T> = anyhow::Result<T>;

impl Axis {
  pub fn new<F: Facade>(context: &F) -> Result<Self> {
    let source_code = SourceCode {
      vertex_shader: VERT_SHADER,
      fragment_shader: FRAG_SHADER,
      tessellation_control_shader: None,
      tessellation_evaluation_shader: None,
      geometry_shader: None,
    };
    let program = Program::new(context, source_code)?;
    let verts = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
    let model_vbo = unsafe {
      VertexBuffer::new_raw(context, &verts, VF_F32x3, size_of::<[f32; 3]>())?
    };

    Ok(Self { model_vbo, program })
  }

  pub fn draw_single(
    &self,
    frame: &mut Frame,
    vp: &Matrix4<f32>,
    rot: [f32; 3],
    scale: f32,
    color: [f32; 3],
  ) -> Result<()> {
    let m = Matrix4::from_scale(scale)
      * Matrix4::from(Euler {
        x: Deg(rot[0]),
        y: Deg(rot[1]),
        z: Deg(rot[2]),
      });
    let mvp: [[f32; 4]; 4] = (vp * m).into();
    let uniforms = uniform! {
      mvp: mvp,
      clr: color,
    };

    let draw_params = DrawParameters {
      point_size: Some(10.0),
      line_width: Some(3.0),
      ..Default::default()
    };

    frame.draw(
      &self.model_vbo,
      glium::index::NoIndices(glium::index::PrimitiveType::LineStrip),
      &self.program,
      &uniforms,
      &draw_params,
    )?;

    frame.draw(
      &self.model_vbo,
      glium::index::NoIndices(glium::index::PrimitiveType::Points),
      &self.program,
      &uniforms,
      &draw_params,
    )?;

    Ok(())
  }

  pub fn draw(&self, frame: &mut Frame, camera: &Matrix4<f32>) -> Result<()> {
    self.draw_single(frame, camera, [0.0, 0.0, 0.0], 1.0, [1.0, 0.0, 0.0])?;
    self.draw_single(frame, camera, [0.0, 0.0, 0.0], -1.0, [0.8, 0.0, 0.0])?;
    self.draw_single(frame, camera, [0.0, 0.0, 90.0], 1.0, [0.0, 1.0, 0.0])?;
    self.draw_single(frame, camera, [0.0, 0.0, 90.0], -1.0, [0.0, 0.8, 0.0])?;
    self.draw_single(frame, camera, [0.0, 90.0, 0.0], 1.0, [0.0, 0.0, 1.0])?;
    self.draw_single(frame, camera, [0.0, 90.0, 0.0], -1.0, [0.0, 0.0, 0.8])?;
    Ok(())
  }
}
