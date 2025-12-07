use std::rc::Rc;
use std::{cell::Cell, marker::PhantomData};

use glow::{HasContext, PixelUnpackData};
use image::DynamicImage;
use image::metadata::Orientation;

use crate::graphics::GlContext;

static TARGET_NAME: &str = "gl.texture";

#[derive(Debug, Copy, Clone)]
pub struct TextureParams {
    pub internal_format: TextureFormat,
    pub wrap: TextureWrap,
    pub min_filter: FilterMode,
    pub mag_filter: FilterMode,
}

impl Default for TextureParams {
    fn default() -> Self {
        TextureParams {
            internal_format: TextureFormat::RGBA8,
            wrap: TextureWrap::Clamp,
            min_filter: FilterMode::Linear,
            mag_filter: FilterMode::Linear,
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
    pub fn new_empty(
        ctx: Rc<GlContext>,
        width: u32,
        height: u32,
        params: TextureParams,
    ) -> Texture {
        let (internal_format, format, pixel_type) = gl_texture_format(params.internal_format);
        let gl_tex = create_and_bind_texture(&ctx, &params);
        unsafe {
            ctx.gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                internal_format as i32,
                width as i32,
                height as i32,
                0,
                format,
                pixel_type,
                glow::PixelUnpackData::Slice(None),
            );
        }
        apply_texture_parameters(&ctx, &params);
        Texture {
            ctx,
            gl_tex,
            width: Cell::new(width),
            height: Cell::new(height),
            format: params.internal_format,
        }
    }

    pub fn new(
        ctx: Rc<GlContext>,
        source: impl Into<DynamicImage>,
        params: TextureParams,
    ) -> Texture {
        let mut source = source.into();
        // OpenGL is expecting the image data to be upside down.
        //
        // REF: https://registry.khronos.org/OpenGL-Refpages/gl4/html/glTexImage2D.xhtml
        source.apply_orientation(Orientation::FlipVertical);
        let (format, pixel_type) = match source.color() {
            image::ColorType::Rgb8 => (glow::RGB, glow::UNSIGNED_BYTE),
            image::ColorType::Rgba8 => (glow::RGBA, glow::UNSIGNED_BYTE),
            image::ColorType::L16 => (glow::DEPTH_COMPONENT, glow::UNSIGNED_SHORT),
            _ => unimplemented!("Unsupported image input"),
        };

        let (internal_format, _, _) = gl_texture_format(params.internal_format);
        let pixels = source.as_bytes();
        let (width, height) = (source.width(), source.height());
        let gl_tex = create_and_bind_texture(&ctx, &params);
        unsafe {
            ctx.gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                internal_format as i32,
                width as i32,
                height as i32,
                0,
                format,
                pixel_type,
                PixelUnpackData::Slice(Some(pixels)),
            );
        }
        apply_texture_parameters(&ctx, &params);
        ctx.check_no_gl_error();
        Texture {
            ctx,
            gl_tex,
            width: Cell::new(width),
            height: Cell::new(height),
            format: params.internal_format,
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

    pub fn set_min_filter(&self, filter: FilterMode) {
        let mut cache = self.ctx.cache.borrow_mut();
        cache.bind_texture(&self.ctx.gl, 0, glow::TEXTURE_2D, self.gl_tex);
        let filter = gl_filter(filter);
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
        let filter = gl_filter(filter);
        unsafe {
            self.ctx.gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                filter as i32,
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

fn create_and_bind_texture(ctx: &GlContext, params: &TextureParams) -> glow::Texture {
    let mut cache = ctx.cache.borrow_mut();
    let gl_tex = unsafe { ctx.gl.create_texture().unwrap() };
    tracing::debug!(
        target: TARGET_NAME,
        params=?params,
        "new: {gl_tex:?}",
    );
    cache.bind_texture(&ctx.gl, 0, glow::TEXTURE_2D, gl_tex);
    unsafe {
        ctx.gl.pixel_store_i32(glow::PACK_ALIGNMENT, 1); // miniquad always uses row alignment of 1
    }
    gl_tex
}

fn apply_texture_parameters(ctx: &GlContext, params: &TextureParams) {
    let wrap = match params.wrap {
        TextureWrap::Repeat => glow::REPEAT,
        TextureWrap::Mirror => glow::MIRRORED_REPEAT,
        TextureWrap::Clamp => glow::CLAMP_TO_EDGE,
    };
    let min_filter = gl_filter(params.min_filter);
    let mag_filter = gl_filter(params.mag_filter);

    unsafe {
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
}

fn gl_filter(filter: FilterMode) -> u32 {
    match filter {
        FilterMode::Nearest => glow::NEAREST,
        FilterMode::Linear => glow::LINEAR,
    }
}

fn gl_texture_format(format: TextureFormat) -> (u32, u32, u32) {
    match format {
        TextureFormat::RGB8 => (glow::RGB, glow::RGB, glow::UNSIGNED_BYTE),
        TextureFormat::RGBA8 => (glow::RGBA, glow::RGBA, glow::UNSIGNED_BYTE),
        TextureFormat::DepthU16 => (
            glow::DEPTH_COMPONENT16,
            glow::DEPTH_COMPONENT,
            glow::UNSIGNED_SHORT,
        ),
        TextureFormat::DepthF32 => (glow::DEPTH_COMPONENT32F, glow::DEPTH_COMPONENT, glow::FLOAT),
    }
}
