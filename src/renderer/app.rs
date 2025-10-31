use crate::{camera::{Camera, CameraMode}, renderer::{window::WindowContext, mesh::create_fullscreen_quad, utils::get_uniform}, shader::create_shader_program};
use crate::gl_bindings::*;
use glfw::{self,Context, Action, Key};

const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;
const TITLE: &str = "Black Hole Renderer";

pub struct App {
	pub window_ctx: WindowContext,
	pub camera: Camera,
	pub vao: u32,
	pub render_disk: bool,
	pub gravitational_lensing: bool,
	pub fov: f32,
	pub passive_tracking: bool,
	pub shader: u32
}

impl App {
	pub fn new() -> Self {
		let mut window_ctx = WindowContext::new(WIDTH, HEIGHT, TITLE);

		window_ctx.window.set_key_polling(true);
		window_ctx.window.set_mouse_button_polling(true);
		window_ctx.window.set_cursor_pos_polling(true);
		window_ctx.window.set_scroll_polling(true);
		window_ctx.window.set_framebuffer_size_polling(true);

		let camera = Camera::new();
		let vao = create_fullscreen_quad();

		Self {
			window_ctx,
			camera,
			vao,
			render_disk: true,
			gravitational_lensing: true,
			fov: 60.0,
			passive_tracking: false,
			shader: create_shader_program("shaders/blackhole.vert", "shaders/blackhole.frag").unwrap()
		}
	}

	pub fn run(&mut self) {
		let start_time = std::time::Instant::now();

		unsafe {
			let (fb_width, fb_height) = self.window_ctx.window.get_framebuffer_size();
			Viewport(0, 0, fb_width, fb_height);
			ClearColor(0.0, 0.0, 0.0, 1.0);
		}

		self.manual();

		while !self.window_ctx.window.should_close() {
			let current_time = self.window_ctx.glfw.get_time();
			self.camera.update(current_time);

			self.window_ctx.poll();

			let events: Vec<_> = glfw::flush_messages(&self.window_ctx.events).collect();
			for (_, event) in events {
				self.process_input(event);
			}

			if self.passive_tracking {
				let (x, y) = self.window_ctx.window.get_cursor_pos();
				let (width, height) = self.window_ctx.window.get_framebuffer_size();
				if x <= 0.0 || x >= width as f64 - 1.0 || y <= 0.0 || y >= height as f64 - 1.0 {
					self.window_ctx.window.set_cursor_pos((width / 2) as f64, (height / 2) as f64);
				}
			}

			unsafe {
				Clear(COLOR_BUFFER_BIT | DEPTH_BUFFER_BIT);
			}

			unsafe {
            	UseProgram(self.shader);
        	}

			let (fb_width, fb_height) = self.window_ctx.window.get_framebuffer_size();
			let elapsed = start_time.elapsed().as_secs_f32();
			let cam_pos = self.camera.get_position();
			let view_mat = self.camera.get_view_matrix();

			unsafe {
				Uniform2f(get_uniform(self.shader, "u_resolution"), fb_width as f32, fb_height as f32);
				Uniform1f(get_uniform(self.shader, "u_time"), elapsed);
				Uniform3f(get_uniform(self.shader, "u_camera_pos"), cam_pos.x, cam_pos.y, cam_pos.z);
				let mat_data = [
					view_mat.m11, view_mat.m12, view_mat.m13,
					view_mat.m21, view_mat.m22, view_mat.m23,
					view_mat.m31, view_mat.m32, view_mat.m33,
				];
				UniformMatrix3fv(get_uniform(self.shader, "u_view_matrix"), 1, FALSE, mat_data.as_ptr());
				Uniform1f(get_uniform(self.shader, "u_fov"), self.fov);
				Uniform1i(get_uniform(self.shader, "u_render_disk"), if self.render_disk { 1 } else { 0 });
				Uniform1i(get_uniform(self.shader, "u_gravitational_lensing"), if self.gravitational_lensing { 1 } else { 0 });
			}

			unsafe {
				BindVertexArray(self.vao);
				DrawArrays(TRIANGLES, 0, 6);
				BindVertexArray(0);
			}

			self.window_ctx.window.swap_buffers();
		}

		unsafe {
			DeleteVertexArrays(1, &self.vao);
		}
	}

