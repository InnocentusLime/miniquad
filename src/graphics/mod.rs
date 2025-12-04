mod cache;
mod index_buffer;
mod pipeline;
mod render_pass;
mod texture;
mod vertex_buffer;

use std::{cell::{Cell, RefCell}, rc::Rc};

use bytemuck::Pod;
use cache::GlCache;
use glow::HasContext;
use image::DynamicImage;

pub use index_buffer::*;
pub use pipeline::*;
pub use render_pass::*;
pub use texture::*;
pub use vertex_buffer::*;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PrimitiveType {
    Triangles,
    Lines,
    Points,
}

#[derive(Debug)]
pub struct GlContext {
    pub(crate) client_area_size: Cell<(u32, u32)>,
    pub(crate) gl: glow::Context,
    pub(crate) vao: glow::VertexArray,
    pub(crate) cache: RefCell<GlCache>,
}

impl GlContext {
    pub fn new(gl: glow::Context, client_area_size: (u32, u32)) -> GlContext {
        let vao = unsafe { gl.create_vertex_array().unwrap() };
        unsafe {
            gl.bind_vertex_array(Some(vao));
        }
        GlContext {
            cache: RefCell::new(GlCache::new()),
            client_area_size: Cell::new(client_area_size),
            gl,
            vao,
        }
    }

    pub fn screen_size(&self) -> (u32, u32) {
        self.client_area_size.get()
    }

    pub fn new_empty_vertex_buffer<T: Pod + Default>(
        self: &Rc<Self>,
        usage: BufferUsage,
        size: usize,
    ) -> VertexBuffer<T> {
        VertexBuffer::new_empty(self.clone(), usage, size)
    }

    pub fn new_vertex_buffer<T: Pod + Default>(
        self: &Rc<Self>,
        usage: BufferUsage,
        data: &[T],
    ) -> VertexBuffer<T> {
        VertexBuffer::new(self.clone(), usage, data)
    }

    pub fn new_empty_index_buffer<I: IndexBufferElement>(
        self: &Rc<Self>,
        usage: BufferUsage,
        size: usize,
    ) -> IndexBuffer<I> {
        IndexBuffer::new_empty(self.clone(), usage, size)
    }

    pub fn new_index_buffer<I: IndexBufferElement>(
        self: &Rc<Self>,
        usage: BufferUsage,
        data: &[I],
    ) -> IndexBuffer<I> {
        IndexBuffer::new(self.clone(), usage, data)
    }

    pub fn new_empty_texture(self: &Rc<Self>, width: u32, height: u32, params: TextureParams) -> Texture {
        Texture::new_empty(self.clone(), width, height, params)
    }

    pub fn new_texture(self: &Rc<Self>, source: impl Into<DynamicImage>, params: TextureParams) -> Texture {
        Texture::new(self.clone(), source, params)
    }

    pub fn new_pipeline<S: Into<String>>(
        self: &Rc<Self>,
        vertex_shader_source: &str,
        fragment_shader_source: &str,
        params: PipelineParams,
        attributes: impl IntoIterator<Item = VertexAttribute>,
        uniforms: impl IntoIterator<Item = UniformDesc>,
        image_uniforms: impl IntoIterator<Item = S>,
    ) -> anyhow::Result<Pipeline> {
        Pipeline::new(
            self.clone(),
            vertex_shader_source,
            fragment_shader_source,
            params,
            attributes,
            uniforms,
            image_uniforms,
        )
    }

    pub fn new_render_pass(
        self: &Rc<Self>,
        color_img: Vec<Texture>,
        depth_img: Option<Texture>,
    ) -> RenderPass {
        RenderPass::new(self.clone(), color_img, depth_img)
    }

    pub fn submit_drawcall(&self, drawcall: DrawCall) {
        drawcall.pipeline.apply(
            drawcall.vertex_buffers,
            drawcall.index_buffer,
            drawcall.textures,
            drawcall.uniform_data,
        );

        let offset = drawcall.index_buffer.sz_elem * drawcall.base_element;
        let mode = match drawcall.pipeline.primitive_type() {
            PrimitiveType::Triangles => glow::TRIANGLES,
            PrimitiveType::Lines => glow::LINES,
            PrimitiveType::Points => glow::POINTS,
        };

        unsafe {
            self.gl.draw_elements_instanced(
                mode,
                drawcall.num_elements,
                drawcall.index_buffer.gl_type,
                offset,
                1,
            );
        }
    }
}

impl Drop for GlContext {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_vertex_array(self.vao);
        }
    }
}

