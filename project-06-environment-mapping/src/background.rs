use std::{fs::read_to_string, path::Path};

use common::{project_asset_path, to_raw_image};
use glium::{
  backend::Facade,
  framebuffer::SimpleFrameBuffer,
  implement_vertex,
  texture::{CubeLayer, Cubemap},
  uniform, BlitTarget, Program, Surface, Texture2d, VertexBuffer,
};

use crate::{camera::Camera, Result};

#[derive(Copy, Clone, derive_more::From)]
struct CameraPlaneVertex {
  clip_pos: [f32; 2],
}

implement_vertex!(CameraPlaneVertex, clip_pos);

pub struct Background {
  vertices: VertexBuffer<CameraPlaneVertex>,
  cubemap: Cubemap,
  shader: Program,
}

impl Background {
  pub fn new(facade: &impl Facade, cubemap_path: &[&Path; 6]) -> Result<Self> {
    // tracing the square into trig strip as Z-shaped path
    let verts = [
      CameraPlaneVertex::from([-1.0, 1.0]),
      CameraPlaneVertex::from([-1.0, -1.0]),
      CameraPlaneVertex::from([1.0, 1.0]),
      CameraPlaneVertex::from([1.0, -1.0]),
    ];
    let vertices = VertexBuffer::new(facade, &verts)?;

    let vert_src = read_to_string(project_asset_path!("camera_plane.vert"))?;
    let frag_src = read_to_string(project_asset_path!("camera_plane.frag"))?;
    let shader = Program::from_source(facade, &vert_src, &frag_src, None)?;

    let cubemap = load_cubemap_from_file(facade, cubemap_path)?;

    Ok(Self {
      vertices,
      shader,
      cubemap,
    })
  }

  pub fn reload_shader(&mut self, facade: &impl Facade) -> Result<()> {
    let vert_src = read_to_string(project_asset_path!("camera_plane.vert"))?;
    let frag_src = read_to_string(project_asset_path!("camera_plane.frag"))?;
    self.shader = Program::from_source(facade, &vert_src, &frag_src, None)?;
    Ok(())
  }

  pub fn draw(&self, target: &mut impl Surface, camera: &Camera) {
    let view_proj_inv: [[f32; 4]; 4] =
      camera.view_projection().inverse().to_cols_array_2d();
    let env_map = self
      .cubemap
      .sampled()
      .minify_filter(glium::uniforms::MinifySamplerFilter::Linear)
      .magnify_filter(glium::uniforms::MagnifySamplerFilter::Linear);

    let uniforms = uniform! {
      view_proj_inv: view_proj_inv,
      env_map: env_map,
    };

    let indices =
      glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip);

    let draw_params = glium::DrawParameters {
      depth: glium::Depth {
        // In vertex shader we will write gl_Position.z=1.0 for all
        // vertices. This way only the background will be drawn
        // because clear_depth=1.0.
        test: glium::draw_parameters::DepthTest::IfEqual,
        write: false,
        ..Default::default()
      },
      ..Default::default()
    };

    target
      .draw(
        &self.vertices,
        indices,
        &self.shader,
        &uniforms,
        &draw_params,
      )
      .unwrap();
  }
}

fn load_cubemap_from_file(
  facade: &impl Facade,
  images: &[&Path; 6],
) -> Result<Cubemap> {
  let images: Vec<Texture2d> = images
    .iter()
    .map(|path| {
      let img = image::open(path).unwrap().to_rgb8();
      let raw_img = to_raw_image(&img);
      let texture = Texture2d::new(facade, raw_img)?;
      Ok(texture)
    })
    .collect::<Result<Vec<_>>>()?;

  let dimension = images[0].get_width();
  let cubemap = Cubemap::empty(facade, dimension)?;
  let full_rect = BlitTarget {
    left: 0,
    bottom: 0,
    width: dimension as i32,
    height: dimension as i32,
  };

  // unfortunately glium doesn't support loading image data directly
  // into cubemap textures. So we have to copy the data from the
  // individual textures.
  let blit_face = |i: usize, face: CubeLayer| -> Result<()> {
    let face = cubemap.main_level().image(face);
    let target = SimpleFrameBuffer::new(facade, face)?;
    let source = images[i].as_surface();
    source.blit_whole_color_to(
      &target,
      &full_rect,
      glium::uniforms::MagnifySamplerFilter::Nearest,
    );
    Ok(())
  };

  blit_face(0, CubeLayer::PositiveX)?;
  blit_face(1, CubeLayer::NegativeX)?;
  blit_face(2, CubeLayer::PositiveY)?;
  blit_face(3, CubeLayer::NegativeY)?;
  blit_face(4, CubeLayer::PositiveZ)?;
  blit_face(5, CubeLayer::NegativeZ)?;

  Ok(cubemap)
}
