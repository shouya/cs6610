mod camera;
mod light;
mod mesh;
mod object;
mod scene;
mod teapot_quad;
mod transform;

use std::time::Duration;

use glium::{glutin::surface::WindowSurface, Display, Surface as _};
use scene::Scene;
use teapot_quad::TeapotQuad;
use winit::{
  application::ApplicationHandler,
  dpi::{LogicalSize, PhysicalPosition, PhysicalSize},
  event::{self, KeyEvent, WindowEvent},
  event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
  keyboard::{ModifiersState, NamedKey},
  window::{Window, WindowAttributes, WindowId},
};

use common::Axis;

pub use camera::{Camera, Projection};
pub use light::Light;
pub use object::{Object, Teapot};
pub use transform::Transform;

type Result<T> = anyhow::Result<T>;

const TARGET_UPS: u32 = 60;
const TARGET_FRAME_TIME: Duration =
  Duration::from_micros(1_000_000 / TARGET_UPS as u64);

struct World {
  t: f32,
  show_axis: bool,
  axis: Option<Axis>,
  scene: Option<Scene>,
}

impl World {
  fn new() -> Self {
    Self {
      t: 0.0,
      axis: None,
      show_axis: true,
      scene: None,
    }
  }

  fn set_axis(&mut self, axis: Axis) {
    self.axis = Some(axis);
  }

  fn set_scene(&mut self, scene: Scene) {
    self.scene = Some(scene);
  }

  fn handle_resize(&mut self, width: u32, height: u32) {
    if let Some(scene) = &mut self.scene {
      scene.handle_resize(width as f32, height as f32);
    }

    self.update_view();
  }

  fn update_view(&mut self) {
    if let Some(scene) = &mut self.scene {
      scene.update_view();
    }
  }

  fn update(&mut self, dt: Duration) {
    self.t += dt.as_secs_f32();

    if let Some(scene) = &mut self.scene {
      scene.update(&dt);
    }
  }

  fn render(&self, context: &Display<WindowSurface>) -> Result<()> {
    let mut frame = context.draw();
    frame.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);

    if let Some(scene) = &self.scene {
      scene.draw(&mut frame)?;

      if self.show_axis {
        if let Some(axis) = &self.axis {
          let vp = scene.view_projection();
          axis.draw(&mut frame, &vp)?;
        }
      }
    }

    frame.finish()?;
    Ok(())
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

  world: World,
}

impl App {
  fn new() -> Result<Self> {
    let last_update = std::time::Instant::now();

    let world = World::new();

    Ok(Self {
      window: None,
      display: None,
      last_update,
      last_pos: [0.0, 0.0],
      mouse_pos: [0.0, 0.0],
      mouse_down: (false, false),
      modifiers: ModifiersState::empty(),
      world,
    })
  }

  fn handle_resize(&mut self, size: PhysicalSize<u32>) {
    println!("Resized to {:?}", size);
    self.world.handle_resize(size.width, size.height);
    self.request_redraw();
  }

  fn request_redraw(&self) {
    if let Some(window) = self.window.as_ref() {
      window.request_redraw();
    }
  }

  fn handle_keyboard(&mut self, event: KeyEvent, event_loop: &ActiveEventLoop) {
    if !event.state.is_pressed() {
      return;
    }

    if event.logical_key == NamedKey::Escape {
      event_loop.exit();
    } else if event.logical_key.to_text() == Some("p") {
      if let Some(scene) = &mut self.world.scene {
        scene.camera.toggle_projection();
        scene.update_view();
      }
      self.request_redraw();
    } else if event.logical_key.to_text() == Some("a") {
      self.world.show_axis = !self.world.show_axis;
      self.request_redraw();
    } else if event.logical_key == NamedKey::F6
      || event.logical_key.to_text() == Some("r")
    {
      self.reload_shader();
      self.request_redraw();
    }

    if let Some(scene) = &mut self.world.scene {
      scene.handle_key(event);
      self.request_redraw();
    }
  }

  fn reload_shader(&mut self) {
    let Some(display) = &self.display else {
      eprintln!("display not ready");
      return;
    };

    let Some(scene) = &mut self.world.scene else {
      eprintln!("scene not loaded");
      return;
    };

    match scene.reload_shader(display) {
      Ok(_) => println!("Reloaded shaders"),
      Err(e) => eprintln!("Failed to reload shader: {}", e),
    }
  }

  fn handle_redraw(&mut self) {
    let Some(display) = &self.display else {
      eprintln!("display not ready");
      return;
    };

    self.world.render(display).expect("Failed to render");
  }

  fn update(&mut self) {
    let dt = self.last_update.elapsed();
    self.world.update(dt);
    self.last_update = std::time::Instant::now();

    let help =
      "Press 'p' to toggle perspective, 'a' to toggle axis, Esc to quit";

    if let Some(window) = &self.window {
      window.set_title(&format!(
        "ups: {:.2} ({})",
        1.0 / dt.as_secs_f32(),
        help
      ));
    }

    self.request_redraw();
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
    let Some(scene) = self.world.scene.as_mut() else {
      return;
    };

    self.mouse_pos = [position.x as f32, position.y as f32];
    let dx = self.mouse_pos[0] - self.last_pos[0];
    let dy = self.mouse_pos[1] - self.last_pos[1];
    self.last_pos = self.mouse_pos;

    if self.mouse_down.0 {
      scene.handle_drag(dx, dy, self.modifiers);
      self.request_redraw();
    }
  }

  fn handle_mouse_wheel(&mut self, delta: event::MouseScrollDelta) {
    let (dx, dy) = match delta {
      event::MouseScrollDelta::LineDelta(x, y) => (x * 30.0, y * 30.0),
      event::MouseScrollDelta::PixelDelta(pos) => (pos.x as f32, pos.y as f32),
    };

    let Some(scene) = self.world.scene.as_mut() else {
      return;
    };

    scene.handle_scroll(dx, dy);
  }

  fn schedule_next_frame(&self, event_loop: &ActiveEventLoop) {
    let wake_up_at = self.last_update + TARGET_FRAME_TIME;
    event_loop.set_control_flow(ControlFlow::WaitUntil(wake_up_at));
  }

  fn handle_init(&mut self, display: &Display<WindowSurface>) -> Result<()> {
    // setup axis object
    let axis = Axis::new(display)?;
    self.world.set_axis(axis);

    // setup the scene
    let mut scene = Scene::new(display)?;
    scene.set_quad(TeapotQuad::new(display)?);

    self.world.set_scene(scene);

    // initial update
    self.world.update(Duration::from_secs(0));

    Ok(())
  }
}

impl ApplicationHandler for App {
  fn resumed(&mut self, event_loop: &ActiveEventLoop) {
    if self.window.is_none() {
      let window_attrs = WindowAttributes::default()
        .with_title("cs5610")
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
  let event_loop = EventLoop::new().expect("Failed to create event loop");
  let mut app = App::new()?;

  event_loop.run_app(&mut app)?;
  Ok(())
}
