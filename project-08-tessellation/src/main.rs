mod camera;
mod light;
mod mesh;
mod object;
mod scene;
mod teapot_quad;
mod transform;

use std::{fmt::Debug, time::Duration};

use glium::{glutin::surface::WindowSurface, Display, Surface as _};
use scene::Scene;
use teapot_quad::TeapotQuad;
use winit::{
  dpi::{LogicalSize, PhysicalSize},
  event::{DeviceId, Event, KeyEvent, WindowEvent},
  event_loop::{EventLoopBuilder, EventLoopProxy, EventLoopWindowTarget},
  keyboard::{ModifiersState, NamedKey},
  platform::x11::WindowBuilderExtX11 as _,
  window::{Window, WindowBuilder, WindowId},
};

use common::Axis;

pub use camera::{Camera, Projection};
pub use light::Light;
pub use object::{Object, Teapot};
pub use transform::Transform;

type Result<T> = anyhow::Result<T>;

#[derive(Debug)]
enum UserSignal {
  Quit,
}

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

  fn render(
    &self,
    context: &Display<WindowSurface>,
    _dt: Duration,
  ) -> Result<()> {
    let mut frame = context.draw();
    frame.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);

    if let Some(scene) = &self.scene {
      scene.draw(&mut frame)?;

      if self.show_axis {
        if let Some(axis) = &self.axis {
          let vp = scene.view_projection().to_cols_array_2d().into();
          axis.draw(&mut frame, &vp)?;
        }
      }
    }

    frame.finish()?;
    Ok(())
  }
}

struct App {
  #[allow(unused)]
  window: Window,
  display: Display<WindowSurface>,
  event_loop: EventLoopProxy<UserSignal>,
  last_update: std::time::Instant,
  last_frame: std::time::Instant,

  // left, right
  mouse_down: (bool, bool),
  last_pos: [f32; 2],
  mouse_pos: [f32; 2],
  modifiers: ModifiersState,

  world: World,
}

impl App {
  fn new(
    window: Window,
    display: Display<WindowSurface>,
    event_loop: EventLoopProxy<UserSignal>,
  ) -> Result<Self> {
    let last_update = std::time::Instant::now();
    let last_frame = std::time::Instant::now();

    let world = World::new();

    Ok(Self {
      window,
      display,
      event_loop,
      last_update,
      last_frame,
      last_pos: [0.0, 0.0],
      mouse_pos: [0.0, 0.0],
      mouse_down: (false, false),
      modifiers: ModifiersState::empty(),
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
      WindowEvent::ModifiersChanged(new_modifiers) => {
        self.modifiers = new_modifiers.state();
      }
      _ => {}
    }
  }

  fn handle_resize(&mut self, size: PhysicalSize<u32>) {
    println!("Resized to {:?}", size);
    self.world.handle_resize(size.width, size.height);
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
      if let Some(scene) = &mut self.world.scene {
        scene.camera.toggle_projection();
        scene.update_view();
      }
      self.window.request_redraw();
    } else if event.logical_key.to_text() == Some("a") {
      self.world.show_axis = !self.world.show_axis;
      self.window.request_redraw();
    } else if event.logical_key == NamedKey::F6
      || event.logical_key.to_text() == Some("r")
    {
      if let Some(scene) = &mut self.world.scene {
        match scene.reload_shader(&self.display) {
          Ok(_) => println!("Reloaded shaders"),
          Err(e) => eprintln!("Failed to reload shader: {}", e),
        }
      }

      self.window.request_redraw();
    }

    if let Some(scene) = &mut self.world.scene {
      scene.handle_key(event);
      self.window.request_redraw();
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
    let new_now = std::time::Instant::now();
    let dt = self.last_update.elapsed();
    self.world.update(dt);
    self.last_update = new_now;

    let help =
      "Press 'p' to toggle perspective, 'a' to toggle axis, Esc to quit";

    self.window.set_title(&format!(
      "ups: {:.2} ({})",
      1.0 / dt.as_secs_f32(),
      help
    ));

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
    let Some(scene) = self.world.scene.as_mut() else {
      return;
    };

    self.mouse_pos = [position.x as f32, position.y as f32];
    let dx = self.mouse_pos[0] - self.last_pos[0];
    let dy = self.mouse_pos[1] - self.last_pos[1];
    self.last_pos = self.mouse_pos;

    if self.mouse_down.0 {
      scene.handle_drag(dx, dy, self.modifiers);
      self.window.request_redraw();
    }
  }

  fn handle_mouse_wheel(
    &mut self,
    _device_id: DeviceId,
    delta: winit::event::MouseScrollDelta,
    _phase: winit::event::TouchPhase,
  ) {
    let (dx, dy) = match delta {
      winit::event::MouseScrollDelta::LineDelta(x, y) => (x * 30.0, y * 30.0),
      winit::event::MouseScrollDelta::PixelDelta(pos) => {
        (pos.x as f32, pos.y as f32)
      }
    };

    let Some(scene) = self.world.scene.as_mut() else {
      return;
    };

    scene.handle_scroll(dx, dy);
  }
}

fn main() -> Result<()> {
  let event_loop = EventLoopBuilder::with_user_event()
    .build()
    .expect("Failed to create event loop");
  let window = WindowBuilder::new()
    .with_name("cs5610", env!("CARGO_BIN_NAME"))
    .with_inner_size(LogicalSize::new(800, 600))
    .build(&event_loop)?;
  let display = common::gl_boilerplate::init_display(&window);

  let event_loop_proxy = event_loop.create_proxy();
  let mut app = App::new(window, display, event_loop_proxy)?;

  // setup axis object
  let axis = Axis::new(&app.display)?;
  app.world.set_axis(axis);

  // setup the scene
  let mut scene = Scene::new(&app.display)?;
  scene.set_quad(TeapotQuad::new(&app.display)?);

  app.world.set_scene(scene);

  // initial update
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
