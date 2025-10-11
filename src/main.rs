use glfw::{Action, Context, Key};
use nalgebra_glm as glm;
use std::ffi::CString;
use std::ptr;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;
const PI: f32 = std::f32::consts::PI;

#[derive(PartialEq, Clone, Copy)]
enum CameraMode {
    FreeOrbit,
    AutoOrbit,
    FrontView,
    TopView, 
}

struct Camera {
    mode: CameraMode,
    azimuth: f32,
    elevation: f32,
    radius: f32,
    target_radius: f32,
    min_radius: f32,
    max_radius: f32,
    orbit_speed: f32,
    zoom_speed: f32,
    auto_orbit_speed: f32,
    lerp_factor: f32,
    dragging: bool,
    last_x: f64,
    last_y: f64,
    roll: f32,
}

impl Camera {
    fn new() -> Self {
        Camera {
            mode: CameraMode::FreeOrbit,
            azimuth: PI * 0.25,      
            elevation: PI * 0.45,
            radius: 6.0e10,
            target_radius: 6.0e10,
            min_radius: 2.0e10,
            max_radius: 2.0e11,
            orbit_speed: 0.003,
            zoom_speed: 5.0e9,
            auto_orbit_speed: 0.05,
            lerp_factor: 0.1,
            dragging: false,
            last_x: 0.0,
            last_y: 0.0,
            roll: 0.0,
        }
    }

    fn update(&mut self, time: f64) {
        self.radius += (self.target_radius - self.radius) * self.lerp_factor;
        
        if self.mode == CameraMode::AutoOrbit {
            self.azimuth = (time as f32) * self.auto_orbit_speed;
            self.elevation = (PI * 0.3) + ((time * 0.05).sin() as f32) * 0.3;
        }
    }

    fn get_position(&self) -> glm::Vec3 {
        match self.mode {
            CameraMode::FrontView => {
                glm::vec3(10.0e10, 1.0e10, 10.0e10)
            }
            CameraMode::TopView => {
                glm::vec3(0.0, 15.0e10, 0.1e10)
            }
            _ => {
                let elev_clamped = self.elevation.clamp(0.01, PI - 0.01);
                glm::vec3(
                    self.radius * elev_clamped.sin() * self.azimuth.cos(),
                    self.radius * elev_clamped.cos(),
                    self.radius * elev_clamped.sin() * self.azimuth.sin(),
                )
            }
        }
    }

    fn get_view_matrix(&self) -> glm::Mat3 {
        let pos = self.get_position();
        let target = glm::vec3(0.0, 0.0, 0.0);
        let forward = glm::normalize(&(target - pos));
        
        // Handle roll for camera rotation
        let world_up = glm::vec3(0.0, 1.0, 0.0);
        let right = glm::normalize(&glm::cross(&forward, &world_up));
        let up = glm::cross(&right, &forward);
        
        // Apply roll if needed
        if self.roll.abs() > 0.001 {
            let cos_roll = self.roll.cos();
            let sin_roll = self.roll.sin();
            let right_rolled = right * cos_roll + up * sin_roll;
            let up_rolled = -right * sin_roll + up * cos_roll;
            
            glm::mat3(
                right_rolled.x, right_rolled.y, right_rolled.z,
                up_rolled.x, up_rolled.y, up_rolled.z,
                forward.x, forward.y, forward.z
            )
        } else {
            glm::mat3(
                right.x, right.y, right.z,
                up.x, up.y, up.z,
                forward.x, forward.y, forward.z
            )
        }
    }

    fn process_mouse_move(&mut self, x: f64, y: f64) {
        if self.dragging && (self.mode == CameraMode::FreeOrbit || self.mode == CameraMode::AutoOrbit) {
            let dx = (x - self.last_x) as f32;
            let dy = (y - self.last_y) as f32;
            
            if self.mode == CameraMode::AutoOrbit {
                self.mode = CameraMode::FreeOrbit;
            }
            
            self.azimuth += dx * self.orbit_speed;
            self.elevation -= dy * self.orbit_speed;
            self.elevation = self.elevation.clamp(0.01, PI - 0.01);
        }
        self.last_x = x;
        self.last_y = y;
    }

    fn process_scroll(&mut self, yoffset: f64) {
        self.target_radius -= yoffset as f32 * self.zoom_speed;
        self.target_radius = self.target_radius.clamp(self.min_radius, self.max_radius);
    }
    
    fn set_mode(&mut self, mode: CameraMode) {
        self.mode = mode;
        println!("Camera mode: {:?}", match mode {
            CameraMode::FreeOrbit => "Free Orbit",
            CameraMode::AutoOrbit => "Auto Orbit",
            CameraMode::FrontView => "Front View",
            CameraMode::TopView => "Top View",
        });
    }
    
