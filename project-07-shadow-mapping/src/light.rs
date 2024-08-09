use glam::{Mat3, Vec3};
use glium::{
  texture::{DepthCubemap, DepthTexture2d},
  uniform,
};

use crate::transform::Transform;

pub enum LightVariant {
  Directional {
    // direction towards the light
    dir: Vec3,
  },
  Point {
    pos: Vec3,
  },
  Spot {
    pos: Vec3,
    fov: f32,
  },
}

impl Default for LightVariant {
  fn default() -> Self {
    LightVariant::Directional {
      dir: Vec3::new(0.5, 1.0, 0.5).normalize(),
    }
  }
}

pub enum ShadowMap {
  Directional { map: DepthTexture2d },
  Point { map: DepthCubemap },
  Spot { map: DepthTexture2d },
}

pub struct Light {
  // each component can be greater than one. pre-multiplied by intensity.
  color: Vec3,
  variant: LightVariant,
  shadow_map: Option<ShadowMap>,
}

impl Default for Light {
  fn default() -> Self {
    Self {
      color: Vec3::ONE,
      variant: LightVariant::default(),
      shadow_map: None,
    }
  }
}

impl Light {
  pub fn color(&self) -> Vec3 {
    self.color
  }

  pub fn uniforms(&self) -> impl glium::uniforms::Uniforms {
    let typ: i32 = match self.variant {
      LightVariant::Directional { .. } => 0,
      LightVariant::Point { .. } => 1,
      LightVariant::Spot { .. } => 2,
    };
    let dir_or_loc: Vec3 = match self.variant {
      LightVariant::Directional { dir } => dir,
      LightVariant::Point { pos } => pos,
      LightVariant::Spot { pos, .. } => pos,
    };
    let cone_angle: f32 = match self.variant {
      LightVariant::Directional { .. } => 0.0,
      LightVariant::Point { .. } => 0.0,
      LightVariant::Spot { fov, .. } => fov.to_radians().cos(),
    };

    uniform! {
      light_type: typ,
      light_dir_or_loc: dir_or_loc.to_array(),
      light_cone_angle: cone_angle,
      light_color: self.color.to_array(),
    }
  }

  pub fn light_object_transform(&self) -> Transform {
    let position = match self.variant {
      LightVariant::Directional { dir } => dir * 2.0,
      LightVariant::Point { pos } => pos,
      LightVariant::Spot { pos, .. } => pos,
    };

    Transform {
      translation: position,
      rotation: Vec3::ZERO,
      scale: Vec3::splat(0.1),
    }
  }

  pub fn rotate(&mut self, dx: f32, _dy: f32) {
    let rot = Mat3::from_rotation_y(dx * 0.1);
    match self.variant {
      LightVariant::Directional { ref mut dir } => {
        *dir = rot * *dir;
      }
      LightVariant::Point { ref mut pos } => {
        *pos = rot * *pos;
      }
      LightVariant::Spot { ref mut pos, .. } => {
        *pos = rot * *pos;
      }
    }
  }
}
