#![allow(non_snake_case)]

use common::{
  mesh::{concat_strips, tear_into_strips},
  DynUniforms, MergedUniform, Mtl,
};
use image::RgbImage;
use std::{borrow::Cow, collections::HashMap, ops::Range};

use crate::Result;
use common::obj_loader::{MtlLib, Obj, VAIdx};
use glium::{
  backend::Facade, implement_vertex, texture::RawImage2d,
  uniforms::UniformValue, Texture2d,
};

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

  fn hash(&self) -> u64 {
    use std::hash::{DefaultHasher, Hasher as _};

    let mut hasher = DefaultHasher::new();
    hasher.write(self.as_bytes());
    hasher.finish()
  }
}

implement_vertex!(Vertex, pos, uv, n);

#[derive(Clone)]
struct Group {
  #[allow(dead_code)]
  name: String,
  index_range: Range<u32>,
  mtl: Option<String>,
}

pub struct Mesh {
  vertices: Vec<Vertex>,
  indices: Vec<u32>,
  mtl_lib: MtlLib,
  groups: Vec<Group>,
}

impl Mesh {
  pub fn load<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
    let obj = Obj::load_from(&path)?;
    Ok(Self::from_obj(obj))
  }

  pub fn from_obj(obj: Obj) -> Self {
    // hash(vertex) -> index
    let mut vert_index: HashMap<u64, usize> = HashMap::new();
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut groups = Vec::new();

    let to_vert_attr = |[v, vt, vn]: VAIdx| Vertex {
      pos: obj.v[v - 1],
      // the y component of the uv-coordinates is reversed!
      uv: [obj.vt[vt - 1][0], -obj.vt[vt - 1][1]],
      n: obj.vn[vn - 1],
    };

    for group in obj.groups {
      let mut group_indices = Vec::new();
      for trig in group.trigs() {
        for v in trig {
          let va = to_vert_attr(v);
          let hash = va.hash();
          let i = vert_index.entry(hash).or_insert_with(|| {
            vertices.push(va);
            vertices.len() - 1
          });
          group_indices.push(*i as u32);
        }
      }
      let strips = tear_into_strips(&group_indices);
      let group_indices = concat_strips(&strips);
      let begin = indices.len() as u32;
      indices.extend_from_slice(&group_indices);
      let range = begin..(indices.len() as u32);
      let group = Group {
        name: group.name,
        index_range: range,
        mtl: group.usemtl,
      };
      groups.push(group);
    }

    let mtl_lib = obj.mtl_lib;
    Self {
      vertices,
      indices,
      mtl_lib,
      groups,
    }
  }

  pub fn upload(&self, facade: &impl Facade) -> Result<GPUMesh> {
    let vbo = glium::VertexBuffer::new(facade, &self.vertices)?;
    let ibo = glium::IndexBuffer::new(
      facade,
      glium::index::PrimitiveType::TriangleStrip,
      &self.indices,
    )?;
    let mtls = self
      .mtl_lib
      .mtls
      .iter()
      .map(|mtl| {
        let gpu_mtl = GPUMtl::upload_from(mtl, facade)?;
        Ok((mtl.name.clone(), gpu_mtl))
      })
      .collect::<Result<HashMap<_, _>>>()?;
    let groups = self.groups.clone();

    Ok(GPUMesh {
      vbo,
      ibo,
      groups,
      mtls,
    })
  }
}

pub struct GPUMtl {
  Ns: f32,
  Ni: f32,
  d: f32,
  Tr: f32,
  Tf: [f32; 3],
  illum: u32,
  Ka: [f32; 3],
  Kd: [f32; 3],
  Ks: [f32; 3],
  Ke: [f32; 3],
  map_Ka: Option<glium::texture::Texture2d>,
  map_Kd: Option<glium::texture::Texture2d>,
  map_Ks: Option<glium::texture::Texture2d>,
  map_bump: Option<glium::texture::Texture2d>,
  use_map_Ka: u32,
  use_map_Kd: u32,
  use_map_Ks: u32,
  use_map_bump: u32,
}

impl GPUMtl {
  fn upload_from(mtl: &Mtl, facade: &impl Facade) -> Result<Self> {
    let mut gpu_mtl = Self {
      Ns: mtl.Ns,
      Ni: mtl.Ni,
      d: mtl.d,
      Tr: mtl.Tr,
      Tf: mtl.Tf,
      illum: mtl.illum,
      Ka: mtl.Ka,
      Kd: mtl.Kd,
      Ks: mtl.Ks,
      Ke: mtl.Ke,
      map_Ka: None,
      map_Kd: None,
      map_Ks: None,
      map_bump: None,
      use_map_Ka: 0,
      use_map_Kd: 0,
      use_map_Ks: 0,
      use_map_bump: 0,
    };

    if let Some(img) = &mtl.map_Ka {
      gpu_mtl.map_Ka = Some(upload_texture(facade, img));
      gpu_mtl.use_map_Ka = 1;
    }

    if let Some(img) = &mtl.map_Kd {
      gpu_mtl.map_Kd = Some(upload_texture(facade, img));
      gpu_mtl.use_map_Kd = 1;
    }

    if let Some(img) = &mtl.map_Ks {
      gpu_mtl.map_Ks = Some(upload_texture(facade, img));
      gpu_mtl.use_map_Ks = 1;
    }

    if let Some(img) = &mtl.map_bump {
      gpu_mtl.map_bump = Some(upload_texture(facade, img));
      gpu_mtl.use_map_bump = 1;
    }

    Ok(gpu_mtl)
  }

