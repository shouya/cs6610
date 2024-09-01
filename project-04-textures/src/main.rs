mod light;
mod mesh;
mod object;

use std::time::Duration;

use glam::{Mat4, Vec3};
use glium::{glutin::surface::WindowSurface, Display, Surface};

use object::{GPUObject, Teapot, Yoda};

use common::Axis;
use light::Light;
use winit::{
  application::ApplicationHandler,
  dpi::{LogicalSize, PhysicalSize},
  event::{self, WindowEvent},
  event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
  keyboard::{ModifiersState, NamedKey},
  window::{Window, WindowAttributes},
};

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
  objects: Vec<object::GPUObject>,
}

struct Camera {
  clear_color: [f32; 4],
  // camera
  aspect_ratio: f32,
  distance: f32,
  rotation: [f32; 2],
  perspective: bool,
  // cached matrices
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
    // default view matrix: eye at (0, 0, 2), looking at (0, 0, -1), up (0, 1, 0)
    let dir = Mat4::from_euler(
      glam::EulerRot::YXZ,
      self.rotation[0],
      self.rotation[1],
      0.0,
    )
    .transform_vector3(Vec3::new(0.0, 0.0, 1.0));
    let eye = Vec3::new(0.0, 0.0, 0.0) + -dir * self.distance;
    let up = Vec3::new(0.0, 1.0, 0.0);
    let origin = Vec3::new(0.0, 0.0, 0.0);
    Mat4::look_at_rh(eye, origin, up)
  }

  fn calc_proj(&self) -> Mat4 {
    if self.perspective {
      Mat4::perspective_rh_gl(60f32.to_radians(), self.aspect_ratio, 0.1, 100.0)
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
      objects: Vec::new(),
    }
  }

  fn set_axis(&mut self, axis: Axis) {
    self.axis = Some(axis);
  }

  fn update_view(&mut self) {
    self.camera.update_view();
  }

  fn update(&mut self, dt: Duration) {
    for obj in &mut self.objects {
      obj.update(&dt);
    }
  }

  fn render(&self, context: &Display<WindowSurface>) -> Result<()> {
    let mut frame = context.draw();
    frame.clear_color_and_depth(self.camera.clear_color.into(), 1.0);

    if let Some(axis) = self.axis.as_ref() {
      if self.show_axis {
        if let Err(e) = axis.draw(&mut frame, &self.camera.view_projection()) {
          eprintln!("Failed to draw axis: {}", e);
        }
      }
    }

    for obj in &self.objects {
      obj.draw(&mut frame, &self.camera, &self.light);
    }

    frame.finish()?;
    Ok(())
  }

  #[allow(unused)]
  fn update_bg_color(&mut self, dt: Duration) {
    self.t += dt.as_secs_f32();
    let t = self.t;
    let r = t.sin().abs();
    let g = (t * 2.0).sin().abs();
    let b = (t * 3.0).sin().abs();
    self.camera.clear_color = [r, g, b, 1.0];
  }

  fn add_object(&mut self, object: GPUObject) {
    self.objects.push(object);
  }
}

struct App {
  window: Option<Window>,
  display: Option<Display<WindowSurface>>,
  last_update: std::time::Instant,

  // left, right
  mouse_down: (bool, bool),
  last_pos: [f32; 2],
  mouse_pos: [f32; 2],
  modifiers: ModifiersState,

  model_name: &'static str,
  world: World,
}

