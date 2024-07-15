use common::RawObj;
use glium::{
  implement_vertex, uniforms::Uniforms, DrawParameters, Program, VertexBuffer,
};

pub trait MeshFormat {
  type GPURepr: GPUMeshFormat;

  fn upload(&self, surface: &impl glium::backend::Facade) -> Self::GPURepr;
}

const VERT_SHADER: &str = include_str!("../assets/mesh.vert");
const FRAG_SHADER: &str = include_str!("../assets/mesh.frag");

pub trait GPUMeshFormat {
  fn draw(
    &self,
    frame: &mut impl glium::Surface,
    uniforms: &impl Uniforms,
    params: &DrawParameters<'_>,
  );
}

#[derive(Copy, Clone)]
struct Vertex {
  pos: [f32; 3],
  uv: [f32; 2],
  n: [f32; 3],
}

implement_vertex!(Vertex, pos, uv, n);

pub struct TriangleList {
  trigs: Box<[Vertex]>,
}

impl TriangleList {
  pub fn from_raw_obj(raw_obj: RawObj) -> Self {
    let mut trigs = Vec::new();
    let to_vert_attr = |[v, vt, vn]: [usize; 3]| Vertex {
      pos: raw_obj.v[v - 1],
      uv: [raw_obj.vt[vt - 1][0], raw_obj.vt[vt - 1][1]],
      n: raw_obj.vn[vn - 1],
    };

    for trig in raw_obj.trigs() {
      let [a, b, c] = trig;
      trigs.push(to_vert_attr(a));
      trigs.push(to_vert_attr(b));
      trigs.push(to_vert_attr(c));
    }

    Self {
      trigs: trigs.into_boxed_slice(),
    }
  }
}

impl MeshFormat for TriangleList {
  type GPURepr = TriangleListGPU;

  fn upload(&self, surface: &impl glium::backend::Facade) -> Self::GPURepr {
    let vbo =
      VertexBuffer::new(surface, &self.trigs).expect("Failed to create VBO");
    let program = Program::from_source(surface, VERT_SHADER, FRAG_SHADER, None)
      .expect("Failed to create program");
    Self::GPURepr { program, vbo }
  }
}

pub struct TriangleListGPU {
  program: Program,
  vbo: VertexBuffer<Vertex>,
}

impl GPUMeshFormat for TriangleListGPU {
  fn draw(
    &self,
    frame: &mut impl glium::Surface,
    uniforms: &impl Uniforms,
    params: &DrawParameters<'_>,
  ) {
    frame
      .draw(
        &self.vbo,
        glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList),
        &self.program,
        uniforms,
        params,
      )
      .expect("Failed to draw");
  }
}
