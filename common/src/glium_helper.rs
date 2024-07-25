use std::{
  borrow::Cow,
  collections::{HashMap, HashSet},
};

use glium::uniforms::{AsUniformValue, UniformValue, Uniforms};

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