    fn adjust_roll(&mut self, delta: f32) {
        self.roll += delta;
        println!("Camera roll: {:.1}°", self.roll.to_degrees());
    }
}

fn load_shader(path: &str, shader_type: u32) -> Result<u32, String> {
    let source = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read shader file {}: {}", path, e))?;
    
    let c_source = CString::new(source.as_bytes())
        .map_err(|e| format!("CString conversion failed: {}", e))?;
    
    unsafe {
        let shader = CreateShader(shader_type);
        ShaderSource(shader, 1, &c_source.as_ptr(), ptr::null());
        CompileShader(shader);
        
        let mut success = 0;
        GetShaderiv(shader, COMPILE_STATUS, &mut success);
        
        if success == 0 {
            let mut len = 0;
            GetShaderiv(shader, INFO_LOG_LENGTH, &mut len);
            let mut buffer = vec![0u8; len as usize];
            GetShaderInfoLog(shader, len, ptr::null_mut(), buffer.as_mut_ptr() as *mut i8);
            let error = String::from_utf8_lossy(&buffer);
            return Err(format!("Shader compilation failed for {}: {}", path, error));
        }
        
        Ok(shader)
    }
}

fn create_shader_program(vert_path: &str, frag_path: &str) -> Result<u32, String> {
    let vert_shader = load_shader(vert_path, VERTEX_SHADER)?;
    let frag_shader = load_shader(frag_path, FRAGMENT_SHADER)?;
    
    unsafe {
        let program = CreateProgram();
        AttachShader(program, vert_shader);
        AttachShader(program, frag_shader);
        LinkProgram(program);
        
        let mut success = 0;
        GetProgramiv(program, LINK_STATUS, &mut success);
        
        if success == 0 {
            let mut len = 0;
            GetProgramiv(program, INFO_LOG_LENGTH, &mut len);
            let mut buffer = vec![0u8; len as usize];
            GetProgramInfoLog(program, len, ptr::null_mut(), buffer.as_mut_ptr() as *mut i8);
            let error = String::from_utf8_lossy(&buffer);
            return Err(format!("Program linking failed: {}", error));
        }
        
        DeleteShader(vert_shader);
        DeleteShader(frag_shader);
        
        Ok(program)
    }
}

fn create_fullscreen_quad() -> u32 {
    let vertices: [f32; 18] = [
        -1.0, -1.0, 0.0,
        1.0, -1.0, 0.0,
        1.0, 1.0, 0.0,
        
        1.0, 1.0, 0.0,
        -1.0, 1.0, 0.0,
        -1.0, -1.0, 0.0,
    ];
    
    unsafe {
        let mut vao = 0;
        let mut vbo = 0;
        
        GenVertexArrays(1, &mut vao);
        GenBuffers(1, &mut vbo);
        
        BindVertexArray(vao);
        BindBuffer(ARRAY_BUFFER, vbo);
        BufferData(
            ARRAY_BUFFER,
            (vertices.len() * std::mem::size_of::<f32>()) as isize,
            vertices.as_ptr() as *const _,
            STATIC_DRAW,
        );
        
        EnableVertexAttribArray(0);
        VertexAttribPointer(0, 3, FLOAT, FALSE, 3 * std::mem::size_of::<f32>() as i32, ptr::null());
        
        BindVertexArray(0);
        
        vao
    }
}

