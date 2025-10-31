use nalgebra_glm as glm;

const PI: f32 = std::f32::consts::PI;

#[derive(PartialEq, Clone, Copy)]
pub enum CameraMode {
    FreeOrbit,
    AutoOrbit,
    FrontView,
    TopView, 
}

pub struct Camera {
    pub azimuth: f32,
    pub mode: CameraMode,
    pub elevation: f32,
    pub radius: f32,
    pub target_radius: f32,
    pub min_radius: f32,
    pub max_radius: f32,
    pub orbit_speed: f32,
    pub zoom_speed: f32,
    pub auto_orbit_speed: f32,
    pub lerp_factor: f32,
    pub dragging: bool,
    pub last_x: f64,
    pub last_y: f64,
    pub roll: f32,
}

impl Camera {
    pub fn new() -> Self {
        Camera {
            mode: CameraMode::FreeOrbit,
            azimuth: PI * 0.25,      
            elevation: PI * 0.45,
            radius: 6.0e10,
            target_radius: 6.0e10,
            min_radius: 2.0e10,
            max_radius: 5.0e11,
            orbit_speed: 0.003,
            zoom_speed: 2.0e10,
            auto_orbit_speed: 0.05,
            lerp_factor: 0.1,
            dragging: false,
            last_x: 0.0,
            last_y: 0.0,
            roll: 0.0,
        }
    }

    pub fn update(&mut self, time: f64) {
        self.radius += (self.target_radius - self.radius) * self.lerp_factor;
        
        if self.mode == CameraMode::AutoOrbit {
            self.azimuth = (time as f32) * self.auto_orbit_speed;
            self.elevation = (PI * 0.3) + ((time * 0.05).sin() as f32) * 0.3;
        }
    }

    pub fn get_position(&self) -> glm::Vec3 {
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

    pub fn get_view_matrix(&self) -> glm::Mat3 {
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

    pub fn process_mouse_move(&mut self, x: f64, y: f64) {
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

    pub fn process_scroll(&mut self, yoffset: f64) {
        self.target_radius -= yoffset as f32 * self.zoom_speed;
        self.target_radius = self.target_radius.clamp(self.min_radius, self.max_radius);
    }
    
    pub fn set_mode(&mut self, mode: CameraMode) {
        self.mode = mode;
        println!("Camera mode: {:?}", match mode {
            CameraMode::FreeOrbit => "Free Orbit",
            CameraMode::AutoOrbit => "Auto Orbit",
            CameraMode::FrontView => "Front View",
            CameraMode::TopView => "Top View",
        });
    }
    
    pub fn adjust_roll(&mut self, delta: f32) {
        self.roll += delta;
        println!("Camera roll: {:.1}°", self.roll.to_degrees());
    }

    pub fn reset_roll(&mut self) {
        self.roll = 0.0;
        println!("Camera roll reset");
    }

    pub fn passive_mouse_move(&mut self, x: f64, y: f64) {
        let dx = (x - self.last_x) as f32;
        let dy = (y - self.last_y) as f32;

        let sensitivity = 0.001;

        self.azimuth += dx * sensitivity;
        self.elevation -= dy * sensitivity;
        self.elevation = self.elevation.clamp(0.01, std::f32::consts::PI - 0.01);

        self.last_x = x;
        self.last_y = y;
    }
}