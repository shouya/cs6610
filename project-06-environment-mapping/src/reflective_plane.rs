use std::{ffi::c_void, fs::read_to_string};

use cgmath::{InnerSpace, Transform, Vector3};
use common::project_asset_path;
use glium::{
  backend::Facade, framebuffer::SimpleFrameBuffer, texture::DepthTexture2d,
  uniform, Program, Surface, Texture2d,
};

use crate::{
  camera::Camera, light::Light, object::GPUObject, scene::Scene, Result,
};

pub struct ReflectivePlane {
  object: GPUObject,
  normal: Vector3<f32>,
  program: Program,
  texture: Texture2d,
  depth: DepthTexture2d,
}

impl ReflectivePlane {
  pub fn new(facade: &impl Facade, object: GPUObject) -> Result<Self> {
    let normal = object
      .model()
      .transform_vector(Vector3::unit_y())
      .normalize();
    let (w, h) = facade.get_context().get_framebuffer_dimensions();
    let texture = Texture2d::empty(facade, w, h)?;
    let depth = DepthTexture2d::empty(facade, w, h)?;
    let program = Self::shader(facade)?;
    Ok(Self {
      object,
      normal,
      program,
      texture,
      depth,
    })
  }

  pub fn handle_resize(&mut self, facade: &impl Facade, (w, h): (u32, u32)) {
    let new_texture = Texture2d::empty(facade, w, h).unwrap();
    // TODO: copy the old texture to the new texture to avoid flickering
    self.texture = new_texture;
    self.depth = DepthTexture2d::empty(facade, w, h).unwrap();
  }

  pub fn shader(facade: &impl Facade) -> Result<Program> {
    // reuse the same vertex shader for normal objects
    let vert_src = read_to_string(project_asset_path!("shader.vert"))?;
    let frag_src =
      read_to_string(project_asset_path!("reflective_plane.frag"))?;
    let program = Program::from_source(facade, &vert_src, &frag_src, None)?;
    Ok(program)
  }

  pub fn reload_shader(&mut self, facade: &impl Facade) -> Result<()> {
    self.program = Self::shader(facade)?;
    Ok(())
  }

  pub fn update_world_texture(
    &self,
    facade: &impl Facade,
    scene: &Scene,
    camera: &Camera,
  ) -> Result<()> {
    // TODO: flip the camera against the plane
    let mut surface =
      SimpleFrameBuffer::with_depth_buffer(facade, &self.texture, &self.depth)?;

    scene.draw_with_camera(
      &mut surface,
      camera,
      self as *const _ as *const c_void,
    );

    Ok(())
  }

  pub fn draw(
    &self,
    target: &mut impl Surface,
    camera: &Camera,
    light: &Light,
  ) {
    let world_texture = self
      .texture
      .sampled()
      .wrap_function(glium::uniforms::SamplerWrapFunction::Repeat)
      // nearest is enough since the texture is expected to have the same
      // resolution as the screen
      .minify_filter(glium::uniforms::MinifySamplerFilter::Nearest)
      .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest);

    let uniforms = uniform! {
      world_texture: world_texture,
    };

    // TODO: only draw the relevant view port of the plane.
    // calculated by projecting the four coordinates to clip space.
    // and then taking the min and max of the x and y values.
    self
      .object
      .draw_raw(target, camera, light, &self.program, uniforms);
  }
}
