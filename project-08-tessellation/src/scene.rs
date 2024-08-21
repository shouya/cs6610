use std::{rc::Rc, time::Duration};

use glam::Mat4;
use glium::backend::Context;
use winit::keyboard::ModifiersState;

use crate::{
  light::ShadowMapVisual, object::LightObject, teapot_quad::TeapotQuad, Camera,
  Light, Object, Result,
};

pub struct Scene {
  pub light: Light,
  pub camera: Camera,
  pub teapot_quad: Option<TeapotQuad>,
  pub show_wireframe: bool,
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
      teapot_quad: None,
      light_obj: LightObject::load(facade)?,
      shadow_map_visual: (false, ShadowMapVisual::new(facade)?),
      show_wireframe: false,
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
      Some("+") => {
        if let Some(quad) = &mut self.teapot_quad {
          quad.update_tess_level(1);
        }
      }
      Some("-") => {
        if let Some(quad) = &mut self.teapot_quad {
          quad.update_tess_level(-1);
        }
      }
      Some("w") => {
        self.show_wireframe = !self.show_wireframe;
      }
      _ => {}
    }
  }

  pub fn update_view(&mut self) {
    self.camera.update_view()
  }

  pub fn update(&mut self, dt: &Duration) {
    if let Some(quad) = &mut self.teapot_quad {
      quad.update(dt);
    }

    self.sync_light_obj();
    self.light_obj.update(dt);
  }

  pub fn set_quad(&mut self, quad: TeapotQuad) {
    self.teapot_quad = Some(quad);
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
    self.light_obj.draw(frame, &self.camera, &self.light)?;

    Ok(())
  }

  fn draw_objects(&self, frame: &mut glium::Frame) -> Result<()> {
    if let Some(quad) = &self.teapot_quad {
      quad.draw(frame, &self.camera, &self.light)?;
      if self.show_wireframe {
        quad.draw_wireframe(frame, &self.camera, &self.light)?;
      }
    }

    Ok(())
  }

  fn shadow_pass(&self) -> Result<()> {
    let mut target =
      self.light.shadow_map_target(&self.context, &self.camera)?;
    target.clear();

    if let Some(quad) = &self.teapot_quad {
      target.draw_object(quad)?;
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
    if let Some(quad) = &mut self.teapot_quad {
      quad.reload_shader(facade)?;
    }
    self.shadow_map_visual.1.reload_shader(facade)?;
    Ok(())
  }
}