pub struct DrawCall<'a> {
    pub pipeline: &'a Pipeline,
    pub base_element: i32,
    pub num_elements: i32,
    pub vertex_buffers: &'a [VertexBufferBinding<'a>],
    pub index_buffer: IndexBufferBinding<'a>,
    pub textures: &'a [TextureBinding<'a>],
    pub uniform_data: &'a [u8],
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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FrontFaceOrder {
    Clockwise,
    CounterClockwise,
}

/// A pixel-wise comparison function.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Comparison {
    Never,
    Less,
    LessOrEqual,
    Greater,
    GreaterOrEqual,
    Equal,
    NotEqual,
}

impl From<Comparison> for u32 {
    fn from(cmp: Comparison) -> Self {
        match cmp {
            Comparison::Never => glow::NEVER,
            Comparison::Less => glow::LESS,
            Comparison::LessOrEqual => glow::LEQUAL,
            Comparison::Greater => glow::GREATER,
            Comparison::GreaterOrEqual => glow::GEQUAL,
            Comparison::Equal => glow::EQUAL,
            Comparison::NotEqual => glow::NOTEQUAL,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CullFace {
    Nothing,
    Front,
    Back,
}

/// Pixel arithmetic description for blending operations.
/// Will be used in an equation:
/// `equation(sfactor * source_color, dfactor * destination_color)`
/// Where source_color is the new pixel color and destination color is color from the destination buffer.
///
/// Example:
///```
///# use miniquad::{BlendState, BlendFactor, BlendValue, Equation};
///BlendState::new(
///    Equation::Add,
///    BlendFactor::Value(BlendValue::SourceAlpha),
///    BlendFactor::OneMinusValue(BlendValue::SourceAlpha)
///);
///```
/// This will be `source_color * source_color.a + destination_color * (1 - source_color.a)`
/// Wich is quite common set up for alpha blending.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct BlendState {
    equation: Equation,
    sfactor: BlendFactor,
    dfactor: BlendFactor,
}

impl BlendState {
    pub fn new(equation: Equation, sfactor: BlendFactor, dfactor: BlendFactor) -> BlendState {
        BlendState {
            equation,
            sfactor,
            dfactor,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct StencilState {
    pub front: StencilFaceState,
    pub back: StencilFaceState,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct StencilFaceState {
    /// Operation to use when stencil test fails
    pub fail_op: StencilOp,

    /// Operation to use when stencil test passes, but depth test fails
    pub depth_fail_op: StencilOp,

    /// Operation to use when both stencil and depth test pass,
    /// or when stencil pass and no depth or depth disabled
    pub pass_op: StencilOp,

    /// Used for stencil testing with test_ref and test_mask: if (test_ref & test_mask) *test_func* (*stencil* && test_mask)
    /// Default is Always, which means "always pass"
    pub test_func: CompareFunc,

    /// Default value: 0
    pub test_ref: i32,

    /// Default value: all 1s
    pub test_mask: u32,

    /// Specifies a bit mask to enable or disable writing of individual bits in the stencil planes
    /// Default value: all 1s
    pub write_mask: u32,
}

/// Operations performed on current stencil value when comparison test passes or fails.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum StencilOp {
    /// Default value
    Keep,
    Zero,
    Replace,
    IncrementClamp,
    DecrementClamp,
    Invert,
    IncrementWrap,
    DecrementWrap,
}

/// Depth and stencil compare function
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum CompareFunc {
    /// Default value
    Always,
    Never,
    Less,
    Equal,
    LessOrEqual,
    Greater,
    NotEqual,
    GreaterOrEqual,
}

type ColorMask = (bool, bool, bool, bool);

/// Specifies how incoming RGBA values (source) and the RGBA in framebuffer (destination)
/// are combined.
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum Equation {
    /// Adds source and destination. Source and destination are multiplied
    /// by blending parameters before addition.
    #[default]
    Add,
    /// Subtracts destination from source. Source and destination are
    /// multiplied by blending parameters before subtraction.
    Subtract,
    /// Subtracts source from destination. Source and destination are
    /// multiplied by blending parameters before subtraction.
    ReverseSubtract,
}

/// Blend values.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BlendValue {
    SourceColor,
    SourceAlpha,
    DestinationColor,
    DestinationAlpha,
}

/// Blend factors.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BlendFactor {
    Zero,
    One,
    Value(BlendValue),
    OneMinusValue(BlendValue),
    SourceAlphaSaturate,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BufferUsage {
    Immutable,
    Dynamic,
    Stream,
}
