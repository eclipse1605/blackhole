use std::ffi::CString;
use std::ptr;

use crate::gl_bindings::*;

pub fn load_shader(path: &str, shader_type: u32) -> Result<u32, String> {
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

pub fn create_shader_program(vert_path: &str, frag_path: &str) -> Result<u32, String> {
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