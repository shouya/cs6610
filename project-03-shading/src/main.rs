mod teapot;

use std::{fmt::Debug, path::Path, time::Duration};

use glium::{glutin::surface::WindowSurface, Display, Surface};

use teapot::Teapot;
use winit::{
  dpi::PhysicalSize,
  event::{DeviceId, Event, KeyEvent, WindowEvent},
  event_loop::{EventLoopBuilder, EventLoopProxy, EventLoopWindowTarget},
  keyboard::NamedKey,
  window::{Window, WindowId},
};

use cgmath::{Deg, Matrix3, Matrix4, Point3, SquareMatrix as _, Vector3};

use common::Axis;

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

const SHADER_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/shader");
const MODEL_PATH: &str = "assets/teapot.obj";

#[derive(Debug)]
enum UserSignal {
  Quit,
}

struct World {
  t: f32,
  camera: Camera,
  show_axis: bool,
  axis: Option<Axis>,
  teapot: Option<Teapot>,
}

struct Camera {
  clear_color: [f32; 4],
  // camera
  aspect_ratio: f32,
  distance: f32,
  rotation: [f32; 2],
  perspective: bool,
  // cached matrices
  m_view: Matrix4<f32>,
  m_proj: Matrix4<f32>,
  m_vp: Matrix4<f32>,
}

impl Camera {
  fn new() -> Self {
    Self {
      clear_color: [0.0, 0.0, 0.0, 1.0],
      aspect_ratio: 1.0,
      distance: 2.0,
      rotation: [0.0, 0.0],
      perspective: true,

      m_view: Matrix4::identity(),
      m_proj: Matrix4::identity(),
      m_vp: Matrix4::identity(),
    }
  }

  fn handle_window_resize(&mut self, new_size: (u32, u32)) {
    self.aspect_ratio = new_size.0 as f32 / new_size.1 as f32;
  }

  fn vp(&self) -> Matrix4<f32> {
    self.m_vp
  }

  fn calc_view(&self) -> Matrix4<f32> {
    // default view matrix: eye at (0, 0, 2), looking at (0, 0, -1), up (0, 1, 0)
    let dir = Matrix3::from_angle_y(Deg(self.rotation[0]))
      * Matrix3::from_angle_x(-Deg(self.rotation[1]))
      * Vector3::new(0.0, 0.0, -1.0);
    let eye = Point3::new(0.0, 0.0, 0.0) + -dir * self.distance;
    let up = Vector3::new(0.0, 1.0, 0.0);
    let origin = Point3::new(0.0, 0.0, 0.0);
    Matrix4::look_at_rh(eye, origin, up)
  }

  fn calc_proj(&self) -> Matrix4<f32> {
    if self.perspective {
      cgmath::perspective(cgmath::Deg(60.0), self.aspect_ratio, 0.1, 100.0)
    } else {
      cgmath::Matrix4::from_scale(1.0 / self.distance)
        * cgmath::ortho(
          -1.0,
          1.0,
          -1.0 / self.aspect_ratio,
          1.0 / self.aspect_ratio,
          0.1,
          100.0,
        )
    }
  }

  fn update_view(&mut self) {
    self.m_view = self.calc_view();
    self.m_proj = self.calc_proj();
    self.m_vp = self.m_proj * self.m_view;
  }
}

impl World {
  fn new() -> Self {
    Self {
      t: 0.0,
      camera: Camera::new(),
      axis: None,
      show_axis: true,
      teapot: None,
    }
  }

  fn set_teapot(&mut self, teapot: Teapot) {
    self.teapot = Some(teapot);
  }

  fn set_axis(&mut self, axis: Axis) {
    self.axis = Some(axis);
  }

  fn update_view(&mut self) {
    self.camera.update_view();
  }

  fn update(&mut self, dt: Duration) {
    // self.update_bg_color(dt);
    if let Some(teapot) = self.teapot.as_mut() {
      teapot.update(dt);
    }
  }