	fn process_input(&mut self, event: glfw::WindowEvent) {
		match event {
			glfw::WindowEvent::Key(Key::T, _, Action::Press, _) => {
				self.passive_tracking = !self.passive_tracking;
				println!("Passive mouse tracking: {}", if self.passive_tracking { "ON" } else { "OFF" });
			}
			glfw::WindowEvent::FramebufferSize(width, height) => {
				unsafe {
					Viewport(0, 0, width, height);
				}
			}
			glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
				self.window_ctx.window.set_should_close(true);
			}
			glfw::WindowEvent::Key(Key::D, _, Action::Press, _) => {
				self.render_disk = !self.render_disk;
				println!("Accretion disk: {}", if self.render_disk { "ON" } else { "OFF" });
			}
			glfw::WindowEvent::Key(Key::G, _, Action::Press, _) => {
				self.gravitational_lensing = !self.gravitational_lensing;
				println!("Gravitational lensing: {}", if self.gravitational_lensing { "ON" } else { "OFF" });
			}
			glfw::WindowEvent::Key(Key::Num1, _, Action::Press, _) => {
				self.camera.set_mode(CameraMode::FreeOrbit);
			}
			glfw::WindowEvent::Key(Key::Num2, _, Action::Press, _) => {
				self.camera.set_mode(CameraMode::AutoOrbit);
			}
			glfw::WindowEvent::Key(Key::Num3, _, Action::Press, _) => {
				self.camera.set_mode(CameraMode::FrontView);
			}
			glfw::WindowEvent::Key(Key::Num4, _, Action::Press, _) => {
				self.camera.set_mode(CameraMode::TopView);
			}
			glfw::WindowEvent::Key(Key::Q, _, Action::Press, _) => {
				self.camera.adjust_roll(-0.1);
			}
			glfw::WindowEvent::Key(Key::E, _, Action::Press, _) => {
				self.camera.adjust_roll(0.1);
			}
			glfw::WindowEvent::Key(Key::R, _, Action::Press, _) => {
				self.camera.reset_roll();
			}
			glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Press, _) => {
				self.camera.dragging = true;
				let (x, y) = self.window_ctx.window.get_cursor_pos();
				self.camera.last_x = x;
				self.camera.last_y = y;
			}
			glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Release, _) => {
				self.camera.dragging = false;
			}
			glfw::WindowEvent::CursorPos(x, y) => {
				if self.passive_tracking {
					self.camera.passive_mouse_move(x, y);
				} else {
					self.camera.process_mouse_move(x, y);
				}
			}
			glfw::WindowEvent::Scroll(_, yoffset) => {
				self.camera.process_scroll(yoffset);
			}
			_ => {}
		}
	}

	fn manual(&self) {
		println!("\n╔════════════════════════════════════════════════════╗");
		println!("║     Black Hole 3D Renderer - Controls              ║");
		println!("╠════════════════════════════════════════════════════╣");
		println!("║ CAMERA                                             ║");
		println!("║   Left Mouse + Drag : Orbit camera                 ║");
		println!("║   Mouse Wheel       : Zoom in/out                  ║");
		println!("║   1 Key             : Free orbit mode              ║");
		println!("║   2 Key             : Auto orbit mode              ║");
		println!("║   3 Key             : Front view                   ║");
		println!("║   4 Key             : Top view                     ║");
		println!("║   Q/E Keys          : Roll camera left/right       ║");
		println!("║   R Key             : Reset camera roll            ║");
		println!("║   T Key             : Active/passive mouse tracking║");
		println!("╠════════════════════════════════════════════════════╣");
		println!("║ RENDERING                                          ║");
		println!("║   D Key             : Toggle accretion disk        ║");
		println!("║   G Key             : Toggle gravitational lensing ║");
		println!("╠════════════════════════════════════════════════════╣");
		println!("║ ESC                 : Exit                         ║");
		println!("╚════════════════════════════════════════════════════╝\n");
		println!("Current shader: SIMPLE");
		println!("Camera mode: Free Orbit");
	}
}
