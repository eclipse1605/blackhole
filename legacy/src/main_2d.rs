use glfw::{Action, Context, Key, WindowMode};
use nalgebra_glm as glm;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 960;
const G: f64 = 6.67430e-11;
const C: f64 = 299792458.0;

struct Engine {
    world_width: f32,
    world_height: f32,
    offset_x: f32,
    offset_y: f32,
    zoom: f32,
    middle_mouse_pressed: bool,
    last_mouse_x: f64,
    last_mouse_y: f64,
}

impl Engine {
    fn new() -> Self {
        Engine {
            world_width: 100000000000.0,
            world_height: 75000000000.0,
            offset_x: 0.0,
            offset_y: 0.0,
            zoom: 1.0,
            middle_mouse_pressed: false,
            last_mouse_x: 0.0,
            last_mouse_y: 0.0,
        }
    }

    fn setup_projection(&self, window_width: f32, window_height: f32) {
        unsafe {
            Clear(COLOR_BUFFER_BIT | DEPTH_BUFFER_BIT);
            MatrixMode(PROJECTION);
            LoadIdentity();
            
            let aspect_ratio = window_width / window_height;
            let world_aspect = self.world_width / self.world_height;
            
            let (left, right, bottom, top) = if aspect_ratio > world_aspect {
                let extended_width = self.world_height * aspect_ratio;
                (
                    -extended_width + self.offset_x,
                    extended_width + self.offset_x,
                    -self.world_height + self.offset_y,
                    self.world_height + self.offset_y,
                )
            } else {
                let extended_height = self.world_width / aspect_ratio;
                (
                    -self.world_width + self.offset_x,
                    self.world_width + self.offset_x,
                    -extended_height + self.offset_y,
                    extended_height + self.offset_y,
                )
            };
            
            Ortho(left as f64, right as f64, bottom as f64, top as f64, -1.0, 1.0);
            MatrixMode(MODELVIEW);
            LoadIdentity();
        }
    }
}

struct BlackHole {
    position: glm::Vec3,
    mass: f64,
    r_s: f64,
}

impl BlackHole {
    fn new(position: glm::Vec3, mass: f64) -> Self {
        let r_s = 2.0 * G * mass / (C * C);
        BlackHole { position, mass, r_s }
    }

    fn draw(&self) {
        unsafe {
            Color3f(1.0, 0.0, 0.0);
            Begin(TRIANGLE_FAN);
            Vertex2f(self.position.x, self.position.y);
            
            let segments = 32;
            for i in 0..=segments {
                let angle = 2.0 * std::f32::consts::PI * i as f32 / segments as f32;
                let x = self.position.x + self.r_s as f32 * angle.cos();
                let y = self.position.y + self.r_s as f32 * angle.sin();
                Vertex2f(x, y);
            }
            End();
        }
    }
}

#[derive(Clone)]
struct Ray {
    x: f64,
    y: f64,
    r: f64,
    phi: f64,
    dr: f64,
    dphi: f64,
    trail: Vec<glm::Vec2>,
    e: f64,
    l: f64,
}

impl Ray {
    fn new(pos: glm::Vec2, dir: glm::Vec2, rs: f64) -> Self {
        let x = pos.x as f64;
        let y = pos.y as f64;
        let r = (x * x + y * y).sqrt();
        let phi = y.atan2(x);
        
        let dr = dir.x as f64 * phi.cos() + dir.y as f64 * phi.sin();
        let dphi = (-dir.x as f64 * phi.sin() + dir.y as f64 * phi.cos()) / r.max(1e-9);
        
        let l = r * r * dphi;
        let f = 1.0 - rs / r;
        let dt_dlambda = ((dr * dr) / (f * f) + (r * r * dphi * dphi) / f).sqrt();
        let e = f * dt_dlambda;
        
        let mut ray = Ray {
            x, y, r, phi, dr, dphi,
            trail: Vec::with_capacity(1024),
            e, l,
        };
        ray.trail.push(glm::vec2(x as f32, y as f32));
        ray
    }

