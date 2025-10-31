use crate::gl_bindings::*;
use std::ptr;

pub fn create_fullscreen_quad() -> u32 {
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