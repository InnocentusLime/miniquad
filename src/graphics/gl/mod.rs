mod buffer;
mod cache;
mod index_buffer;
mod pipeline;
mod render_pass;
mod texture;

use std::cell::{Cell, RefCell};

use glow::HasContext;
use glutin::context::PossiblyCurrentContext;

use super::*;
use cache::*;

pub use buffer::{Buffer, BufferBinding};
pub use index_buffer::{IndexBuffer, IndexBufferElement};
pub use pipeline::Pipeline;
pub use render_pass::RenderPass;
pub use texture::Texture;

#[derive(Debug)]
pub struct GlContext {
    pub(crate) glutin_ctx: PossiblyCurrentContext,
    pub(crate) client_area_size: Cell<(u32, u32)>,
    pub(crate) gl: glow::Context,
    pub(crate) vao: glow::VertexArray,
    pub(crate) cache: RefCell<GlCache>,
}

impl GlContext {
    pub fn new(
        glutin_ctx: PossiblyCurrentContext,
        gl: glow::Context, 
        client_area_size: (u32, u32),
    ) -> GlContext {
        let vao = unsafe { gl.create_vertex_array().unwrap() };
        unsafe {
            gl.bind_vertex_array(Some(vao));
        }
        let cache = GlCache {
            index_buffer: None,
            vertex_buffer: None,
            textures: [None; MAX_SHADERSTAGE_IMAGES],
            cur_pipeline: None,

            color_blend: None,
            alpha_blend: None,
            stencil: None,
            color_write: (true, true, true, true),
            cull_face: CullFace::Nothing,
        };

        GlContext {
            glutin_ctx,
            client_area_size: Cell::new(client_area_size),
            gl,
            vao,
            cache: RefCell::new(cache),
        }
    }

    pub fn screen_size(&self) -> (u32, u32) {
        self.client_area_size.get()
    }
}

impl Drop for GlContext {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_vertex_array(self.vao);
        }
    }
}

pub struct DrawCall<'a, I: IndexBufferElement> {
    pub ctx: &'a GlContext,
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

        let offset = std::mem::size_of::<I>() as i32 * self.base_element;
        let mode = match self.pipeline.primitive_type() {
            PrimitiveType::Triangles => glow::TRIANGLES,
            PrimitiveType::Lines => glow::LINES,
            PrimitiveType::Points => glow::POINTS,
        };

        unsafe {
            self.ctx
                .gl
                .draw_elements_instanced(mode, self.num_elements, I::GL_TYPE, offset, 1);
        }
    }
}

impl From<Equation> for u32 {
    fn from(eq: Equation) -> Self {
        match eq {
            Equation::Add => glow::FUNC_ADD,
            Equation::Subtract => glow::FUNC_SUBTRACT,
            Equation::ReverseSubtract => glow::FUNC_REVERSE_SUBTRACT,
        }
    }
}

impl From<BlendFactor> for u32 {
    fn from(factor: BlendFactor) -> u32 {
        match factor {
            BlendFactor::Zero => glow::ZERO,
            BlendFactor::One => glow::ONE,
            BlendFactor::Value(BlendValue::SourceColor) => glow::SRC_COLOR,
            BlendFactor::Value(BlendValue::SourceAlpha) => glow::SRC_ALPHA,
            BlendFactor::Value(BlendValue::DestinationColor) => glow::DST_COLOR,
            BlendFactor::Value(BlendValue::DestinationAlpha) => glow::DST_ALPHA,
            BlendFactor::OneMinusValue(BlendValue::SourceColor) => glow::ONE_MINUS_SRC_COLOR,
            BlendFactor::OneMinusValue(BlendValue::SourceAlpha) => glow::ONE_MINUS_SRC_ALPHA,
            BlendFactor::OneMinusValue(BlendValue::DestinationColor) => glow::ONE_MINUS_DST_COLOR,
            BlendFactor::OneMinusValue(BlendValue::DestinationAlpha) => glow::ONE_MINUS_DST_ALPHA,
            BlendFactor::SourceAlphaSaturate => glow::SRC_ALPHA_SATURATE,
        }
    }
}

impl From<StencilOp> for u32 {
    fn from(op: StencilOp) -> Self {
        match op {
            StencilOp::Keep => glow::KEEP,
            StencilOp::Zero => glow::ZERO,
            StencilOp::Replace => glow::REPLACE,
            StencilOp::IncrementClamp => glow::INCR,
            StencilOp::DecrementClamp => glow::DECR,
            StencilOp::Invert => glow::INVERT,
            StencilOp::IncrementWrap => glow::INCR_WRAP,
            StencilOp::DecrementWrap => glow::DECR_WRAP,
        }
    }
}

impl From<CompareFunc> for u32 {
    fn from(cf: CompareFunc) -> Self {
        match cf {
            CompareFunc::Always => glow::ALWAYS,
            CompareFunc::Never => glow::NEVER,
            CompareFunc::Less => glow::LESS,
            CompareFunc::Equal => glow::EQUAL,
            CompareFunc::LessOrEqual => glow::LEQUAL,
            CompareFunc::Greater => glow::GREATER,
            CompareFunc::NotEqual => glow::NOTEQUAL,
            CompareFunc::GreaterOrEqual => glow::GEQUAL,
        }
    }
}

fn gl_usage(usage: BufferUsage) -> u32 {
    match usage {
        BufferUsage::Immutable => glow::STATIC_DRAW,
        BufferUsage::Dynamic => glow::DYNAMIC_DRAW,
        BufferUsage::Stream => glow::STREAM_DRAW,
    }
}
