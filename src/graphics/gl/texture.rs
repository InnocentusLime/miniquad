use crate::graphics::gl::GlContext;
use crate::graphics::{
    FilterMode, MipmapFilterMode, RawId, TextureFormat, TextureId, TextureKind, TextureParams,
    TextureSource, TextureWrap,
};
use crate::native::gl::*;
use crate::TextureIdInner;

#[derive(Clone, Copy, Debug)]
pub struct Texture {
    pub gl_tex: GLuint,
    pub params: TextureParams,
}

impl Texture {
    pub fn new(ctx: &mut GlContext, source: TextureSource, params: TextureParams) -> Texture {
        if let TextureSource::Bytes(bytes_data) = source {
            assert_eq!(
                params.format.size(params.width, params.height) as usize,
                bytes_data.len()
            );
        }
        let (internal_format, format, pixel_type) = params.format.into();

        ctx.cache.store_texture_binding(0);

        let mut gl_tex: GLuint = 0;

        unsafe {
            glGenTextures(1, &mut gl_tex as *mut _);
            ctx.cache.bind_texture(0, params.kind.into(), gl_tex);
            glPixelStorei(GL_UNPACK_ALIGNMENT, 1); // miniquad always uses row alignment of 1

            if cfg!(not(target_arch = "wasm32")) {
                // if not WASM
                if params.format == TextureFormat::Alpha {
                    // if alpha miniquad texture, the value on non-WASM is stored in red channel
                    // swizzle red -> alpha
                    glTexParameteri(params.kind.into(), GL_TEXTURE_SWIZZLE_A, GL_RED as _);
                } else {
                    // keep alpha -> alpha
                    glTexParameteri(params.kind.into(), GL_TEXTURE_SWIZZLE_A, GL_ALPHA as _);
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
                    assert!(params.kind == TextureKind::Texture2D, "incompatible TextureKind and TextureSource. Cubemaps require TextureSource::Array of 6 textures.");
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
                TextureSource::Array(array) => {
                    if params.kind == TextureKind::CubeMap {
                        assert!(
                            array.len() == 6,
                            "Cubemaps require TextureSource::Array of 6 textures."
                        );
                    }
                    for (cubemap_face, mipmaps) in array.iter().enumerate() {
                        if mipmaps.len() != 1 {
                            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_BASE_LEVEL, 0);
                            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAX_LEVEL, array.len() as _);
                        }
                        for (mipmap_level, bytes) in mipmaps.iter().enumerate() {
                            let target = match params.kind {
                                TextureKind::Texture2D => GL_TEXTURE_2D,
                                TextureKind::CubeMap => {
                                    GL_TEXTURE_CUBE_MAP_POSITIVE_X + cubemap_face as u32
                                }
                            };
                            glTexImage2D(
                                target,
                                mipmap_level as _,
                                internal_format as i32,
                                params.width as i32,
                                params.height as i32,
                                0,
                                format,
                                pixel_type,
                                bytes.as_ptr() as *const _,
                            );
                        }
                    }
                }
            }

            let wrap = match params.wrap {
                TextureWrap::Repeat => GL_REPEAT,
                TextureWrap::Mirror => GL_MIRRORED_REPEAT,
                TextureWrap::Clamp => GL_CLAMP_TO_EDGE,
            };

            let min_filter = Self::gl_filter(params.min_filter, params.mipmap_filter);
            let mag_filter = match params.mag_filter {
                FilterMode::Nearest => GL_NEAREST,
                FilterMode::Linear => GL_LINEAR,
            };

            glTexParameteri(params.kind.into(), GL_TEXTURE_WRAP_S, wrap as i32);
            glTexParameteri(params.kind.into(), GL_TEXTURE_WRAP_T, wrap as i32);
            glTexParameteri(params.kind.into(), GL_TEXTURE_MIN_FILTER, min_filter as i32);
            glTexParameteri(params.kind.into(), GL_TEXTURE_MAG_FILTER, mag_filter as i32);
        }
        ctx.cache.restore_texture_binding(0);

