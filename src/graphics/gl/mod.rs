mod buffer;
mod cache;
mod index_buffer;
mod pipeline;
mod render_pass;
mod texture;

use std::cell::RefCell;

use super::*;
use cache::*;

pub use buffer::{Buffer, BufferBinding};
pub use index_buffer::{IndexBuffer, IndexBufferElement};
pub use pipeline::Pipeline;
pub use render_pass::RenderPass;
pub use texture::Texture;

/// Raw OpenGL bindings
/// Highly unsafe, some of the functions could be missing due to incompatible GL version
/// or all of them might be missing alltogether if rendering context is not a GL one.
pub mod raw_gl {
    use super::*;

    #[doc(inline)]
    pub use crate::native::gl::*;

    pub fn texture_format_into_gl(format: TextureFormat) -> (GLenum, GLenum, GLenum) {
        format.into()
    }
}

pub struct GlContext {
    pub(crate) cache: RefCell<GlCache>,
    pub(crate) info: ContextInfo,
}

impl Default for GlContext {
    fn default() -> Self {
        Self::new()
    }
}

impl GlContext {
    pub fn new() -> GlContext {
        unsafe {
            let mut vao = 0;

            glGenVertexArrays(1, &mut vao as *mut _);
            glBindVertexArray(vao);
            let info = gl_info();
            let cache = GlCache {
                stored_index_buffer: 0,
                stored_index_size: 0,
                stored_vertex_buffer: 0,
                index_buffer: 0,
                index_size: 0,
                vertex_buffer: 0,
                cur_pipeline: None,
                color_blend: None,
                alpha_blend: None,
                stencil: None,
                color_write: (true, true, true, true),
                cull_face: CullFace::Nothing,
                stored_texture: 0,
                stored_target: 0,
                textures: [CachedTexture {
                    target: 0,
                    texture: 0,
                }; MAX_SHADERSTAGE_IMAGES],
                attributes: [None; MAX_VERTEX_ATTRIBUTES],
            };

            GlContext {
                info,
                cache: RefCell::new(cache),
            }
        }
    }

    pub fn features(&self) -> &Features {
        &self.info.features
    }
}

pub struct DrawCall<'a, I: IndexBufferElement> {
    pub pipeline: &'a Pipeline,
    pub base_element: i32,
    pub num_elements: i32,
    pub vertex_buffers: &'a [BufferBinding],
    pub index_buffer: &'a IndexBuffer<I>,
    pub textures: &'a [&'a Texture],
    pub uniform_data: &'a [u8],
}

impl<'a, I: IndexBufferElement> DrawCall<'a, I> {
    pub fn execute(self) {
        self.pipeline.apply(
            self.vertex_buffers,
            self.index_buffer,
            self.textures,
            self.uniform_data,
        );

        let indices = std::mem::size_of::<I>() as i32 * self.base_element;
        let primitive_type = self.pipeline.primitive_type().into();

        unsafe {
            glDrawElementsInstanced(
                primitive_type,
                self.num_elements,
                I::GL_TYPE,
                indices as *mut _,
                1,
            );
        }
    }
}

impl RenderingBackend for GlContext {
    fn info(&self) -> ContextInfo {
        self.info.clone()
    }
}

