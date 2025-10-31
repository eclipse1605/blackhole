use glfw::{Action, Context, Key};
use std::ffi::CString;

mod gl_bindings;
use gl_bindings::*;

const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;
const TITLE: &str = "Black Hole Renderer";

mod camera;
mod shader;
mod renderer;
use camera::{Camera, CameraMode};
use renderer::{window::WindowContext, mesh::create_fullscreen_quad, shader_manager::ShaderManager};

fn main() {
    
    let mut window_ctx = WindowContext::new(WIDTH, HEIGHT, TITLE);

    window_ctx.window.set_key_polling(true);
    window_ctx.window.set_mouse_button_polling(true);
    window_ctx.window.set_cursor_pos_polling(true);
    window_ctx.window.set_scroll_polling(true);
    window_ctx.window.set_framebuffer_size_polling(true);

    unsafe {
        let version = std::ffi::CStr::from_ptr(GetString(VERSION) as *const i8);
        println!("OpenGL version: {}", version.to_str().unwrap());
    }

    let mut shaders = ShaderManager::new();

    let quad_vao = create_fullscreen_quad();
    println!("Fullscreen quad created");

    let mut camera = Camera::new();
    let mut passive_tracking = false;
    
    let mut render_disk = true;
    let mut gravitational_lensing = true;
    let fov = 60.0f32;
    
    let start_time = std::time::Instant::now();

    unsafe {
        let (fb_width, fb_height) = window_ctx.window.get_framebuffer_size();
        Viewport(0, 0, fb_width, fb_height);
        ClearColor(0.0, 0.0, 0.0, 1.0);
    }

    println!("\n╔════════════════════════════════════════════════════╗");
    println!("║     Black Hole 3D Renderer - Controls             ║");
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
    println!("║   S Key             : Switch shader mode           ║");
    println!("╠════════════════════════════════════════════════════╣");
    println!("║ ESC                 : Exit                         ║");
    println!("╚════════════════════════════════════════════════════╝\n");
    println!("Current shader: SIMPLE");
    println!("Camera mode: Free Orbit");

    while !window_ctx.window.should_close() {
        let current_time = window_ctx.glfw.get_time();
        camera.update(current_time);
        
        window_ctx.poll();
        
        for (_, event) in glfw::flush_messages(&window_ctx.events) {
            match event {
                glfw::WindowEvent::Key(Key::T, _, Action::Press, _) => {
                    passive_tracking = !passive_tracking;
                    println!("Passive mouse tracking: {}", if passive_tracking { "ON" } else { "OFF" });
                },
                glfw::WindowEvent::FramebufferSize(width, height) => {
                    unsafe {
                        Viewport(0, 0, width, height);
                    }
                }
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    window_ctx.window.set_should_close(true);
                }
                glfw::WindowEvent::Key(Key::D, _, Action::Press, _) => {
                    render_disk = !render_disk;
                    println!("Accretion disk: {}", if render_disk { "ON" } else { "OFF" });
                }
                glfw::WindowEvent::Key(Key::G, _, Action::Press, _) => {
                    gravitational_lensing = !gravitational_lensing;
                    println!("Gravitational lensing: {}", if gravitational_lensing { "ON" } else { "OFF" });
                }
                glfw::WindowEvent::Key(Key::S, _, Action::Press, _) => {
                    shaders.switch();
                }
                glfw::WindowEvent::Key(Key::Num1, _, Action::Press, _) => {
                    camera.set_mode(CameraMode::FreeOrbit);
                }
                glfw::WindowEvent::Key(Key::Num2, _, Action::Press, _) => {
                    camera.set_mode(CameraMode::AutoOrbit);
                }
                glfw::WindowEvent::Key(Key::Num3, _, Action::Press, _) => {
                    camera.set_mode(CameraMode::FrontView);
                }
                glfw::WindowEvent::Key(Key::Num4, _, Action::Press, _) => {
                    camera.set_mode(CameraMode::TopView);
                }
                glfw::WindowEvent::Key(Key::Q, _, Action::Press, _) => {
                    camera.adjust_roll(-0.1);
                }
                glfw::WindowEvent::Key(Key::E, _, Action::Press, _) => {
                    camera.adjust_roll(0.1);
                }
                glfw::WindowEvent::Key(Key::R, _, Action::Press, _) => {
                    camera.reset_roll();
                }
                glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Press, _) => {
                    camera.dragging = true;
                    let (x, y) = window_ctx.window.get_cursor_pos();
                    camera.last_x = x;
                    camera.last_y = y;
                }
                glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Release, _) => {
                    camera.dragging = false;
                }
                glfw::WindowEvent::CursorPos(x, y) => {
                    if passive_tracking {
                        camera.passive_mouse_move(x, y, WIDTH as f64, HEIGHT as f64);
                    } else {
                        camera.process_mouse_move(x, y);
                    }
                },
                glfw::WindowEvent::Scroll(_, yoffset) => {
                    camera.process_scroll(yoffset);
                }
                _ => {}
            }
        }

        if passive_tracking {
            let (x, y) = window_ctx.window.get_cursor_pos();
            if x <= 0.0 || x >= WIDTH as f64 - 1.0 || y <= 0.0 || y >= HEIGHT as f64 - 1.0 {
                window_ctx.window.set_cursor_pos((WIDTH / 2) as f64, (HEIGHT / 2) as f64);
            }
        }

        unsafe {
            Clear(COLOR_BUFFER_BIT | DEPTH_BUFFER_BIT);
        }

        shaders.use_current();

        let (fb_width, fb_height) = window_ctx.window.get_framebuffer_size();
        let elapsed = start_time.elapsed().as_secs_f32();
        let cam_pos = camera.get_position();
        let view_mat = camera.get_view_matrix();

        unsafe {
            Uniform2f(shaders.get_uniform("u_resolution"), fb_width as f32, fb_height as f32);
            Uniform1f(shaders.get_uniform("u_time"), elapsed);
            Uniform3f(shaders.get_uniform("u_camera_pos"), cam_pos.x, cam_pos.y, cam_pos.z);
            let mat_data = [
                view_mat.m11, view_mat.m12, view_mat.m13,
                view_mat.m21, view_mat.m22, view_mat.m23,
                view_mat.m31, view_mat.m32, view_mat.m33,
            ];
            UniformMatrix3fv(shaders.get_uniform("u_view_matrix"), 1, FALSE, mat_data.as_ptr());
            Uniform1f(shaders.get_uniform("u_fov"), fov);
            Uniform1i(shaders.get_uniform("u_render_disk"), if render_disk { 1 } else { 0 });
            Uniform1i(shaders.get_uniform("u_gravitational_lensing"), if gravitational_lensing { 1 } else { 0 });
        }

        unsafe {
            BindVertexArray(quad_vao);
            DrawArrays(TRIANGLES, 0, 6);
            BindVertexArray(0);
        }

        window_ctx.window.swap_buffers();
    }

    unsafe {
        DeleteProgram(shaders.simple);
        DeleteProgram(shaders.full);
        DeleteProgram(shaders.debug);
        DeleteVertexArrays(1, &quad_vao);
    }

    println!("\nGoodbye!");
}
