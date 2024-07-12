mod obj_loader;

use std::{mem::size_of, path::Path, time::Duration};

use glium::{
  backend::{glutin::SimpleWindowBuilder, Facade},
  glutin::surface::WindowSurface,
  program::SourceCode,
  uniform, Display, Frame, Program, Surface, VertexBuffer,
};
use winit::{
  dpi::PhysicalSize,
  event::{DeviceId, Event, KeyEvent, WindowEvent},
  event_loop::{EventLoopBuilder, EventLoopProxy, EventLoopWindowTarget},
  keyboard::NamedKey,
  window::{Window, WindowId},
};

use cgmath::{Matrix4, SquareMatrix};

use obj_loader::RawObj;

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
enum UserSignal {
  Quit,
}

struct World {
  t: f32,
  clear_color: [f32; 4],
  m_view: Matrix4<f32>,
  m_proj: Matrix4<f32>,
  teapot: Option<Teapot>,
}

impl World {
  fn new() -> Self {
    Self {
      t: 0.0,
      clear_color: [0.0, 0.0, 0.0, 1.0],
      m_view: Matrix4::identity(),
      m_proj: Matrix4::identity(),
      teapot: None,
    }
  }

  fn set_teapot(&mut self, teapot: Teapot) {
    self.teapot = Some(teapot);
  }
}

struct Teapot {
  rotation: f32,
  rotation_speed: f32,
  model_vbo: VertexBuffer<[f32; 3]>,
  program: Program,
  mvp: Matrix4<f32>,
}

impl World {
  fn update(&mut self, dt: Duration) {
    self.update_bg_color(dt);
    self.rotate_teapot(dt);
  }

  fn render(
    &self,
    context: &Display<WindowSurface>,
    _dt: Duration,
  ) -> Result<()> {
    let mut frame = context.draw();
    frame.clear_color_and_depth(self.clear_color.into(), 1.0);

    if let Some(teapot) = self.teapot.as_ref() {
      if let Err(e) = teapot.draw(&mut frame) {
        eprintln!("Failed to draw teapot: {}", e);
      }
    }

    frame.finish()?;
    Ok(())
  }

  fn update_bg_color(&mut self, dt: Duration) {
    self.t += dt.as_secs_f32();
    let t = self.t;
    let r = t.sin().abs();
    let g = (t * 2.0).sin().abs();
    let b = (t * 3.0).sin().abs();
    self.clear_color = [r, g, b, 1.0];
  }

  fn rotate_teapot(&mut self, dt: Duration) {
    let Some(teapot) = self.teapot.as_mut() else {
      return;
    };

    teapot.rotation += dt.as_secs_f32() * teapot.rotation_speed;
    let m_model = Matrix4::from_angle_y(cgmath::Rad(teapot.rotation));
    teapot.mvp = self.m_proj * self.m_view * m_model;
  }
}

impl Teapot {
  fn load_file<F: Facade>(
    context: &F,
    model_path: &Path,
    shaders_path: &Path,
  ) -> Result<Self> {
    let model = RawObj::load_from(model_path)?;
    let vert_shader_path = shaders_path.with_extension("vert");
    let frag_shader_path = shaders_path.with_extension("frag");

    let source_code = SourceCode {
      vertex_shader: &std::fs::read_to_string(vert_shader_path)?,
      fragment_shader: &std::fs::read_to_string(frag_shader_path)?,
      tessellation_control_shader: None,
      tessellation_evaluation_shader: None,
      geometry_shader: None,
    };
    let program = Program::new(context, source_code)?;
    Self::new(context, &model, program)
  }

  fn new<F: Facade>(
    context: &F,
    model: &RawObj,
    program: Program,
  ) -> Result<Self> {
    let model_vbo = unsafe {
      VertexBuffer::new_raw(context, &model.v, VF_F32x3, size_of::<[f32; 3]>())?
    };
    let mvp = Matrix4::<f32>::identity();

    Ok(Self {
      rotation: 0.0,
      rotation_speed: 1.0,
      model_vbo,
      mvp,
      program,
    })
  }

  fn draw(&self, frame: &mut Frame) -> Result<()> {
    let mpv: [[f32; 4]; 4] = self.mvp.into();
    let uniforms = uniform! {
      mvp: mpv
    };

    frame.draw(
      &self.model_vbo,
      glium::index::NoIndices(glium::index::PrimitiveType::Points),
      &self.program,
      &uniforms,
      &Default::default(),
    )?;

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

  world: World,
}

#[allow(non_upper_case_globals)]
const VF_F32x3: glium::vertex::VertexFormat = &[(
  // attribute name
  std::borrow::Cow::Borrowed("pos"),
  // byte offset
  0,
  // what's this?
  0,
  // attribute type (F32F32F32)
  glium::vertex::AttributeType::F32F32F32,
  // does it need to be normalized?
  false,
)];

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
    self
      .world
      .render(&self.display, self.last_frame.elapsed())
      .expect("Failed to render");
    self.last_frame = std::time::Instant::now();

    self.window.request_redraw();
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

    let elapsed = self.boot_time.elapsed().as_millis();
    self.window.set_title(&format!("Elapsed: {}ms", elapsed));

    self.window.request_redraw();
  }
}

fn main() -> Result<()> {
  let event_loop = EventLoopBuilder::with_user_event()
    .build()
    .expect("Failed to create event loop");
  let (window, display) = SimpleWindowBuilder::new()
    .with_title("Hello world!")
    .build(&event_loop);
  let event_loop_proxy = event_loop.create_proxy();
  let mut app = App::new(window, display, event_loop_proxy)?;

  let teapot = Teapot::load_file(
    &app.display,
    Path::new("project-02-transformations/assets/teapot.obj"),
    Path::new("project-02-transformations/assets/shader"),
  )?;
  app.world.set_teapot(teapot);
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
