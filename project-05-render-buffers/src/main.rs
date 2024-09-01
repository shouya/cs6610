mod light;
mod mesh;
mod object;

use std::time::Duration;

use glium::{
  dynamic_uniform, glutin::surface::WindowSurface, Display, Surface,
};

use object::{GPUObject, IndirectScene, Teapot, Yoda};
use winit::{
  application::ApplicationHandler,
  dpi::{LogicalSize, PhysicalPosition, PhysicalSize},
  event::{self, KeyEvent, WindowEvent},
  event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
  keyboard::{ModifiersState, NamedKey},
  window::{Window, WindowAttributes, WindowId},
};

use glam::{EulerRot, Mat3, Mat4, Vec3};

use common::Axis;
use light::Light;

type Result<T> = anyhow::Result<T>;

const TARGET_UPS: u32 = 60;
const TARGET_FRAME_TIME: Duration =
  Duration::from_micros(1_000_000 / TARGET_UPS as u64);

struct World {
  t: f32,
  camera: Camera,
  show_axis: bool,
  axis: Option<Axis>,
  light: Light,
  indirect_scene: Option<IndirectScene>,
  object: Option<GPUObject>,
}

struct Camera {
  clear_color: [f32; 4],
  aspect_ratio: f32,
  distance: f32,
  rotation: [f32; 2],
  perspective: bool,
  m_view: Mat4,
  m_proj: Mat4,
  m_view_proj: Mat4,
}

impl Camera {
  fn new() -> Self {
    Self {
      clear_color: [0.0, 0.0, 0.0, 1.0],
      aspect_ratio: 1.0,
      distance: 2.0,
      rotation: [0.0, 0.0],
      perspective: true,
      m_view: Mat4::IDENTITY,
      m_proj: Mat4::IDENTITY,
      m_view_proj: Mat4::IDENTITY,
    }
  }

  fn handle_window_resize(&mut self, new_size: (u32, u32)) {
    self.aspect_ratio = new_size.0 as f32 / new_size.1 as f32;
  }

  fn view(&self) -> Mat4 {
    self.m_view
  }

  fn view_projection(&self) -> Mat4 {
    self.m_view_proj
  }

  fn projection(&self) -> Mat4 {
    self.m_proj
  }

  fn calc_view(&self) -> Mat4 {
    let dir = Mat3::from_euler(
      EulerRot::YXZ,
      self.rotation[0].to_radians(),
      -self.rotation[1].to_radians(),
      0.0,
    ) * Vec3::new(0.0, 0.0, 1.0);
    let eye = Vec3::new(0.0, 0.0, 0.0) + -dir * self.distance;
    let up = Vec3::new(0.0, 1.0, 0.0);
    let origin = Vec3::new(0.0, 0.0, 0.0);
    Mat4::look_at_rh(eye, origin, up)
  }

  fn calc_proj(&self) -> Mat4 {
    if self.perspective {
      Mat4::perspective_rh_gl(
        60.0_f32.to_radians(),
        self.aspect_ratio,
        0.1,
        100.0,
      )
    } else {
      Mat4::orthographic_rh_gl(
        -1.0,
        1.0,
        -1.0 / self.aspect_ratio,
        1.0 / self.aspect_ratio,
        0.1,
        100.0,
      ) * (1.0 / self.distance)
    }
  }

  fn update_view(&mut self) {
    self.m_view = self.calc_view();
    self.m_proj = self.calc_proj();
    self.m_view_proj = self.m_proj * self.m_view;
  }
}

impl World {
  fn new() -> Self {
    Self {
      t: 0.0,
      camera: Camera::new(),
      axis: None,
      show_axis: true,
      light: Light::new(),
      object: None,
      indirect_scene: None,
    }
  }

  fn set_axis(&mut self, axis: Axis) {
    self.axis = Some(axis);
  }

  fn update_view(&mut self) {
    self.camera.update_view();
  }

  fn update(&mut self, dt: Duration) {
    self.t += dt.as_secs_f32();

    if let Some(object) = &mut self.object {
      object.update(&dt);
    }

    if let Some(indirect_scene) = &mut self.indirect_scene {
      for obj in &mut indirect_scene.objects {
        obj.update(&dt);
      }
    }
  }

