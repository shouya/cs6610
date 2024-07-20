pub mod axis;
pub mod gl_boilerplate;
pub mod obj_loader;

use std::path::PathBuf;

pub use axis::Axis;
pub use obj_loader::RawObj;

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
