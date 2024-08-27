use std::path::{Path, PathBuf};

use common::{asset_path, teapot_path, DynUniforms, MergedUniform};
use glam::{Mat3, Mat4, Vec3};
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

    object.model = Mat4::from_scale(Vec3::splat(0.05))
            * Mat4::from_rotation_y(0.0)
            // the object itself is rotated 90 to the front, let's rotate it back a little.
            * Mat4::from_rotation_x(-90.0f32.to_radians());

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

    object.model = Mat4::from_translation(Vec3::new(0.0, -0.3, 0.0))
      * Mat4::from_scale(Vec3::splat(0.0003))
      * Mat4::from_rotation_y(180.0f32.to_radians())
      * Mat4::from_rotation_x(-90.0f32.to_radians());

    Ok(object)
  }
}

pub struct Plane;

impl Plane {
  pub fn create(facade: &impl Facade) -> Result<GPUObject> {
    let plane = genmesh::generators::Plane::new();
    let mesh = Mesh::from_genmesh(plane).upload(facade)?;
    let model = Mat4::from_rotation_x(-90.0f32.to_radians());
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
  model: Mat4,
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
  #[allow(unused)]
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
    let mut vertices = [
      Vec3::new(x1, y1, z1),
      Vec3::new(x1, y1, z2),
      Vec3::new(x1, y2, z1),
      Vec3::new(x1, y2, z2),
      Vec3::new(x2, y1, z1),
      Vec3::new(x2, y1, z2),
      Vec3::new(x2, y2, z1),
      Vec3::new(x2, y2, z2),
    ];

    for vert in &mut vertices {
      *vert = self.model.transform_point3(*vert);
    }

    let (mut xmin, mut ymin, mut zmin) = (f32::MAX, f32::MAX, f32::MAX);
    let (mut xmax, mut ymax, mut zmax) = (f32::MIN, f32::MIN, f32::MIN);

    for v in vertices {
      xmin = xmin.min(v.x);
      ymin = ymin.min(v.y);
      zmin = zmin.min(v.z);
      xmax = xmax.max(v.x);
      ymax = ymax.max(v.y);
      zmax = zmax.max(v.z);
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
    let model = Mat4::IDENTITY;

    Ok(Self {
      shader_path,
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
    extra_uniforms: impl Uniforms,
  ) {
    let mv: Mat4 = camera.view() * self.model;
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
    let uniforms = MergedUniform::new(&extra_uniforms, &uniforms);

    let scissor = camera.scissor();
    let culling = if camera.mirror() {
      glium::draw_parameters::BackfaceCullingMode::CullCounterClockwise
    } else {
      glium::draw_parameters::BackfaceCullingMode::CullClockwise
    };
    let draw_params = DrawParameters {
      depth: glium::Depth {
        test: glium::draw_parameters::DepthTest::IfLess,
        write: true,
        ..Default::default()
      },
      backface_culling: culling,
      scissor,
      ..Default::default()
    };

    self.mesh.draw(frame, program, &uniforms, &draw_params);
  }

  pub fn world_pos(&self) -> Vec3 {
    self.model.w_axis.truncate()
  }

  pub fn translated(self, d: [f32; 3]) -> Self {
    let model = Mat4::from_translation(Vec3::from(d)) * self.model;
    Self { model, ..self }
  }

  pub fn rotated_y(self, degree: f32) -> Self {
    let model = Mat4::from_rotation_y(degree.to_radians()) * self.model;
    Self { model, ..self }
  }

  pub fn scaled(self, sx: f32, sy: f32, sz: f32) -> Self {
    let model = Mat4::from_scale(Vec3::new(sx, sy, sz)) * self.model;
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
