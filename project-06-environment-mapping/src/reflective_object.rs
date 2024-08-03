use std::{cell::Cell, fs::read_to_string, time::Duration};

use cgmath::SquareMatrix as _;
use common::{math, project_asset_path};
use glium::{
  backend::Facade,
  framebuffer::SimpleFrameBuffer,
  texture::{CubeLayer, Cubemap, DepthTexture2d},
  uniform, Program, Surface,
};

use crate::{
  camera::Camera, light::Light, object::GPUObject, scene::Scene, Result,
};

pub struct ReflectiveObject {
  object: GPUObject,
  program: Program,
  cubemap: Cubemap,
  cubemap_depth: DepthTexture2d,
  cubemap_initialized: Cell<bool>,
  cubemap_next_face: Cell<usize>,
}

const CUBEMAP_RESOLUTION: u32 = 128;

impl ReflectiveObject {
  pub fn new(facade: &impl Facade, object: GPUObject) -> Result<Self> {
    let cubemap = Cubemap::empty(facade, CUBEMAP_RESOLUTION)?;
    let cubemap_depth =
      DepthTexture2d::empty(facade, CUBEMAP_RESOLUTION, CUBEMAP_RESOLUTION)?;
    let program = Self::shader(facade)?;
    Ok(Self {
      object,
      cubemap,
      cubemap_depth,
      cubemap_initialized: Cell::new(false),
      cubemap_next_face: Cell::new(0),
      program,
    })
  }

  pub fn shader(facade: &impl Facade) -> Result<Program> {
    // reuse the same vertex shader for normal objects
    let vert_src = read_to_string(project_asset_path!("shader.vert"))?;
    let frag_src =
      read_to_string(project_asset_path!("reflective_object.frag"))?;
    let program = Program::from_source(facade, &vert_src, &frag_src, None)?;
    Ok(program)
  }

  pub fn reload_shader(&mut self, facade: &impl Facade) -> Result<()> {
    self.program = Self::shader(facade)?;
    Ok(())
  }

  pub fn update_cubemap(
    &self,
    facade: &impl Facade,
    scene: &Scene,
  ) -> Result<()> {
    if !self.cubemap_initialized.get() {
      for i in 0..6 {
        self.redraw_cubemap(facade, i, scene)?;
      }
      self.cubemap_initialized.set(true);
    }

    // simulate slow cubemap update by skipping some frames
    if std::env::var("SLOW_CUBEMAP_UPDATE") == Ok("1".into())
      && rand::random::<f32>() > 0.1
    {
      return Ok(());
    }

    let next_face_id = self.cubemap_next_face.get();
    self.cubemap_next_face.set(next_face_id + 1);

    self.redraw_cubemap(facade, next_face_id, scene)?;

    Ok(())
  }

  pub fn redraw_cubemap(
    &self,
    facade: &impl Facade,
    face_id: usize,
    scene: &Scene,
  ) -> Result<()> {
    // reference: https://www.khronos.org/opengl/wiki/Cubemap_Texture
    let layers = [
      (CubeLayer::PositiveX, [1, 0, 0], [0, -1, 0]),
      (CubeLayer::NegativeX, [-1, 0, 0], [0, -1, 0]),
      (CubeLayer::PositiveY, [0, 1, 0], [0, 0, 1]),
      (CubeLayer::NegativeY, [0, -1, 0], [0, 0, -1]),
      (CubeLayer::PositiveZ, [0, 0, 1], [0, -1, 0]),
      (CubeLayer::NegativeZ, [0, 0, -1], [0, -1, 0]),
    ];
    let layer_id = face_id % layers.len();
    let (layer, direction, up) = layers[layer_id];

    let image = self.cubemap.main_level().image(layer);
    let mut framebuffer =
      SimpleFrameBuffer::with_depth_buffer(facade, image, &self.cubemap_depth)?;

    let world_pos = self.object.world_pos();
    // object size
    let camera = Camera::for_cubemap_face(world_pos, direction, up, false);

    scene.draw_with_camera(
      &mut framebuffer,
      &camera,
      self as *const _ as *const _,
    );

    Ok(())
  }

  pub fn draw(
    &self,
    target: &mut impl Surface,
    camera: &Camera,
    light: &Light,
  ) {
    let cubemap = self
      .cubemap
      .sampled()
      .wrap_function(glium::uniforms::SamplerWrapFunction::Repeat)
      .minify_filter(glium::uniforms::MinifySamplerFilter::Linear)
      .magnify_filter(glium::uniforms::MagnifySamplerFilter::Linear);
    let view_inv: [[f32; 3]; 3] =
      math::mat4_to_3(camera.view()).invert().unwrap().into();
    let uniforms = uniform! {
      use_cubemap: 1u32,
      cubemap: cubemap,
      view_inv: view_inv,
    };

    self.object.draw_with_program(
      target,
      camera,
      light,
      &self.program,
      uniforms,
    );
  }

  pub fn update(&mut self, dt: &Duration) {
    self.object.update(dt);
  }
}
