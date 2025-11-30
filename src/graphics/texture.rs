use std::num::NonZeroU32;
use std::rc::Rc;
use std::{cell::Cell, marker::PhantomData};

use glow::{HasContext, NativeFramebuffer, PixelPackData, PixelUnpackData};

use crate::graphics::GlContext;

#[derive(Debug, Copy, Clone)]
pub struct TextureParams {
    pub format: TextureFormat,
    pub wrap: TextureWrap,
    pub min_filter: FilterMode,
    pub mag_filter: FilterMode,
    pub mipmap_filter: MipmapFilterMode,
    pub width: u32,
    pub height: u32,
    pub allocate_mipmaps: bool,
}

impl Default for TextureParams {
    fn default() -> Self {
        TextureParams {
            format: TextureFormat::RGBA8,
            wrap: TextureWrap::Clamp,
            min_filter: FilterMode::Linear,
            mag_filter: FilterMode::Linear,
            mipmap_filter: MipmapFilterMode::None,
            width: 0,
            height: 0,
            allocate_mipmaps: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Texture {
    internal: Rc<TextureInternal>,
    pub(crate) gl_tex: glow::Texture,
}

impl Texture {
    pub fn new(ctx: Rc<GlContext>, source: TextureSource, params: TextureParams) -> Texture {
        if let TextureSource::Bytes(bytes_data) = source {
            assert_eq!(
                params.format.size(params.width, params.height) as usize,
                bytes_data.len()
            );
        }
        let (internal_format, format, pixel_type) = gl_texture_format(params.format);
        let wrap = match params.wrap {
            TextureWrap::Repeat => glow::REPEAT,
            TextureWrap::Mirror => glow::MIRRORED_REPEAT,
            TextureWrap::Clamp => glow::CLAMP_TO_EDGE,
        };
        let min_filter = gl_filter(params.min_filter, params.mipmap_filter);
        let mag_filter = match params.mag_filter {
            FilterMode::Nearest => glow::NEAREST,
            FilterMode::Linear => glow::LINEAR,
        };
        let tex_internal = TextureInternal::new(ctx, params);
        let mut cache = tex_internal.ctx.cache.borrow_mut();
        let gl = &tex_internal.ctx.gl;

        cache.bind_texture(gl, 0, glow::TEXTURE_2D, tex_internal.gl_tex);
        unsafe {
            gl.pixel_store_i32(glow::PACK_ALIGNMENT, 1); // miniquad always uses row alignment of 1

            if cfg!(not(target_arch = "wasm32")) {
                // if not WASM
                if params.format == TextureFormat::Alpha {
                    // if alpha miniquad texture, the value on non-WASM is stored in red channel
                    // swizzle red -> alpha
                    gl.tex_parameter_i32(
                        glow::TEXTURE_2D,
                        glow::TEXTURE_SWIZZLE_A,
                        glow::RED as i32,
                    );
                } else {
                    // keep alpha -> alpha
                    gl.tex_parameter_i32(
                        glow::TEXTURE_2D,
                        glow::TEXTURE_SWIZZLE_A,
                        glow::ALPHA as i32,
                    );
                }
            }

            match source {
                TextureSource::Empty => {
                    // not quite sure if glTexImage2D(null) is really a requirement
                    // but it was like this for quite a while and apparantly it works?
                    gl.tex_image_2d(
                        glow::TEXTURE_2D,
                        0,
                        internal_format as i32,
                        params.width as i32,
                        params.height as i32,
                        0,
                        format,
                        pixel_type,
                        glow::PixelUnpackData::Slice(None),
                    );
                }
                TextureSource::Bytes(source) => {
                    gl.tex_image_2d(
                        glow::TEXTURE_2D,
                        0,
                        internal_format as i32,
                        params.width as i32,
                        params.height as i32,
                        0,
                        format,
                        pixel_type,
                        glow::PixelUnpackData::Slice(Some(source)),
                    );
                }
            }

            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, wrap as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, wrap as i32);
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                min_filter as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                mag_filter as i32,
            );
        }

        std::mem::drop(cache);
        Texture {
            gl_tex: tex_internal.gl_tex,
            internal: Rc::new(tex_internal),
        }
    }

    pub fn resize(&self, width: u32, height: u32, source: Option<&[u8]>) {
        let mut cache = self.internal.ctx.cache.borrow_mut();
        let gl = &self.internal.ctx.gl;

        cache.bind_texture(gl, 0, glow::TEXTURE_2D, self.gl_tex);
        let (internal_format, format, pixel_type) = gl_texture_format(self.internal.format);
        self.internal.width.set(width);
        self.internal.height.set(height);
        unsafe {
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                internal_format as i32,
                width as i32,
                height as i32,
                0,
                format,
                pixel_type,
                glow::PixelUnpackData::Slice(source),
            );
        }
    }

