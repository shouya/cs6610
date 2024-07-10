use glium::{
  backend::glutin::SimpleWindowBuilder, glutin::surface::WindowSurface,
  Display, Surface,
};
use winit::{
  dpi::PhysicalSize,
  event::{DeviceId, Event, KeyEvent, WindowEvent},
  event_loop::{EventLoopBuilder, EventLoopProxy, EventLoopWindowTarget},
  keyboard::NamedKey,
  window::{Window, WindowId},
};

#[derive(Debug)]
enum UserSignal {
  Quit,
}

struct App {
  #[allow(unused)]
  window: Window,
  color: [f32; 4],
  boot_time: std::time::Instant,
  display: Display<WindowSurface>,
  event_loop: EventLoopProxy<UserSignal>,
}

impl App {
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
      _ => {}
    }
  }

  fn handle_resize(&mut self, size: PhysicalSize<u32>) {
    println!("Resized to {:?}", size);
  }

  fn handle_keyboard(
    &mut self,
    _device_id: DeviceId,
    event: KeyEvent,
    _is_synthetic: bool,
  ) {
    if event.logical_key == NamedKey::Escape {
      self.event_loop.send_event(UserSignal::Quit).unwrap();
    }
  }

  fn handle_redraw(&mut self) {
    let mut frame = self.display.draw();
    frame.clear_color_and_depth(self.color.into(), 0.0);
    frame.finish().unwrap();
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
    let elapsed = self.boot_time.elapsed().as_millis();
    let t = elapsed as f32 / 2000.0;
    let r = t.sin().abs();
    let g = (t * 2.0).sin().abs();
    let b = (t * 3.0).sin().abs();
    self.color = [r, g, b, 1.0];
    self.window.set_title(&format!("Elapsed: {}ms", elapsed));
    self.window.request_redraw();
  }
}

fn main() {
  let event_loop = EventLoopBuilder::with_user_event()
    .build()
    .expect("Failed to create event loop");
  let (window, display) = SimpleWindowBuilder::new()
    .with_title("Hello world!")
    .build(&event_loop);
  let event_loop_proxy = event_loop.create_proxy();
  let mut app = App {
    window,
    display,
    event_loop: event_loop_proxy,
    boot_time: std::time::Instant::now(),
    color: [0.0, 0.0, 0.0, 1.0],
  };

  event_loop
    .run(move |event, window_target| match event {
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
    })
    .unwrap();
}
