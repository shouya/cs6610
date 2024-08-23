use std::time::Duration;

use common::{
  to_raw_image, CameraLike, Draw, DynUniforms, HasShadow, MergedUniform,
  OwnedMergedUniform,
};
use glam::{Mat3, Mat4};
use glium::{
  backend::Facade,
  implement_vertex,
  index::{NoIndices, PrimitiveType},
  program::SourceCode,
  uniform,
  uniforms::{UniformValue, Uniforms},
  Depth, DepthTest, DrawParameters, Program, Surface, Texture2d, VertexBuffer,
};
use image::RgbImage;

use crate::{Camera, Light, Result, Transform};

#[derive(Copy, Clone)]
struct Vertex {
  pos: [f32; 3],
  uv: [f32; 2],
}

implement_vertex!(Vertex, pos, uv);

const LOCAL_ASSETS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets");

pub struct TeapotQuad {
  vbo: VertexBuffer<Vertex>,
  normal_map: Texture2d,
  model: Transform,
  displacement_map: Texture2d,
  program: Program,
  shadow_program: Program,
  wireframe_program: Program,
  tess_level_outer: usize,
  tess_level_inner: usize,
}

impl TeapotQuad {
  pub fn new(facade: &impl Facade) -> Result<Self> {
    let verts = [
      Vertex {
        pos: [-1.0, -1.0, 0.0],
        uv: [0.0, 1.0],
      },
      Vertex {
        pos: [1.0, -1.0, 0.0],
        uv: [1.0, 1.0],
      },
      Vertex {
        pos: [-1.0, 1.0, 0.0],
        uv: [0.0, 0.0],
      },
      Vertex {
        pos: [1.0, 1.0, 0.0],
        uv: [1.0, 0.0],
      },
    ];
    let vbo = VertexBuffer::new(facade, &verts)?;

    let program = Self::load_program(facade)?;
    let shadow_program = Self::load_shadow_program(facade)?;
    let wireframe_program = Self::load_wireframe_program(facade)?;

    let normal_map =
      load_texture(facade, format!("{LOCAL_ASSETS}/teapot_normal.png"))?;
    let displacement_map =
      load_texture(facade, format!("{LOCAL_ASSETS}/teapot_disp.png"))?;

    Ok(Self {
      model: Transform::default(),
      vbo,
      program,
      shadow_program,
      wireframe_program,
      normal_map,
      displacement_map,
      tess_level_outer: 18,
      tess_level_inner: 18,
    })
  }

  pub fn draw(
    &self,
    target: &mut impl Surface,
    camera: &Camera,
    light: &Light,
  ) -> Result<()> {
    self.draw_raw(
      target,
      camera,
      &self.program,
      light.uniforms(camera),
      None,
    )?;

    Ok(())
  }

  pub fn draw_wireframe(
    &self,
    target: &mut glium::Frame,
    camera: &Camera,
    light: &Light,
  ) -> Result<()> {
    let mut params = default_draw_params();
    params.depth = Depth {
      test: DepthTest::Overwrite,
      write: false,
      ..Default::default()
    };
    params.backface_culling =
      glium::draw_parameters::BackfaceCullingMode::CullingDisabled;

    self.draw_raw(
      target,
      camera,
      &self.wireframe_program,
      light.uniforms(camera),
      Some(params),
    )?;

    Ok(())
  }

