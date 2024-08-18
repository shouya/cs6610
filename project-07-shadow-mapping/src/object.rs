use std::path::{Path, PathBuf};

use common::{asset_path, load_program, teapot_path, MergedUniform};
use glam::{Mat3, Mat4, Quat, Vec3};
use glium::backend::Facade;
use glium::uniforms::Uniforms;
use glium::{uniform, DrawParameters, Program, Surface};

use crate::light::Light;
use crate::mesh::{GPUMesh, Mesh};
use crate::transform::Transform;
use crate::{Camera, Result};

const SHADER_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/object");

#[allow(unused)]
pub struct Teapot;

impl Teapot {
  pub fn load(facade: &impl Facade) -> Result<Object> {
    let mut object = Object::load(&teapot_path(), &SHADER_PATH, facade)?;

    object.transform = Transform {
      scale: Vec3::splat(0.05),
      // the object itself is rotated 90 degrees to the front, let's
      // rotate it back a little.
      rotation: Quat::from_rotation_x(-90f32.to_radians()),
      ..Transform::default()
    };

    Ok(object)
  }
}

#[allow(unused)]
pub struct Plane;

impl Plane {
  pub fn load(facade: &impl Facade) -> Result<Object> {
    let model_path = asset_path("plane.obj");
    let object = Object::load(&model_path, &SHADER_PATH, facade)?;

    Ok(object)
  }
}

#[allow(unused)]
pub struct LightObject;

impl LightObject {
  pub fn load(facade: &impl Facade) -> Result<Object> {
    let model_path = asset_path("light.obj");
    let mut object = Object::load(&model_path, &SHADER_PATH, facade)?;

    object.transform = Transform {
      scale: Vec3::splat(0.1),
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
      Ok(program) => {
        self.program = Some(program);
        println!("Shader reloaded: {:?}", shader_path);
      }
      Err(e) => {
        eprintln!("Failed to reload shader: {}", e);
      }
    }

    Ok(())
  }

  pub fn set_transform(&mut self, transform: Transform) {
    self.transform = transform;
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
    self.transform.to_mat4()
  }

  pub fn draw(&self, frame: &mut impl Surface, camera: &Camera, light: &Light) {
    if let Some(program) = &self.program {
      let light_uniforms = light.uniforms(camera);
      self.draw_with_program(frame, camera, program, light_uniforms);
    } else {
      eprintln!("GPUObject::draw: program is not loaded");
    }
  }

  pub fn draw_with_program(
    &self,
    frame: &mut impl Surface,
    camera: &Camera,
    program: &Program,
    uniforms: impl Uniforms,
  ) {
    let m: Mat4 = self.model();
    let v: Mat4 = camera.view();
    let mv: Mat4 = v * m;
    let mv3: Mat3 = Mat3::from_mat4(mv);
    let mv_n: Mat3 = mv3.inverse().transpose();
    let mvp: Mat4 = camera.projection() * mv;

    let model_uniforms = uniform! {
      v: v.to_cols_array_2d(),
      m: m.to_cols_array_2d(),
      mvp: mvp.to_cols_array_2d(),
      mv: mv.to_cols_array_2d(),
      mv3: mv3.to_cols_array_2d(),
      mv_n: mv_n.to_cols_array_2d(),
    };
    let uniforms = MergedUniform::new(&uniforms, &model_uniforms);

    let draw_params = DrawParameters {
      depth: glium::Depth {
        test: glium::draw_parameters::DepthTest::IfLess,
        write: true,
        ..Default::default()
      },
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

  pub fn rotated_y(mut self, angle: f32) -> Self {
    self.transform.rotation = Quat::from_rotation_y(angle);
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
