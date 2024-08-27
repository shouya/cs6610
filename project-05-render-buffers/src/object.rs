use std::path::{Path, PathBuf};
use std::rc::Rc;

use common::{asset_path, teapot_path, DynUniforms, MergedUniform};
use glam::{Mat3, Mat4, Vec3};
use glium::backend::{Context, Facade};
use glium::framebuffer::{DepthRenderBuffer, SimpleFrameBuffer};
use glium::uniforms::Uniforms;
use glium::{uniform, DrawParameters, Program, Surface, Texture2d};

use crate::light::Light;
use crate::mesh::{GPUMesh, Mesh};
use crate::{Camera, Result};

pub struct RenderBuffer {
  texture: Texture2d,
  depth: DepthRenderBuffer,
}

impl RenderBuffer {
  fn new(facade: &impl Facade, width: u32, height: u32) -> Result<Self> {
    let texture = Texture2d::empty(facade, width, height)?;
    let depth = DepthRenderBuffer::new(
      facade,
      glium::texture::DepthFormat::F32,
      width,
      height,
    )?;

    Ok(Self { texture, depth })
  }

  fn framebuffer(&self, facade: &impl Facade) -> Result<SimpleFrameBuffer<'_>> {
    let fb =
      SimpleFrameBuffer::with_depth_buffer(facade, &self.texture, &self.depth)?;
    Ok(fb)
  }
}

pub struct IndirectScene {
  pub camera: Camera,
  pub light: Light,
  pub objects: Vec<GPUObject>,
  pub buffer: RenderBuffer,
  context: Rc<Context>,
}

impl IndirectScene {
  fn new(
    facade: &impl Facade,
    camera: Camera,
    light: Light,
    buffer: RenderBuffer,
  ) -> Self {
    Self {
      camera,
      light,
      buffer,
      objects: Vec::new(),
      context: facade.get_context().clone(),
    }
  }

  pub fn add_object(&mut self, object: GPUObject) {
    self.objects.push(object);
  }

  pub fn render(&self) -> Result<&Texture2d> {
    let mut fb = self.buffer.framebuffer(&self.context)?;

    fb.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);
    for obj in &self.objects {
      obj.draw(&mut fb, &self.camera, &self.light);
    }

    Ok(&self.buffer.texture)
  }
}

pub struct Teapot;

impl Teapot {
  pub fn load(facade: &impl Facade) -> Result<GPUObject> {
    const SHADER_PATH: &str =
      concat!(env!("CARGO_MANIFEST_DIR"), "/assets/shader");
    let mut object = GPUObject::load(&teapot_path(), &SHADER_PATH, facade)?;

    object.model = Mat4::from_scale(Vec3::splat(0.05))
      * Mat4::from_rotation_y(0.0)
      * Mat4::from_rotation_x((-90.0f32).to_radians());

    Ok(object)
  }

  #[allow(dead_code)]
  pub fn load_indirect_scene(facade: &impl Facade) -> Result<IndirectScene> {
    let camera = Camera::new();
    let light = Light::new();
    let object = Self::load(facade)?;
    let mut scene = IndirectScene::new(
      facade,
      camera,
      light,
      RenderBuffer::new(facade, 1024, 1024)?,
    );
    scene.add_object(object);

    Ok(scene)
  }
}

pub struct Yoda;

impl Yoda {
  pub fn load(facade: &impl Facade) -> Result<GPUObject> {
    const SHADER_PATH: &str =
      concat!(env!("CARGO_MANIFEST_DIR"), "/assets/shader");
    let yoda_model = asset_path("yoda/yoda.obj");
    let mut object = GPUObject::load(&yoda_model, &SHADER_PATH, facade)?;

    object.model = Mat4::from_translation(Vec3::new(0.0, -0.3, 0.0))
      * Mat4::from_scale(Vec3::splat(0.0003))
      * Mat4::from_rotation_y(180.0f32.to_radians())
      * Mat4::from_rotation_x((-90.0f32).to_radians());

    Ok(object)
  }

  pub fn load_indirect_scene(facade: &impl Facade) -> Result<IndirectScene> {
    let camera = Camera::new();
    let light = Light::new();
    let object = Self::load(facade)?;
    let mut scene = IndirectScene::new(
      facade,
      camera,
      light,
      RenderBuffer::new(facade, 1024, 1024)?,
    );
    scene.add_object(object);

    Ok(scene)
  }
}

pub struct GPUObject {
  shader_path: PathBuf,
  program: Program,
  mesh: GPUMesh,
  model: Mat4,
}

impl GPUObject {
  pub fn reload_shader(&mut self, facade: &impl Facade) -> Result<()> {
    let shader_path = &self.shader_path;

    match load_program(shader_path, facade) {
      Ok(program) => self.program = program,
      Err(e) => {
        eprintln!("Failed to reload shader: {}", e);
      }
    }

    Ok(())
  }

  pub fn load(
    obj_path: &impl AsRef<Path>,
    shader_path: &impl AsRef<Path>,
    facade: &impl Facade,
  ) -> Result<Self> {
    let mesh = Mesh::load(obj_path)?;
    let mesh = mesh.upload(facade)?;
    let program = load_program(shader_path.as_ref(), facade)?;
    let model = Mat4::IDENTITY;

    Ok(Self {
      shader_path: shader_path.as_ref().to_path_buf(),
      program,
      mesh,
      model,
    })
  }

  pub fn update(&mut self, _dt: &std::time::Duration) {}

  pub fn model(&self) -> Mat4 {
    self.model
  }

  pub fn draw(&self, frame: &mut impl Surface, camera: &Camera, light: &Light) {
    self.draw_with_extra_uniforms(frame, camera, light, DynUniforms::new());
  }

  pub fn draw_with_extra_uniforms(
    &self,
    frame: &mut impl Surface,
    camera: &Camera,
    light: &Light,
    extra_uniforms: impl Uniforms,
  ) {
    let mv: Mat4 = camera.view() * self.model();
    let mv3: Mat3 = Mat3::from_mat4(mv);
    let mv_n: Mat3 = mv3.inverse().transpose();
    let mvp: Mat4 = camera.projection() * mv;

    // in view space
    let light_pos: [f32; 3] = (camera.view()
      * light.position_world().extend(1.0))
    .truncate()
    .into();

    let uniforms = uniform! {
        mvp: mvp.to_cols_array_2d(),
        mv: mv.to_cols_array_2d(),
        mv_n: mv_n.to_cols_array_2d(),
        light_pos: light_pos,
        light_color: light.color(),
    };

    let draw_params = DrawParameters {
      depth: glium::Depth {
        test: glium::draw_parameters::DepthTest::IfLess,
        write: true,
        ..Default::default()
      },
      ..Default::default()
    };
    let program = &self.program;
    let uniforms = MergedUniform::new(&extra_uniforms, &uniforms);

    self.mesh.draw(frame, program, &uniforms, &draw_params);
  }
}

fn load_program(path: &Path, facade: &impl Facade) -> Result<Program> {
  let vert_path = path.with_extension("vert");
  let frag_path = path.with_extension("frag");

  let vert_src = std::fs::read_to_string(vert_path)?;
  let frag_src = std::fs::read_to_string(frag_path)?;

  Ok(Program::from_source(facade, &vert_src, &frag_src, None)?)
}
