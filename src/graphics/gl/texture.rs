use std::cell::RefCell;
use std::rc::Rc;

use crate::graphics::gl::GlContext;
use crate::graphics::{
    FilterMode, MipmapFilterMode, TextureFormat, TextureParams, TextureSource, TextureWrap,
};
use crate::native::gl::*;

#[derive(Clone)]
#[repr(transparent)]
pub struct Texture(Rc<TextureInternal>);

impl Texture {
    pub fn new(ctx: Rc<GlContext>, source: TextureSource, params: TextureParams) -> Texture {
        if let TextureSource::Bytes(bytes_data) = source {
            assert_eq!(
                params.format.size(params.width, params.height) as usize,
                bytes_data.len()
            );
        }
        let (internal_format, format, pixel_type) = params.format.into();
        let wrap = match params.wrap {
            TextureWrap::Repeat => GL_REPEAT,
            TextureWrap::Mirror => GL_MIRRORED_REPEAT,
            TextureWrap::Clamp => GL_CLAMP_TO_EDGE,
        };
        let min_filter = gl_filter(params.min_filter, params.mipmap_filter);
        let mag_filter = match params.mag_filter {
            FilterMode::Nearest => GL_NEAREST,
            FilterMode::Linear => GL_LINEAR,
        };
        let tex_internal = Rc::new(TextureInternal::new(ctx, params));
        let mut cache = tex_internal.ctx.cache.borrow_mut();

        cache.store_texture_binding(0);
        cache.bind_texture(0, GL_TEXTURE_2D, tex_internal.gl_tex);
        unsafe {
            glPixelStorei(GL_UNPACK_ALIGNMENT, 1); // miniquad always uses row alignment of 1

            if cfg!(not(target_arch = "wasm32")) {
                // if not WASM
                if params.format == TextureFormat::Alpha {
                    // if alpha miniquad texture, the value on non-WASM is stored in red channel
                    // swizzle red -> alpha
                    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_SWIZZLE_A, GL_RED as _);
                } else {
                    // keep alpha -> alpha
                    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_SWIZZLE_A, GL_ALPHA as _);
                }
            }

