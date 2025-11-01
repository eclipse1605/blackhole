use std::ffi::CString;
use crate::gl_bindings::*;
use gl::types::{GLuint};

pub fn get_uniform(program_id: u32, name: &str) -> i32 {
    unsafe {
        let cname = CString::new(name).unwrap();
        GetUniformLocation(program_id, cname.as_ptr())
    }
}

pub fn load_texture(path: &str) -> Result<u32, String> {
    let img = image::open(path).map_err(|e| e.to_string())?.flipv().to_rgb8();
    let (width, height) = img.dimensions();
    let data = img.into_raw();

    let mut texture_id: GLuint = 0;
    unsafe {
        GenTextures(1, &mut texture_id);
        BindTexture(TEXTURE_2D, texture_id);
        TexImage2D(
            TEXTURE_2D,
            0,
            RGB as i32,
            width as i32,
            height as i32,
            0,
            RGB,
            UNSIGNED_BYTE,
            data.as_ptr() as *const _,
        );
        GenerateMipmap(TEXTURE_2D);
    }

    Ok(texture_id)
}
