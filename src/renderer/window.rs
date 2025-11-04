use crate::gl_bindings::*;
use glfw::{self, Context, WindowEvent};

pub struct WindowContext {
	pub glfw: glfw::Glfw,
	pub window: glfw::PWindow,
	pub events: glfw::GlfwReceiver<(f64, WindowEvent)>,
}

impl WindowContext {
	pub fn new(width: u32, height: u32, title: &str) -> Self {
		let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();
		glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
		glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
		glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

		let (mut window, events) = glfw
			.create_window(width, height, title, glfw::WindowMode::Windowed)
			.expect("Failed to create GLFW window");

		window.make_current();
		glfw.set_swap_interval(glfw::SwapInterval::Sync(1));

		load_with(|symbol| window.get_proc_address(symbol) as *const _);

		// Print GL vendor/renderer/version so the user can verify which GPU is being used.
		unsafe {
			let vendor = std::ffi::CStr::from_ptr(GetString(VENDOR) as *const i8).to_string_lossy();
			let renderer = std::ffi::CStr::from_ptr(GetString(RENDERER) as *const i8).to_string_lossy();
			let version = std::ffi::CStr::from_ptr(GetString(VERSION) as *const i8).to_string_lossy();
			eprintln!("OpenGL vendor: {} | renderer: {} | version: {}", vendor, renderer, version);
		}

		Self { glfw, window, events }
	}

	pub fn poll(&mut self) {
		self.glfw.poll_events();
	}
}