  fn render(
    &self,
    context: &Display<WindowSurface>,
    _dt: Duration,
  ) -> Result<()> {
    let mut frame = context.draw();
    frame.clear_color_and_depth(self.camera.clear_color.into(), 1.0);

    if let Some(teapot) = self.teapot.as_ref() {
      if let Err(e) = teapot.draw(&mut frame, &self.camera) {
        eprintln!("Failed to draw teapot: {}", e);
      }
    }

    if let Some(axis) = self.axis.as_ref() {
      if self.show_axis {
        if let Err(e) = axis.draw(&mut frame, &self.camera.vp()) {
          eprintln!("Failed to draw axis: {}", e);
        }
      }
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
}

struct App {
  #[allow(unused)]
  window: Window,
  display: Display<WindowSurface>,
  event_loop: EventLoopProxy<UserSignal>,
  boot_time: std::time::Instant,
  last_update: std::time::Instant,
  last_frame: std::time::Instant,

  // left, right
  mouse_down: (bool, bool),
  last_pos: [f32; 2],
  mouse_pos: [f32; 2],

  world: World,
}

impl App {
  fn new(
    window: Window,
    display: Display<WindowSurface>,
    event_loop: EventLoopProxy<UserSignal>,
  ) -> Result<Self> {
    let boot_time = std::time::Instant::now();
    let last_update = boot_time;
    let last_frame = boot_time;

    let world = World::new();

    Ok(Self {
      window,
      display,
      event_loop,
      boot_time,
      last_update,
      last_frame,
      last_pos: [0.0, 0.0],
      mouse_pos: [0.0, 0.0],
      mouse_down: (false, false),
      world,
    })
  }

  fn handle_widow_event(
    &mut self,
    _window_id: WindowId,
    event: WindowEvent,
    window_target: &EventLoopWindowTarget<UserSignal>,
  ) {
    match event {
      WindowEvent::Resized(size) => self.handle_resize(size),
      WindowEvent::KeyboardInput {
        device_id,
        event,
        is_synthetic,
      } => self.handle_keyboard(device_id, event, is_synthetic),
      WindowEvent::RedrawRequested => self.handle_redraw(),
      WindowEvent::CloseRequested => {
        window_target.exit();
      }
      WindowEvent::MouseInput {
        device_id,
        state,
        button,
      } => {
        self.handle_mouse_input(device_id, state, button);
      }
      WindowEvent::MouseWheel {
        device_id,
        delta,
        phase,
      } => {
        self.handle_mouse_wheel(device_id, delta, phase);
      }
      WindowEvent::CursorMoved {
        device_id,
        position,
      } => {
        self.handle_cursor_moved(device_id, position);
      }
      _ => {}
    }
  }

  fn handle_resize(&mut self, size: PhysicalSize<u32>) {
    println!("Resized to {:?}", size);
    self
      .world
      .camera
      .handle_window_resize((size.width, size.height));
    self.world.update_view();
    self.window.request_redraw();
  }

  fn handle_keyboard(
    &mut self,
    _device_id: DeviceId,
    event: KeyEvent,
    _is_synthetic: bool,
  ) {
    if !event.state.is_pressed() {
      return;
    }
    if event.logical_key == NamedKey::Escape {
      self.event_loop.send_event(UserSignal::Quit).unwrap();
    } else if event.logical_key.to_text() == Some("p") {
      self.world.camera.perspective ^= true;
      self.world.update_view();
      self.window.request_redraw();
    } else if event.logical_key.to_text() == Some("a") {
      self.world.show_axis = !self.world.show_axis;
      self.window.request_redraw();
    } else if event.logical_key == NamedKey::F6 {
      if let Some(teapot) = self.world.teapot.as_mut() {
        if let Err(e) = teapot.recompile_shader(&self.display) {
          eprintln!("Failed to recompile shader: {}", e);
        } else {
          println!("Recompiled shader");
          self.window.request_redraw();
        }
      }
    }
  }

  fn handle_redraw(&mut self) {
    self
      .world
      .render(&self.display, self.last_frame.elapsed())
      .expect("Failed to render");
    self.last_frame = std::time::Instant::now();
  }

  fn handle_user_event(
    &mut self,
    user_event: UserSignal,
    window_target: &EventLoopWindowTarget<UserSignal>,
  ) {
    match user_event {
      UserSignal::Quit => window_target.exit(),
    }
  }

  fn handle_idle(&mut self) {
    self.world.update(self.last_update.elapsed());
    self.last_update = std::time::Instant::now();

    let help =
      "Press 'p' to toggle perspective, 'a' to toggle axis, Esc to quit";

    let elapsed = self.boot_time.elapsed().as_secs_f32();
    self
      .window
      .set_title(&format!("Elapsed: {:.2}s ({help})", elapsed));

    self.window.request_redraw();
  }

  fn handle_mouse_input(
    &mut self,
    _device_id: DeviceId,
    state: winit::event::ElementState,
    button: winit::event::MouseButton,
  ) {
    let pressed = state.is_pressed();

    match button {
      winit::event::MouseButton::Left => {
        self.mouse_down.0 = pressed;
      }
      winit::event::MouseButton::Right => {
        self.mouse_down.1 = pressed;
      }
      _ => {}
    }
  }

  fn handle_cursor_moved(
    &mut self,
    _device_id: DeviceId,
    position: winit::dpi::PhysicalPosition<f64>,
  ) {
    self.mouse_pos = [position.x as f32, position.y as f32];

    // left drag: rotate camera
    if self.mouse_down.0 {
      let dx = self.mouse_pos[0] - self.last_pos[0];
      let dy = self.mouse_pos[1] - self.last_pos[1];
      self.world.camera.rotation[0] += dx * 0.1;
      self.world.camera.rotation[1] += dy * 0.1;
      self.world.update_view();
      self.window.request_redraw();
    }

    // right drag: change camera distance
    if self.mouse_down.1 {
      let dy = self.mouse_pos[1] - self.last_pos[1];

      self.world.camera.distance += dy * 0.01;
      self.world.camera.distance = self.world.camera.distance.clamp(0.1, 10.0);
      self.world.update_view();
      self.window.request_redraw();
    }

    self.last_pos = self.mouse_pos;
  }

  fn handle_mouse_wheel(
    &mut self,
    _device_id: DeviceId,
    delta: winit::event::MouseScrollDelta,
    _phase: winit::event::TouchPhase,
  ) {
    let d = match delta {
      winit::event::MouseScrollDelta::LineDelta(_x, y) => y * 20.0,
      winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
    };

    self.world.camera.distance -= d * 0.01;
    self.world.camera.distance = self.world.camera.distance.clamp(0.1, 10.0);
    self.world.update_view();
  }
}

fn main() -> Result<()> {
  let event_loop = EventLoopBuilder::with_user_event()
    .build()
    .expect("Failed to create event loop");
  let window = Window::new(&event_loop)?;
  let display = common::gl_boilerplate::init_display(&window);

  let event_loop_proxy = event_loop.create_proxy();
  let mut app = App::new(window, display, event_loop_proxy)?;

  let teapot = Teapot::load_file(
    &app.display,
    Path::new(MODEL_PATH),
    Path::new(SHADER_PATH),
  )?;
  let axis = Axis::new(&app.display)?;
  app.world.set_teapot(teapot);
  app.world.set_axis(axis);
  app.world.update(std::time::Duration::from_secs(0));

  event_loop.run(move |event, window_target| match event {
    Event::WindowEvent { window_id, event } => {
      app.handle_widow_event(window_id, event, window_target);
    }
    Event::UserEvent(user_event) => {
      app.handle_user_event(user_event, window_target);
    }
    Event::AboutToWait => {
      app.handle_idle();
    }
    _ => {}
  })?;

  Ok(())
}
