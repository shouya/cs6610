use std::{rc::Rc, time::Duration};

use glam::Mat4;
use glium::backend::Context;
use winit::keyboard::ModifiersState;

use crate::{object::LightObject, Camera, Light, Object, Result};

#[derive(Default)]
pub struct Scene {
  pub light: Light,
  pub camera: Camera,
  pub objects: Vec<Object>,
  // tracking the position and orientation of the light
  pub light_obj: Option<Object>,
  // used internally
  context: Option<Rc<Context>>,
}

// Event handling
impl Scene {
  pub fn handle_drag(&mut self, dx: f32, dy: f32, modifiers: ModifiersState) {
    if modifiers.shift_key() {
      self.light.rotate(dx, dy);
      self.sync_light_obj();
    } else {
      self.camera.rotate(dx, dy);
    }
    self.camera.update_view();
  }

  fn sync_light_obj(&mut self) {
    if let Some(light_obj) = &mut self.light_obj {
      light_obj.set_transform(self.light.light_object_transform());
    }
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

  pub fn update(&mut self, dt: &Duration) {
    for object in &mut self.objects {
      object.update(dt);
    }

    self.sync_light_obj();
    if let Some(light_obj) = &mut self.light_obj {
      light_obj.update(dt);
    }
  }

  pub fn add_object(&mut self, teapot: Object) {
    self.objects.push(teapot);
  }

  pub fn init_light_object(
    &mut self,
    facade: &impl glium::backend::Facade,
  ) -> Result<()> {
    let light_obj = LightObject::load(facade)?;
    self.light_obj = Some(light_obj);
    self.sync_light_obj();
    Ok(())
  }

  pub fn init_light(
    &mut self,
    facade: &impl glium::backend::Facade,
  ) -> Result<()> {
    self.light.load_shadow_program(facade)?;

    // save the context for later use
    let _ = self.context.insert(facade.get_context().clone());

    Ok(())
  }
}

impl Scene {
  pub fn draw(&self, frame: &mut glium::Frame) -> Result<()> {
    self.shadow_pass()?;

    self.draw_objects(frame)?;

    if let Some(light_obj) = &self.light_obj {
      light_obj.draw(frame, &self.camera, &self.light);
    }

    Ok(())
  }

  fn draw_objects(&self, frame: &mut glium::Frame) -> Result<()> {
    for object in &self.objects {
      object.draw(frame, &self.camera, &self.light);
    }
    Ok(())
  }

  fn shadow_pass(&self) -> Result<()> {
    let facade = self
      .context
      .as_ref()
      .expect("Context not initialized, please call init_light");
    let mut target = self.light.shadow_map_target(facade, &self.camera)?;
    target.clear();

    for object in &self.objects {
      target.draw_object(object);
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
