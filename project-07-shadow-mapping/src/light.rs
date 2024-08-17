use std::{cell::OnceCell, rc::Rc};

use common::{load_program, DynUniforms, OwnedMergedUniform};
use glam::{Mat3, Mat4, Quat, Vec3};
use glium::{
  backend::Facade,
  framebuffer::SimpleFrameBuffer,
  texture::{DepthCubemap, DepthTexture2d},
  uniform,
  uniforms::DepthTextureComparison,
  Surface as _,
};

use crate::{transform::Transform, Camera, Object, Result};

const SHADOW_MAP_RESOLUTION: u32 = 8196;

pub enum LightVariant {
  Directional {
    // direction towards the light
    dir: Vec3,
    map: OnceCell<DepthTexture2d>,
  },
  #[allow(unused)]
  Point {
    pos: Vec3,
    map: OnceCell<DepthCubemap>,
  },
  #[allow(unused)]
  Spot {
    pos: Vec3,
    fov: f32,
    map: OnceCell<DepthTexture2d>,
  },
}

impl LightVariant {
  fn light_uniforms(
    &self,
    light_color: Vec3,
  ) -> impl glium::uniforms::Uniforms + '_ {
    let typ: i32 = match *self {
      LightVariant::Directional { .. } => 0,
      LightVariant::Point { .. } => 1,
      LightVariant::Spot { .. } => 2,
    };
    let dir_or_loc: Vec3 = match *self {
      LightVariant::Directional { dir, .. } => dir,
      LightVariant::Point { pos, .. } => pos,
      LightVariant::Spot { pos, .. } => pos,
    };
    let cone_angle: f32 = match self {
      LightVariant::Directional { .. } => 0.0,
      LightVariant::Point { .. } => 0.0,
      LightVariant::Spot { fov, .. } => fov.to_radians().cos(),
    };

    uniform! {
      light_type: typ,
      light_dir_or_loc: dir_or_loc.to_array(),
      light_cone_angle: cone_angle,
      light_color: light_color.to_array(),
    }
  }

  fn shadow_uniforms(
    &self,
    camera: &Camera,
  ) -> impl glium::uniforms::Uniforms + '_ {
    match self {
      LightVariant::Directional { map, .. } => {
        let camera = self.shadow_space_camera(camera);
        let transform = Mat4::from_translation(Vec3::new(0.5, 0.5, 0.5))
          * Mat4::from_scale(Vec3::new(0.5, 0.5, 0.5));
        let vp = transform * camera.view_projection();

        let sampled_shadow_map = map
          .get()
          .unwrap()
          .sampled()
          .depth_texture_comparison(Some(DepthTextureComparison::LessOrEqual))
          .magnify_filter(glium::uniforms::MagnifySamplerFilter::Linear)
          .minify_filter(glium::uniforms::MinifySamplerFilter::Linear)
          .wrap_function(glium::uniforms::SamplerWrapFunction::Clamp);

        let sampled_shadow_map_debug = map
          .get()
          .unwrap()
          .sampled()
          .magnify_filter(glium::uniforms::MagnifySamplerFilter::Linear)
          .minify_filter(glium::uniforms::MinifySamplerFilter::Linear);

        uniform! {
          shadow_map: sampled_shadow_map,
          shadow_map_debug: sampled_shadow_map_debug,
          shadow_transform: vp.to_cols_array_2d(),
        }
      }
      _ => {
        panic!("Not implemented");
      }
    }
  }

  fn shadow_space_camera(&self, camera: &Camera) -> Camera {
    match self {
      LightVariant::Directional { dir, .. } => {
        // TODO: calculate orthographic projection based on camera frustum
        let proj = crate::Projection::Custom(Mat4::orthographic_rh_gl(
          -10.0, 10.0, -10.0, 10.0, -20.0, 20.0,
        ));
        Camera::new(*dir * 10.0, Vec3::ZERO, proj, 1.0)
      }
      _ => {
        panic!("Not implemented");
      }
    }
  }
}

impl Default for LightVariant {
  fn default() -> Self {
    LightVariant::Directional {
      dir: Vec3::new(0.5, 1.0, 0.5).normalize(),
      map: OnceCell::new(),
    }
  }
}

pub struct Light {
  // each component can be greater than one. pre-multiplied by intensity.
  color: Vec3,
  variant: LightVariant,
  program: Option<Rc<glium::Program>>,
}

