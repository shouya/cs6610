use std::{rc::Rc, time::Duration};

use glam::Mat4;
use glium::backend::Context;
use winit::keyboard::ModifiersState;

use crate::{
  light::ShadowMapVisual, object::LightObject, Camera, Light, Object, Result,
};

pub struct Scene {
  pub light: Light,
  pub camera: Camera,
  pub objects: Vec<Object>,
  // tracking the position and orientation of the light
  pub light_obj: Object,
  // the boolean is used to toggle the shadow map visual
  shadow_map_visual: (bool, ShadowMapVisual),
  // used internally
  context: Rc<Context>,
}

// Event handling
impl Scene {
  pub fn new(facade: &impl glium::backend::Facade) -> Result<Self> {
    Ok(Self {
      light: Light::new(facade)?,
      camera: Camera::default(),
      objects: Vec::new(),
      light_obj: LightObject::load(facade)?,
      shadow_map_visual: (false, ShadowMapVisual::new(facade)?),
      context: facade.get_context().clone(),
    })
  }

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
    self
      .light_obj
      .set_transform(self.light.light_object_transform());
  }

  pub fn handle_resize(&mut self, width: f32, height: f32) {
    self.camera.handle_resize(width, height);
    self.camera.update_view();
  }

  pub fn handle_scroll(&mut self, _dx: f32, dy: f32) {
    self.camera.change_distance(dy);
    self.camera.update_view();
  }

  pub fn handle_key(&mut self, key: winit::event::KeyEvent) {
    let key = key.logical_key.to_text();

    match key {
      Some("s") => {
        self.shadow_map_visual.0 = !self.shadow_map_visual.0;
      }
      Some("x") => {
        self.light.toggle_light_variant();
      }
      _ => {}
    }
  }

  pub fn update_view(&mut self) {
    self.camera.update_view()
  }

  pub fn update(&mut self, dt: &Duration) {
    for object in &mut self.objects {
      object.update(dt);
    }

    self.sync_light_obj();
    self.light_obj.update(dt);
  }

  pub fn add_object(&mut self, teapot: Object) {
    self.objects.push(teapot);
  }
}

impl Scene {
  pub fn draw(&self, frame: &mut glium::Frame) -> Result<()> {
    self.shadow_pass()?;

    if self.shadow_map_visual.0 {
      self.shadow_map_visual.1.draw(frame, &self.light)?;
      return Ok(());
    }

    self.draw_objects(frame)?;
    self.light_obj.draw(frame, &self.camera, &self.light);

    Ok(())
  }

  fn draw_objects(&self, frame: &mut glium::Frame) -> Result<()> {
    for object in &self.objects {
      object.draw(frame, &self.camera, &self.light);
    }
    Ok(())
  }

  fn shadow_pass(&self) -> Result<()> {
    let mut target =
      self.light.shadow_map_target(&self.context, &self.camera)?;
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
    self.shadow_map_visual.1.reload_shader(facade)?;
    Ok(())
  }
}
