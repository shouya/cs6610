use std::rc::Rc;

use common::{load_program, Draw, DynUniforms, HasShadow, OwnedMergedUniform};
use glam::{Mat3, Mat4, Quat, Vec3, Vec4Swizzles as _};
use glium::{
  backend::Facade,
  framebuffer::SimpleFrameBuffer,
  implement_vertex,
  texture::{DepthTexture2d, MipmapsOption},
  uniform,
  uniforms::DepthTextureComparison,
  DrawParameters, Program, Surface as _,
};

use crate::{transform::Transform, Camera, Result};

const SHADOW_MAP_RESOLUTION: u32 = 1024;

pub enum LightVariant {
  Directional {
    // direction towards the light
    dir: Vec3,
  },
  Spot {
    pos: Vec3,
    fov: f32,
  },
}

impl LightVariant {
  fn new() -> Self {
    LightVariant::Directional {
      dir: Vec3::new(2.1, 2.2, 1.0).normalize(),
    }
  }

  fn toggle(&mut self) {
    *self = match *self {
      LightVariant::Directional { dir } => LightVariant::Spot {
        pos: dir * 2.0,
        fov: 60f32.to_radians(),
      },
      LightVariant::Spot { pos, .. } => LightVariant::Directional {
        dir: pos.normalize(),
      },
    };
  }

  fn light_uniforms(
    &self,
    light_color: Vec3,
  ) -> impl glium::uniforms::Uniforms + '_ {
    let typ: i32 = match *self {
      LightVariant::Directional { .. } => 0,
      LightVariant::Spot { .. } => 1,
    };
    let dir_or_loc: Vec3 = match *self {
      LightVariant::Directional { dir, .. } => dir,
      LightVariant::Spot { pos, .. } => pos,
    };
    let cone_angle: f32 = match self {
      LightVariant::Directional { .. } => 0.0,
      LightVariant::Spot { fov, .. } => *fov,
    };

    uniform! {
      light_type: typ,
      light_dir_or_loc: dir_or_loc.to_array(),
      light_cone_angle: cone_angle,
      light_color: light_color.to_array(),
    }
  }

  fn shadow_uniforms<'a>(
    &'a self,
    camera: &Camera,
    map: &'a DepthTexture2d,
  ) -> impl glium::uniforms::Uniforms + 'a {
    let camera = self.shadow_space_camera(camera);
    let transform = Mat4::from_translation(Vec3::new(0.5, 0.5, 0.5))
      * Mat4::from_scale(Vec3::new(0.5, 0.5, 0.5));
    let vp = transform * camera.view_projection();

    let sampled_shadow_map = map
      .sampled()
      .depth_texture_comparison(Some(DepthTextureComparison::LessOrEqual))
      .magnify_filter(glium::uniforms::MagnifySamplerFilter::Linear)
      .minify_filter(glium::uniforms::MinifySamplerFilter::Nearest)
      .wrap_function(glium::uniforms::SamplerWrapFunction::BorderClamp)
      .border_color(Some([1.0, 1.0, 1.0, 1.0]));

    uniform! {
      shadow_map: sampled_shadow_map,
      shadow_transform: vp.to_cols_array_2d(),
    }
  }

  fn shadow_space_camera(&self, camera: &Camera) -> Camera {
    match self {
      LightVariant::Directional { dir, .. } => {
        let view = Mat4::look_at_rh(*dir, Vec3::ZERO, Vec3::Y);
        // calculate orthographic projection based on camera frustum
        let proj = find_bounding_box_projection(camera, view);
        let proj = crate::Projection::Custom(proj);
        Camera::new(Vec3::ZERO, -*dir, proj, 1.0)
      }
      LightVariant::Spot { pos, fov } => {
        let proj = Mat4::perspective_rh_gl(*fov, 1.0, 0.1, 100.0);
        let proj = crate::Projection::Custom(proj);
        Camera::new(*pos, Vec3::ZERO, proj, 1.0)
      }
    }
  }
}

fn find_bounding_box_projection(camera: &Camera, view: Mat4) -> Mat4 {
  let camera_frustum_world_bbox: (Vec3, Vec3) = camera.world_bounding_box();
  let get_corner = |x: bool, y: bool, z: bool| -> Vec3 {
    Vec3::new(
      if x {
        camera_frustum_world_bbox.1.x
      } else {
        camera_frustum_world_bbox.0.x
      },
      if y {
        camera_frustum_world_bbox.1.y
      } else {
        camera_frustum_world_bbox.0.y
      },
      if z {
        camera_frustum_world_bbox.1.z
      } else {
        camera_frustum_world_bbox.0.z
      },
    )
  };
  let corners = [
    get_corner(false, false, false),
    get_corner(true, false, false),
    get_corner(false, true, false),
    get_corner(true, true, false),
    get_corner(false, false, true),
    get_corner(true, false, true),
    get_corner(false, true, true),
    get_corner(true, true, true),
  ];

  let mut min = Vec3::splat(f32::INFINITY);
  let mut max = Vec3::splat(f32::NEG_INFINITY);

  for corner in &corners {
    let corner = (view * corner.extend(1.0)).xyz();
    min = min.min(corner);
    max = max.max(corner);
  }

  Mat4::orthographic_rh_gl(-1.0, 1.0, -1.0, 1.0, -1.0, 1.0)
}

pub struct Light {
  // each component can be greater than one. pre-multiplied by intensity.
  color: Vec3,
  variant: LightVariant,
  map: DepthTexture2d,
  program: Rc<glium::Program>,
}

