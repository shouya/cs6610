use common::SimpleObj;
use glam::{EulerRot, Mat3, Mat4, Vec3};
use glium::{backend::Facade, uniform, DrawParameters};

use crate::{
  mesh::{GPUMeshFormat as _, MeshFormat, TriangleIndex, TriangleIndexGPU},
  Camera, Result,
};

pub struct Light {
  // note the light's color can exceed 1.0
  color: [f32; 3],

  // in world space
  distance: f32,

  // rotation around the y axis
  rotation: f32,

  // used for rendering the light
  gpu: Option<GPULight>,
}

struct GPULight {
  mesh: TriangleIndexGPU,
}

impl Light {
  pub fn new() -> Self {
    Self {
      color: [1.0, 1.0, 1.0],
      distance: 1.0,
      rotation: 0.0,
      gpu: None,
    }
  }
  pub fn position_world(&self) -> Vec3 {
    let location = Vec3::new(self.distance, self.distance, 0.0);
    let transform =
      Mat4::from_euler(EulerRot::XYZ, 0.0, self.rotation.to_radians(), 0.0);
    transform.transform_point3(location)
  }

  pub fn add_rotation(&mut self, delta: f32) {
    self.rotation += delta;
  }

  fn model(&self) -> Mat4 {
    Mat4::from_translation(self.position_world())
      * Mat4::from_scale(Vec3::splat(0.05))
  }

  pub fn color(&self) -> [f32; 3] {
    self.color
  }

  pub fn draw(&self, frame: &mut glium::Frame, camera: &Camera) -> Result<()> {
    let Some(gpu) = self.gpu.as_ref() else {
      return Ok(());
    };

    let mv: Mat4 = camera.view() * self.model();
    let mv3: Mat3 = Mat3::from_mat4(mv);
    let mv_n: Mat3 = mv3.inverse().transpose();
    let mvp: Mat4 = camera.projection() * mv;

    // assume the light is lit up by something on the origin
    let light_pos: Vec3 = camera.view().transform_point3(Vec3::ZERO);

    let uniforms = uniform! {
      mvp: mvp.to_cols_array_2d(),
      mv: mv.to_cols_array_2d(),
      mv_n: mv_n.to_cols_array_2d(),
      mode: 9, // plain color
      k_a: [0.1, 0.1, 0.1f32],
      k_d: self.color,
      k_s: [1.0, 1.0, 1.0f32],
      light_pos: light_pos.to_array(),
      light_color: [1.0f32; 3],
      shininess: 10.0f32,
    };

    let params = DrawParameters {
      depth: glium::Depth {
        test: glium::draw_parameters::DepthTest::IfLess,
        write: true,
        ..Default::default()
      },
      ..Default::default()
    };

    gpu.mesh.draw(frame, &uniforms, &params);

    Ok(())
  }

  pub fn upload(&mut self, surface: &impl Facade) -> Result<()> {
    let obj = SimpleObj::load_from(&common::sphere_path())?;
    let mesh = TriangleIndex::from_simple_obj(obj).upload(surface);
    let gpu = GPULight { mesh };
    self.gpu = Some(gpu);
    Ok(())
  }
}
