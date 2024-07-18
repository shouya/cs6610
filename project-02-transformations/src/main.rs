mod obj_loader;

use std::{fmt::Debug, mem::size_of, path::Path, time::Duration};

use glium::{
  backend::Facade, glutin::surface::WindowSurface, program::SourceCode,
  uniform, Display, DrawParameters, Frame, Program, Surface, VertexBuffer,
};

use winit::{
  dpi::PhysicalSize,
  event::{DeviceId, Event, KeyEvent, WindowEvent},
  event_loop::{EventLoopBuilder, EventLoopProxy, EventLoopWindowTarget},
  keyboard::NamedKey,
  window::{Window, WindowId},
};

use cgmath::{Deg, Euler, Matrix3, Matrix4, Point3, SquareMatrix, Vector3};

use common::RawObj;

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

const SHADER_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/shader");

#[derive(Debug)]
enum UserSignal {
  Quit,
}

struct World {
  t: f32,
  clear_color: [f32; 4],
  // camera
  aspect_ratio: f32,
  camera_distance: f32,
  camera_rotation: [f32; 2],
  perspective: bool,
  // world space to camera space
  m_view: Matrix4<f32>,
  // view space to clip space
  m_proj: Matrix4<f32>,
  axis: Option<Axis>,
  show_axis: bool,
  teapot: Option<Teapot>,
}

struct Teapot {
  rotation: f32,
  rotation_speed: f32,
  model_vbo: VertexBuffer<[f32; 3]>,
  program: Program,
  center: Vector3<f32>,
  mvp: Matrix4<f32>,
}

struct Axis {
  model_vbo: VertexBuffer<[f32; 3]>,
  program: Program,
  mvp: Matrix4<f32>,
}

