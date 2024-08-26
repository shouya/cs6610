use std::time::Duration;

use common::gl_boilerplate::init_display;
use glium::{glutin::surface::WindowSurface, Display, Surface as _};
use winit::{
  application::ApplicationHandler,
  event::{StartCause, WindowEvent},
  event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
  keyboard::NamedKey,
  window::{Window, WindowAttributes, WindowId},
};

const TARGET_UPS: u32 = 60;
const TARGET_FRAME_TIME: Duration =
  Duration::from_micros(1_000_000 / TARGET_UPS as u64);

struct App {
  window: Option<Window>,
  color: [f32; 4],
  boot_time: std::time::Instant,
  last_update: std::time::Instant,
  display: Option<Display<WindowSurface>>,
}

impl App {
  fn new() -> Self {
    Self {
      window: None,
      color: [0.0, 0.0, 0.0, 1.0],
      boot_time: std::time::Instant::now(),
      last_update: std::time::Instant::now(),
      display: None,
    }
  }
}

impl ApplicationHandler for App {
  fn resumed(&mut self, event_loop: &ActiveEventLoop) {
    if self.window.is_none() {
      let window_attrs = WindowAttributes::default().with_title("Hello world!");

      match event_loop.create_window(window_attrs) {
        Ok(window) => {
          let display = init_display(&window);
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
    let wake_up_at = self.last_update + TARGET_FRAME_TIME;
    event_loop.set_control_flow(ControlFlow::WaitUntil(wake_up_at));

    match event {
      WindowEvent::CloseRequested => {
        event_loop.exit();
      }
      WindowEvent::Resized(size) => {
        println!("Resized: {:?}", size);
      }
      WindowEvent::KeyboardInput { event, .. } => {
        if event.logical_key == NamedKey::Escape {
          event_loop.exit();
        }
      }
      WindowEvent::RedrawRequested => {
        self.handle_redraw();
        // self.handle_idle();
      }
      _ => {}
    }
  }

  fn new_events(&mut self, _event_loop: &ActiveEventLoop, _cause: StartCause) {
    if self.last_update.elapsed() > TARGET_FRAME_TIME {
      self.update();
    }
  }
}

impl App {
  fn handle_redraw(&mut self) {
    if let Some(display) = &self.display {
      let mut frame = display.draw();
      frame.clear_color_and_depth(self.color.into(), 0.0);
      frame.finish().unwrap();
    }
  }

  fn update(&mut self) {
    let update_dt = self.last_update.elapsed();
    self.last_update = std::time::Instant::now();

    let elapsed = self.boot_time.elapsed().as_millis();
    let t = elapsed as f32 / 2000.0;
    let r = t.sin().abs();
    let g = (t * 2.0).sin().abs();
    let b = (t * 3.0).sin().abs();
    self.color = [r, g, b, 1.0];

    if let Some(window) = &self.window {
      window.set_title(&format!(
        "Update time: {:.02}ms",
        update_dt.as_secs_f32() * 1000.0
      ));
      window.request_redraw();
    }
  }
}

fn main() {
  let event_loop = EventLoop::new().unwrap();
  event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);

  let mut app = App::new();
  app.update();
  event_loop.run_app(&mut app).expect("Failed to run app");
}
