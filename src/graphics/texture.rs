use std::rc::Rc;
use std::{cell::Cell, marker::PhantomData};

use glow::{HasContext, PixelUnpackData};

use crate::graphics::GlContext;

static TARGET_NAME: &str = "gl.texture";

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

#[derive(Debug)]
pub struct Texture {
    ctx: Rc<GlContext>,
    pub(crate) gl_tex: glow::Texture,
    width: Cell<u32>,
    height: Cell<u32>,
    format: TextureFormat,
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

        let gl_tex = unsafe { ctx.gl.create_texture().unwrap() };
        tracing::debug!(
            target: TARGET_NAME,
            params=?params,
            "new: {gl_tex:?}",
        );
        let mut cache = ctx.cache.borrow_mut();
        cache.bind_texture(&ctx.gl, 0, glow::TEXTURE_2D, gl_tex);
        unsafe {
            ctx.gl.pixel_store_i32(glow::PACK_ALIGNMENT, 1); // miniquad always uses row alignment of 1

            match source {
                TextureSource::Empty => {
                    ctx.gl.tex_image_2d(
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
                    ctx.gl.tex_image_2d(
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

            ctx.gl
                .tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, wrap as i32);
            ctx.gl
                .tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, wrap as i32);
            ctx.gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                min_filter as i32,
            );
            ctx.gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                mag_filter as i32,
            );
        }

        std::mem::drop(cache);
        Texture {
            ctx,
            gl_tex,
            width: Cell::new(params.width),
            height: Cell::new(params.height),
            format: params.format,
        }
    }

    pub fn resize(&self, width: u32, height: u32, source: Option<&[u8]>) {
        let mut cache = self.ctx.cache.borrow_mut();
        cache.bind_texture(&self.ctx.gl, 0, glow::TEXTURE_2D, self.gl_tex);
        let (internal_format, format, pixel_type) = gl_texture_format(self.format);
        self.width.set(width);
        self.height.set(height);
        unsafe {
            self.ctx.gl.tex_image_2d(
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
        let mut cache = self.ctx.cache.borrow_mut();
        cache.bind_texture(&self.ctx.gl, 0, glow::TEXTURE_2D, self.gl_tex);
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
            self.ctx
                .gl
                .tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, wrap_x as i32);
            self.ctx
                .gl
                .tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, wrap_y as i32);
        }
    }

    pub fn set_min_filter(&self, filter: FilterMode, mipmap_filter: MipmapFilterMode) {
        let mut cache = self.ctx.cache.borrow_mut();
        cache.bind_texture(&self.ctx.gl, 0, glow::TEXTURE_2D, self.gl_tex);
        let filter = gl_filter(filter, mipmap_filter);
        unsafe {
            self.ctx.gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                filter as i32,
            );
        }
    }

    pub fn set_mag_filter(&mut self, filter: FilterMode) {
        let mut cache = self.ctx.cache.borrow_mut();
        cache.bind_texture(&self.ctx.gl, 0, glow::TEXTURE_2D, self.gl_tex);
        let filter = match filter {
            FilterMode::Nearest => glow::NEAREST,
            FilterMode::Linear => glow::LINEAR,
        };
        unsafe {
            self.ctx.gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                filter as i32,
            );
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
        let mut cache = self.ctx.cache.borrow_mut();
        assert_eq!(self.size(width as _, height as _), source.len());
        assert!(x_offset + width <= self.width() as _);
        assert!(y_offset + height <= self.height() as _);

        cache.bind_texture(&self.ctx.gl, 0, glow::TEXTURE_2D, self.gl_tex);
        let (_, format, pixel_type) = gl_texture_format(self.format);

        unsafe {
            self.ctx.gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 1); // miniquad always uses row alignment of 1
            self.ctx.gl.tex_sub_image_2d(
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

    pub fn size(&self, width: u32, height: u32) -> usize {
        self.format.size(width, height) as usize
    }

    pub fn generate_mipmaps(&self) {
        let mut cache = self.ctx.cache.borrow_mut();
        cache.bind_texture(&self.ctx.gl, 0, glow::TEXTURE_2D, self.gl_tex);
        unsafe {
            self.ctx.gl.generate_mipmap(glow::TEXTURE_2D);
        }
    }

    pub fn width(&self) -> u32 {
        self.width.get()
    }

    pub fn height(&self) -> u32 {
        self.height.get()
    }

    pub fn bind(&self) -> TextureBinding<'_> {
        TextureBinding {
            gl_tex: self.gl_tex,
            _phantom: PhantomData,
        }
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        tracing::debug!(
            target: TARGET_NAME,
            "dropping: {:?}",
            self.gl_tex,
        );
        unsafe {
            self.ctx.gl.delete_texture(self.gl_tex);
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TextureBinding<'a> {
    pub(crate) gl_tex: glow::Texture,
    _phantom: PhantomData<&'a glow::Texture>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum TextureFormat {
    RGB8,
    RGBA8,
    RGBAF16,
    DepthU16,
    DepthF32,
}

impl TextureFormat {
    /// Returns the size in bytes of texture with `dimensions`.
    pub fn size(self, width: u32, height: u32) -> u32 {
        let square = width * height;
        match self {
            TextureFormat::RGB8 => 3 * square,
            TextureFormat::RGBA8 => 4 * square,
            TextureFormat::RGBAF16 => 8 * square,
            TextureFormat::DepthU16 => 2 * square,
            TextureFormat::DepthF32 => 4 * square,
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
    // Depth textures are a special case when it comes to OpenGL vs WebGL.
    // In OpenGL GL_DEPTH_COMPONENT is the ONLY valid internal format for
    // value for depth textures.
    // In WebGL and OpenGL ES that is not true and the call must specify
    // a SIZED value (e.g. GL_DEPTH_COMPONENT16 or GL_DEPTH_COMPONENT32F).
    //
    // NOTE:
    // This is still imperfect. If we run on a native platform with
    // a OpenGL ES context -- the code will most like not work.
    //
    // REF:
    // * OpenGL: https://registry.khronos.org/OpenGL-Refpages/gl4/html/glTexImage2D.xhtml
    // * OpenGL ES: https://registry.khronos.org/OpenGL-Refpages/es3.0/html/glTexImage2D.xhtml

    #[cfg(not(target_family = "wasm"))]
    const DEPTH_U16_INTERNAL_FORMAT: u32 = glow::DEPTH_COMPONENT;
    #[cfg(target_family = "wasm")]
    const DEPTH_U16_INTERNAL_FORMAT: u32 = glow::DEPTH_COMPONENT16;
    #[cfg(not(target_family = "wasm"))]
    const DEPTH_F32_INTERNAL_FORMAT: u32 = glow::DEPTH_COMPONENT;
    #[cfg(target_family = "wasm")]
    const DEPTH_F32_INTERNAL_FORMAT: u32 = glow::DEPTH_COMPONENT32F;

    match format {
        TextureFormat::RGB8 => (glow::RGB, glow::RGB, glow::UNSIGNED_BYTE),
        TextureFormat::RGBA8 => (glow::RGBA, glow::RGBA, glow::UNSIGNED_BYTE),
        TextureFormat::RGBAF16 => (glow::RGBA16F, glow::RGBA, glow::FLOAT),
        TextureFormat::DepthU16 => (
            DEPTH_U16_INTERNAL_FORMAT,
            glow::DEPTH_COMPONENT,
            glow::UNSIGNED_SHORT,
        ),
        TextureFormat::DepthF32 => (
            DEPTH_F32_INTERNAL_FORMAT,
            glow::DEPTH_COMPONENT,
            glow::FLOAT,
        ),
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