    pub fn set_wrap(&self, wrap_x: TextureWrap, wrap_y: TextureWrap) {
        let gl = &self.internal.ctx.gl;
        let mut cache = self.internal.ctx.cache.borrow_mut();

        cache.bind_texture(gl, 0, glow::TEXTURE_2D, self.gl_tex);
        let wrap_x = match wrap_x {
            TextureWrap::Repeat => glow::REPEAT,
            TextureWrap::Mirror => glow::MIRRORED_REPEAT,
            TextureWrap::Clamp => glow::CLAMP_TO_EDGE,
        };

        let wrap_y = match wrap_y {
            TextureWrap::Repeat => glow::REPEAT,
            TextureWrap::Mirror => glow::MIRRORED_REPEAT,
            TextureWrap::Clamp => glow::CLAMP_TO_EDGE,
        };

        unsafe {
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, wrap_x as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, wrap_y as i32);
        }
    }

    pub fn set_min_filter(&self, filter: FilterMode, mipmap_filter: MipmapFilterMode) {
        let gl = &self.internal.ctx.gl;
        let mut cache = self.internal.ctx.cache.borrow_mut();

        cache.bind_texture(gl, 0, glow::TEXTURE_2D, self.gl_tex);
        let filter = gl_filter(filter, mipmap_filter);
        unsafe {
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, filter as i32);
        }
    }

    pub fn set_mag_filter(&mut self, filter: FilterMode) {
        let gl = &self.internal.ctx.gl;
        let mut cache = self.internal.ctx.cache.borrow_mut();

        cache.bind_texture(gl, 0, glow::TEXTURE_2D, self.gl_tex);
        let filter = match filter {
            FilterMode::Nearest => glow::NEAREST,
            FilterMode::Linear => glow::LINEAR,
        };
        unsafe {
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, filter as i32);
        }
    }

    pub fn update_part(
        &self,
        x_offset: i32,
        y_offset: i32,
        width: i32,
        height: i32,
        source: &[u8],
    ) {
        let gl = &self.internal.ctx.gl;
        let mut cache = self.internal.ctx.cache.borrow_mut();

        assert_eq!(self.size(width as _, height as _), source.len());
        assert!(x_offset + width <= self.width() as _);
        assert!(y_offset + height <= self.height() as _);

        cache.bind_texture(gl, 0, glow::TEXTURE_2D, self.gl_tex);
        let (_, format, pixel_type) = gl_texture_format(self.internal.format);

        unsafe {
            gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 1); // miniquad always uses row alignment of 1

            if cfg!(not(target_arch = "wasm32")) {
                // if not WASM
                if self.internal.format == TextureFormat::Alpha {
                    // if alpha miniquad texture, the value on non-WASM is stored in red channel
                    // swizzle red -> alpha
                    gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_SWIZZLE_A, glow::RED as _);
                } else {
                    // keep alpha -> alpha
                    gl.tex_parameter_i32(
                        glow::TEXTURE_2D,
                        glow::TEXTURE_SWIZZLE_A,
                        glow::ALPHA as _,
                    );
                }
            }

            gl.tex_sub_image_2d(
                glow::TEXTURE_2D,
                0,
                x_offset as _,
                y_offset as _,
                width as _,
                height as _,
                format,
                pixel_type,
                PixelUnpackData::Slice(Some(source)),
            );
        }
    }

    /// Read texture data into CPU memory
    pub fn read_pixels(&self, bytes: &mut [u8]) {
        let gl = &self.internal.ctx.gl;
        let (_, format, pixel_type) = gl_texture_format(self.internal.format);
        assert_eq!(bytes.len() as u32, self.width() * self.height());
        unsafe {
            let curr_fbo = gl.get_parameter_i32(glow::DRAW_FRAMEBUFFER_BINDING);
            let curr_fbo = NativeFramebuffer(NonZeroU32::new(curr_fbo as u32).unwrap());
            let temp_fbo = gl.create_framebuffer().unwrap();

            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(temp_fbo));
            gl.framebuffer_texture_2d(
                glow::FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0,
                glow::TEXTURE_2D,
                Some(self.gl_tex),
                0,
            );
            gl.read_pixels(
                0,
                0,
                self.width() as _,
                self.height() as _,
                format,
                pixel_type,
                PixelPackData::Slice(Some(bytes)),
            );

            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(curr_fbo));
            gl.delete_framebuffer(temp_fbo);
        }
    }

    pub fn size(&self, width: u32, height: u32) -> usize {
        self.internal.format.size(width, height) as usize
    }

    pub fn generate_mipmaps(&self) {
        let gl = &self.internal.ctx.gl;
        let mut cache = self.internal.ctx.cache.borrow_mut();

        cache.bind_texture(gl, 0, glow::TEXTURE_2D, self.gl_tex);
        unsafe {
            gl.generate_mipmap(glow::TEXTURE_2D);
        }
    }

    pub fn width(&self) -> u32 {
        self.internal.width.get()
    }

    pub fn height(&self) -> u32 {
        self.internal.height.get()
    }

    pub fn bind(&self) -> TextureBinding<'_> {
        TextureBinding {
            gl_tex: self.gl_tex,
            _phantom: PhantomData,
        }
    }
}

