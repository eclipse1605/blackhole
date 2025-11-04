use nalgebra_glm as glm;

const PI: f32 = std::f32::consts::PI;

#[derive(PartialEq, Clone, Copy)]
pub enum CameraMode {
    FreeOrbit,
    AutoOrbit,
    FrontView,
    TopView, 
}

#[derive(PartialEq, Clone, Copy)]
pub enum CameraType {
    LockedCam,
    FreeCam,
}

#[derive(PartialEq, Clone, Copy)]
pub enum FreeCamDirection {
    Up,
    Down,
    Left,
    Right,
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
    pub camera_type: CameraType,
    pub free_position: glm::Vec3,
    pub move_speed: f32,
    pub target_distance: f32,
}

impl Camera {
    pub fn new() -> Self {
    // Use shader-friendly units (schwarzschild units). Keep camera distances small
    // so zooming/fov behave sensibly inside the shader.
    let initial_radius = 15.0;
        let initial_azimuth = PI * 0.25;
        let initial_elevation = PI * 0.45;
        let elev_clamped = initial_elevation.clamp(0.01, PI - 0.01);
        let initial_pos = glm::vec3(
            initial_radius * elev_clamped.sin() * initial_azimuth.cos(),
            initial_radius * elev_clamped.cos(),
            initial_radius * elev_clamped.sin() * initial_azimuth.sin(),
        );
        
        Camera {
            mode: CameraMode::FreeOrbit,
            azimuth: initial_azimuth,      
            elevation: initial_elevation,
            radius: initial_radius,
            target_radius: initial_radius,
            min_radius: 2.0,
            max_radius: 500.0,
            orbit_speed: 0.003,
            zoom_speed: 2.0,
            auto_orbit_speed: 0.05,
            lerp_factor: 0.1,
            dragging: false,
            last_x: 0.0,
            last_y: 0.0,
            roll: 0.0,
            camera_type: CameraType::LockedCam,
            free_position: initial_pos,
            move_speed: 1.0,
            target_distance: initial_radius,
        }
    }

    pub fn update(&mut self, time: f64) {
        match self.camera_type {
            CameraType::LockedCam => {
                self.radius += (self.target_radius - self.radius) * self.lerp_factor;
                
                if self.mode == CameraMode::AutoOrbit {
                    self.azimuth = (time as f32) * self.auto_orbit_speed;
                    self.elevation = (PI * 0.3) + ((time * 0.05).sin() as f32) * 0.3;
                }
            }
            CameraType::FreeCam => {
                let current_distance = glm::length(&self.free_position);
                if current_distance > 0.001 {
                    let new_distance = current_distance + (self.target_distance - current_distance) * self.lerp_factor;
                    let direction = self.free_position / current_distance;
                    self.free_position = direction * new_distance;
                }
                
                if self.mode == CameraMode::AutoOrbit {
                    self.azimuth = (time as f32) * self.auto_orbit_speed;
                    self.elevation = (PI * 0.3) + ((time * 0.05).sin() as f32) * 0.3;
                    
                    let radius = self.target_distance;
                    let elev_clamped = self.elevation.clamp(0.01, PI - 0.01);
                    self.free_position = glm::vec3(
                        radius * elev_clamped.sin() * self.azimuth.cos(),
                        radius * elev_clamped.cos(),
                        radius * elev_clamped.sin() * self.azimuth.sin(),
                    );
                    self.target_distance = radius;
                }
            }
        }
    }