        Texture { gl_tex, params }
    }

    pub fn resize(&mut self, ctx: &mut GlContext, width: u32, height: u32, source: Option<&[u8]>) {
        ctx.cache.store_texture_binding(0);
        ctx.cache
            .bind_texture(0, self.params.kind.into(), self.gl_tex);

        let (internal_format, format, pixel_type) = self.params.format.into();

        self.params.width = width;
        self.params.height = height;

        unsafe {
            glTexImage2D(
                GL_TEXTURE_2D,
                0,
                internal_format as i32,
                self.params.width as i32,
                self.params.height as i32,
                0,
                format,
                pixel_type,
                match source {
                    Some(source) => source.as_ptr() as *const _,
                    Option::None => std::ptr::null(),
                },
            );
        }

        ctx.cache.restore_texture_binding(0);
    }

    pub fn update_texture_part(
        &self,
        ctx: &mut GlContext,
        x_offset: i32,
        y_offset: i32,
        width: i32,
        height: i32,
        source: &[u8],
    ) {
        assert_eq!(self.size(width as _, height as _), source.len());
        assert!(x_offset + width <= self.params.width as _);
        assert!(y_offset + height <= self.params.height as _);

        ctx.cache.store_texture_binding(0);
        ctx.cache
            .bind_texture(0, self.params.kind.into(), self.gl_tex);

        let (_, format, pixel_type) = self.params.format.into();

        unsafe {
            glPixelStorei(GL_UNPACK_ALIGNMENT, 1); // miniquad always uses row alignment of 1

            if cfg!(not(target_arch = "wasm32")) {
                // if not WASM
                if self.params.format == TextureFormat::Alpha {
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

        ctx.cache.restore_texture_binding(0);
    }

    /// Read texture data into CPU memory
    pub fn read_pixels(&self, bytes: &mut [u8]) {
        let (_, format, pixel_type) = self.params.format.into();

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
                self.gl_tex,
                0,
            );

            glReadPixels(
                0,
                0,
                self.params.width as _,
                self.params.height as _,
                format,
                pixel_type,
                bytes.as_mut_ptr() as _,
            );

            glBindFramebuffer(GL_FRAMEBUFFER, binded_fbo as _);
            glDeleteFramebuffers(1, &fbo);
        }
    }

    #[inline]
    fn size(&self, width: u32, height: u32) -> usize {
        self.params.format.size(width, height) as usize
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

impl From<TextureKind> for GLuint {
    fn from(kind: TextureKind) -> GLuint {
        match kind {
            TextureKind::Texture2D => GL_TEXTURE_2D,
            TextureKind::CubeMap => GL_TEXTURE_CUBE_MAP,
        }
    }
}

#[derive(Default)]
pub struct Textures(Vec<Texture>);

impl Textures {
    pub fn get(&self, texture: TextureId) -> Texture {
        match texture.0 {
            TextureIdInner::Raw(RawId::OpenGl(texture)) => Texture {
                gl_tex: texture,
                params: Default::default(),
            },
            #[cfg(target_vendor = "apple")]
            TextureIdInner::Raw(RawId::Metal(..)) => panic!("Metal texture in OpenGL context!"),
            TextureIdInner::Managed(texture) => self.0[texture],
        }
    }
}

impl GlContext {
    pub fn new_gl_texture(&mut self, source: TextureSource, params: TextureParams) -> TextureId {
        let texture = Texture::new(self, source, params);
        self.textures.0.push(texture);
        TextureId(TextureIdInner::Managed(self.textures.0.len() - 1))
    }

    pub fn delete_gl_texture(&mut self, texture: TextureId) {
        //self.cache.clear_texture_bindings();

        let t = self.textures.get(texture);
        unsafe {
            glDeleteTextures(1, t.gl_tex as *const _);
        }
    }

    pub fn gl_texture_set_wrap(
        &mut self,
        texture: TextureId,
        wrap_x: TextureWrap,
        wrap_y: TextureWrap,
    ) {
        let t = self.textures.get(texture);

        self.cache.store_texture_binding(0);
        self.cache.bind_texture(0, t.params.kind.into(), t.gl_tex);
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
        self.cache.restore_texture_binding(0);
    }

    pub fn gl_texture_set_min_filter(
        &mut self,
        texture: TextureId,
        filter: FilterMode,
        mipmap_filter: MipmapFilterMode,
    ) {
        let t = self.textures.get(texture);

        self.cache.store_texture_binding(0);
        self.cache.bind_texture(0, t.params.kind.into(), t.gl_tex);

        let filter = Texture::gl_filter(filter, mipmap_filter);
        unsafe {
            glTexParameteri(t.params.kind.into(), GL_TEXTURE_MIN_FILTER, filter as i32);
        }
        self.cache.restore_texture_binding(0);
    }

    pub fn gl_texture_set_mag_filter(&mut self, texture: TextureId, filter: FilterMode) {
        let t = self.textures.get(texture);

        self.cache.store_texture_binding(0);
        self.cache.bind_texture(0, t.params.kind.into(), t.gl_tex);

        let filter = match filter {
            FilterMode::Nearest => GL_NEAREST,
            FilterMode::Linear => GL_LINEAR,
        };
        unsafe {
            glTexParameteri(t.params.kind.into(), GL_TEXTURE_MAG_FILTER, filter as i32);
        }
        self.cache.restore_texture_binding(0);
    }

    pub fn gl_texture_resize(
        &mut self,
        texture: TextureId,
        width: u32,
        height: u32,
        source: Option<&[u8]>,
    ) {
        let mut t = self.textures.get(texture);
        t.resize(self, width, height, source);
        if let TextureIdInner::Managed(tex_id) = texture.0 {
            self.textures.0[tex_id].params = t.params;
        };
    }

    pub fn gl_texture_read_pixels(&mut self, texture: TextureId, source: &mut [u8]) {
        let t = self.textures.get(texture);
        t.read_pixels(source);
    }

    pub fn gl_texture_generate_mipmaps(&mut self, texture: TextureId) {
        let t = self.textures.get(texture);

        self.cache.store_texture_binding(0);
        self.cache.bind_texture(0, t.params.kind.into(), t.gl_tex);
        unsafe {
            glGenerateMipmap(t.params.kind.into());
        }
        self.cache.restore_texture_binding(0);
    }

    pub fn gl_texture_update_part(
        &mut self,
        texture: TextureId,
        x_offset: i32,
        y_offset: i32,
        width: i32,
        height: i32,
        source: &[u8],
    ) {
        let t = self.textures.get(texture);
        t.update_texture_part(self, x_offset, y_offset, width, height, source);
    }

    pub fn gl_texture_params(&self, texture: TextureId) -> TextureParams {
        let texture = self.textures.get(texture);
        texture.params
    }

    pub unsafe fn gl_texture_raw_id(&self, texture: TextureId) -> RawId {
        let texture = self.textures.get(texture);

        RawId::OpenGl(texture.gl_tex)
    }
}
