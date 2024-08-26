use std::{mem::size_of, path::Path, time::Duration};

use glam::{EulerRot, Mat3, Mat4, Vec3};
use glium::{
  backend::Facade, glutin::surface::WindowSurface, program::SourceCode,
  uniform, Display, DrawParameters, Frame, Program, Surface, VertexBuffer,
};

use common::{gl_boilerplate::init_display, Axis, SimpleObj};
use winit::{
  application::ApplicationHandler,
  dpi::{LogicalSize, PhysicalPosition, PhysicalSize},
  event::{self, KeyEvent, WindowEvent},
  event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
  keyboard::NamedKey,
  window::{Window, WindowAttributes, WindowId},
};

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

const SHADER_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/shader");

const TARGET_UPS: u32 = 60;
const TARGET_FRAME_TIME: Duration =
  Duration::from_micros(1_000_000 / TARGET_UPS as u64);

struct World {
  t: f32,
  clear_color: [f32; 4],
  // camera
  aspect_ratio: f32,
  camera_distance: f32,
  camera_rotation: [f32; 2],
  perspective: bool,
  // world space to camera space
  m_view: Mat4,
  // view space to clip space
  m_proj: Mat4,
  axis: Option<Axis>,
  show_axis: bool,
  teapot: Option<Teapot>,
}

struct Teapot {
  rotation: f32,
  rotation_speed: f32,
  model_vbo: VertexBuffer<[f32; 3]>,
  program: Program,
  center: Vec3,
  mvp: Mat4,
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
      m_view: Mat4::IDENTITY,
      m_proj: Mat4::IDENTITY,
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
    let dir = Mat3::from_euler(
      EulerRot::YXZ,
      self.camera_rotation[0].to_radians(),
      self.camera_rotation[1].to_radians(),
      0.0,
    ) * Vec3::new(0.0, 0.0, -1.0);
    let eye = Vec3::new(0.0, 0.0, 0.0) + -dir * self.camera_distance;
    let up = Vec3::new(0.0, 1.0, 0.0);
    let origin = Vec3::new(0.0, 0.0, 0.0);
    self.m_view = Mat4::look_at_rh(eye, origin, up);
  }

  fn update(&mut self, dt: Duration) {
    // self.update_bg_color(dt);
    self.rotate_teapot(dt);
  }

  fn update_projection(&mut self) {
    // fov, aspect ratio, near, far
    self.m_proj = if self.perspective {
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
      ) * (1.0 / self.camera_distance)
    };
  }

  fn render(&self, context: &Display<WindowSurface>) -> Result<()> {
    let mut frame = context.draw();
    frame.clear_color_and_depth(self.clear_color.into(), 1.0);

    if let Some(teapot) = self.teapot.as_ref() {
      if let Err(e) = teapot.draw(&mut frame) {
        eprintln!("Failed to draw teapot: {}", e);
      }
    }

    if let Some(axis) = self.axis.as_ref() {
      if self.show_axis {
        let m_view_proj = self.m_proj * self.m_view;
        if let Err(e) = axis.draw(&mut frame, &m_view_proj) {
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

    let m_model = Mat4::from_scale(Vec3::splat(0.05))
    // the object itself is rotated 90 to the front, let's rotate it back a little.
      * Mat4::from_euler(
        EulerRot::YXZ,
        teapot.rotation,
        (-90f32).to_radians(),
        0.0,
      )
      * Mat4::from_translation(-teapot.center);
    teapot.mvp = self.m_proj * self.m_view * m_model;
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
    let model = SimpleObj::load_from(&model_path)?;
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
    model: &SimpleObj,
    program: Program,
  ) -> Result<Self> {
    eprintln!("Loaded model with {} vertices", model.v.len());
    let model_vbo = unsafe {
      VertexBuffer::new_raw(context, &model.v, VF_F32x3, size_of::<[f32; 3]>())?
    };
    let mvp = Mat4::IDENTITY;
    let center = Vec3::from(model.center());

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
    let mvp: [[f32; 4]; 4] = self.mvp.to_cols_array_2d();
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

struct App {
  window: Option<Window>,
  display: Option<Display<WindowSurface>>,
  last_update: std::time::Instant,

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
  fn new() -> Self {
    let last_update = std::time::Instant::now();

    let world = World::new();

    Self {
      window: None,
      display: None,
      last_update,
      last_pos: [0.0, 0.0],
      mouse_pos: [0.0, 0.0],
      mouse_down: (false, false),
      world,
    }
  }

  fn handle_resize(&mut self, size: PhysicalSize<u32>) {
    println!("Resized to {:?}", size);
    self.world.aspect_ratio = size.width as f32 / size.height as f32;
    self.world.update_projection();
    self.world.update_view();

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
      self.world.perspective = !self.world.perspective;
      self.world.update_projection();
      self.request_redraw();
    } else if event.logical_key.to_text() == Some("a") {
      self.world.show_axis = !self.world.show_axis;
      self.request_redraw();
    } else if event.logical_key == NamedKey::F6 {
      self.recompile_shader();
    }
  }

  fn handle_redraw(&mut self) {
    if let Some(display) = &self.display {
      self.world.render(display).expect("Failed to render");
    }
  }

  fn recompile_shader(&mut self) {
    if let Some(display) = &self.display {
      if let Some(teapot) = self.world.teapot.as_mut() {
        if let Err(e) = teapot.recompile_shader(display) {
          eprintln!("Failed to recompile shader: {}", e);
        } else {
          println!("Recompiled shader");
          self.request_redraw();
        }
      }
    }
  }

  fn update(&mut self) {
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

  fn handle_cursor_moved(&mut self, position: PhysicalPosition<f64>) {
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
    self.request_redraw();
  }

  fn handle_mouse_wheel(&mut self, delta: event::MouseScrollDelta) {
    let d = match delta {
      event::MouseScrollDelta::LineDelta(_x, y) => y * 20.0,
      event::MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
    };

    self.world.camera_distance -= d * 0.01;
    self.world.camera_distance = self.world.camera_distance.clamp(0.1, 10.0);
    self.world.update_projection();
    self.world.update_view();
  }

  fn schedule_next_frame(&self, event_loop: &ActiveEventLoop) {
    let wake_up_at = self.last_update + TARGET_FRAME_TIME;
    event_loop.set_control_flow(ControlFlow::WaitUntil(wake_up_at));
  }

  fn handle_init(&mut self, display: &Display<WindowSurface>) -> Result<()> {
    let teapot = Teapot::load_file(
      display,
      &common::teapot_path(),
      Path::new(SHADER_PATH),
    )?;
    let axis = Axis::new(display)?;
    self.world.set_teapot(teapot);
    self.world.set_axis(axis);
    self.world.update(std::time::Duration::from_secs(0));
    Ok(())
  }
}

impl ApplicationHandler for App {
  fn resumed(&mut self, event_loop: &ActiveEventLoop) {
    if self.window.is_none() {
      let window_attrs = WindowAttributes::default()
        .with_title("Teapot")
        .with_inner_size(LogicalSize::new(800, 600));

      match event_loop.create_window(window_attrs) {
        Ok(window) => {
          let display = init_display(&window);
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
  let mut app = App::new();

  event_loop.run_app(&mut app)?;
  Ok(())
}