  fn render(
    &self,
    context: &Display<WindowSurface>,
    _dt: Duration,
  ) -> Result<()> {
    let sampler = self
      .indirect_scene
      .as_ref()
      .and_then(|scene| scene.render().ok())
      .map(|texture| {
        texture
          .sampled()
          .magnify_filter(glium::uniforms::MagnifySamplerFilter::Linear)
          .minify_filter(glium::uniforms::MinifySamplerFilter::Linear)
      });

    let mut frame = context.draw();
    frame.clear_color_and_depth(self.camera.clear_color.into(), 1.0);

    if let Some(axis) = self.axis.as_ref() {
      if self.show_axis {
        if let Err(e) = axis.draw(&mut frame, &self.camera.view_projection()) {
          eprintln!("Failed to draw axis: {}", e);
        }
      }
    }

    if let Some(obj) = &self.object {
      let uniforms = if let Some(sampler) = &sampler {
        dynamic_uniform! {
            map_Kd: sampler,
            use_map_Kd: &1u32,
            map_Ka: sampler,
            use_map_Ka: &1u32,
            map_Ks: sampler,
            use_map_Ks: &1u32,
        }
      } else {
        dynamic_uniform! {}
      };

      obj.draw_with_extra_uniforms(
        &mut frame,
        &self.camera,
        &self.light,
        uniforms,
      );
    }

    frame.finish()?;
    Ok(())
  }

  fn set_indirect_scene(&mut self, scene: IndirectScene) {
    let _ = self.indirect_scene.insert(scene);
  }

  fn set_object(&mut self, object: GPUObject) {
    let _ = self.object.insert(object);
  }
}

struct App {
  window: Option<Window>,
  display: Option<Display<WindowSurface>>,
  last_update: std::time::Instant,

  mouse_down: (bool, bool),
  last_pos: [f32; 2],
  mouse_pos: [f32; 2],
  modifiers: ModifiersState,

  world: World,
}

impl App {
  fn new() -> Self {
    let last_update = std::time::Instant::now();

    Self {
      window: None,
      display: None,
      last_update,
      last_pos: [0.0, 0.0],
      mouse_pos: [0.0, 0.0],
      mouse_down: (false, false),
      modifiers: ModifiersState::empty(),
      world: World::new(),
    }
  }

  fn handle_resize(&mut self, size: PhysicalSize<u32>) {
    println!("Resized to {:?}", size);
    self
      .world
      .camera
      .handle_window_resize((size.width, size.height));
    self.world.update_view();
    self.request_redraw();
  }

  fn handle_keyboard(&mut self, event: KeyEvent, event_loop: &ActiveEventLoop) {
    if !event.state.is_pressed() {
      return;
    }
    if event.logical_key == NamedKey::Escape {
      event_loop.exit();
    } else if event.logical_key.to_text() == Some("p") {
      self.world.camera.perspective ^= true;
      self.world.update_view();
      self.request_redraw();
    } else if event.logical_key.to_text() == Some("a") {
      self.world.show_axis = !self.world.show_axis;
      self.request_redraw();
    } else if event.logical_key == NamedKey::F6 {
      if let Some(indirect_scene) = &mut self.world.indirect_scene {
        for obj in &mut indirect_scene.objects {
          obj.reload_shader(self.display.as_ref().unwrap()).unwrap();
        }
      }

      if let Some(object) = &mut self.world.object {
        object
          .reload_shader(self.display.as_ref().unwrap())
          .unwrap();
      }

      println!("Reloaded shaders");
      self.request_redraw();
    }
  }

  fn handle_redraw(&mut self) {
    if let Some(display) = &self.display {
      self
        .world
        .render(display, self.last_update.elapsed())
        .expect("Failed to render");
    }
  }

  fn update(&mut self) {
    self.world.update(self.last_update.elapsed());

    let help =
      "Press 'p' to toggle perspective, 'a' to toggle axis, Esc to quit";
    let title = format!(
      "3D Object Viewer - {:.0} UPS",
      1_000_000_000 / self.last_update.elapsed().as_nanos()
    );

    if let Some(window) = self.window.as_ref() {
      window.set_title(&format!("{} - {}", title, help));
      window.request_redraw();
    }

    self.last_update = std::time::Instant::now();
  }

  fn handle_mouse_input(
    &mut self,
    state: event::ElementState,
    button: event::MouseButton,
  ) {
    let pressed = state.is_pressed();

    match button {
      event::MouseButton::Left => {
        self.mouse_down.0 = pressed;
      }
      event::MouseButton::Right => {
        self.mouse_down.1 = pressed;
      }
      _ => {}
    }
  }