#[allow(clippy::field_reassign_with_default)]
fn gl_info() -> ContextInfo {
    let version_string = unsafe { glGetString(super::gl::GL_VERSION) };
    let gl_version_string = unsafe { std::ffi::CStr::from_ptr(version_string as _) }
        .to_str()
        .unwrap()
        .to_string();
    //let gles2 = !gles3 && gl_version_string.contains("OpenGL ES");

    let gl2 = gl_version_string.is_empty()
        || gl_version_string.starts_with("2")
        || gl_version_string.starts_with("OpenGL ES 2");
    let webgl1 = gl_version_string == "WebGL 1.0";

    let features = Features {
        instancing: !gl2,
        resolve_attachments: !webgl1 && !gl2,
    };

    let mut glsl_support = GlslSupport::default();

    // this is not quite documented,
    // but somehow even GL2.1 usually have all the compatibility extensions to support glsl100
    // It was tested on really old windows machines, virtual machines etc. glsl100 always works!
    glsl_support.v100 = true;

    // on wasm miniquad always creates webgl1 context, with the only glsl available being version 100
    #[cfg(target_arch = "wasm32")]
    {
        // on web, miniquad always loads EXT_shader_texture_lod and OES_standard_derivatives
        glsl_support.v100_ext = true;

        let webgl2 = gl_version_string.contains("WebGL 2.0");
        if webgl2 {
            glsl_support.v300es = true;
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let gles3 = gl_version_string.contains("OpenGL ES 3");

        if gles3 {
            glsl_support.v300es = true;
        }
    }

    // there is no gl3.4, so 4+ and 3.3 covers all modern OpenGL
    if gl_version_string.starts_with("3.2") {
        glsl_support.v150 = true; // MacOS is defaulting to 3.2 and GLSL 150
    } else if gl_version_string.starts_with("4") || gl_version_string.starts_with("3.3") {
        glsl_support.v330 = true;
    // gl 3.0, 3.1, 3.2 maps to 1.30, 1.40, 1.50 glsl versions
    } else if gl_version_string.starts_with("3") {
        glsl_support.v130 = true;
    }

    ContextInfo {
        gl_version_string,
        glsl_support,
        features,
    }
}

impl From<Equation> for GLenum {
    fn from(eq: Equation) -> Self {
        match eq {
            Equation::Add => GL_FUNC_ADD,
            Equation::Subtract => GL_FUNC_SUBTRACT,
            Equation::ReverseSubtract => GL_FUNC_REVERSE_SUBTRACT,
        }
    }
}

impl From<BlendFactor> for GLenum {
    fn from(factor: BlendFactor) -> GLenum {
        match factor {
            BlendFactor::Zero => GL_ZERO,
            BlendFactor::One => GL_ONE,
            BlendFactor::Value(BlendValue::SourceColor) => GL_SRC_COLOR,
            BlendFactor::Value(BlendValue::SourceAlpha) => GL_SRC_ALPHA,
            BlendFactor::Value(BlendValue::DestinationColor) => GL_DST_COLOR,
            BlendFactor::Value(BlendValue::DestinationAlpha) => GL_DST_ALPHA,
            BlendFactor::OneMinusValue(BlendValue::SourceColor) => GL_ONE_MINUS_SRC_COLOR,
            BlendFactor::OneMinusValue(BlendValue::SourceAlpha) => GL_ONE_MINUS_SRC_ALPHA,
            BlendFactor::OneMinusValue(BlendValue::DestinationColor) => GL_ONE_MINUS_DST_COLOR,
            BlendFactor::OneMinusValue(BlendValue::DestinationAlpha) => GL_ONE_MINUS_DST_ALPHA,
            BlendFactor::SourceAlphaSaturate => GL_SRC_ALPHA_SATURATE,
        }
    }
}

impl From<StencilOp> for GLenum {
    fn from(op: StencilOp) -> Self {
        match op {
            StencilOp::Keep => GL_KEEP,
            StencilOp::Zero => GL_ZERO,
            StencilOp::Replace => GL_REPLACE,
            StencilOp::IncrementClamp => GL_INCR,
            StencilOp::DecrementClamp => GL_DECR,
            StencilOp::Invert => GL_INVERT,
            StencilOp::IncrementWrap => GL_INCR_WRAP,
            StencilOp::DecrementWrap => GL_DECR_WRAP,
        }
    }
}

impl From<CompareFunc> for GLenum {
    fn from(cf: CompareFunc) -> Self {
        match cf {
            CompareFunc::Always => GL_ALWAYS,
            CompareFunc::Never => GL_NEVER,
            CompareFunc::Less => GL_LESS,
            CompareFunc::Equal => GL_EQUAL,
            CompareFunc::LessOrEqual => GL_LEQUAL,
            CompareFunc::Greater => GL_GREATER,
            CompareFunc::NotEqual => GL_NOTEQUAL,
            CompareFunc::GreaterOrEqual => GL_GEQUAL,
        }
    }
}
