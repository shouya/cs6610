pub mod axis;
pub mod gl_boilerplate;
pub mod obj_loader;

use std::path::Path;

pub use axis::Axis;
pub use obj_loader::RawObj;

pub fn teapot_path() -> &'static Path {
  let path = Path::new("assets/teapot.obj");
  if path.exists() {
    return path;
  }

  let path = Path::new("../assets/teapot.obj");
  if path.exists() {
    return path;
  }

  let path = Path::new("../../assets/teapot.obj");
  if path.exists() {
    return path;
  }

  panic!("Could not find teapot.obj")
}
