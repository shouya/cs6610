pub mod axis;
pub mod gl_boilerplate;
mod glium_helper;
pub mod mesh;
pub mod obj_loader;

use std::path::PathBuf;

pub use axis::Axis;
pub use glium_helper::{DynUniforms, MergedUniform};
pub use obj_loader::{Group, Mtl, MtlLib, Obj, SimpleObj, VAIdx};

pub fn teapot_path() -> PathBuf {
  asset_path("teapot.obj")
}

pub fn sphere_path() -> PathBuf {
  asset_path("sphere.obj")
}

pub fn asset_path(name: &str) -> PathBuf {
  let path = PathBuf::from(format!("assets/{}", name));
  if path.exists() {
    return path;
  }

  let path = PathBuf::from(format!("../assets/{}", name));
  if path.exists() {
    return path;
  }

  let path = PathBuf::from(format!("../../assets/{}", name));
  if path.exists() {
    return path;
  }

  panic!("Could not find {}", name)
}