impl Default for Light {
  fn default() -> Self {
    Self {
      color: Vec3::ONE,
      variant: LightVariant::default(),
      program: None,
    }
  }
}

impl Light {
  pub fn color(&self) -> Vec3 {
    self.color
  }

  pub fn uniforms(
    &self,
    camera: &Camera,
  ) -> impl glium::uniforms::Uniforms + '_ {
    OwnedMergedUniform::new(
      self.variant.light_uniforms(self.color),
      self.variant.shadow_uniforms(camera),
    )
  }

  pub fn light_object_transform(&self) -> Transform {
    let position = match self.variant {
      LightVariant::Directional { dir, .. } => dir * 2.0,
      LightVariant::Point { pos, .. } => pos,
      LightVariant::Spot { pos, .. } => pos,
    };

    let dir = match self.variant {
      LightVariant::Directional { dir, .. } => dir.normalize(),
      LightVariant::Point { pos, .. } => pos.normalize(),
      LightVariant::Spot { pos, .. } => pos.normalize(),
    };

    Transform {
      translation: position,
      scale: Vec3::splat(0.1),
      rotation: Quat::from_rotation_arc_colinear(Vec3::X, -dir),
    }
  }

  pub fn rotate(&mut self, dx: f32, _dy: f32) {
    let rot = Mat3::from_rotation_y(dx * 0.1);
    match self.variant {
      LightVariant::Directional { ref mut dir, .. } => {
        *dir = rot * *dir;
      }
      LightVariant::Point { ref mut pos, .. } => {
        *pos = rot * *pos;
      }
      LightVariant::Spot { ref mut pos, .. } => {
        *pos = rot * *pos;
      }
    }
  }

  pub fn shadow_map_target(
    &self,
    facade: &impl Facade,
    camera: &Camera,
  ) -> Result<ShadowMapFramebuffer<'_>> {
    match &self.variant {
      LightVariant::Directional { map, .. } => {
        let camera = self.variant.shadow_space_camera(camera);
        let map = map.get_or_init(|| create_shadow_map(facade));
        let framebuffer = SimpleFrameBuffer::depth_only(facade, map)?;

        Ok(ShadowMapFramebuffer::Single {
          camera: Box::new(camera),
          framebuffer: Box::new(framebuffer),
          program: self.program.as_ref().expect("Shadow program not loaded"),
        })
      }
      _ => {
        panic!("Not implemented");
      }
    }
  }

  pub fn load_shadow_program(&mut self, facade: &impl Facade) -> Result<()> {
    const SHADER_PATH: &str =
      concat!(env!("CARGO_MANIFEST_DIR"), "/assets/shadow");
    let program = load_program(SHADER_PATH, facade)?;
    self.program = Some(Rc::new(program));
    Ok(())
  }
}

pub enum ShadowMapFramebuffer<'a> {
  Single {
    camera: Box<Camera>,
    framebuffer: Box<SimpleFrameBuffer<'a>>,
    program: &'a glium::Program,
  },
  Cube {
    cameras: Box<[Camera; 6]>,
    framebuffers: Box<[SimpleFrameBuffer<'a>; 6]>,
    program: &'a glium::Program,
  },
}

impl<'a> ShadowMapFramebuffer<'a> {
  pub fn clear(&mut self) {
    match self {
      ShadowMapFramebuffer::Single { framebuffer, .. } => {
        framebuffer.clear_depth(1.0);
      }
      ShadowMapFramebuffer::Cube { framebuffers, .. } => {
        for framebuffer in framebuffers.iter_mut() {
          framebuffer.clear_depth(1.0);
        }
      }
    }
  }

  pub fn draw_object(&mut self, object: &Object) {
    match self {
      ShadowMapFramebuffer::Single {
        framebuffer,
        camera,
        program,
      } => object.draw_with_program(
        framebuffer.as_mut(),
        camera,
        program,
        DynUniforms::new(),
      ),
      ShadowMapFramebuffer::Cube { .. } => {
        // TODO: draw cubemap with different camera
      }
    }
  }
}

fn create_shadow_map(facade: &impl Facade) -> DepthTexture2d {
  DepthTexture2d::empty(facade, SHADOW_MAP_RESOLUTION, SHADOW_MAP_RESOLUTION)
    .unwrap()
}