impl App {
  fn new(model_name: &'static str) -> Self {
    let last_update = std::time::Instant::now();

    let world = World::new();

    Self {
      window: None,
      display: None,
      last_update,
      last_pos: [0.0, 0.0],
      mouse_pos: [0.0, 0.0],
      mouse_down: (false, false),
      modifiers: ModifiersState::empty(),
      world,
      model_name,
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

  fn handle_keyboard(
    &mut self,
    event: event::KeyEvent,
    event_loop: &ActiveEventLoop,
  ) {
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
      self.reload_shaders();
    }
  }

  fn handle_redraw(&mut self) {
    if let Some(display) = &self.display {
      self.world.render(display).expect("Failed to render");
    }
  }

  fn handle_idle(&mut self) {
    self.world.update(self.last_update.elapsed());

    let help =
      "Press 'p' to toggle perspective, 'a' to toggle axis, Esc to quit";

    let title = format!(
      "Teapot - {:.0} UPS",
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

  fn handle_cursor_moved(
    &mut self,
    position: winit::dpi::PhysicalPosition<f64>,
  ) {
    self.mouse_pos = [position.x as f32, position.y as f32];

    // left drag: rotate camera
    if self.mouse_down.0 && !self.modifiers.control_key() {
      let dx = self.mouse_pos[0] - self.last_pos[0];
      let dy = self.mouse_pos[1] - self.last_pos[1];
      self.world.camera.rotation[0] += dx * 0.01;
      self.world.camera.rotation[1] += dy * 0.01;
      self.world.update_view();
      self.request_redraw();
    }

    // ctrl + left drag: rotate light
    if self.mouse_down.0 && self.modifiers.control_key() {
      let dx = self.mouse_pos[0] - self.last_pos[0];
      let _dy = self.mouse_pos[1] - self.last_pos[1];
      self.world.light.add_rotation(dx * 0.01);
      self.request_redraw();
    }

    // right drag: change camera distance
    if self.mouse_down.1 {
      let dy = self.mouse_pos[1] - self.last_pos[1];

      self.world.camera.distance += dy * 0.01;
      self.world.camera.distance = self.world.camera.distance.clamp(0.1, 10.0);
      self.world.update_view();
      self.request_redraw();
    }

    self.last_pos = self.mouse_pos;
  }

  fn handle_mouse_wheel(&mut self, delta: event::MouseScrollDelta) {
    let d = match delta {
      event::MouseScrollDelta::LineDelta(_x, y) => y * 20.0,
      event::MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
    };

    self.world.camera.distance -= d * 0.01;
    self.world.camera.distance = self.world.camera.distance.clamp(0.1, 10.0);
    self.world.update_view();
  }

  fn request_redraw(&self) {
    if let Some(window) = &self.window {
      window.request_redraw();
    }
  }

  fn reload_shaders(&mut self) {
    if let Some(display) = &self.display {
      for obj in self.world.objects.iter_mut() {
        if let Err(e) = obj.reload_shader(display) {
          eprintln!("Failed to reload shader: {}", e);
          return;
        }
      }
      println!("Reloaded shaders");
      self.request_redraw();
    }
  }

  fn handle_init(&mut self, display: &Display<WindowSurface>) -> Result<()> {
    let model = match self.model_name {
      "teapot" => Teapot::load(display)?,
      "yoda" => Yoda::load(display)?,
      _ => unreachable!(),
    };
    self.world.add_object(model);

    // setup axis object
    let axis = Axis::new(display)?;
    self.world.set_axis(axis);

    // initial update
    self.world.update(std::time::Duration::from_secs(0));
    Ok(())
  }
}

impl ApplicationHandler for App {
  fn resumed(&mut self, event_loop: &ActiveEventLoop) {
    if self.window.is_none() {
      let window_attrs = WindowAttributes::default()
        .with_title("Project 04")
        .with_inner_size(LogicalSize::new(800, 600));

      let window = match event_loop.create_window(window_attrs) {
        Ok(window) => window,
        Err(e) => {
          eprintln!("Failed to create window: {}", e);
          event_loop.exit();
          return;
        }
      };

      let display = common::gl_boilerplate::init_display(&window);

      match self.handle_init(&display) {
        Ok(_) => {
          self.window = Some(window);
          self.display = Some(display);
        }
        Err(e) => {
          eprintln!("Failed to initialize: {}", e);
          event_loop.exit();
        }
      }
    }
  }

  fn window_event(
    &mut self,
    event_loop: &ActiveEventLoop,
    _window_id: winit::window::WindowId,
    event: event::WindowEvent,
  ) {
    match event {
      WindowEvent::Resized(size) => self.handle_resize(size),
      WindowEvent::KeyboardInput { event, .. } => {
        self.handle_keyboard(event, event_loop)
      }
      WindowEvent::RedrawRequested => self.handle_redraw(),
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
    let wake_up_at = self.last_update + TARGET_FRAME_TIME;
    event_loop.set_control_flow(ControlFlow::WaitUntil(wake_up_at));

    if self.last_update.elapsed() >= TARGET_FRAME_TIME {
      self.handle_idle();
    }
  }
}

fn main() -> Result<()> {
  let event_loop = EventLoop::new()?;

  let mut args = std::env::args();
  let bin_name = args.next().unwrap(); // skip $0
  let model_name = match args.next().as_deref() {
    Some("teapot") => "teapot",
    Some("yoda") => "yoda",
    _ => {
      eprintln!("Usage: {} [teapot|yoda]", bin_name);
      eprintln!("Loading the yoda model by default");
      "yoda"
    }
  };

  let mut app = App::new(model_name);
  event_loop.run_app(&mut app)?;

  Ok(())
}
