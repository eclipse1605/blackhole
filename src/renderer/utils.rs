use std::ffi::CString;

use crate::gl_bindings::GetUniformLocation;

pub fn get_uniform(program_id: u32, name: &str) -> i32 {
    unsafe {
        let cname = CString::new(name).unwrap();
        GetUniformLocation(program_id, cname.as_ptr())
    }
}
