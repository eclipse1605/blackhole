use crate::{camera::{Camera, CameraMode, FreeCamDirection}, fps::FpsCounter, renderer::{window::WindowContext, mesh::create_fullscreen_quad, utils::get_uniform}, shader::create_shader_program};
use crate::gl_bindings::*;
use crate::renderer::skybox::Skybox;
use crate::renderer::utils::load_texture;
use glfw::{self,Context, Action, Key};
use std::fs;
use std::path::Path;
use chrono::Local;
use image::{ImageBuffer, Rgba};

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
	pub shader: u32,
	pub fps_counter: FpsCounter,
	pub skybox: Skybox,
    pub color_map: u32,
	pub screenshot_icon: u32,   
    pub icon_size: f32,
	pub is_fullscreen: bool,
    pub windowed_pos: (i32, i32),
    pub windowed_size: (i32, i32),
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

		let skybox = Skybox::load_from_folder("assets/skybox_nebula_dark")
			.expect("Failed to load skybox");

		let color_map = load_texture("assets/color_map.png")
			.expect("Failed to load color map texture");

		let screenshot_icon = load_texture("assets/ss.png")
    		.expect("Failed to load screenshot icon");

		Self {
			window_ctx,
			camera,
			vao,
			render_disk: true,
			gravitational_lensing: true,
			fov: 60.0,
			passive_tracking: false,
			shader: create_shader_program("shaders/blackhole.vert", "shaders/blackhole.frag").unwrap(),
			fps_counter: FpsCounter::new(),
			color_map,
    		skybox,
			screenshot_icon,
			icon_size: 64.0,
			is_fullscreen: false,
			windowed_pos: (100, 100),
			windowed_size: (WIDTH as i32, HEIGHT as i32),
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
				ActiveTexture(TEXTURE0);
				BindTexture(TEXTURE_2D, self.color_map);
				Uniform1i(get_uniform(self.shader, "colorMap"), 0);

				self.skybox.bind(1);
				Uniform1i(get_uniform(self.shader, "skybox"), 1);
			}

			unsafe {
				BindVertexArray(self.vao);
				DrawArrays(TRIANGLES, 0, 6);
				BindVertexArray(0);
			}

			unsafe {
				Disable(DEPTH_TEST);
				Enable(BLEND);
				BlendFunc(SRC_ALPHA, ONE_MINUS_SRC_ALPHA);

				MatrixMode(PROJECTION);
				PushMatrix();
				LoadIdentity();
				Ortho(0.0, fb_width as f64, 0.0, fb_height as f64, -1.0, 1.0);

				MatrixMode(MODELVIEW);
				PushMatrix();
				LoadIdentity();

				BindTexture(TEXTURE_2D, self.screenshot_icon);

				let size = self.icon_size;
				let x = 20.0;
				let y = 20.0;

				Begin(QUADS);
				TexCoord2f(0.0, 0.0); Vertex2f(x, y);
				TexCoord2f(1.0, 0.0); Vertex2f(x + size, y);
				TexCoord2f(1.0, 1.0); Vertex2f(x + size, y + size);
				TexCoord2f(0.0, 1.0); Vertex2f(x, y + size);
				End();

				PopMatrix();
				MatrixMode(PROJECTION);
				PopMatrix();
				MatrixMode(MODELVIEW);

				Enable(DEPTH_TEST);
				Disable(BLEND);
			}

			self.window_ctx.window.swap_buffers();
			self.fps_counter.update();
		}

		unsafe {
			DeleteVertexArrays(1, &self.vao);
		}
	}

	fn toggle_fullscreen(&mut self) {
		if self.is_fullscreen {
			self.window_ctx.window.set_monitor(
				glfw::WindowMode::Windowed,
				self.windowed_pos.0,
				self.windowed_pos.1,
				self.windowed_size.0 as u32,
				self.windowed_size.1 as u32,
				None,
			);
			self.is_fullscreen = false;
			println!("Switched to windowed mode");
		} else {
			self.windowed_pos = self.window_ctx.window.get_pos();
			let (w, h) = self.window_ctx.window.get_size();
			self.windowed_size = (w, h);

			self.window_ctx.glfw.with_primary_monitor(|_, monitor_opt| {
				if let Some(monitor) = monitor_opt {
					if let Some(mode) = monitor.get_video_mode() {
						self.window_ctx.window.set_monitor(
							glfw::WindowMode::FullScreen(monitor),
							0,
							0,
							mode.width,
							mode.height,
							Some(mode.refresh_rate),
						);
						self.is_fullscreen = true;
						println!("Switched to fullscreen mode");
					}
				} else {
					println!("No primary monitor found!");
				}
			});
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
			glfw::WindowEvent::Key(Key::F, _, Action::Press, _) 
			| glfw::WindowEvent::Key(Key::F12, _, Action::Press, _) => {
				self.toggle_fullscreen();
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
			glfw::WindowEvent::Key(Key::C, _, Action::Press, _) => {
				self.camera.toggle_camera_type();
			}
			glfw::WindowEvent::Key(Key::Up, _, action, _) => {
				if action == Action::Press || action == Action::Repeat {
					self.camera.move_freecam(FreeCamDirection::Up);
				}
			}
			glfw::WindowEvent::Key(Key::Down, _, action, _) => {
				if action == Action::Press || action == Action::Repeat {
					self.camera.move_freecam(FreeCamDirection::Down);
				}
			}
			glfw::WindowEvent::Key(Key::Left, _, action, _) => {
				if action == Action::Press || action == Action::Repeat {
					self.camera.move_freecam(FreeCamDirection::Left);
				}
			}
			glfw::WindowEvent::Key(Key::Right, _, action, _) => {
				if action == Action::Press || action == Action::Repeat {
					self.camera.move_freecam(FreeCamDirection::Right);
				}
			}
			glfw::WindowEvent::Key(Key::P, _, Action::Press, _) => {
				self.take_screenshot();
			}
			glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Press, _) => {
				let (x, y) = self.window_ctx.window.get_cursor_pos();
				let (width, height) = self.window_ctx.window.get_framebuffer_size();

				let y = height as f64 - y;

				let icon_x = 20.0;
				let icon_y = 20.0;
				let icon_size = self.icon_size as f64;

				if x >= icon_x && x <= icon_x + icon_size && y >= icon_y && y <= icon_y + icon_size {
					println!("Screenshot button clicked!");
					self.take_screenshot();
				} else {
					self.camera.dragging = true;
					let (x, y) = self.window_ctx.window.get_cursor_pos();
					self.camera.last_x = x;
					self.camera.last_y = y;
				}
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

	fn take_screenshot(&self) {
		unsafe {
			let (width, height) = self.window_ctx.window.get_framebuffer_size();
			let mut pixels = vec![0u8; (width * height * 4) as usize];
			ReadPixels(
				0,
				0,
				width,
				height,
				RGBA,
				UNSIGNED_BYTE,
				pixels.as_mut_ptr() as *mut std::ffi::c_void,
			);

			let mut flipped = vec![0u8; pixels.len()];
			for y in 0..height {
				let src = (y * width * 4) as usize;
				let dst = ((height - 1 - y) * width * 4) as usize;
				flipped[dst..dst + (width * 4) as usize]
					.copy_from_slice(&pixels[src..src + (width * 4) as usize]);
			}

			let dir = Path::new("screenshots");
			if !dir.exists() {
				fs::create_dir_all(dir).expect("Failed to create screenshots directory");
			}

			let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S");
			let filename = format!("screenshots/screenshot_{}.png", timestamp);

			let img = ImageBuffer::<Rgba<u8>, _>::from_raw(width as u32, height as u32, flipped)
				.expect("Failed to create ImageBuffer");
			img.save(&filename).expect("Failed to save screenshot");

			println!("Screenshot saved to {}", filename);
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
		println!("║   P Key             : Take screenshot              ║");
		println!("║   T Key             : Active/passive mouse tracking║");
		println!("║   C Key             : Toggle FreeCam/LockedCam     ║");
		println!("║   Arrow Keys        : Move camera (FreeCam only)   ║");
		println!("║   F / F12 Keys       : Toggle fullscreen mode      ║");
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
