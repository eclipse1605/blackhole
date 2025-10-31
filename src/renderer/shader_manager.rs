use crate::shader::create_shader_program;
use crate::gl_bindings::*;
use std::ffi::CString;

pub enum ShaderType {
    Simple,
    Full,
    Debug,
}

pub struct ShaderManager {
    pub simple: u32,
    pub full: u32,
    pub debug: u32,
    pub current: ShaderType,
}

impl ShaderManager {
    pub fn new() -> Self {
        let simple = create_shader_program("shaders/blackhole.vert", "shaders/blackhole_simple.frag").unwrap();
        let full   = create_shader_program("shaders/blackhole.vert", "shaders/blackhole.frag").unwrap();
        let debug  = create_shader_program("shaders/blackhole.vert", "shaders/debug.frag").unwrap();
        Self { simple, full, debug, current: ShaderType::Simple }
    }

    pub fn use_current(&self) {
        unsafe {
            UseProgram(self.current_id());
        }
    }

    pub fn switch(&mut self) {
        self.current = match self.current {
            ShaderType::Simple => ShaderType::Full,
            ShaderType::Full => ShaderType::Debug,
            ShaderType::Debug => ShaderType::Simple,
        };
    }

    pub fn current_id(&self) -> u32 {
        match self.current {
            ShaderType::Simple => self.simple,
            ShaderType::Full => self.full,
            ShaderType::Debug => self.debug,
        }
    }

    pub fn get_uniform(&self, name: &str) -> i32 {
        unsafe {
            let cname = CString::new(name).unwrap();
            GetUniformLocation(self.current_id(), cname.as_ptr())
        }
    }
}
