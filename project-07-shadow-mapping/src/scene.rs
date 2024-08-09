use std::time::Duration;

use glam::Mat4;
use winit::keyboard::ModifiersState;

use crate::{Camera, Light, Object, Result};

#[derive(Default)]
pub struct Scene {
  pub light: Light,
  pub camera: Camera,
  pub objects: Vec<Object>,
}

// Event handling
impl Scene {
  pub fn handle_drag(&mut self, dx: f32, dy: f32, _modifiers: ModifiersState) {
    self.camera.rotate(dx, dy);
    self.camera.update_view();
  }

  pub fn handle_resize(&mut self, width: f32, height: f32) {
    self.camera.handle_resize(width, height);
    self.camera.update_view();
  }

  pub fn handle_scroll(&mut self, _dx: f32, dy: f32) {
    self.camera.change_distance(dy);
    self.camera.update_view();
  }

  pub fn update_view(&mut self) {
    self.camera.update_view()
  }

  pub fn update(&mut self, _dt: &Duration) {}

  pub fn add_object(&mut self, teapot: Object) {
    self.objects.push(teapot);
  }
}

impl Scene {
  pub fn draw(&self, frame: &mut glium::Frame) -> Result<()> {
    for object in &self.objects {
      object.draw(frame, &self.camera, &self.light);
    }

    Ok(())
  }

  pub fn view_projection(&self) -> Mat4 {
    self.camera.view_projection()
  }

  pub fn reload_shader(
    &mut self,
    facade: &impl glium::backend::Facade,
  ) -> Result<()> {
    for object in &mut self.objects {
      object.reload_shader(facade)?;
    }
    Ok(())
  }
}
