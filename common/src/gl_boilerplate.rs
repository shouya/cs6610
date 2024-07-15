use glium::{glutin::surface::WindowSurface, Display};
use winit::window::Window;

// I would use glutin::SimpleWindowBuilder but it has no way to turn
// on debug with the public API.
pub fn init_display(window: &Window) -> Display<WindowSurface> {
  use glium::{
    debug::DebugCallbackBehavior,
    glutin::{
      context::NotCurrentGlContext,
      display::{DisplayApiPreference, GlDisplay as _},
      surface::SurfaceAttributesBuilder,
    },
  };
  use raw_window_handle::{HasRawDisplayHandle as _, HasRawWindowHandle as _};

  let display_handle = window.raw_display_handle();
  let window_handle = window.raw_window_handle();

  let disp = unsafe {
    glium::glutin::display::Display::new(
      display_handle,
      DisplayApiPreference::Egl,
    )
    .expect("Failed to create display")
  };

  eprintln!("GL Version: {}", disp.version_string());

  let config = unsafe {
    disp
      .find_configs(Default::default())
      .expect("Failed to find configs")
      .next()
      .expect("No config found")
  };
  let context = unsafe {
    disp
      .create_context(&config, &Default::default())
      .expect("Failed to create context")
  };
  let surface_attr = SurfaceAttributesBuilder::<WindowSurface>::new().build(
    window_handle,
    window.inner_size().width.try_into().unwrap(),
    window.inner_size().height.try_into().unwrap(),
  );
  let surface = unsafe {
    disp
      .create_window_surface(&config, &surface_attr)
      .expect("Failed to create surface")
  };
  let context = context
    .make_current(&surface)
    .expect("Failed to make context current");

  Display::with_debug(
    context,
    surface,
    DebugCallbackBehavior::DebugMessageOnError,
  )
  .expect("Failed to create display")
}
