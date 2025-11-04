use std::path::Path;
use crate::gl_bindings::*;
use gl::types::{GLuint, GLenum};

pub struct Skybox {
    pub id: GLuint,
}

impl Skybox {
    pub fn load_from_folder<P: AsRef<Path>>(folder: P) -> Result<Self, String> {
        let faces = [
            "right.png",
            "left.png",
            "top.png",
            "bottom.png",
            "front.png",
            "back.png",
        ];

        let mut texture_id: GLuint = 0;
        unsafe {
            GenTextures(1, &mut texture_id);
            BindTexture(TEXTURE_CUBE_MAP, texture_id);
        }

        for (i, face) in faces.iter().enumerate() {
            let path = folder.as_ref().join(face);
            let mut dyn_img = image::open(&path)
                .map_err(|_| format!("Failed to load cubemap face {:?}", path))?
                .flipv();

                if *face == "top.png" || *face == "bottom.png" {
                dyn_img = dyn_img.rotate180();
            }

            let img = dyn_img.to_rgb8();

            let (width, height) = img.dimensions();
            let data = img.into_raw();

            unsafe {
                TexImage2D(
                    TEXTURE_CUBE_MAP_POSITIVE_X + i as u32,
                    0,
                    RGB as i32,
                    width as i32,
                    height as i32,
                    0,
                    RGB,
                    UNSIGNED_BYTE,
                    data.as_ptr() as *const _,
                );
            }
        }

        unsafe {
            TexParameteri(TEXTURE_CUBE_MAP, TEXTURE_MIN_FILTER, LINEAR as i32);
            TexParameteri(TEXTURE_CUBE_MAP, TEXTURE_MAG_FILTER, LINEAR as i32);
            TexParameteri(
                TEXTURE_CUBE_MAP,
                TEXTURE_WRAP_S,
                CLAMP_TO_EDGE as i32,
            );
            TexParameteri(
                TEXTURE_CUBE_MAP,
                TEXTURE_WRAP_T,
                CLAMP_TO_EDGE as i32,
            );
            TexParameteri(
                TEXTURE_CUBE_MAP,
                TEXTURE_WRAP_R,
                CLAMP_TO_EDGE as i32,
            );
                // Generate mipmaps for better sampling performance and enable trilinear filtering.
                GenerateMipmap(TEXTURE_CUBE_MAP);
                TexParameteri(TEXTURE_CUBE_MAP, TEXTURE_MIN_FILTER, LINEAR_MIPMAP_LINEAR as i32);
        }

        Ok(Skybox { id: texture_id })
    }

    pub fn bind(&self, unit: GLenum) {
        unsafe {
            ActiveTexture(TEXTURE0 + unit);
            BindTexture(TEXTURE_CUBE_MAP, self.id);
        }
    }
}