            match source {
                TextureSource::Empty => {
                    // not quite sure if glTexImage2D(null) is really a requirement
                    // but it was like this for quite a while and apparantly it works?
                    glTexImage2D(
                        GL_TEXTURE_2D,
                        0,
                        internal_format as i32,
                        params.width as i32,
                        params.height as i32,
                        0,
                        format,
                        pixel_type,
                        std::ptr::null() as _,
                    );
                }
                TextureSource::Bytes(source) => {
                    glTexImage2D(
                        GL_TEXTURE_2D,
                        0,
                        internal_format as i32,
                        params.width as i32,
                        params.height as i32,
                        0,
                        format,
                        pixel_type,
                        source.as_ptr() as *const _,
                    );
                }
            }

            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, wrap as i32);
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, wrap as i32);
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, min_filter as i32);
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, mag_filter as i32);
        }
        cache.restore_texture_binding(0);

        std::mem::drop(cache);
        Texture(tex_internal)
    }

    pub fn resize(&self, width: u32, height: u32, source: Option<&[u8]>) {
        let mut params = self.0.params.borrow_mut();
        let mut cache = self.0.ctx.cache.borrow_mut();

        cache.store_texture_binding(0);
        cache.bind_texture(0, GL_TEXTURE_2D, self.gl_tex());

        let (internal_format, format, pixel_type) = params.format.into();
        params.width = width;
        params.height = height;
        let pixels = source.map(<[u8]>::as_ptr).unwrap_or_default();
        unsafe {
            glTexImage2D(
                GL_TEXTURE_2D,
                0,
                internal_format as i32,
                params.width as i32,
                params.height as i32,
                0,
                format,
                pixel_type,
                pixels as *const _,
            );
        }

        cache.restore_texture_binding(0);
    }

    pub fn set_wrap(&self, wrap_x: TextureWrap, wrap_y: TextureWrap) {
        let mut cache = self.0.ctx.cache.borrow_mut();

        cache.store_texture_binding(0);
        cache.bind_texture(0, GL_TEXTURE_2D, self.gl_tex());
        let wrap_x = match wrap_x {
            TextureWrap::Repeat => GL_REPEAT,
            TextureWrap::Mirror => GL_MIRRORED_REPEAT,
            TextureWrap::Clamp => GL_CLAMP_TO_EDGE,
        };

        let wrap_y = match wrap_y {
            TextureWrap::Repeat => GL_REPEAT,
            TextureWrap::Mirror => GL_MIRRORED_REPEAT,
            TextureWrap::Clamp => GL_CLAMP_TO_EDGE,
        };

        unsafe {
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, wrap_x as i32);
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, wrap_y as i32);
        }
        cache.restore_texture_binding(0);
    }

    pub fn set_min_filter(&self, filter: FilterMode, mipmap_filter: MipmapFilterMode) {
        let mut cache = self.0.ctx.cache.borrow_mut();

        cache.store_texture_binding(0);
        cache.bind_texture(0, GL_TEXTURE_2D, self.gl_tex());

        let filter = gl_filter(filter, mipmap_filter);
        unsafe {
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, filter as i32);
        }
        cache.restore_texture_binding(0);
    }

    pub fn set_mag_filter(&mut self, filter: FilterMode) {
        let mut cache = self.0.ctx.cache.borrow_mut();

        cache.store_texture_binding(0);
        cache.bind_texture(0, GL_TEXTURE_2D, self.gl_tex());

        let filter = match filter {
            FilterMode::Nearest => GL_NEAREST,
            FilterMode::Linear => GL_LINEAR,
        };
        unsafe {
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, filter as i32);
        }
        cache.restore_texture_binding(0);
    }

    pub fn update_part(
        &self,
        x_offset: i32,
        y_offset: i32,
        width: i32,
        height: i32,
        source: &[u8],
    ) {
        let params = self.0.params.borrow_mut();
        let mut cache = self.0.ctx.cache.borrow_mut();

        assert_eq!(self.size(width as _, height as _), source.len());
        assert!(x_offset + width <= params.width as _);
        assert!(y_offset + height <= params.height as _);

        cache.store_texture_binding(0);
        cache.bind_texture(0, GL_TEXTURE_2D, self.gl_tex());
        let (_, format, pixel_type) = params.format.into();

        unsafe {
            glPixelStorei(GL_UNPACK_ALIGNMENT, 1); // miniquad always uses row alignment of 1

            if cfg!(not(target_arch = "wasm32")) {
                // if not WASM
                if params.format == TextureFormat::Alpha {
                    // if alpha miniquad texture, the value on non-WASM is stored in red channel
                    // swizzle red -> alpha
                    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_SWIZZLE_A, GL_RED as _);
                } else {
                    // keep alpha -> alpha
                    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_SWIZZLE_A, GL_ALPHA as _);
                }
            }

            glTexSubImage2D(
                GL_TEXTURE_2D,
                0,
                x_offset as _,
                y_offset as _,
                width as _,
                height as _,
                format,
                pixel_type,
                source.as_ptr() as *const _,
            );
        }

        cache.restore_texture_binding(0);
    }

    /// Read texture data into CPU memory
    pub fn read_pixels(&self, bytes: &mut [u8]) {
        let params = self.0.params.borrow_mut();

        let (_, format, pixel_type) = params.format.into();

        let mut fbo = 0;
        unsafe {
            let mut binded_fbo: i32 = 0;
            glGetIntegerv(GL_DRAW_FRAMEBUFFER_BINDING, &mut binded_fbo);
            glGenFramebuffers(1, &mut fbo);
            glBindFramebuffer(GL_FRAMEBUFFER, fbo);
            glFramebufferTexture2D(
                GL_FRAMEBUFFER,
                GL_COLOR_ATTACHMENT0,
                GL_TEXTURE_2D,
                self.gl_tex(),
                0,
            );

            glReadPixels(
                0,
                0,
                params.width as _,
                params.height as _,
                format,
                pixel_type,
                bytes.as_mut_ptr() as _,
            );

            glBindFramebuffer(GL_FRAMEBUFFER, binded_fbo as _);
            glDeleteFramebuffers(1, &fbo);
        }
    }

    #[inline]
    pub fn size(&self, width: u32, height: u32) -> usize {
        let params = self.0.params.borrow_mut();
        params.format.size(width, height) as usize
    }

    pub fn generate_mipmaps(&self) {
        let mut cache = self.0.ctx.cache.borrow_mut();

        cache.store_texture_binding(0);
        cache.bind_texture(0, GL_TEXTURE_2D, self.gl_tex());
        unsafe {
            glGenerateMipmap(GL_TEXTURE_2D);
        }
        cache.restore_texture_binding(0);
    }

    pub fn gl_tex(&self) -> GLuint {
        self.0.gl_tex()
    }

    pub fn width(&self) -> u32 {
        self.0.params.borrow().width
    }

    pub fn height(&self) -> u32 {
        self.0.params.borrow().height
    }
}