    fn draw(rays: &[Ray]) {
        unsafe {
            Enable(BLEND);
            BlendFunc(SRC_ALPHA, ONE_MINUS_SRC_ALPHA);
            
            for ray in rays {
                if ray.trail.len() >= 2 {
                    Begin(LINE_STRIP);
                    for (i, point) in ray.trail.iter().enumerate() {
                        let fade = (i as f32) / (ray.trail.len() as f32);
                        let alpha = 0.2 + fade * 0.6; 
                        
                        let blue = 0.3 + fade * 0.4;  
                        let green = 0.5 + fade * 0.4; 
                        let red = fade * 0.3;         
                        
                        Color4f(red, green, blue, alpha);
                        Vertex2f(point.x, point.y);
                    }
                    End();
                }
                
                Color4f(1.0, 0.8, 0.2, 1.0); 
                PointSize(4.0);
                Begin(POINTS);
                Vertex2f(ray.x as f32, ray.y as f32);
                End();
            }
            
            Disable(BLEND);
        }
    }

    fn step(&mut self, dlam: f64, rs: f64) -> bool {
        if self.r <= rs {
            return false;
        }
        
        rk4_step(self, dlam, rs);
        
        self.x = self.r * self.phi.cos();
        self.y = self.r * self.phi.sin();
        
        let max_distance = 2e11; 
        if self.r > max_distance {
            return false;
        }
        
        let current_pos = glm::vec2(self.x as f32, self.y as f32);
        let should_add_point = if let Some(last_point) = self.trail.last() {
            let distance = ((current_pos.x - last_point.x).powi(2) + 
                           (current_pos.y - last_point.y).powi(2)).sqrt();
            distance > 1e8
        } else {
            true
        };
        
        if should_add_point {
            self.trail.push(current_pos);
            
            if self.trail.len() > 800 {
                self.trail.remove(0); 
            }
        }
        
        true 
    }
}

fn geodesic_rhs(ray: &Ray, out: &mut [f64; 4], rs: f64) {
    let r = ray.r;
    let dr = ray.dr;
    let dphi = ray.dphi;
    let e = ray.e;
    let f = 1.0 - rs / r;
    
    out[0] = dr;
    out[1] = dphi;
    
    let dt_dlambda = e / f;
    out[2] = -(rs / (2.0 * r * r)) * f * (dt_dlambda * dt_dlambda)
           + (rs / (2.0 * r * r * f)) * (dr * dr)
           + (r - rs) * (dphi * dphi);
    
    out[3] = -2.0 * dr * dphi / r;
}

fn add_state(a: &[f64; 4], b: &[f64; 4], factor: f64, out: &mut [f64; 4]) {
    for i in 0..4 {
        out[i] = a[i] + b[i] * factor;
    }
}

fn rk4_step(ray: &mut Ray, dlam: f64, rs: f64) {
    let y0 = [ray.r, ray.phi, ray.dr, ray.dphi];
    let mut k1 = [0.0f64; 4];
    let mut k2 = [0.0f64; 4];
    let mut k3 = [0.0f64; 4];
    let mut k4 = [0.0f64; 4];
    
    geodesic_rhs(ray, &mut k1, rs);
    
    let mut temp = [0.0f64; 4];
    add_state(&y0, &k1, dlam / 2.0, &mut temp);
    let mut r2 = ray.clone();
    r2.r = temp[0]; r2.phi = temp[1]; r2.dr = temp[2]; r2.dphi = temp[3];
    geodesic_rhs(&r2, &mut k2, rs);
    
    add_state(&y0, &k2, dlam / 2.0, &mut temp);
    let mut r3 = ray.clone();
    r3.r = temp[0]; r3.phi = temp[1]; r3.dr = temp[2]; r3.dphi = temp[3];
    geodesic_rhs(&r3, &mut k3, rs);
    
    add_state(&y0, &k3, dlam, &mut temp);
    let mut r4 = ray.clone();
    r4.r = temp[0]; r4.phi = temp[1]; r4.dr = temp[2]; r4.dphi = temp[3];
    geodesic_rhs(&r4, &mut k4, rs);
    
    ray.r += (dlam / 6.0) * (k1[0] + 2.0 * k2[0] + 2.0 * k3[0] + k4[0]);
    ray.phi += (dlam / 6.0) * (k1[1] + 2.0 * k2[1] + 2.0 * k3[1] + k4[1]);
    ray.dr += (dlam / 6.0) * (k1[2] + 2.0 * k2[2] + 2.0 * k3[2] + k4[2]);
    ray.dphi += (dlam / 6.0) * (k1[3] + 2.0 * k2[3] + 2.0 * k3[3] + k4[3]);
}

