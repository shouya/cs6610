use std::path::{Path, PathBuf};

use common::{teapot_path, DynUniforms, MergedUniform};
use glam::{Mat3, Mat4, Vec3};
use glium::backend::Facade;
use glium::uniforms::Uniforms;
use glium::{uniform, DrawParameters, Program, Surface};

use crate::light::Light;
use crate::mesh::{GPUMesh, Mesh};
use crate::transform::Transform;
use crate::{Camera, Result};

#[allow(unused)]
pub struct Teapot;

impl Teapot {
  pub fn load(facade: &impl Facade) -> Result<Object> {
    const SHADER_PATH: &str =
      concat!(env!("CARGO_MANIFEST_DIR"), "/assets/object");
    let mut object = Object::load(&teapot_path(), &SHADER_PATH, facade)?;

    object.transform = Transform {
      scale: Vec3::splat(0.05),
      // the object itself is rotated 90 to the front, let's rotate it back a little.
      rotation: Vec3::new(-90.0, 0.0, 0.0),
      ..Transform::default()
    };

    Ok(object)
  }
}

pub struct Object {
  shader_path: Option<PathBuf>,
  program: Option<Program>,
  mesh: GPUMesh,
  transform: Transform,
  cast_shadow: bool,
  receive_shadow: bool,
}

impl Object {
  pub fn reload_shader(&mut self, facade: &impl Facade) -> Result<()> {
    let Some(shader_path) = &self.shader_path else {
      return Ok(());
    };

    match load_program(shader_path, facade) {
      Ok(program) => self.program = Some(program),
      Err(e) => {
        eprintln!("Failed to reload shader: {}", e);
      }
    }

    Ok(())
  }

  // world space
  pub fn bounding_box(&self) -> [[f32; 2]; 3] {
    let [[x1, x2], [y1, y2], [z1, z2]] = self.mesh.bounding_box();
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

    let mat = self.transform.to_mat4();
    for vert in &mut vertices {
      *vert = mat.transform_point3((*vert).into()).into();
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

  pub fn load(
    obj_path: &impl AsRef<Path>,
    shader_path: &impl AsRef<Path>,
    facade: &impl Facade,
  ) -> Result<Self> {
    let mesh = Mesh::load(obj_path)?;
    let mesh = mesh.upload(facade)?;
    let shader_path = shader_path.as_ref().to_path_buf();
    let program = Some(load_program(&shader_path, facade)?);
    let shader_path = Some(shader_path);

    Ok(Self {
      shader_path,
      program,
      mesh,
      transform: Transform::default(),
      cast_shadow: true,
      receive_shadow: true,
    })
  }

  pub fn update(&mut self, _dt: &std::time::Duration) {}

  pub fn model(&self) -> Mat4 {
    dbg!(self.transform.to_mat4())
  }

  pub fn draw(&self, frame: &mut impl Surface, camera: &Camera, light: &Light) {
    if let Some(program) = &self.program {
      self.draw_with_program(frame, camera, light, program, DynUniforms::new());
    } else {
      eprintln!("GPUObject::draw: program is not loaded");
    }
  }

  pub fn draw_with_program(
    &self,
    frame: &mut impl Surface,
    camera: &Camera,
    light: &Light,
    program: &Program,
    uniforms: impl Uniforms,
  ) {
    let mv: Mat4 = camera.view() * self.model();
    let mv3: Mat3 = Mat3::from_mat4(mv);
    let mv_n: Mat3 = mv3.inverse().transpose();
    let mvp: Mat4 = camera.projection() * mv;

    // in view space
    let light_dir: [f32; 3] = light.light_dir(self.world_pos()).into();

    let model_uniforms = uniform! {
      mvp: mvp.to_cols_array_2d(),
      mv: mv.to_cols_array_2d(),
      mv_n: mv_n.to_cols_array_2d(),
      light_dir: light_dir,
      light_color: light.color().to_array(),
    };
    let uniforms = MergedUniform::new(&uniforms, &model_uniforms);

    let culling = glium::draw_parameters::BackfaceCullingMode::CullClockwise;
    let draw_params = DrawParameters {
      depth: glium::Depth {
        test: glium::draw_parameters::DepthTest::IfLess,
        write: true,
        ..Default::default()
      },
      backface_culling: culling,
      ..Default::default()
    };

    self.mesh.draw(frame, program, &uniforms, &draw_params);
  }

  pub fn world_pos(&self) -> Vec3 {
    self.transform.translation
  }

  pub fn translated(mut self, d: [f32; 3]) -> Self {
    self.transform.translation = Vec3::from(d);
    self
  }

  pub fn rotated_y(mut self, degree: f32) -> Self {
    self.transform.rotation = Vec3::new(0.0, degree, 0.0);
    self
  }

  pub fn scaled(mut self, sx: f32, sy: f32, sz: f32) -> Self {
    self.transform.scale = Vec3::new(sx, sy, sz);
    self
  }

  pub fn cast_no_shadow(mut self) -> Self {
    self.cast_shadow = false;
    self
  }

  pub fn receive_no_shadow(mut self) -> Self {
    self.receive_shadow = false;
    self
  }
}

fn load_program(path: &Path, facade: &impl Facade) -> Result<Program> {
  let vert_path = path.with_extension("vert");
  let frag_path = path.with_extension("frag");

  let vert_src = std::fs::read_to_string(vert_path)?;
  let frag_src = std::fs::read_to_string(frag_path)?;

  Ok(Program::from_source(facade, &vert_src, &frag_src, None)?)
}