#[derive(Debug)]
struct TextureInternal {
    ctx: Rc<GlContext>,
    gl_tex: glow::Texture,
    width: Cell<u32>,
    height: Cell<u32>,
    format: TextureFormat,
}

impl TextureInternal {
    pub fn new(ctx: Rc<GlContext>, params: TextureParams) -> TextureInternal {
        let gl_tex = unsafe { ctx.gl.create_texture().unwrap() };
        TextureInternal {
            ctx,
            gl_tex,
            width: Cell::new(params.width),
            height: Cell::new(params.height),
            format: params.format,
        }
    }
}

impl Drop for TextureInternal {
    fn drop(&mut self) {
        unsafe {
            self.ctx.gl.delete_texture(self.gl_tex);
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TextureBinding<'a> {
    pub(crate) gl_tex: glow::Texture,
    _phantom: PhantomData<&'a TextureInternal>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum TextureFormat {
    RGB8,
    RGBA8,
    RGBA16F,
    Depth,
    Depth32,
    Alpha,
}

impl TextureFormat {
    /// Returns the size in bytes of texture with `dimensions`.
    pub fn size(self, width: u32, height: u32) -> u32 {
        let square = width * height;
        match self {
            TextureFormat::RGB8 => 3 * square,
            TextureFormat::RGBA8 => 4 * square,
            TextureFormat::RGBA16F => 8 * square,
            TextureFormat::Depth => 2 * square,
            TextureFormat::Depth32 => 4 * square,
            TextureFormat::Alpha => square,
        }
    }
}

/// Sets the wrap parameter for texture.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TextureWrap {
    /// Samples at coord x + 1 map to coord x.
    Repeat,
    /// Samples at coord x + 1 map to coord 1 - x.
    Mirror,
    /// Samples at coord x + 1 map to coord 1.
    Clamp,
}

#[derive(Clone, Copy, Debug, PartialEq, Hash)]
pub enum FilterMode {
    Linear,
    Nearest,
}

#[derive(Clone, Copy, Debug, PartialEq, Hash)]
pub enum MipmapFilterMode {
    None,
    Linear,
    Nearest,
}

pub enum TextureSource<'a> {
    Empty,
    Bytes(&'a [u8]),
}

fn gl_texture_format(format: TextureFormat) -> (u32, u32, u32) {
    match format {
        TextureFormat::RGB8 => (glow::RGB, glow::RGB, glow::UNSIGNED_BYTE),
        TextureFormat::RGBA8 => (glow::RGBA, glow::RGBA, glow::UNSIGNED_BYTE),
        TextureFormat::RGBA16F => (glow::RGBA16F, glow::RGBA, glow::FLOAT),
        TextureFormat::Depth => (
            glow::DEPTH_COMPONENT,
            glow::DEPTH_COMPONENT,
            glow::UNSIGNED_SHORT,
        ),
        TextureFormat::Depth32 => (glow::DEPTH_COMPONENT, glow::DEPTH_COMPONENT, glow::FLOAT),
        #[cfg(target_arch = "wasm32")]
        TextureFormat::Alpha => (glow::ALPHA, glow::ALPHA, glow::UNSIGNED_BYTE),
        #[cfg(not(target_arch = "wasm32"))]
        TextureFormat::Alpha => (glow::R8, glow::RED, glow::UNSIGNED_BYTE), // texture updates will swizzle Red -> Alpha to match WASM
    }
}

fn gl_filter(filter: FilterMode, mipmap_filter: MipmapFilterMode) -> u32 {
    match filter {
        FilterMode::Nearest => match mipmap_filter {
            MipmapFilterMode::None => glow::NEAREST,
            MipmapFilterMode::Nearest => glow::NEAREST_MIPMAP_NEAREST,
            MipmapFilterMode::Linear => glow::NEAREST_MIPMAP_LINEAR,
        },
        FilterMode::Linear => match mipmap_filter {
            MipmapFilterMode::None => glow::LINEAR,
            MipmapFilterMode::Nearest => glow::LINEAR_MIPMAP_NEAREST,
            MipmapFilterMode::Linear => glow::LINEAR_MIPMAP_LINEAR,
        },
    }
}
