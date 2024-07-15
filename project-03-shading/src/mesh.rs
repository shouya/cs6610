use std::{
  collections::HashMap,
  hash::{DefaultHasher, Hasher},
};

use common::RawObj;
use glium::{
  implement_vertex, index::PrimitiveType, uniforms::Uniforms, DrawParameters,
  IndexBuffer, Program, VertexBuffer,
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

impl Vertex {
  fn as_bytes(&self) -> &[u8] {
    unsafe {
      std::slice::from_raw_parts(
        self as *const Self as *const u8,
        std::mem::size_of::<Self>(),
      )
    }
  }
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

    eprintln!("TriangleList, buffer size: {} (v)", vbo.get_size());

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

pub struct TriangleIndex {
  vertices: Vec<Vertex>,
  indices: Vec<u32>,
}

impl TriangleIndex {
  pub fn from_raw_obj(raw_obj: RawObj) -> Self {
    // we cannot store vertex because f32 is not Eq. Here we are
    // assuming the hash function is one-to-one for all values we have.
    let mut vert_index: HashMap<u64, usize> = Default::default();
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let to_vert_attr = |[v, vt, vn]: [usize; 3]| Vertex {
      pos: raw_obj.v[v - 1],
      uv: [raw_obj.vt[vt - 1][0], raw_obj.vt[vt - 1][1]],
      n: raw_obj.vn[vn - 1],
    };

    for trig in raw_obj.trigs() {
      for v in trig {
        let va = to_vert_attr(v);
        let hash = {
          let mut hasher = DefaultHasher::new();
          hasher.write(va.as_bytes());
          hasher.finish()
        };

        let i = vert_index.entry(hash).or_insert_with(|| {
          let i = vertices.len();
          vertices.push(va);
          i
        });

        indices.push(*i as u32);
      }
    }

    Self { vertices, indices }
  }
}

impl MeshFormat for TriangleIndex {
  type GPURepr = TriangleIndexGPU;

  fn upload(&self, surface: &impl glium::backend::Facade) -> Self::GPURepr {
    let vbo =
      VertexBuffer::new(surface, &self.vertices).expect("Failed to create VBO");
    let ibo =
      IndexBuffer::new(surface, PrimitiveType::TrianglesList, &self.indices)
        .expect("Failed to create IBO");
    let program = Program::from_source(surface, VERT_SHADER, FRAG_SHADER, None)
      .expect("Failed to create program");

    eprintln!(
      "TriangleIndex, buffer size: {} ({}/{})",
      vbo.get_size() + ibo.get_size(),
      vbo.get_size(),
      ibo.get_size()
    );

    TriangleIndexGPU { program, vbo, ibo }
  }
}

pub struct TriangleIndexGPU {
  program: Program,
  vbo: VertexBuffer<Vertex>,
  ibo: IndexBuffer<u32>,
}

impl GPUMeshFormat for TriangleIndexGPU {
  fn draw(
    &self,
    frame: &mut impl glium::Surface,
    uniforms: &impl Uniforms,
    params: &DrawParameters<'_>,
  ) {
    frame
      .draw(&self.vbo, &self.ibo, &self.program, uniforms, params)
      .expect("Failed to draw");
  }
}
