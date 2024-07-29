mod background;
mod camera;
mod light;
mod mesh;
mod object;
mod reflective_object;
mod scene;

use std::{fmt::Debug, time::Duration};

use glium::{glutin::surface::WindowSurface, Display};
use object::Teapot;
use scene::Scene;
use winit::{
  dpi::PhysicalSize,
  event::{DeviceId, Event, KeyEvent, WindowEvent},
  event_loop::{EventLoopBuilder, EventLoopProxy, EventLoopWindowTarget},
  keyboard::{ModifiersState, NamedKey},
  window::{Window, WindowId},
};

use common::{asset_path, Axis};

use crate::{camera::Camera, light::Light};

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

  fn update_view(&mut self) {
    if let Some(scene) = &mut self.scene {
      scene.camera.update_view();
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

    if let Some(scene) = &self.scene {
      scene.draw(&mut frame);

      if self.show_axis {
        if let Some(axis) = &self.axis {
          axis.draw(&mut frame, &scene.camera.view_projection())?;
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
  boot_time: std::time::Instant,
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
    if let Some(scene) = &mut self.world.scene {
      scene.camera.handle_window_resize((size.width, size.height));
    }
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
      if let Some(scene) = &mut self.world.scene {
        scene.camera.toggle_perspective();
        scene.camera.update_view();
      }
      self.window.request_redraw();
    } else if event.logical_key.to_text() == Some("a") {
      self.world.show_axis = !self.world.show_axis;
      self.window.request_redraw();
    } else if event.logical_key == NamedKey::F6 {
      if let Some(scene) = &mut self.world.scene {
        match scene.reload_shader(&self.display) {
          Ok(_) => println!("Reloaded shaders"),
          Err(e) => eprintln!("Failed to reload shader: {}", e),
        }
      }

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
    let Some(scene) = self.world.scene.as_mut() else {
      return;
    };

    let camera_target: &mut Camera = &mut scene.camera;
    let light_target: &mut Light = &mut scene.light;

    // left drag: rotate camera
    if self.mouse_down.0 && !self.modifiers.control_key() {
      let dx = self.mouse_pos[0] - self.last_pos[0];
      let dy = self.mouse_pos[1] - self.last_pos[1];
      camera_target.add_rotation([dx * 0.1, dy * 0.1]);
    }

    // ctrl + left drag: rotate light
    if self.mouse_down.0 && self.modifiers.control_key() {
      let dx = self.mouse_pos[0] - self.last_pos[0];
      let _dy = self.mouse_pos[1] - self.last_pos[1];
      light_target.add_rotation(dx * 0.01);
    }

    // right drag: change camera distance
    if self.mouse_down.1 {
      let dy = self.mouse_pos[1] - self.last_pos[1];

      camera_target.add_distance(dy * 0.01);
    }

    self.world.update_view();
    self.last_pos = self.mouse_pos;
    self.window.request_redraw();
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

    let Some(scene) = self.world.scene.as_mut() else {
      return;
    };

    let camera_target: &mut Camera = &mut scene.camera;
    camera_target.add_distance(d * 0.01);
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

  // setup axis object
  let axis = Axis::new(&app.display)?;
  app.world.set_axis(axis);

  // setup the scene
  let mut scene = Scene::new(
    &app.display,
    &[
      &asset_path("cubemap/cubemap_posx.png"),
      &asset_path("cubemap/cubemap_negx.png"),
      &asset_path("cubemap/cubemap_posy.png"),
      &asset_path("cubemap/cubemap_negy.png"),
      &asset_path("cubemap/cubemap_posz.png"),
      &asset_path("cubemap/cubemap_negz.png"),
    ],
  )?;

  let teapot1 = Teapot::load(&app.display)?
    .translated([-1.0, 0.0, 0.0])
    .rotated_y(-45.0);
  scene.add_object(teapot1, true)?;
  let teapot2 = Teapot::load(&app.display)?
    .rotated_y(45.0)
    .translated([0.5, 0.0, 0.0]);
  scene.add_object(teapot2, false)?;
  let teapot3 = Teapot::load(&app.display)?
    .translated([-0.3, 0.8, 0.0])
    .rotated_y(90.0);
  scene.add_object(teapot3, true)?;
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