fn main() {
    let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));

    let (mut window, events) = glfw
        .create_window(WIDTH, HEIGHT, "Black Hole 3D Renderer", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window");

    window.set_key_polling(true);
    window.set_mouse_button_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_scroll_polling(true);
    window.set_framebuffer_size_polling(true);
    window.make_current();
    glfw.set_swap_interval(glfw::SwapInterval::Sync(1));

    load_with(|symbol| window.get_proc_address(symbol) as *const _);

    unsafe {
        let version = std::ffi::CStr::from_ptr(GetString(VERSION) as *const i8);
        println!("OpenGL version: {}", version.to_str().unwrap());
    }

    let shader_program_simple = match create_shader_program(
        "shaders/blackhole.vert",
        "shaders/blackhole_simple.frag"
    ) {
        Ok(prog) => {
            println!("Simple shader compiled");
            prog
        },
        Err(e) => {
            eprintln!("Simple shader error: {}", e);
            return;
        }
    };
    
    let shader_program_full = match create_shader_program(
        "shaders/blackhole.vert",
        "shaders/blackhole.frag"
    ) {
        Ok(prog) => {
            println!("Full shader compiled");
            prog
        },
        Err(e) => {
            eprintln!("Full shader error: {}", e);
            return;
        }
    };
    
    let shader_program_debug = match create_shader_program(
        "shaders/blackhole.vert",
        "shaders/debug.frag"
    ) {
        Ok(prog) => {
            println!("Debug shader compiled");
            prog
        },
        Err(e) => {
            eprintln!("Debug shader error: {}", e);
            return;
        }
    };
    
    let mut current_shader = 0; // 0=simple, 1=full, 2=debug
    let mut shader_program = shader_program_simple;

    let quad_vao = create_fullscreen_quad();
    println!("Fullscreen quad created");

    let u_resolution = unsafe {
        let name = CString::new("u_resolution").unwrap();
        GetUniformLocation(shader_program, name.as_ptr())
    };
    let u_time = unsafe {
        let name = CString::new("u_time").unwrap();
        GetUniformLocation(shader_program, name.as_ptr())
    };
    let u_camera_pos = unsafe {
        let name = CString::new("u_camera_pos").unwrap();
        GetUniformLocation(shader_program, name.as_ptr())
    };
    let u_view_matrix = unsafe {
        let name = CString::new("u_view_matrix").unwrap();
        GetUniformLocation(shader_program, name.as_ptr())
    };
    let u_fov = unsafe {
        let name = CString::new("u_fov").unwrap();
        GetUniformLocation(shader_program, name.as_ptr())
    };
    let u_render_disk = unsafe {
        let name = CString::new("u_render_disk").unwrap();
        GetUniformLocation(shader_program, name.as_ptr())
    };
    let u_gravitational_lensing = unsafe {
        let name = CString::new("u_gravitational_lensing").unwrap();
        GetUniformLocation(shader_program, name.as_ptr())
    };

    let mut camera = Camera::new();
    
    let mut render_disk = true;
    let mut gravitational_lensing = true;
    let fov = 60.0f32;
    
    let start_time = std::time::Instant::now();

    unsafe {
        let (fb_width, fb_height) = window.get_framebuffer_size();
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

    while !window.should_close() {
        let current_time = glfw.get_time();
        camera.update(current_time);
        
        glfw.poll_events();
        
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::FramebufferSize(width, height) => {
                    unsafe {
                        Viewport(0, 0, width, height);
                    }
                }
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    window.set_should_close(true);
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
                    current_shader = (current_shader + 1) % 3;
                    shader_program = match current_shader {
                        0 => {
                            println!("Switched to: SIMPLE shader (normalized units, fast)");
                            shader_program_simple
                        },
                        1 => {
                            println!("Switched to: FULL shader (physical units, detailed)");
                            shader_program_full
                        },
                        _ => {
                            println!("Switched to: DEBUG shader (testing)");
                            shader_program_debug
                        },
                    };
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
                    camera.roll = 0.0;
                    println!("Camera roll reset");
                }
                glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Press, _) => {
                    camera.dragging = true;
                    let (x, y) = window.get_cursor_pos();
                    camera.last_x = x;
                    camera.last_y = y;
                }
                glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Release, _) => {
                    camera.dragging = false;
                }
                glfw::WindowEvent::CursorPos(x, y) => {
                    camera.process_mouse_move(x, y);
                }
                glfw::WindowEvent::Scroll(_, yoffset) => {
                    camera.process_scroll(yoffset);
                }
                _ => {}
            }
        }

        unsafe {
            Clear(COLOR_BUFFER_BIT | DEPTH_BUFFER_BIT);
        }

        unsafe {
            UseProgram(shader_program);
        }

        let (fb_width, fb_height) = window.get_framebuffer_size();
        let elapsed = start_time.elapsed().as_secs_f32();
        let cam_pos = camera.get_position();
        let view_mat = camera.get_view_matrix();

        unsafe {
            Uniform2f(u_resolution, fb_width as f32, fb_height as f32);
            Uniform1f(u_time, elapsed);
            Uniform3f(u_camera_pos, cam_pos.x, cam_pos.y, cam_pos.z);
            let mat_data = [
                view_mat.m11, view_mat.m12, view_mat.m13,
                view_mat.m21, view_mat.m22, view_mat.m23,
                view_mat.m31, view_mat.m32, view_mat.m33,
            ];
            UniformMatrix3fv(u_view_matrix, 1, FALSE, mat_data.as_ptr());
            Uniform1f(u_fov, fov);
            Uniform1i(u_render_disk, if render_disk { 1 } else { 0 });
            Uniform1i(u_gravitational_lensing, if gravitational_lensing { 1 } else { 0 });
        }

        unsafe {
            BindVertexArray(quad_vao);
            DrawArrays(TRIANGLES, 0, 6);
            BindVertexArray(0);
        }

        window.swap_buffers();
    }

    unsafe {
        DeleteProgram(shader_program_simple);
        DeleteProgram(shader_program_full);
        DeleteProgram(shader_program_debug);
        DeleteVertexArrays(1, &quad_vao);
    }

    println!("\nGoodbye!");
}