  fn handle_cursor_moved(&mut self, position: PhysicalPosition<f64>) {
    self.mouse_pos = [position.x as f32, position.y as f32];
    let indirect_scene = self.world.indirect_scene.as_mut().unwrap();

    let camera_target: &mut Camera = if self.modifiers.shift_key() {
      &mut indirect_scene.camera
    } else {
      &mut self.world.camera
    };
    let light_target: &mut Light = if self.modifiers.shift_key() {
      &mut indirect_scene.light
    } else {
      &mut self.world.light
    };

    if self.mouse_down.0 && !self.modifiers.control_key() {
      let dx = self.mouse_pos[0] - self.last_pos[0];
      let dy = self.mouse_pos[1] - self.last_pos[1];
      camera_target.rotation[0] += dx * 0.1;
      camera_target.rotation[1] += dy * 0.1;
    }

    if self.mouse_down.0 && self.modifiers.control_key() {
      let dx = self.mouse_pos[0] - self.last_pos[0];
      let _dy = self.mouse_pos[1] - self.last_pos[1];
      light_target.add_rotation(dx * 0.01);
    }

    if self.mouse_down.1 {
      let dy = self.mouse_pos[1] - self.last_pos[1];

      camera_target.distance += dy * 0.01;
      camera_target.distance = camera_target.distance.clamp(0.1, 10.0);
    }

    indirect_scene.camera.update_view();
    self.world.update_view();
    self.last_pos = self.mouse_pos;
    self.request_redraw();
  }

  fn handle_mouse_wheel(&mut self, delta: event::MouseScrollDelta) {
    let d = match delta {
      event::MouseScrollDelta::LineDelta(_x, y) => y * 20.0,
      event::MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
    };

    let indirect_scene = self.world.indirect_scene.as_mut().unwrap();
    let camera_target: &mut Camera = if self.modifiers.shift_key() {
      &mut indirect_scene.camera
    } else {
      &mut self.world.camera
    };
    camera_target.distance -= d * 0.01;
    camera_target.distance = camera_target.distance.clamp(0.1, 10.0);
    indirect_scene.camera.update_view();
    self.world.update_view();
  }

  fn request_redraw(&self) {
    if let Some(window) = self.window.as_ref() {
      window.request_redraw();
    }
  }

  fn schedule_next_frame(&self, event_loop: &ActiveEventLoop) {
    let wake_up_at = self.last_update + TARGET_FRAME_TIME;
    event_loop.set_control_flow(ControlFlow::WaitUntil(wake_up_at));
  }

  fn handle_init(&mut self, display: &Display<WindowSurface>) -> Result<()> {
    let axis = Axis::new(display)?;
    self.world.set_axis(axis);

    self.world.set_object(Teapot::load(display)?);

    let indirect_scene = Yoda::load_indirect_scene(display)?;
    self.world.set_indirect_scene(indirect_scene);

    self.world.update(Duration::from_secs(0));
    Ok(())
  }
}

impl ApplicationHandler for App {
  fn resumed(&mut self, event_loop: &ActiveEventLoop) {
    if self.window.is_none() {
      let window_attrs = WindowAttributes::default()
        .with_title("3D Object Viewer")
        .with_inner_size(LogicalSize::new(800, 600));

      match event_loop.create_window(window_attrs) {
        Ok(window) => {
          let display = common::gl_boilerplate::init_display(&window);
          self.handle_init(&display).expect("Failed to init world");

          self.window = Some(window);
          self.display = Some(display);
        }
        Err(e) => {
          eprintln!("Failed to create window: {}", e);
          event_loop.exit();
        }
      }
    }
  }

  fn window_event(
    &mut self,
    event_loop: &ActiveEventLoop,
    _window_id: WindowId,
    event: WindowEvent,
  ) {
    match event {
      WindowEvent::Resized(size) => self.handle_resize(size),
      WindowEvent::KeyboardInput { event, .. } => {
        self.handle_keyboard(event, event_loop)
      }
      WindowEvent::RedrawRequested => {
        self.handle_redraw();
      }
      WindowEvent::CloseRequested => {
        event_loop.exit();
      }
      WindowEvent::MouseInput { state, button, .. } => {
        self.handle_mouse_input(state, button);
      }
      WindowEvent::MouseWheel { delta, .. } => {
        self.handle_mouse_wheel(delta);
      }
      WindowEvent::CursorMoved { position, .. } => {
        self.handle_cursor_moved(position);
      }
      WindowEvent::ModifiersChanged(new_modifiers) => {
        self.modifiers = new_modifiers.state();
      }
      _ => {}
    }
  }

  fn new_events(
    &mut self,
    event_loop: &ActiveEventLoop,
    _cause: event::StartCause,
  ) {
    self.schedule_next_frame(event_loop);

    if self.last_update.elapsed() > TARGET_FRAME_TIME {
      self.update();
    }
  }
}

fn main() -> Result<()> {
  let event_loop = EventLoop::new()?;
  let mut app = App::new();

  event_loop.run_app(&mut app)?;
  Ok(())
}
