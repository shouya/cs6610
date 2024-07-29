use std::{ffi::c_void, path::Path, rc::Rc, time::Duration};

use glium::{
  backend::{Context, Facade},
  Surface,
};

use crate::{
  background::Background, camera::Camera, light::Light, object::GPUObject,
  reflective_object::ReflectiveObject, Result,
};

pub struct Scene {
  pub camera: Camera,
  pub light: Light,
  objects: Vec<GPUObject>,
  reflective_objects: Vec<ReflectiveObject>,
  background: Background,
  // context is used to create framebuffers for updating the cubemap
  // without access to the Display.
  context: Rc<Context>,
}

impl Scene {
  pub fn new(facade: &impl Facade, cubemap: &[&Path; 6]) -> Result<Self> {
    let camera = Camera::new();
    let light = Light::new();
    let background = Background::new(facade, cubemap)?;
    let context = facade.get_context().clone();

    Ok(Self {
      camera,
      light,
      objects: vec![],
      reflective_objects: vec![],
      background,
      context,
    })
  }

  pub fn add_object(
    &mut self,
    object: GPUObject,
    reflective: bool,
  ) -> Result<()> {
    if reflective {
      self
        .reflective_objects
        .push(ReflectiveObject::new(&self.context, object)?);
    } else {
      self.objects.push(object);
    }
    Ok(())
  }

  pub fn reload_shader(&mut self, facade: &impl Facade) -> Result<()> {
    // TODO: reload shader for background as well
    self.background.reload_shader(facade)?;

    for obj in &mut self.objects {
      obj.reload_shader(facade)?;
    }
    for obj in &mut self.reflective_objects {
      obj.reload_shader(facade)?;
    }
    Ok(())
  }

  pub fn draw(&self, target: &mut impl Surface) {
    target.clear_color_and_depth(self.camera.clear_color().into(), 1.0);

    for obj in &self.objects {
      obj.draw(target, &self.camera, &self.light);
    }

    for obj in &self.reflective_objects {
      obj.draw(target, &self.camera, &self.light);
    }

    self.background.draw(target, &self.camera);
  }

  pub fn draw_with_camera(
    &self,
    target: &mut impl Surface,
    camera: &Camera,
    skip_obj: *const c_void,
  ) {
    target.clear_color_and_depth(camera.clear_color().into(), 1.0);

    for obj in &self.objects {
      // avoid rendering the object on its own cubemap
      if obj as *const _ as *const c_void == skip_obj {
        continue;
      }

      obj.draw(target, camera, &self.light);
    }

    for obj in &self.reflective_objects {
      if obj as *const _ as *const c_void == skip_obj {
        continue;
      }

      obj.draw(target, camera, &self.light);
    }

    self.background.draw(target, camera);
  }

  pub fn update(&mut self, dt: &Duration) {
    // do nothing for now
    for obj in &mut self.objects {
      obj.update(dt);
    }

    match self.update_cubemap() {
      Ok(_) => {}
      Err(e) => eprintln!("Failed to update cubemap: {}", e),
    }
  }

  pub fn update_cubemap(&self) -> Result<()> {
    for obj in &self.reflective_objects {
      obj.update_cubemap(&self.context, self)?;
    }
    Ok(())
  }
}
