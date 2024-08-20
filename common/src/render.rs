use glium::{uniforms::Uniforms, DrawParameters};

pub trait CameraLike {
  fn view(&self) -> [[f32; 4]; 4];
  fn projection(&self) -> [[f32; 4]; 4];
}

pub trait ToUniforms<'a> {
  fn to_uniforms(&self) -> impl Uniforms;
}

pub trait Draw {
  fn draw_raw(
    &self,
    frame: &mut impl glium::Surface,
    camera: &impl CameraLike,
    program: &glium::Program,
    uniforms: impl Uniforms,
    draw_params: Option<DrawParameters>,
  ) -> anyhow::Result<()>;
}

pub trait HasProgram {
  fn program(&self) -> &glium::Program;
}

pub trait HasShadow {
  fn shadow_program(&self) -> Option<&glium::Program> {
    None
  }
  fn casts_shadow(&self) -> bool {
    true
  }
}

pub trait HasModel {
  fn model(&self) -> [[f32; 4]; 4];
}