impl Light {
  pub fn new(facade: &impl Facade) -> Result<Self> {
    const SHADER_PATH: &str =
      concat!(env!("CARGO_MANIFEST_DIR"), "/assets/shadow");
    let program = load_program(SHADER_PATH, facade)?;
    Ok(Self {
      color: Vec3::ONE,
      variant: LightVariant::new(),
      map: create_shadow_map(facade),
      program: Rc::new(program),
    })
  }

  pub fn toggle_light_variant(&mut self) {
    self.variant.toggle();
  }

  pub fn color(&self) -> Vec3 {
    self.color
  }

  pub fn uniforms(
    &self,
    camera: &Camera,
  ) -> impl glium::uniforms::Uniforms + '_ {
    OwnedMergedUniform::new(
      self.variant.light_uniforms(self.color),
      self.variant.shadow_uniforms(camera, &self.map),
    )
  }

  pub fn light_object_transform(&self) -> Transform {
    let position = match self.variant {
      LightVariant::Directional { dir, .. } => dir * 2.0,
      LightVariant::Spot { pos, .. } => pos,
    };

    let dir = match self.variant {
      LightVariant::Directional { dir, .. } => dir.normalize(),
      LightVariant::Spot { pos, .. } => pos.normalize(),
    };

    Transform {
      translation: position,
      scale: Vec3::splat(0.1),
      rotation: Quat::from_rotation_arc_colinear(Vec3::Z, -dir),
    }
  }

  pub fn rotate(&mut self, dx: f32, dy: f32) {
    let rot = Mat3::from_rotation_z(-dx * 0.01);
    match self.variant {
      LightVariant::Directional { ref mut dir, .. } => {
        *dir = rot * *dir;
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
    let camera = self.variant.shadow_space_camera(camera);
    let framebuffer = SimpleFrameBuffer::depth_only(facade, &self.map)?;

    Ok(ShadowMapFramebuffer {
      camera: Box::new(camera),
      framebuffer: Box::new(framebuffer),
      program: &self.program,
    })
  }
}

pub struct ShadowMapFramebuffer<'a> {
  camera: Box<Camera>,
  framebuffer: Box<SimpleFrameBuffer<'a>>,
  program: &'a glium::Program,
}

impl<'a> ShadowMapFramebuffer<'a> {
  pub fn clear(&mut self) {
    self.framebuffer.clear_depth(1.0);
  }

  pub fn draw_object<O>(&mut self, object: &O) -> Result<()>
  where
    O: Draw + HasShadow,
  {
    if !object.casts_shadow() {
      return Ok(());
    }
    let params = DrawParameters {
      depth: glium::Depth {
        test: glium::draw_parameters::DepthTest::IfLess,
        write: true,
        ..Default::default()
      },
      ..Default::default()
    };

    let program = object.shadow_program().unwrap_or(self.program);

    object.draw_raw(
      self.framebuffer.as_mut(),
      self.camera.as_ref(),
      program,
      DynUniforms::new(),
      Some(params),
    )?;

    Ok(())
  }
}

fn create_shadow_map(facade: &impl Facade) -> DepthTexture2d {
  DepthTexture2d::empty_with_format(
    facade,
    glium::texture::DepthFormat::I24,
    MipmapsOption::NoMipmap,
    SHADOW_MAP_RESOLUTION,
    SHADOW_MAP_RESOLUTION,
  )
  .unwrap()
}

#[derive(Copy, Clone)]
struct Vertex {
  pos: [f32; 2],
  uv: [f32; 2],
}

implement_vertex!(Vertex, pos, uv);

pub struct ShadowMapVisual {
  program: Program,
  vbo: glium::VertexBuffer<Vertex>,
}

impl ShadowMapVisual {
  const SHADER_PATH: &'static str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/assets/preview");

  pub fn new(facade: &impl Facade) -> Result<Self> {
    let program = load_program(Self::SHADER_PATH, facade)?;
    let verts = [
      Vertex {
        pos: [-1.0, -1.0],
        uv: [-0.1, -0.1],
      },
      Vertex {
        pos: [1.0, -1.0],
        uv: [1.1, -0.1],
      },
      Vertex {
        pos: [-1.0, 1.0],
        uv: [-0.1, 1.1],
      },
      Vertex {
        pos: [1.0, 1.0],
        uv: [1.1, 1.1],
      },
    ];
    let vbo = glium::VertexBuffer::new(facade, &verts)?;
    Ok(Self { program, vbo })
  }

  pub fn reload_shader(&mut self, facade: &impl Facade) -> Result<()> {
    let program = load_program(Self::SHADER_PATH, facade)?;
    self.program = program;
    Ok(())
  }

  pub fn draw(
    &self,
    target: &mut impl glium::Surface,
    light: &Light,
  ) -> Result<()> {
    let sampler = light
      .map
      .sampled()
      .magnify_filter(glium::uniforms::MagnifySamplerFilter::Linear)
      .minify_filter(glium::uniforms::MinifySamplerFilter::Linear)
      .wrap_function(glium::uniforms::SamplerWrapFunction::BorderClamp)
      .border_color(Some([0.0, 0.0, 0.0, 1.0]));

    let uniforms = uniform! {
      shadow_map: sampler,
    };

    let draw_parameters = DrawParameters {
      depth: glium::Depth {
        test: glium::draw_parameters::DepthTest::Overwrite,
        write: false,
        ..Default::default()
      },
      ..Default::default()
    };

    target.draw(
      &self.vbo,
      glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip),
      &self.program,
      &uniforms,
      &draw_parameters,
    )?;

    Ok(())
  }
}
