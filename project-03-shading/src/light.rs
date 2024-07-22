use cgmath::{Matrix as _, Matrix3, Matrix4, Point3, Rad, Transform};
use common::SimpleObj;
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
  pub fn position_world(&self) -> [f32; 3] {
    let location = Point3::new(self.distance, self.distance, 0.0);
    Matrix4::from_angle_y(Rad(self.rotation))
      .transform_point(location)
      .into()
  }

  pub fn add_rotation(&mut self, delta: f32) {
    self.rotation += delta;
  }

  fn model(&self) -> Matrix4<f32> {
    Matrix4::from_translation(self.position_world().into())
      * Matrix4::from_scale(0.05)
  }

  pub fn color(&self) -> [f32; 3] {
    self.color
  }

  pub fn draw(&self, frame: &mut glium::Frame, camera: &Camera) -> Result<()> {
    let Some(gpu) = self.gpu.as_ref() else {
      return Ok(());
    };

    let mv: Matrix4<f32> = camera.view() * self.model();
    let mv3: Matrix3<f32> = Matrix3 {
      x: mv.x.truncate(),
      y: mv.y.truncate(),
      z: mv.z.truncate(),
    };
    let mv_n: Matrix3<f32> =
      <Matrix3<f32> as Transform<Point3<f32>>>::inverse_transform(&mv3)
        .unwrap()
        .transpose();
    let mvp: Matrix4<f32> = camera.projection() * mv;

    // assume the light is lit up by something on the origin
    let light_pos: [f32; 3] =
      camera.view().transform_point([0.0; 3].into()).into();

    let uniforms = uniform! {
      mvp: <Matrix4<f32> as Into<[[f32; 4]; 4]>>::into(mvp),
      mv: <Matrix4<f32> as Into<[[f32; 4]; 4]>>::into(mv),
      mv_n: <Matrix3<f32> as Into<[[f32; 3]; 3]>>::into(mv_n),
      mode: 9, // plain color
      k_a: [0.1, 0.1, 0.1f32],
      k_d: self.color,
      k_s: [1.0, 1.0, 1.0f32],
      light_pos: light_pos,
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
    let obj = SimpleObj::load_from(common::sphere_path())?;
    let mesh = TriangleIndex::from_simple_obj(obj).upload(surface);
    let gpu = GPULight { mesh };
    self.gpu = Some(gpu);
    Ok(())
  }
}
