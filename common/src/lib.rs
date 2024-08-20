pub mod axis;
pub mod gl_boilerplate;
mod glium_helper;
pub mod math;
pub mod mesh;
pub mod obj_loader;
pub mod render;

use std::path::PathBuf;

pub use axis::Axis;
pub use glium_helper::{
  load_program, to_raw_image, DynUniforms, MergedUniform, OwnedMergedUniform,
};
pub use obj_loader::{Group, Mtl, MtlLib, Obj, SimpleObj, VAIdx};
pub use render::{CameraLike, Draw, HasProgram, HasShadow, ToUniforms};

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

  // print cwd
  println!(
    "Current working directory: {:?}",
    std::env::current_dir().unwrap()
  );
  panic!("Could not find {}", name)
}

#[macro_export]
macro_rules! project_asset_path {
  ($name:literal) => {
    concat!(env!("CARGO_MANIFEST_DIR"), "/assets/", $name)
  };
}
