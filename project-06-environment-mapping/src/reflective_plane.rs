use core::f32;
use std::{ffi::c_void, fs::read_to_string};

use cgmath::{InnerSpace, Matrix4, Transform, Vector3};
use common::{math::reflect4x4, project_asset_path};
use glium::{
  backend::Facade, framebuffer::SimpleFrameBuffer, texture::DepthTexture2d,
  uniform, Program, Rect, Surface, Texture2d,
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
      .transform_vector(Vector3::unit_z())
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

  pub fn reflected_view(&self, view: &Matrix4<f32>) -> Matrix4<f32> {
    let point = self.object.world_pos();
    let refl = reflect4x4(point, self.normal);
    view * refl
  }

  pub fn update_world_texture(
    &self,
    facade: &impl Facade,
    scene: &Scene,
    camera: &Camera,
  ) -> Result<()> {
    let mut surface =
      SimpleFrameBuffer::with_depth_buffer(facade, &self.texture, &self.depth)?;

    let bbox_clip =
      // needs to use the original camera to get the final screen
      // locations of the plane.
      map_bounding_box(self.object.bounding_box(), camera.view_projection());

    let (view, proj, mirror) =
      (camera.view(), camera.projection(), camera.mirror());
    let view = self.reflected_view(&view);
    // reflect the "mirror" flag to make back-face culling working correctly
    let mut camera = Camera::from_view_projection(view, proj, !mirror);

    // only draw the relevant view port of the plane.
    // calculated by projecting the four coordinates to clip space.
    // and then taking the min and max of the x and y values.
    let (screen_w, screen_h) = surface.get_dimensions();
    let left = screen_w as f32 * (bbox_clip[0][0] + 1.0) / 2.0;
    let width = screen_w as f32 * (bbox_clip[0][1] - bbox_clip[0][0]) / 2.0;
    let bottom = screen_h as f32 * (bbox_clip[1][0] + 1.0) / 2.0;
    let height = screen_h as f32 * (bbox_clip[1][1] - bbox_clip[1][0]) / 2.0;
    let scissor = Rect {
      left: left.floor().max(0.0) as u32,
      width: width.ceil().max(0.0) as u32,
      bottom: bottom.floor().max(0.0) as u32,
      height: height.ceil().max(0.0) as u32,
    };
    camera.set_scissor(scissor);

    scene.draw_with_camera(
      &mut surface,
      &camera,
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

    self.object.draw_with_program(
      target,
      camera,
      light,
      &self.program,
      uniforms,
    );
  }
}

fn map_bounding_box(bbox: [[f32; 2]; 3], map: Matrix4<f32>) -> [[f32; 2]; 3] {
  let [[x1, x2], [y1, y2], [z1, z2]] = bbox;
  let mut vertices = [
    [x1, y1, z1],
    [x1, y1, z2],
    [x1, y2, z1],
    [x1, y2, z2],
    [x2, y1, z1],
    [x2, y1, z2],
    [x2, y2, z1],
    [x2, y2, z2],
  ];

  for vert in &mut vertices {
    *vert = map.transform_point((*vert).into()).into();
  }

  let (mut xmin, mut ymin, mut zmin) = (f32::MAX, f32::MAX, f32::MAX);
  let (mut xmax, mut ymax, mut zmax) = (f32::MIN, f32::MIN, f32::MIN);

  for [x, y, z] in vertices {
    xmin = xmin.min(x);
    ymin = ymin.min(y);
    zmin = zmin.min(z);
    xmax = xmax.max(x);
    ymax = ymax.max(y);
    zmax = zmax.max(z);
  }

  [[xmin, xmax], [ymin, ymax], [zmin, zmax]]
}
