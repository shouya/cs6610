use glam::Mat4;
use glium::{
  backend::Facade, implement_vertex, index::PrimitiveType, program::SourceCode,
  uniform, DrawParameters, Frame, IndexBuffer, Program, Surface as _,
  VertexBuffer,
};

#[derive(Copy, Clone)]
struct AxisVert {
  pos: [f32; 3],
  clr: [f32; 3],
}

implement_vertex!(AxisVert, pos, clr);

pub struct Axis {
  vbo: VertexBuffer<AxisVert>,
  lines_ibo: IndexBuffer<u8>,
  points_ibo: IndexBuffer<u8>,
  program: Program,
}

const VERT_SHADER: &str = r#"
#version 330 core

layout(location = 0) in vec3 pos;
layout(location = 1) in vec3 clr;

out vec3 frag_color;

uniform mat4 mvp;

void main()
{
    gl_Position = mvp * vec4(pos, 1.0);
    frag_color = clr;
}
"#;

const FRAG_SHADER: &str = r#"
#version 330 core

layout(location = 0) out vec4 color;
in vec3 frag_color;

void main() {
  color = vec4(frag_color, 1.0);
}
"#;

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
    let vert = |x, y, z, r, g, b, dim| {
      let factor = if dim == 1 { 0.5 } else { 1.0 };

      AxisVert {
        pos: [x as f32, y as f32, z as f32],
        clr: [r as f32 * factor, g as f32 * factor, b as f32 * factor],
      }
    };
    let verts = [
      // x axis
      vert(0, 0, 0, 1, 0, 0, 0),
      vert(1, 0, 0, 1, 0, 0, 0),
      vert(0, 0, 0, 1, 0, 0, 1),
      vert(-1, 0, 0, 1, 0, 0, 1),
      // y axis
      vert(0, 0, 0, 0, 1, 0, 0),
      vert(0, 1, 0, 0, 1, 0, 0),
      vert(0, 0, 0, 0, 1, 0, 1),
      vert(0, -1, 0, 0, 1, 0, 1),
      // z axis
      vert(0, 0, 0, 0, 0, 1, 0),
      vert(0, 0, 1, 0, 0, 1, 0),
      vert(0, 0, 0, 0, 0, 1, 1),
      vert(0, 0, -1, 0, 0, 1, 1),
    ];
    let lines_indices = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
    let points_indices = [1, 3, 5, 7, 9, 11];

    let vbo = VertexBuffer::new(context, &verts)?;
    let lines_ibo =
      IndexBuffer::new(context, PrimitiveType::LinesList, &lines_indices)?;
    let points_ibo =
      IndexBuffer::new(context, PrimitiveType::Points, &points_indices)?;

    Ok(Self {
      vbo,
      lines_ibo,
      points_ibo,
      program,
    })
  }

  pub fn draw(&self, frame: &mut Frame, view_projection: &Mat4) -> Result<()> {
    let mvp: [[f32; 4]; 4] = view_projection.to_cols_array_2d();

    let uniforms = uniform! {
      mvp: mvp,
    };

    let draw_params = DrawParameters {
      point_size: Some(10.0),
      line_width: Some(3.0),
      ..Default::default()
    };

    frame.draw(
      &self.vbo,
      &self.lines_ibo,
      &self.program,
      &uniforms,
      &draw_params,
    )?;

    frame.draw(
      &self.vbo,
      &self.points_ibo,
      &self.program,
      &uniforms,
      &draw_params,
    )?;

    Ok(())
  }
}