    pub fn get_position(&self) -> glm::Vec3 {
        match self.camera_type {
            CameraType::FreeCam => {
                match self.mode {
                    CameraMode::FrontView => {
                        glm::vec3(10.0, 1.0, 10.0)
                    }
                    CameraMode::TopView => {
                        glm::vec3(0.0, 15.0, 0.1)
                    }
                    _ => {
                        self.free_position
                    }
                }
            }
            CameraType::LockedCam => {
                match self.mode {
                    CameraMode::FrontView => {
                        glm::vec3(10.0, 1.0, 10.0)
                    }
                    CameraMode::TopView => {
                        glm::vec3(0.0, 15.0, 0.1)
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
        }
    }

    pub fn get_view_matrix(&self) -> glm::Mat3 {
        let pos = self.get_position();
        
        let forward = match self.camera_type {
            CameraType::LockedCam => {
                let target = glm::vec3(0.0, 0.0, 0.0);
                glm::normalize(&(target - pos))
            }
            CameraType::FreeCam => {
                match self.mode {
                    CameraMode::AutoOrbit => {
                        let target = glm::vec3(0.0, 0.0, 0.0);
                        glm::normalize(&(target - pos))
                    }
                    CameraMode::FrontView | CameraMode::TopView => {
                        let target = glm::vec3(0.0, 0.0, 0.0);
                        glm::normalize(&(target - pos))
                    }
                    _ => {
                        let elev_clamped = self.elevation.clamp(0.01, PI - 0.01);
                        glm::normalize(&glm::vec3(
                            elev_clamped.sin() * self.azimuth.cos(),
                            elev_clamped.cos(),
                            elev_clamped.sin() * self.azimuth.sin(),
                        ))
                    }
                }
            }
        };
        
        let world_up = glm::vec3(0.0, 1.0, 0.0);
        let right = glm::normalize(&glm::cross(&forward, &world_up));
        let up = glm::cross(&right, &forward);
        
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
        if self.dragging {
            let should_rotate = match self.camera_type {
                CameraType::FreeCam => {
                    if self.mode == CameraMode::AutoOrbit {
                        let pos = self.free_position;
                        let target = glm::vec3(0.0, 0.0, 0.0);
                        let direction_to_origin = glm::normalize(&(target - pos));
                        
                        self.elevation = direction_to_origin.y.acos();
                        self.azimuth = direction_to_origin.x.atan2(direction_to_origin.z);
                        self.elevation = self.elevation.clamp(0.01, PI - 0.01);
                        
                        self.mode = CameraMode::FreeOrbit;
                        
                        self.last_x = x;
                        self.last_y = y;
                        return;
                    }
                    true
                }
                CameraType::LockedCam => {
                    if self.mode == CameraMode::AutoOrbit {
                        self.mode = CameraMode::FreeOrbit;
                    }
                    self.mode == CameraMode::FreeOrbit || self.mode == CameraMode::AutoOrbit
                }
            };
            
            if should_rotate {
                let dx = (x - self.last_x) as f32;
                let dy = (y - self.last_y) as f32;
                
                self.azimuth += dx * self.orbit_speed;
                self.elevation -= dy * self.orbit_speed;
                self.elevation = self.elevation.clamp(0.01, PI - 0.01);
            }
        }
        self.last_x = x;
        self.last_y = y;
    }

    pub fn process_scroll(&mut self, yoffset: f64) {
        match self.camera_type {
            CameraType::FreeCam => {
                self.target_distance -= yoffset as f32 * self.zoom_speed;
                self.target_distance = self.target_distance.clamp(self.min_radius, self.max_radius);
            }
            CameraType::LockedCam => {
                self.target_radius -= yoffset as f32 * self.zoom_speed;
                self.target_radius = self.target_radius.clamp(self.min_radius, self.max_radius);
            }
        }
    }
    
    pub fn set_mode(&mut self, mode: CameraMode) {
        if self.camera_type == CameraType::FreeCam {
            match mode {
                CameraMode::FrontView => {
                    self.free_position = glm::vec3(10.0, 1.0, 10.0);
                    let target = glm::vec3(0.0, 0.0, 0.0);
                    let direction = glm::normalize(&(target - self.free_position));
                    self.elevation = direction.y.acos();
                    self.azimuth = direction.x.atan2(direction.z);
                    self.elevation = self.elevation.clamp(0.01, PI - 0.01);
                }
                CameraMode::TopView => {
                    self.free_position = glm::vec3(0.0, 15.0, 0.1);
                    let target = glm::vec3(0.0, 0.0, 0.0);
                    let direction = glm::normalize(&(target - self.free_position));
                    self.elevation = direction.y.acos();
                    self.azimuth = direction.x.atan2(direction.z);
                    self.elevation = self.elevation.clamp(0.01, PI - 0.01);
                }
                CameraMode::AutoOrbit => {
                }
                CameraMode::FreeOrbit => {
                    if self.mode == CameraMode::AutoOrbit {
                        let pos = self.get_position();
                        let target = glm::vec3(0.0, 0.0, 0.0);
                        let direction_to_origin = glm::normalize(&(target - pos));
                        
                        self.elevation = direction_to_origin.y.acos();
                        self.azimuth = direction_to_origin.x.atan2(direction_to_origin.z);
                        self.elevation = self.elevation.clamp(0.01, PI - 0.01);
                        
                        self.free_position = pos;
                        self.target_distance = glm::length(&pos);
                    }
                }
            }
        }
        
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
        if self.mode == CameraMode::AutoOrbit && self.camera_type == CameraType::FreeCam {
            let pos = self.free_position;
            let target = glm::vec3(0.0, 0.0, 0.0);
            let direction_to_origin = glm::normalize(&(target - pos));
            
            self.elevation = direction_to_origin.y.acos();
            self.azimuth = direction_to_origin.x.atan2(direction_to_origin.z);
            self.elevation = self.elevation.clamp(0.01, PI - 0.01);
            
            self.mode = CameraMode::FreeOrbit;
            
            self.last_x = x;
            self.last_y = y;
            return; 
        } else if self.mode == CameraMode::AutoOrbit {
            self.mode = CameraMode::FreeOrbit;
        }
        
        let dx = (x - self.last_x) as f32;
        let dy = (y - self.last_y) as f32;

        let sensitivity = 0.001;

        self.azimuth += dx * sensitivity;
        self.elevation -= dy * sensitivity;
        self.elevation = self.elevation.clamp(0.01, std::f32::consts::PI - 0.01);

        self.last_x = x;
        self.last_y = y;
    }

    pub fn move_freecam(&mut self, direction: FreeCamDirection) {
        if self.camera_type != CameraType::FreeCam {
            return;
        }

        if self.mode == CameraMode::AutoOrbit {
            self.mode = CameraMode::FreeOrbit;
        }

        let elev_clamped = self.elevation.clamp(0.01, PI - 0.01);
        let forward = glm::normalize(&glm::vec3(
            elev_clamped.sin() * self.azimuth.cos(),
            elev_clamped.cos(),
            elev_clamped.sin() * self.azimuth.sin(),
        ));
        let world_up = glm::vec3(0.0, 1.0, 0.0);
        let right = glm::normalize(&glm::cross(&forward, &world_up));
        let up = glm::cross(&right, &forward);

        let movement = match direction {
            FreeCamDirection::Up => -up * self.move_speed,
            FreeCamDirection::Down => up * self.move_speed,
            FreeCamDirection::Left => -right * self.move_speed,
            FreeCamDirection::Right => right * self.move_speed,
        };

        self.free_position = self.free_position + movement;
    }

    pub fn toggle_camera_type(&mut self) {
        match self.camera_type {
            CameraType::LockedCam => {
                let current_pos = self.get_position();
                
                self.free_position = current_pos;
                
                self.target_distance = glm::length(&current_pos);
                
                let target = glm::vec3(0.0, 0.0, 0.0);
                let direction_to_origin = glm::normalize(&(target - current_pos));
                
                self.elevation = direction_to_origin.y.acos();
                self.azimuth = direction_to_origin.x.atan2(direction_to_origin.z);
                self.elevation = self.elevation.clamp(0.01, PI - 0.01);
                
                self.camera_type = CameraType::FreeCam;
                println!("Camera type: FreeCam");
            }
            CameraType::FreeCam => {
                let pos = match self.mode {
                    CameraMode::FrontView | CameraMode::TopView => {
                        self.get_position()
                    }
                    _ => {
                        self.free_position
                    }
                };
                
                let current_dist = glm::length(&pos);
                self.radius = current_dist;
                self.target_radius = self.target_distance;
                
                if self.radius > 0.001 {
                    let normalized = pos / self.radius;
                    self.elevation = normalized.y.acos();
                    self.azimuth = normalized.x.atan2(normalized.z);
                    self.elevation = self.elevation.clamp(0.01, PI - 0.01);
                }
                
                if self.mode == CameraMode::FrontView || self.mode == CameraMode::TopView {
                    self.mode = CameraMode::FreeOrbit;
                }
                self.camera_type = CameraType::LockedCam;
                println!("Camera type: LockedCam");
            }
        }
    }
}
