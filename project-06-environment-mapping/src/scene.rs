use std::{ffi::c_void, path::Path, rc::Rc, time::Duration};

use glium::{
  backend::{Context, Facade},
  Surface,
};

use crate::{
  background::Background, camera::Camera, light::Light, object::GPUObject,
  reflective_object::ReflectiveObject, reflective_plane::ReflectivePlane,
  Result,
};

pub struct Scene {
  pub camera: Camera,
  pub light: Light,
  objects: Vec<GPUObject>,
  reflective_objects: Vec<ReflectiveObject>,
  reflective_planes: Vec<ReflectivePlane>,
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
      reflective_planes: vec![],
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

  pub fn add_plane(
    &mut self,
    object: GPUObject,
    reflective: bool,
  ) -> Result<()> {
    if reflective {
      self
        .reflective_planes
        .push(ReflectivePlane::new(&self.context, object)?);
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
    for plane in &mut self.reflective_planes {
      plane.reload_shader(facade)?;
    }
    Ok(())
  }

  pub fn draw(&self, target: &mut impl Surface) {
    self.draw_with_camera(target, &self.camera, std::ptr::null());
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

    for plane in &self.reflective_planes {
      if plane as *const _ as *const c_void == skip_obj {
        continue;
      }
      plane.draw(target, camera, &self.light);
    }

    self.background.draw(target, camera);
  }

  pub fn update(&mut self, dt: &Duration) {
    // do nothing for now
    for obj in &mut self.objects {
      obj.update(dt);
    }

    for obj in &mut self.reflective_objects {
      obj.update(dt);
    }

    for plane in &mut self.reflective_planes {
      plane.update(dt);
    }

    match self.update_cubemaps() {
      Ok(_) => {}
      Err(e) => eprintln!("Failed to update cubemap: {}", e),
    }

    match self.update_world_textures() {
      Ok(_) => {}
      Err(e) => eprintln!("Failed to update world textures: {}", e),
    }
  }

  pub fn handle_resize(&mut self, facade: &impl Facade) {
    let new_size = facade.get_context().get_framebuffer_dimensions();
    self.camera.handle_window_resize(new_size);

    for plane in &mut self.reflective_planes {
      plane.handle_resize(facade, new_size);
    }
  }

  pub fn update_cubemaps(&self) -> Result<()> {
    for obj in &self.reflective_objects {
      obj.update_cubemap(&self.context, self)?;
    }
    Ok(())
  }

  pub fn update_world_textures(&self) -> Result<()> {
    for plane in &self.reflective_planes {
      plane.update_world_texture(&self.context, self, &self.camera)?;
    }
    Ok(())
  }
}