fn main() {
    let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Compat));

    let (mut window, events) = glfw
        .create_window(WIDTH, HEIGHT, "Black Hole Simulation", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window");

    window.set_key_polling(true);
    window.set_mouse_button_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_framebuffer_size_polling(true);
    window.make_current();

    load_with(|symbol| window.get_proc_address(symbol) as *const _);

    let (fb_width, fb_height) = window.get_framebuffer_size();
    unsafe {
        Viewport(0, 0, fb_width, fb_height);
        ClearColor(0.0, 0.0, 0.0, 1.0);
        Enable(DEPTH_TEST);
    }

    let engine = Engine::new();
    let sag_mass = 8.54e36;
    let sag_a = BlackHole::new(glm::vec3(0.0, 0.0, 0.0), sag_mass);
    let mut rays: Vec<Ray> = Vec::new();
    
    let mut is_fullscreen = false;
    let mut windowed_pos = (100, 100);
    let mut windowed_size = (WIDTH, HEIGHT);

    while !window.should_close() {
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::FramebufferSize(width, height) => {
                    unsafe {
                        Viewport(0, 0, width, height);
                    }
                }
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    window.set_should_close(true)
                }
                glfw::WindowEvent::Key(Key::F11, _, Action::Press, _) => {
                    if is_fullscreen {
                        window.set_monitor(
                            WindowMode::Windowed,
                            windowed_pos.0,
                            windowed_pos.1,
                            windowed_size.0,
                            windowed_size.1,
                            None,
                        );
                        is_fullscreen = false;
                    } else {
                        windowed_pos = window.get_pos();
                        windowed_size = {
                            let (w, h) = window.get_size();
                            (w as u32, h as u32)
                        };
                        
                        glfw.with_primary_monitor(|_, monitor| {
                            if let Some(monitor) = monitor {
                                if let Some(mode) = monitor.get_video_mode() {
                                    window.set_monitor(
                                        WindowMode::FullScreen(monitor),
                                        0,
                                        0,
                                        mode.width,
                                        mode.height,
                                        Some(mode.refresh_rate),
                                    );
                                    is_fullscreen = true;
                                }
                            }
                        });
                    }
                }
                glfw::WindowEvent::Key(Key::Space, _, Action::Press, _) => {
                    let spawn_x = -engine.world_width * 0.9;
                    let spawn_count = 50;
                    let speed = C as f32;
                    
                    for i in 0..spawn_count {
                        let t = i as f32 / (spawn_count - 1) as f32;
                        let y = -engine.world_height * 0.8 + t * (engine.world_height * 1.6);
                        let pos = glm::vec2(spawn_x, y);
                        let dir = glm::vec2(speed, 0.0);
                        rays.push(Ray::new(pos, dir, sag_a.r_s));
                    }
                }
                _ => {}
            }
        }

        rays.retain_mut(|ray| ray.step(1.0, sag_a.r_s));

        let (window_width, window_height) = window.get_framebuffer_size();
        engine.setup_projection(window_width as f32, window_height as f32);

        sag_a.draw();
        Ray::draw(&rays);
        
        window.swap_buffers();
    }
}
