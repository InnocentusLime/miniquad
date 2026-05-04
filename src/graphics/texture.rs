use std::rc::Rc;

use glam::{UVec2, uvec2};
use glow::{HasContext, PixelUnpackData};
use image::DynamicImage;
use image::metadata::Orientation;

use crate::check_gl;
use crate::graphics::{Error, GlContext, ImageUniformVal, Result};

static TARGET_NAME: &str = "gl.texture";

#[derive(Debug)]
pub struct Texture2D {
    ctx: Rc<GlContext>,
    pub(crate) gl_tex: glow::Texture,
    width: u32,
    height: u32,
    format: Texture2DFormat,
}

impl Texture2D {
    pub fn new_empty(
        ctx: Rc<GlContext>,
        format: Texture2DFormat,
        width: u32,
        height: u32,
        wrap: TextureWrap,
        min_filter: FilterMode,
        mag_filter: FilterMode,
    ) -> Result<Texture2D> {
        let gl_tex = create_and_bind_texture(
            &ctx,
            format,
            width,
            height,
            wrap,
            min_filter,
            mag_filter,
            glow::PixelUnpackData::Slice(None),
        )?;
        Ok(Texture2D { ctx, gl_tex, width, height, format })
    }

    pub fn new(
        ctx: Rc<GlContext>,
        source: impl Into<DynamicImage>,
        wrap: TextureWrap,
        min_filter: FilterMode,
        mag_filter: FilterMode,
    ) -> Result<Texture2D> {
        let mut source = source.into();
        // OpenGL is expecting the image data to be upside down.
        //
        // REF: https://registry.khronos.org/OpenGL-Refpages/gl4/html/glTexImage2D.xhtml
        source.apply_orientation(Orientation::FlipVertical);
        let format = match source.color() {
            image::ColorType::Rgb8 => Texture2DFormat::RGB8,
            image::ColorType::Rgba8 => Texture2DFormat::RGBA8,
            image::ColorType::L16 => Texture2DFormat::DepthU16,
            format => return Err(Error::UnsupportedTextureFormat { format }),
        };

        let (width, height) = (source.width(), source.height());
        let gl_tex = create_and_bind_texture(
            &ctx,
            format,
            width,
            height,
            wrap,
            min_filter,
            mag_filter,
            PixelUnpackData::Slice(Some(source.as_bytes())),
        )?;
        Ok(Texture2D { ctx, gl_tex, width, height, format })
    }

    pub fn data_size(&self) -> usize {
        self.format.data_size(self.width(), self.height()) as usize
    }

    pub fn size(&self) -> UVec2 {
        uvec2(self.width(), self.height())
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }
}

impl Drop for Texture2D {
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

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Texture2DFormat {
    RGB8,
    RGBA8,
    DepthU16,
    DepthF32,
}

impl Texture2DFormat {
    /// Returns the size in bytes of texture with `dimensions`.
    pub fn data_size(self, width: u32, height: u32) -> u32 {
        let square = width * height;
        match self {
            Texture2DFormat::RGB8 => 3 * square,
            Texture2DFormat::RGBA8 => 4 * square,
            Texture2DFormat::DepthU16 => 2 * square,
            Texture2DFormat::DepthF32 => 4 * square,
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

fn create_and_bind_texture(
    ctx: &GlContext,
    format: Texture2DFormat,
    width: u32,
    height: u32,
    wrap: TextureWrap,
    min_filter: FilterMode,
    mag_filter: FilterMode,
    data: glow::PixelUnpackData,
) -> Result<glow::Texture> {
    let mut cache = ctx.cache.borrow_mut();
    let Ok(gl_tex) = (unsafe { ctx.gl.create_texture() }) else {
        return Err(Error::TextureAlloc);
    };
    tracing::debug!(
        target: TARGET_NAME,
        ?format, width, height,
        "new: {gl_tex:?}",
    );
    cache.bind_texture(&ctx.gl, 0, glow::TEXTURE_2D, gl_tex);
    unsafe {
        ctx.gl.pixel_store_i32(glow::PACK_ALIGNMENT, 1); // miniquad always uses row alignment of 1
    }

    let (gl_internal_format, gl_format, gl_pixel_type) = gl_texture_format(format);
    unsafe {
        ctx.gl.tex_image_2d(
            glow::TEXTURE_2D,
            0,
            gl_internal_format as i32,
            width as i32,
            height as i32,
            0,
            gl_format,
            gl_pixel_type,
            data,
        );
    }

    apply_texture_parameters(ctx, wrap, min_filter, mag_filter);
    check_gl!(&ctx.gl, Error::TextureInit);

    Ok(gl_tex)
}

fn apply_texture_parameters(
    ctx: &GlContext,
    wrap: TextureWrap,
    min_filter: FilterMode,
    mag_filter: FilterMode,
) {
    let wrap = match wrap {
        TextureWrap::Repeat => glow::REPEAT,
        TextureWrap::Mirror => glow::MIRRORED_REPEAT,
        TextureWrap::Clamp => glow::CLAMP_TO_EDGE,
    };
    let min_filter = gl_filter(min_filter);
    let mag_filter = gl_filter(mag_filter);

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

fn gl_texture_format(format: Texture2DFormat) -> (u32, u32, u32) {
    match format {
        Texture2DFormat::RGB8 => (glow::RGB, glow::RGB, glow::UNSIGNED_BYTE),
        Texture2DFormat::RGBA8 => (glow::RGBA, glow::RGBA, glow::UNSIGNED_BYTE),
        Texture2DFormat::DepthU16 => (
            glow::DEPTH_COMPONENT16,
            glow::DEPTH_COMPONENT,
            glow::UNSIGNED_SHORT,
        ),
        Texture2DFormat::DepthF32 => (glow::DEPTH_COMPONENT32F, glow::DEPTH_COMPONENT, glow::FLOAT),
    }
}

impl ImageUniformVal for Texture2D {
    const GL_TYPE: u32 = glow::TEXTURE_2D;

    fn bind(&self, slot: u32) {
        let mut cache = self.ctx.cache.borrow_mut();
        cache.bind_texture(&self.ctx.gl, slot, Self::GL_TYPE, self.gl_tex);
    }
}