#[derive(Clone)]
struct TextureInternal {
    ctx: Rc<GlContext>,
    gl_tex: GLuint,
    params: RefCell<TextureParams>,
}

impl TextureInternal {
    pub fn new(ctx: Rc<GlContext>, params: TextureParams) -> TextureInternal {
        let mut gl_tex: GLuint = 0;
        unsafe {
            glGenTextures(1, &mut gl_tex as *mut _);
        }
        TextureInternal {
            ctx,
            gl_tex,
            params: RefCell::new(params),
        }
    }

    pub fn gl_tex(&self) -> GLuint {
        self.gl_tex
    }
}

impl Drop for TextureInternal {
    fn drop(&mut self) {
        unsafe {
            glDeleteTextures(1, &self.gl_tex as *const _);
        }
    }
}

/// Converts from TextureFormat to (internal_format, format, pixel_type)
impl From<TextureFormat> for (GLenum, GLenum, GLenum) {
    fn from(format: TextureFormat) -> Self {
        match format {
            TextureFormat::RGB8 => (GL_RGB, GL_RGB, GL_UNSIGNED_BYTE),
            TextureFormat::RGBA8 => (GL_RGBA, GL_RGBA, GL_UNSIGNED_BYTE),
            TextureFormat::RGBA16F => (GL_RGBA16F, GL_RGBA, GL_FLOAT),
            TextureFormat::Depth => (GL_DEPTH_COMPONENT, GL_DEPTH_COMPONENT, GL_UNSIGNED_SHORT),
            TextureFormat::Depth32 => (GL_DEPTH_COMPONENT, GL_DEPTH_COMPONENT, GL_FLOAT),
            #[cfg(target_arch = "wasm32")]
            TextureFormat::Alpha => (GL_ALPHA, GL_ALPHA, GL_UNSIGNED_BYTE),
            #[cfg(not(target_arch = "wasm32"))]
            TextureFormat::Alpha => (GL_R8, GL_RED, GL_UNSIGNED_BYTE), // texture updates will swizzle Red -> Alpha to match WASM
        }
    }
}

fn gl_filter(filter: FilterMode, mipmap_filter: MipmapFilterMode) -> GLenum {
    match filter {
        FilterMode::Nearest => match mipmap_filter {
            MipmapFilterMode::None => GL_NEAREST,
            MipmapFilterMode::Nearest => GL_NEAREST_MIPMAP_NEAREST,
            MipmapFilterMode::Linear => GL_NEAREST_MIPMAP_LINEAR,
        },
        FilterMode::Linear => match mipmap_filter {
            MipmapFilterMode::None => GL_LINEAR,
            MipmapFilterMode::Nearest => GL_LINEAR_MIPMAP_NEAREST,
            MipmapFilterMode::Linear => GL_LINEAR_MIPMAP_LINEAR,
        },
    }
}