impl World {
  fn new() -> Self {
    Self {
      t: 0.0,
      aspect_ratio: 1.0,
      camera_distance: 2.0,
      camera_rotation: [0.0, 0.0],
      perspective: true,
      clear_color: [0.0, 0.0, 0.0, 1.0],
      m_view: Matrix4::identity(),
      m_proj: Matrix4::identity(),
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
    // default view matrix: eye at (0, 0, 2), looking at (0, 0, -1), up (0, 1, 0)
    let dir = Matrix3::from_angle_y(Deg(self.camera_rotation[0]))
      * Matrix3::from_angle_x(-Deg(self.camera_rotation[1]))
      * Vector3::new(0.0, 0.0, -1.0);
    let eye = Point3::new(0.0, 0.0, 0.0) + -dir * self.camera_distance;
    let up = Vector3::new(0.0, 1.0, 0.0);
    let origin = Point3::new(0.0, 0.0, 0.0);
    self.m_view = Matrix4::look_at_rh(eye, origin, up);

    self.update_axis();
  }

  fn update(&mut self, dt: Duration) {
    // self.update_bg_color(dt);
    self.rotate_teapot(dt);
  }

  fn update_projection(&mut self) {
    // fov, aspect ratio, near, far
    self.m_proj = if self.perspective {
      cgmath::perspective(cgmath::Deg(60.0), self.aspect_ratio, 0.1, 100.0)
    } else {
      cgmath::Matrix4::from_scale(1.0 / self.camera_distance)
        * cgmath::ortho(
          -1.0,
          1.0,
          -1.0 / self.aspect_ratio,
          1.0 / self.aspect_ratio,
          0.1,
          100.0,
        )
    };

    self.update_axis();
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

    if let Some(axis) = self.axis.as_ref() {
      if self.show_axis {
        if let Err(e) = axis.draw(&mut frame) {
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
    self.clear_color = [r, g, b, 1.0];
  }

  fn rotate_teapot(&mut self, dt: Duration) {
    let Some(teapot) = self.teapot.as_mut() else {
      return;
    };

    teapot.rotation += dt.as_secs_f32() * teapot.rotation_speed;
    let m_model = Matrix4::from_scale(0.05)
      * Matrix4::from_angle_y(cgmath::Rad(teapot.rotation))
    // the object itself is rotated 90 to the front, let's rotate it back a little.
      * Matrix4::from_angle_x(cgmath::Deg(-90.0))
      * Matrix4::from_translation(-teapot.center);
    teapot.mvp = self.m_proj * self.m_view * m_model;
  }

  fn update_axis(&mut self) {
    if let Some(axis) = self.axis.as_mut() {
      axis.mvp = self.m_proj * self.m_view;
    }
  }
}

impl Teapot {
  fn recompile_shader<F: Facade>(&mut self, context: &F) -> Result<()> {
    let shaders_path = Path::new(SHADER_PATH);
    let vert_shader_path = shaders_path.with_extension("vert");
    let frag_shader_path = shaders_path.with_extension("frag");

    self.program = Program::new(
      context,
      SourceCode {
        vertex_shader: &std::fs::read_to_string(vert_shader_path)?,
        fragment_shader: &std::fs::read_to_string(frag_shader_path)?,
        tessellation_control_shader: None,
        tessellation_evaluation_shader: None,
        geometry_shader: None,
      },
    )?;

    Ok(())
  }

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
    eprintln!("Loaded model with {} vertices", model.v.len());
    let model_vbo = unsafe {
      VertexBuffer::new_raw(context, &model.v, VF_F32x3, size_of::<[f32; 3]>())?
    };
    let mvp = Matrix4::<f32>::identity();
    let center = Vector3::from(model.center());

    Ok(Self {
      rotation: 0.0,
      rotation_speed: 1.0,
      model_vbo,
      mvp,
      program,
      center,
    })
  }

  fn draw(&self, frame: &mut Frame) -> Result<()> {
    let mvp: [[f32; 4]; 4] = self.mvp.into();
    let uniforms = uniform! {
      mvp: mvp,
      clr: [1.0, 0.0, 1.0f32],
    };

    let draw_params = DrawParameters {
      point_size: Some(2.0),
      ..Default::default()
    };

    frame.draw(
      &self.model_vbo,
      glium::index::NoIndices(glium::index::PrimitiveType::Points),
      &self.program,
      &uniforms,
      &draw_params,
    )?;

    Ok(())
  }
}

impl Axis {
  fn load_file<F: Facade>(context: &F, shaders_path: &Path) -> Result<Self> {
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
    Self::new(context, program)
  }

  fn new<F: Facade>(context: &F, program: Program) -> Result<Self> {
    let verts = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
    let model_vbo = unsafe {
      VertexBuffer::new_raw(context, &verts, VF_F32x3, size_of::<[f32; 3]>())?
    };
    let mvp = Matrix4::<f32>::identity();

    Ok(Self {
      model_vbo,
      mvp,
      program,
    })
  }

  fn draw_single(
    &self,
    frame: &mut Frame,
    rot: [f32; 3],
    scale: f32,
    color: [f32; 3],
  ) -> Result<()> {
    let trans = Matrix4::from_scale(scale)
      * Matrix4::from(Euler {
        x: Deg(rot[0]),
        y: Deg(rot[1]),
        z: Deg(rot[2]),
      });
    let mvp: [[f32; 4]; 4] = (self.mvp * trans).into();
    let uniforms = uniform! {
      mvp: mvp,
      clr: color,
    };

    let draw_params = DrawParameters {
      point_size: Some(10.0),
      line_width: Some(3.0),
      ..Default::default()
    };

    frame.draw(
      &self.model_vbo,
      glium::index::NoIndices(glium::index::PrimitiveType::LineStrip),
      &self.program,
      &uniforms,
      &draw_params,
    )?;

    frame.draw(
      &self.model_vbo,
      glium::index::NoIndices(glium::index::PrimitiveType::Points),
      &self.program,
      &uniforms,
      &draw_params,
    )?;

    Ok(())
  }

  fn draw(&self, frame: &mut Frame) -> Result<()> {
    self.draw_single(frame, [0.0, 0.0, 0.0], 1.0, [1.0, 0.0, 0.0])?;
    self.draw_single(frame, [0.0, 0.0, 0.0], -1.0, [0.8, 0.0, 0.0])?;
    self.draw_single(frame, [0.0, 0.0, 90.0], 1.0, [0.0, 1.0, 0.0])?;
    self.draw_single(frame, [0.0, 0.0, 90.0], -1.0, [0.0, 0.8, 0.0])?;
    self.draw_single(frame, [0.0, 90.0, 0.0], 1.0, [0.0, 0.0, 1.0])?;
    self.draw_single(frame, [0.0, 90.0, 0.0], -1.0, [0.0, 0.0, 0.8])?;
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

  world: World,
}

#[allow(non_upper_case_globals)]
const VF_F32x3: glium::vertex::VertexFormat = &[(
  // attribute name
  std::borrow::Cow::Borrowed("pos"),
  // byte offset
  0,
  // this field was undocumented, maybe stride?
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
    self.world.aspect_ratio = size.width as f32 / size.height as f32;
    self.world.update_projection();
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
      self.world.perspective = !self.world.perspective;
      self.world.update_projection();
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
      self.world.camera_rotation[0] += dx * 0.1;
      self.world.camera_rotation[1] += dy * 0.1;
      self.world.update_view();
    }

    // right drag: change camera distance
    if self.mouse_down.1 {
      let dy = self.mouse_pos[1] - self.last_pos[1];

      self.world.camera_distance += dy * 0.01;
      self.world.camera_distance = self.world.camera_distance.clamp(0.1, 10.0);
      self.world.update_projection();
      self.world.update_view();
    }

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

    self.world.camera_distance -= d * 0.01;
    self.world.camera_distance = self.world.camera_distance.clamp(0.1, 10.0);
    self.world.update_projection();
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
    Path::new(common::teapot_path()),
    Path::new(SHADER_PATH),
  )?;
  let axis = Axis::load_file(&app.display, Path::new(SHADER_PATH))?;
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