  fn to_uniforms(&self) -> impl glium::uniforms::Uniforms + '_ {
    let mut uniforms = DynUniforms::new();

    uniforms.add("Ns", &self.Ns);
    uniforms.add("Ni", &self.Ni);
    uniforms.add("d", &self.d);
    uniforms.add("Tr", &self.Tr);
    uniforms.add("Tf", &self.Tf);
    uniforms.add("illum", &self.illum);
    uniforms.add("Ka", &self.Ka);
    uniforms.add("Kd", &self.Kd);
    uniforms.add("Ks", &self.Ks);
    uniforms.add("Ke", &self.Ke);
    uniforms.add("use_map_Ka", &self.use_map_Ka);
    uniforms.add("use_map_Kd", &self.use_map_Kd);
    uniforms.add("use_map_Ks", &self.use_map_Ks);
    uniforms.add("use_map_bump", &self.use_map_bump);

    if let Some(map_Ka) = &self.map_Ka {
      uniforms.add_raw(
        "map_Ka",
        UniformValue::Texture2d(map_Ka, Some(sampler_behavior_Ka())),
      );
    }

    if let Some(map_Kd) = &self.map_Kd {
      uniforms.add_raw(
        "map_Kd",
        UniformValue::Texture2d(map_Kd, Some(sampler_behavior_Kd())),
      );
    }

    if let Some(map_Ks) = &self.map_Ks {
      uniforms.add_raw(
        "map_Ks",
        UniformValue::Texture2d(map_Ks, Some(sampler_behavior_Ks())),
      );
    }

    if let Some(map_bump) = &self.map_bump {
      uniforms.add_raw(
        "map_bump",
        UniformValue::Texture2d(map_bump, Some(sampler_behavior_bump())),
      );
    }

    uniforms
  }
}

pub struct GPUMesh {
  vbo: glium::VertexBuffer<Vertex>,
  ibo: glium::IndexBuffer<u32>,
  groups: Vec<Group>,
  mtls: HashMap<String, GPUMtl>,
}

impl GPUMesh {
  pub fn draw(
    &self,
    frame: &mut impl glium::Surface,
    program: &glium::Program,
    uniforms: &impl glium::uniforms::Uniforms,
    params: &glium::DrawParameters<'_>,
  ) {
    for group in &self.groups {
      let mtl = group.mtl.as_deref().and_then(|name| self.mtls.get(name));
      let range: Range<usize> =
        (group.index_range.start as usize)..(group.index_range.end as usize);
      let ibo_slice = self.ibo.slice(range).unwrap();

      if let Some(mtl) = mtl {
        let mtl_uniforms = mtl.to_uniforms();
        let uniforms = MergedUniform::new(uniforms, &mtl_uniforms);
        frame
          .draw(&self.vbo, &ibo_slice, program, &uniforms, params)
          .expect("Failed to draw");
      } else {
        frame
          .draw(&self.vbo, &ibo_slice, program, uniforms, params)
          .expect("Failed to draw");
      }
    }
  }
}

pub const fn sampler_behavior_Kd() -> glium::uniforms::SamplerBehavior {
  use glium::uniforms::SamplerWrapFunction;

  glium::uniforms::SamplerBehavior {
    wrap_function: (
      SamplerWrapFunction::Repeat,
      SamplerWrapFunction::Repeat,
      SamplerWrapFunction::Repeat,
    ),
    magnify_filter: glium::uniforms::MagnifySamplerFilter::Linear,
    minify_filter: glium::uniforms::MinifySamplerFilter::Linear,
    max_anisotropy: 4,
    depth_texture_comparison: None,
  }
}

pub const fn sampler_behavior_Ks() -> glium::uniforms::SamplerBehavior {
  glium::uniforms::SamplerBehavior {
    max_anisotropy: 1,
    ..sampler_behavior_Kd()
  }
}

pub const fn sampler_behavior_Ka() -> glium::uniforms::SamplerBehavior {
  sampler_behavior_Ks()
}

const fn sampler_behavior_bump() -> glium::uniforms::SamplerBehavior {
  sampler_behavior_Ks()
}

fn to_raw_image(image: &RgbImage) -> RawImage2d<'_, u8> {
  let width = image.width();
  let height = image.height();
  let format = glium::texture::ClientFormat::U8U8U8;
  let data = image.as_raw().clone();

  RawImage2d {
    data: Cow::Owned(data),
    width,
    height,
    format,
  }
}

fn upload_texture(facade: &impl Facade, image: &RgbImage) -> Texture2d {
  let texture = Texture2d::new(facade, to_raw_image(image)).unwrap();
  unsafe { texture.generate_mipmaps() };
  texture
}
