use std::path::{Path, PathBuf};

use cgmath::{
  Matrix as _, Matrix3, Matrix4, Point3, SquareMatrix as _, Transform,
};
use common::{asset_path, teapot_path, DynUniforms, MergedUniform};
use glium::backend::Facade;
use glium::uniforms::Uniforms;
use glium::{uniform, DrawParameters, Program, Surface};

use crate::light::Light;
use crate::mesh::{GPUMesh, Mesh};
use crate::{Camera, Result};

#[allow(unused)]
pub struct Teapot;

impl Teapot {
  pub fn load(facade: &impl Facade) -> Result<GPUObject> {
    const SHADER_PATH: &str =
      concat!(env!("CARGO_MANIFEST_DIR"), "/assets/shader");
    let mut object = GPUObject::load(&teapot_path(), &SHADER_PATH, facade)?;

    object.model = Matrix4::from_scale(0.05)
      * Matrix4::from_angle_y(cgmath::Rad(0.0))
    // the object itself is rotated 90 to the front, let's rotate it back a little.
      * Matrix4::from_angle_x(cgmath::Deg(-90.0));

    Ok(object)
  }
}

#[allow(unused)]
pub struct Yoda;

impl Yoda {
  #[allow(unused)]
  pub fn load(facade: &impl Facade) -> Result<GPUObject> {
    const SHADER_PATH: &str =
      concat!(env!("CARGO_MANIFEST_DIR"), "/assets/shader");
    let yoda_model = asset_path("yoda/yoda.obj");
    let mut object = GPUObject::load(&yoda_model, &SHADER_PATH, facade)?;

    object.model = Matrix4::from_translation([0.0, -0.3, 0.0].into())
      * Matrix4::from_scale(0.0003)
      * Matrix4::from_angle_y(cgmath::Deg(180.0))
      * Matrix4::from_angle_x(cgmath::Deg(-90.0));

    Ok(object)
  }
}

pub struct Plane;

impl Plane {
  pub fn new(facade: &impl Facade) -> Result<GPUObject> {
    let plane = genmesh::generators::Plane::new();
    let mesh = Mesh::from_genmesh(plane).upload(facade)?;
    let model = Matrix4::from_angle_x(cgmath::Deg(-90.0));
    let object = GPUObject {
      shader_path: None,
      program: None,
      mesh,
      model,
    };
    Ok(object)
  }
}

pub struct GPUObject {
  shader_path: Option<PathBuf>,
  program: Option<Program>,
  mesh: GPUMesh,
  model: Matrix4<f32>,
}

impl GPUObject {
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
  pub fn dimensions(&self) -> [f32; 3] {
    let [[xmin, xmax], [ymin, ymax], [zmin, zmax]] = self.bounding_box();
    let dx = xmax - xmin;
    let dy = ymax - ymin;
    let dz = zmax - zmin;
    [dx, dy, dz]
  }

  // world space
  pub fn bounding_box(&self) -> [[f32; 2]; 3] {
    let [[x1, x2], [y1, y2], [z1, z2]] = self.mesh.bounding_box();
    let vertices = [
      [x1, y1, z1],
      [x1, y1, z2],
      [x1, y2, z1],
      [x1, y2, z2],
      [x2, y1, z1],
      [x2, y1, z2],
      [x2, y2, z1],
      [x2, y2, z2],
    ];

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
    let model = Matrix4::identity();

    Ok(Self {
      shader_path,
      program,
      mesh,
      model,
    })
  }

  pub fn update(&mut self, _dt: &std::time::Duration) {}

  pub fn model(&self) -> Matrix4<f32> {
    self.model
  }

  pub fn draw(&self, frame: &mut impl Surface, camera: &Camera, light: &Light) {
    if let Some(program) = &self.program {
      self.draw_raw(frame, camera, light, program, DynUniforms::new());
    } else {
      eprintln!("GPUObject::draw: program is not loaded");
    }
  }

  pub fn draw_raw(
    &self,
    frame: &mut impl Surface,
    camera: &Camera,
    light: &Light,
    program: &Program,
    extra_uniforms: impl Uniforms,
  ) {
    let mv: Matrix4<f32> = camera.view() * self.model();
    let mv3: Matrix3<f32> = common::math::mat4_to_3(mv);
    let mv_n: Matrix3<f32> = mv3.invert().unwrap().transpose();
    let mvp: Matrix4<f32> = camera.projection() * mv;

    // in view space
    let light_pos: [f32; 3] = camera
      .view()
      .transform_point(light.position_world().into())
      .into();

    let uniforms = uniform! {
      mvp: Into::<[[f32; 4]; 4]>::into(mvp),
      mv: Into::<[[f32; 4]; 4]>::into(mv),
      mv_n: Into::<[[f32; 3]; 3]>::into(mv_n),
      light_pos: light_pos,
      light_color: light.color(),
    };

    let draw_params = DrawParameters {
      depth: glium::Depth {
        test: glium::draw_parameters::DepthTest::IfLess,
        write: true,
        ..Default::default()
      },
      backface_culling:
        glium::draw_parameters::BackfaceCullingMode::CullClockwise,
      ..Default::default()
    };
    let uniforms = MergedUniform::new(&extra_uniforms, &uniforms);

    self.mesh.draw(frame, program, &uniforms, &draw_params);
  }

  pub fn world_pos(&self) -> Point3<f32> {
    let d = self.model.w.truncate() / self.model.w.w;
    Point3::new(d.x, d.y, d.z)
  }

  pub fn translated(self, d: [f32; 3]) -> Self {
    let model = Matrix4::from_translation(d.into()) * self.model;
    Self { model, ..self }
  }

  pub fn rotated_y(self, degree: f32) -> Self {
    let model = Matrix4::from_angle_y(cgmath::Deg(degree)) * self.model;
    Self { model, ..self }
  }

  pub fn scaled(self, sx: f32, sy: f32, sz: f32) -> Self {
    let model = Matrix4::from_nonuniform_scale(sx, sy, sz) * self.model;
    Self { model, ..self }
  }
}

fn load_program(path: &Path, facade: &impl Facade) -> Result<Program> {
  let vert_path = path.with_extension("vert");
  let frag_path = path.with_extension("frag");

  let vert_src = std::fs::read_to_string(vert_path)?;
  let frag_src = std::fs::read_to_string(frag_path)?;

  Ok(Program::from_source(facade, &vert_src, &frag_src, None)?)
}
