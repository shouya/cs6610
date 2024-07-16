use std::{
  collections::{BTreeMap, BTreeSet, HashMap},
  hash::{DefaultHasher, Hasher},
  ops::Range,
};

use common::RawObj;
use glium::{
  implement_vertex, index::PrimitiveType, uniforms::Uniforms, DrawParameters,
  IndexBuffer, Program, VertexBuffer,
};
use rand::Rng as _;

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

type DebuggingColor = [f32; 3];

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

pub struct TriangleStrip {
  vertices: Vec<Vertex>,
  indices: Vec<u32>,
  // used for debugging only
  ranges: Vec<(Range<usize>, DebuggingColor)>,
}

impl From<&TriangleIndex> for TriangleStrip {
  fn from(
    TriangleIndex {
      ref vertices,
      ref indices,
    }: &TriangleIndex,
  ) -> Self {
    let mut ranges = Vec::new();
    let mut total_trigs = 0;
    let expected_total_trigs = indices.len() / 3;

    let strips = tear_into_strips(indices);
    let mut indices = Vec::new();

    for strip in strips {
      total_trigs += strip.len() - 2;

      let mut begin = indices.len();
      let mut end = begin + strip.len();
      if !indices.is_empty() {
        // duplicate the last triangle in previous strip
        indices.extend_from_within(indices.len() - 3..);
        // duplicate the first triangle in next strip
        indices.extend_from_slice(&strip[..3]);

        indices.extend_from_slice(&strip);
        begin += 3;
        end += 3;
      } else {
        indices.extend_from_slice(&strip);
      }

      ranges.push((begin..end, rand_color()));
    }

    // sanity check
    if total_trigs != expected_total_trigs {
      eprintln!(
        "total_trigs: {}, expected_total_trigs: {}",
        total_trigs, expected_total_trigs
      );
      // assert!(total_trigs == expected_total_trigs);
    }

    Self {
      vertices: vertices.clone(),
      ranges,
      indices,
    }
  }
}

impl MeshFormat for TriangleStrip {
  type GPURepr = TriangleStripGPU;

  fn upload(&self, surface: &impl glium::backend::Facade) -> Self::GPURepr {
    let vbo =
      VertexBuffer::new(surface, &self.vertices).expect("Failed to create VBO");
    let ibo =
      IndexBuffer::new(surface, PrimitiveType::TriangleStrip, &self.indices)
        .expect("Failed to create IBO");
    let program = Program::from_source(surface, VERT_SHADER, FRAG_SHADER, None)
      .expect("Failed to create program");

    eprintln!(
      "TriangleStrip, buffer size: {} ({}/{})",
      vbo.get_size() + ibo.get_size(),
      vbo.get_size(),
      ibo.get_size()
    );

    TriangleStripGPU {
      program,
      vbo,
      ibo,
      ranges: self.ranges.clone(),
    }
  }
}

pub struct TriangleStripGPU {
  program: Program,
  vbo: VertexBuffer<Vertex>,
  ibo: IndexBuffer<u32>,
  ranges: Vec<(Range<usize>, DebuggingColor)>,
}

impl GPUMeshFormat for TriangleStripGPU {
  fn draw(
    &self,
    frame: &mut impl glium::Surface,
    uniforms: &impl Uniforms,
    params: &DrawParameters<'_>,
  ) {
    if std::env::var("DEBUG_TRIANGLE_STRIP").as_deref() == Ok("1") {
      let mut new_uniforms = OverridingUniforms::from(uniforms);

      for (range, color) in &self.ranges {
        new_uniforms.set("clr", color);
        let ibo = self.ibo.slice(range.clone()).unwrap();
        frame
          .draw(&self.vbo, &ibo, &self.program, &new_uniforms, params)
          .expect("Failed to draw");
      }
    } else {
      frame
        .draw(&self.vbo, &self.ibo, &self.program, uniforms, params)
        .expect("Failed to draw");
    }
  }
}

fn tear_into_strips(indices: &[u32]) -> Vec<Vec<u32>> {
  let mut trigs: BTreeSet<[u32; 3]> = BTreeSet::new();
  // line => (trig, vert)
  let mut colinear_trigs: BTreeMap<[u32; 2], Vec<([u32; 3], u32)>> =
    BTreeMap::new();

  for trig in indices.chunks(3) {
    let mut t = [trig[0], trig[1], trig[2]];
    t.sort();
    trigs.insert(t);

    colinear_trigs
      .entry([t[0], t[1]])
      .or_default()
      .push((t, t[2]));
    colinear_trigs
      .entry([t[0], t[2]])
      .or_default()
      .push((t, t[1]));
    colinear_trigs
      .entry([t[1], t[2]])
      .or_default()
      .push((t, t[0]));
  }

  let mut strips = Vec::new();

  while let Some([a, mut b, mut c]) = trigs.pop_first() {
    let mut strip = vec![a, b, c];

    loop {
      let mut line = [b, c];
      line.sort();

      let next_trig = colinear_trigs[&line]
        .iter()
        .filter_map(|(t, d)| trigs.remove(t).then_some(*d))
        .next();

      let Some(d) = next_trig else {
        break;
      };

      // add vertex to strip and mark the triangle as processed
      strip.push(d);

      b = c;
      c = d;
    }

    strips.push(strip);
  }

  validate_trig_strips(indices, &strips);

  strips
}

fn rand_color() -> DebuggingColor {
  let mut rng = rand::thread_rng();

  [
    rng.gen_range(0.0..1.0),
    rng.gen_range(0.0..1.0),
    rng.gen_range(0.0..1.0),
  ]
}

struct OverridingUniforms<'a, U> {
  existing: U,
  overrides: HashMap<&'static str, glium::uniforms::UniformValue<'a>>,
}

impl<U> From<U> for OverridingUniforms<'_, U> {
  fn from(existing: U) -> Self {
    Self {
      existing,
      overrides: Default::default(),
    }
  }
}

impl<'a, U> OverridingUniforms<'a, U> {
  fn set(
    &mut self,
    name: &'static str,
    value: &'a impl glium::uniforms::AsUniformValue,
  ) {
    self.overrides.insert(name, value.as_uniform_value());
  }
}

impl<U: Uniforms> Uniforms for OverridingUniforms<'_, &U> {
  fn visit_values<'a, F: FnMut(&str, glium::uniforms::UniformValue<'a>)>(
    &'a self,
    mut output: F,
  ) {
    self.existing.visit_values(|name, value| {
      if let Some(override_value) = self.overrides.get(name) {
        output(name, *override_value);
      } else {
        output(name, value);
      }
    });
  }
}

fn validate_trig_strips(indices: &[u32], strips: &[Vec<u32>]) {
  let mut original_trigs: Vec<[u32; 3]> = indices
    .chunks(3)
    .map(|x| x.try_into().unwrap())
    .map(|mut trig: [u32; 3]| {
      trig.sort();
      trig
    })
    .collect();

  original_trigs.sort();

  let mut strip_trigs: Vec<[u32; 3]> = strips
    .iter()
    .flat_map(|strip| {
      strip.windows(3).map(|x| x.try_into().unwrap()).map(
        |mut trig: [u32; 3]| {
          trig.sort();
          trig
        },
      )
    })
    .collect();
  strip_trigs.sort();

  assert_eq!(original_trigs, strip_trigs);
}