  fn uniforms<'a>(
    &'a self,
    camera: &'a impl CameraLike,
    program: &Program,
  ) -> impl Uniforms + 'a {
    fn mat4_uniform(mat: &Mat4) -> UniformValue<'static> {
      glium::uniforms::UniformValue::Mat4(mat.to_cols_array_2d())
    }
    fn mat3_uniform(mat3: &Mat3) -> UniformValue<'static> {
      glium::uniforms::UniformValue::Mat3(mat3.to_cols_array_2d())
    }

    let mut camera_uniforms = DynUniforms::new();
    let model = self.model.to_mat4();
    let view = Mat4::from_cols_array_2d(&camera.view());
    let proj = Mat4::from_cols_array_2d(&camera.projection());

    camera_uniforms.add_raw("model", mat4_uniform(&model));
    camera_uniforms.add_raw("view", mat4_uniform(&view));
    camera_uniforms.add_raw("projection", mat4_uniform(&proj));

    if program.get_uniform("view_projection").is_some() {
      let view_proj = proj * view;
      camera_uniforms.add_raw("view_projection", mat4_uniform(&view_proj));
    }

    if program.get_uniform("model_view_projection").is_some() {
      let model_view_proj = proj * view * model;
      camera_uniforms
        .add_raw("model_view_projection", mat4_uniform(&model_view_proj));
    }

    if program.get_uniform("model_view_normal").is_some() {
      let model_view = view * model;
      let model_view_normal = Mat3::from_mat4(model_view).inverse().transpose();
      camera_uniforms
        .add_raw("model_view_normal", mat3_uniform(&model_view_normal));
    }

    let normal_map = self
      .normal_map
      .sampled()
      .magnify_filter(glium::uniforms::MagnifySamplerFilter::Linear)
      .minify_filter(glium::uniforms::MinifySamplerFilter::LinearMipmapLinear);
    let displacement_map = self
      .displacement_map
      .sampled()
      .magnify_filter(glium::uniforms::MagnifySamplerFilter::Linear)
      .minify_filter(glium::uniforms::MinifySamplerFilter::LinearMipmapLinear);

    let extra_uniforms = uniform! {
      tess_level_inner: self.tess_level_inner as f32,
      tess_level_outer: self.tess_level_outer as f32,
      normal_map: normal_map,
      displacement_map: displacement_map,
    };

    OwnedMergedUniform::new(camera_uniforms, extra_uniforms)
  }

  pub fn load_program(facade: &impl Facade) -> Result<Program> {
    use std::fs::read_to_string;
    let vert = read_to_string(format!("{LOCAL_ASSETS}/tess_obj.vert"))?;
    let tcs = read_to_string(format!("{LOCAL_ASSETS}/tess_obj.tcs"))?;
    let tes = read_to_string(format!("{LOCAL_ASSETS}/tess_obj.tes"))?;
    let frag = read_to_string(format!("{LOCAL_ASSETS}/tess_obj.frag"))?;

    let program = SourceCode {
      vertex_shader: &vert,
      tessellation_control_shader: Some(&tcs),
      tessellation_evaluation_shader: Some(&tes),
      fragment_shader: &frag,
      geometry_shader: None,
    };

    let program = Program::new(facade, program)?;
    Ok(program)
  }

  fn load_wireframe_program(facade: &impl Facade) -> Result<Program> {
    use std::fs::read_to_string;
    let vert = read_to_string(format!("{LOCAL_ASSETS}/tess_obj.vert"))?;
    let tcs = read_to_string(format!("{LOCAL_ASSETS}/tess_obj.tcs"))?;
    let tes = read_to_string(format!("{LOCAL_ASSETS}/tess_obj.tes"))?;
    let geom = read_to_string(format!("{LOCAL_ASSETS}/tess_obj.geom"))?;
    let frag = read_to_string(format!("{LOCAL_ASSETS}/tess_obj_wf.frag"))?;

    let program = SourceCode {
      vertex_shader: &vert,
      tessellation_control_shader: Some(&tcs),
      tessellation_evaluation_shader: Some(&tes),
      fragment_shader: &frag,
      geometry_shader: Some(&geom),
    };

    Ok(Program::new(facade, program)?)
  }

  fn load_shadow_program(facade: &impl Facade) -> Result<Program> {
    use std::fs::read_to_string;
    let vert = read_to_string(format!("{LOCAL_ASSETS}/tess_obj.vert"))?;
    let tcs = read_to_string(format!("{LOCAL_ASSETS}/tess_obj.tcs"))?;
    let tes = read_to_string(format!("{LOCAL_ASSETS}/tess_obj.tes"))?;
    let frag = r#"
      #version 330 core
      void main() {}
    "#;

    let program = SourceCode {
      vertex_shader: &vert,
      tessellation_control_shader: Some(&tcs),
      tessellation_evaluation_shader: Some(&tes),
      fragment_shader: frag,
      geometry_shader: None,
    };

    Ok(Program::new(facade, program)?)
  }

  pub fn update(&self, _dt: &Duration) {}

  pub fn reload_shader(&mut self, facade: &impl Facade) -> Result<()> {
    self.program = Self::load_program(facade)?;
    self.wireframe_program = Self::load_wireframe_program(facade)?;
    self.shadow_program = Self::load_shadow_program(facade)?;
    Ok(())
  }

  pub fn update_tess_level(&mut self, delta: isize) {
    self.tess_level_inner =
      (self.tess_level_inner as isize + delta).max(1) as usize;
    self.tess_level_outer =
      (self.tess_level_outer as isize + delta).max(1) as usize;
  }
}

impl common::Draw for TeapotQuad {
  fn draw_raw(
    &self,
    frame: &mut impl glium::Surface,
    camera: &impl common::CameraLike,
    program: &glium::Program,
    uniforms: impl glium::uniforms::Uniforms,
    draw_params: Option<DrawParameters>,
  ) -> Result<()> {
    let own_uniforms = self.uniforms(camera, program);
    let uniforms = MergedUniform::new(&uniforms, &own_uniforms);
    let params = draw_params.unwrap_or(default_draw_params());

    frame.draw(
      &self.vbo,
      NoIndices(PrimitiveType::Patches {
        vertices_per_patch: 4,
      }),
      program,
      &uniforms,
      &params,
    )?;

    Ok(())
  }
}

impl HasShadow for TeapotQuad {
  fn shadow_program(&self) -> Option<&Program> {
    Some(&self.shadow_program)
  }
}

fn load_texture(
  facade: &impl Facade,
  path: impl AsRef<std::path::Path>,
) -> Result<Texture2d> {
  let image = image::open(path)?.to_rgb8();
  Ok(upload_texture(facade, &image))
}

fn upload_texture(facade: &impl Facade, image: &RgbImage) -> Texture2d {
  let texture = Texture2d::new(facade, to_raw_image(image)).unwrap();
  unsafe { texture.generate_mipmaps() };
  texture
}

fn default_draw_params() -> DrawParameters<'static> {
  DrawParameters {
    depth: Depth {
      test: DepthTest::IfLess,
      write: true,
      ..Default::default()
    },
    // backface_culling: BackfaceCullingMode::CullClockwise,
    ..Default::default()
  }
}
