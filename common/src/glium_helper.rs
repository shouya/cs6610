use std::{
  borrow::Cow,
  collections::{HashMap, HashSet},
  path::Path,
};

use glium::{
  backend::Facade,
  texture::RawImage2d,
  uniforms::{AsUniformValue, UniformValue, Uniforms},
};
use image::RgbImage;

#[derive(Default)]
pub struct DynUniforms<'a> {
  uniforms: HashMap<Cow<'static, str>, UniformValue<'a>>,
}

impl<'a> DynUniforms<'a> {
  pub fn new() -> Self {
    Self {
      uniforms: Default::default(),
    }
  }

  pub fn add(&mut self, name: &'static str, value: &'a dyn AsUniformValue) {
    if self.uniforms.contains_key(name) {
      return;
    }

    self.add_override(name, value);
  }

  pub fn add_raw(&mut self, name: &'static str, value: UniformValue<'a>) {
    if self.uniforms.contains_key(name) {
      return;
    }
    self.add_raw_override(name, value);
  }

  pub fn add_override(
    &mut self,
    name: &'static str,
    value: &'a dyn AsUniformValue,
  ) {
    self.add_raw_override(name, value.as_uniform_value());
  }

  pub fn add_raw_override(
    &mut self,
    name: &'static str,
    value: UniformValue<'a>,
  ) {
    self.uniforms.insert(Cow::Borrowed(name), value);
  }
}

impl Uniforms for DynUniforms<'_> {
  fn visit_values<'a, F: FnMut(&str, UniformValue<'a>)>(&'a self, mut f: F) {
    for (name, value) in &self.uniforms {
      f(name, *value);
    }
  }
}

pub struct OwnedMergedUniform<U1, U2> {
  u1: U1,
  u2: U2,
}

impl<U1, U2> OwnedMergedUniform<U1, U2> {
  pub fn new(u1: U1, u2: U2) -> Self {
    Self { u1, u2 }
  }
}

impl<U1, U2> Uniforms for OwnedMergedUniform<U1, U2>
where
  U1: Uniforms,
  U2: Uniforms,
{
  fn visit_values<'a, F: FnMut(&str, UniformValue<'a>)>(&'a self, mut f: F) {
    let mut visited = HashSet::new();

    self.u1.visit_values(|name, value| {
      if visited.insert(name.to_owned()) {
        f(name, value);
      }
    });

    self.u2.visit_values(|name, value| {
      if !visited.contains(name) && visited.insert(name.to_owned()) {
        f(name, value);
      }
    });
  }
}

pub struct MergedUniform<'a, U1, U2> {
  u1: &'a U1,
  u2: &'a U2,
}

impl<'a, U1, U2> MergedUniform<'a, U1, U2> {
  pub fn new(u1: &'a U1, u2: &'a U2) -> Self {
    Self { u1, u2 }
  }
}

impl<U1, U2> Uniforms for MergedUniform<'_, U1, U2>
where
  U1: Uniforms,
  U2: Uniforms,
{
  fn visit_values<'a, F: FnMut(&str, UniformValue<'a>)>(&'a self, mut f: F) {
    let mut visited = HashSet::new();

    self.u1.visit_values(|name, value| {
      if visited.insert(name.to_owned()) {
        f(name, value);
      }
    });
    self.u2.visit_values(|name, value| {
      if !visited.contains(name) && visited.insert(name.to_owned()) {
        f(name, value)
      }
    });
  }
}

pub fn to_raw_image(image: &RgbImage) -> RawImage2d<'_, u8> {
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

pub fn load_program<P: AsRef<Path>>(
  path: P,
  facade: &impl Facade,
) -> Result<glium::Program, anyhow::Error> {
  let vert_path = path.as_ref().with_extension("vert");
  let frag_path = path.as_ref().with_extension("frag");

  let vert_src = std::fs::read_to_string(vert_path)?;
  let frag_src = std::fs::read_to_string(frag_path)?;

  Ok(glium::Program::from_source(
    facade, &vert_src, &frag_src, None,
  )?)
}
