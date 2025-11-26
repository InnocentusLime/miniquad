use std::{error::Error, fmt::Display};

mod gl;

pub use gl::*;

#[derive(Clone, Copy, Debug)]
pub enum UniformType {
    /// One 32-bit wide float (equivalent to `f32`)
    Float1,
    /// Two 32-bit wide floats (equivalent to `[f32; 2]`)
    Float2,
    /// Three 32-bit wide floats (equivalent to `[f32; 3]`)
    Float3,
    /// Four 32-bit wide floats (equivalent to `[f32; 4]`)
    Float4,
    /// One unsigned 32-bit integers (equivalent to `[u32; 1]`)
    Int1,
    /// Two unsigned 32-bit integers (equivalent to `[u32; 2]`)
    Int2,
    /// Three unsigned 32-bit integers (equivalent to `[u32; 3]`)
    Int3,
    /// Four unsigned 32-bit integers (equivalent to `[u32; 4]`)
    Int4,
    /// Four by four matrix of 32-bit floats
    Mat4,
}

impl UniformType {
    /// Byte size for a given UniformType
    pub fn size(&self) -> usize {
        match self {
            UniformType::Float1 => 4,
            UniformType::Float2 => 8,
            UniformType::Float3 => 12,
            UniformType::Float4 => 16,
            UniformType::Int1 => 4,
            UniformType::Int2 => 8,
            UniformType::Int3 => 12,
            UniformType::Int4 => 16,
            UniformType::Mat4 => 64,
        }
    }
}

#[derive(Debug, Clone)]
pub struct UniformDesc {
    pub name: String,
    pub uniform_type: UniformType,
    pub array_count: usize,
}

impl UniformDesc {
    pub fn new(name: &str, uniform_type: UniformType) -> UniformDesc {
        UniformDesc {
            name: name.to_string(),
            uniform_type,
            array_count: 1,
        }
    }

    pub fn array(self, array_count: usize) -> UniformDesc {
        UniformDesc {
            array_count,
            ..self
        }
    }
}

#[derive(Clone)]
pub struct ShaderMeta {
    pub uniforms: Vec<UniformDesc>,
    pub images: Vec<String>,
    pub attributes: Vec<VertexAttribute>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum VertexFormat {
    /// One 32-bit wide float (equivalent to `f32`)
    Float1,
    /// Two 32-bit wide floats (equivalent to `[f32; 2]`)
    Float2,
    /// Three 32-bit wide floats (equivalent to `[f32; 3]`)
    Float3,
    /// Four 32-bit wide floats (equivalent to `[f32; 4]`)
    Float4,
    /// One unsigned 8-bit integer (equivalent to `u8`)
    Byte1,
    /// Two unsigned 8-bit integers (equivalent to `[u8; 2]`)
    Byte2,
    /// Three unsigned 8-bit integers (equivalent to `[u8; 3]`)
    Byte3,
    /// Four unsigned 8-bit integers (equivalent to `[u8; 4]`)
    Byte4,
    /// One unsigned 16-bit integer (equivalent to `u16`)
    Short1,
    /// Two unsigned 16-bit integers (equivalent to `[u16; 2]`)
    Short2,
    /// Tree unsigned 16-bit integers (equivalent to `[u16; 3]`)
    Short3,
    /// Four unsigned 16-bit integers (equivalent to `[u16; 4]`)
    Short4,
    /// One unsigned 32-bit integers (equivalent to `[u32; 1]`)
    Int1,
    /// Two unsigned 32-bit integers (equivalent to `[u32; 2]`)
    Int2,
    /// Three unsigned 32-bit integers (equivalent to `[u32; 3]`)
    Int3,
    /// Four unsigned 32-bit integers (equivalent to `[u32; 4]`)
    Int4,
}

impl VertexFormat {
    /// Number of components in this VertexFormat
    /// it is called size in OpenGl, but do not confuse this with bytes size
    /// basically, its an N from FloatN/IntN
    pub fn components(&self) -> i32 {
        match self {
            VertexFormat::Float1 => 1,
            VertexFormat::Float2 => 2,
            VertexFormat::Float3 => 3,
            VertexFormat::Float4 => 4,
            VertexFormat::Byte1 => 1,
            VertexFormat::Byte2 => 2,
            VertexFormat::Byte3 => 3,
            VertexFormat::Byte4 => 4,
            VertexFormat::Short1 => 1,
            VertexFormat::Short2 => 2,
            VertexFormat::Short3 => 3,
            VertexFormat::Short4 => 4,
            VertexFormat::Int1 => 1,
            VertexFormat::Int2 => 2,
            VertexFormat::Int3 => 3,
            VertexFormat::Int4 => 4,
        }
    }

    /// Size in bytes
    pub fn size_bytes(&self) -> i32 {
        match self {
            VertexFormat::Float1 => 1 * 4,
            VertexFormat::Float2 => 2 * 4,
            VertexFormat::Float3 => 3 * 4,
            VertexFormat::Float4 => 4 * 4,
            VertexFormat::Byte1 => 1,
            VertexFormat::Byte2 => 2,
            VertexFormat::Byte3 => 3,
            VertexFormat::Byte4 => 4,
            VertexFormat::Short1 => 1 * 2,
            VertexFormat::Short2 => 2 * 2,
            VertexFormat::Short3 => 3 * 2,
            VertexFormat::Short4 => 4 * 2,
            VertexFormat::Int1 => 1 * 4,
            VertexFormat::Int2 => 2 * 4,
            VertexFormat::Int3 => 3 * 4,
            VertexFormat::Int4 => 4 * 4,
        }
    }

    fn type_(&self) -> u32 {
        match self {
            VertexFormat::Float1 => glow::FLOAT,
            VertexFormat::Float2 => glow::FLOAT,
            VertexFormat::Float3 => glow::FLOAT,
            VertexFormat::Float4 => glow::FLOAT,
            VertexFormat::Byte1 => glow::UNSIGNED_BYTE,
            VertexFormat::Byte2 => glow::UNSIGNED_BYTE,
            VertexFormat::Byte3 => glow::UNSIGNED_BYTE,
            VertexFormat::Byte4 => glow::UNSIGNED_BYTE,
            VertexFormat::Short1 => glow::UNSIGNED_SHORT,
            VertexFormat::Short2 => glow::UNSIGNED_SHORT,
            VertexFormat::Short3 => glow::UNSIGNED_SHORT,
            VertexFormat::Short4 => glow::UNSIGNED_SHORT,
            VertexFormat::Int1 => glow::UNSIGNED_INT,
            VertexFormat::Int2 => glow::UNSIGNED_INT,
            VertexFormat::Int3 => glow::UNSIGNED_INT,
            VertexFormat::Int4 => glow::UNSIGNED_INT,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum VertexStep {
    #[default]
    PerVertex,
    PerInstance,
}

#[derive(Clone, Debug)]
pub struct VertexAttribute {
    pub name: &'static str,
    pub format: VertexFormat,
    /// This flag affects integer VertexFormats, Byte*, Short*, Int*
    /// Taking Byte4 as an example:
    /// On Metal, it might be received as either `float4` or `uint4`
    /// On OpenGl and `gl_pass_as_float = true` shaders should receive it as `vec4`
    /// With `gl_pass_as_float = false`, as `uvec4`
    ///
    /// Note that `uvec4` requires at least `150` glsl version
    /// Before setting `gl_pass_as_float` to false, better check `context.info().has_integer_attributes()` and double check that shaders are at least `150`
    pub gl_pass_as_float: bool,
}

impl VertexAttribute {
    pub const fn new(name: &'static str, format: VertexFormat) -> VertexAttribute {
        Self::with_buffer(name, format)
    }

    pub const fn with_buffer(name: &'static str, format: VertexFormat) -> VertexAttribute {
        VertexAttribute {
            name,
            format,
            gl_pass_as_float: true,
        }
    }
}

#[derive(Clone, Debug, Copy)]
pub enum ShaderType {
    Vertex,
    Fragment,
}

impl Display for ShaderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Vertex => write!(f, "Vertex"),
            Self::Fragment => write!(f, "Fragment"),
        }
    }
}

#[derive(Clone, Debug)]
pub enum ShaderError {
    MissingUniform {
        uniform: String,
    },
    MissingAttribute {
        attribute: String,
    },
    CompilationError {
        shader_type: ShaderType,
        error_message: String,
    },
    LinkError(String),
    /// Shader strings should never contains \00 in the middle
    FFINulError(std::ffi::NulError),
}

impl From<std::ffi::NulError> for ShaderError {
    fn from(e: std::ffi::NulError) -> ShaderError {
        ShaderError::FFINulError(e)
    }
}

impl Display for ShaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingUniform { uniform } => write!(f, "No such uniform:{uniform}"),
            Self::MissingAttribute { attribute } => write!(f, "No such attribute:{attribute}"),
            Self::CompilationError {
                shader_type,
                error_message,
            } => write!(f, "{shader_type} shader error:\n{error_message}"),
            Self::LinkError(msg) => write!(f, "Link shader error:\n{msg}"),
            Self::FFINulError(e) => write!(f, "{e}"),
        }
    }
}

impl Error for ShaderError {}

/// List of all the possible formats of input data when uploading to texture.
/// The list is built by intersection of texture formats supported by 3.3 core profile and webgl1.
#[repr(u8)]
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
            TextureFormat::Alpha => 1 * square,
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

#[derive(Debug, Copy, Clone)]
pub struct TextureParams {
    pub format: TextureFormat,
    pub wrap: TextureWrap,
    pub min_filter: FilterMode,
    pub mag_filter: FilterMode,
    pub mipmap_filter: MipmapFilterMode,
    pub width: u32,
    pub height: u32,
    // All miniquad API could work without this flag being explicit.
    // We can decide if mipmaps are required by the data provided
    // And reallocate non-mipmapped texture(on metal) on generateMipmaps call
    // But! Reallocating cubemaps is too much struggle, so leave it for later.
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

#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash)]
pub struct ShaderId(usize);

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

#[derive(Clone, Copy)]
pub struct PassAction {
    color: Option<(f32, f32, f32, f32)>,
    depth: Option<f32>,
    stencil: Option<i32>,
}

impl PassAction {
    pub const NOTHING: PassAction = PassAction {
        color: None,
        depth: None,
        stencil: None,
    };

    pub fn clear_color(r: f32, g: f32, b: f32, a: f32) -> PassAction {
        PassAction {
            color: Some((r, g, b, a)),
            depth: Some(1.),
            stencil: None,
        }
    }
}

impl Default for PassAction {
    fn default() -> PassAction {
        PassAction {
            color: Some((0.0, 0.0, 0.0, 0.0)),
            depth: Some(1.),
            stencil: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderPassId(usize);

pub const MAX_VERTEX_ATTRIBUTES: usize = 16;
pub const MAX_SHADERSTAGE_IMAGES: usize = 12;

#[derive(Clone, Debug)]
pub struct Features {
    pub instancing: bool,
    /// Does current rendering backend support automatic resolve of
    /// multisampled render passes on end_render_pass.
    /// Would be false on WebGl1 and GL2.
    ///
    /// With resolve_attachments: false, not-none resolve_img in new_render_pass will
    /// result in a runtime panic.
    pub resolve_attachments: bool,
}

impl Default for Features {
    fn default() -> Features {
        Features {
            instancing: true,
            resolve_attachments: true,
        }
    }
}

/// Specify whether front- or back-facing polygons can be culled.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CullFace {
    Nothing,
    Front,
    Back,
}

/// Define front- and back-facing polygons.
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
    Always,
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
            Comparison::Always => glow::ALWAYS,
        }
    }
}

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

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PrimitiveType {
    Triangles,
    Lines,
    Points,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct PipelineParams {
    pub cull_face: CullFace,
    pub front_face_order: FrontFaceOrder,
    pub depth_test: Comparison,
    pub depth_write: bool,
    pub depth_write_offset: Option<(f32, f32)>,
    /// Color (RGB) blend function. If None - blending will be disabled for this pipeline.
    /// Usual use case to get alpha-blending:
    ///```
    ///# use miniquad::{PipelineParams, BlendState, BlendValue, BlendFactor, Equation};
    ///PipelineParams {
    ///    color_blend: Some(BlendState::new(
    ///        Equation::Add,
    ///        BlendFactor::Value(BlendValue::SourceAlpha),
    ///        BlendFactor::OneMinusValue(BlendValue::SourceAlpha))
    ///    ),
    ///    ..Default::default()
    ///};
    ///```
    pub color_blend: Option<BlendState>,
    /// Alpha blend function. If None - alpha will be blended with same equation than RGB colors.
    /// One of possible separate alpha channel blend settings is to avoid blending with WebGl background.
    /// On webgl canvas's resulting alpha channel will be used to blend the whole canvas background.
    /// To avoid modifying only alpha channel, but keep usual transparency:
    ///```
    ///# use miniquad::{PipelineParams, BlendState, BlendValue, BlendFactor, Equation};
    ///PipelineParams {
    ///    color_blend: Some(BlendState::new(
    ///        Equation::Add,
    ///        BlendFactor::Value(BlendValue::SourceAlpha),
    ///        BlendFactor::OneMinusValue(BlendValue::SourceAlpha))
    ///    ),
    ///    alpha_blend: Some(BlendState::new(
    ///        Equation::Add,
    ///        BlendFactor::Zero,
    ///        BlendFactor::One)
    ///    ),
    ///    ..Default::default()
    ///};
    ///```
    /// The same results may be achieved with ColorMask(true, true, true, false)
    pub alpha_blend: Option<BlendState>,
    pub stencil_test: Option<StencilState>,
    pub color_write: ColorMask,
    pub primitive_type: PrimitiveType,
}

// TODO(next major version bump): should be PipelineId
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct PipelineId(usize);

impl Default for PipelineParams {
    fn default() -> PipelineParams {
        PipelineParams {
            cull_face: CullFace::Nothing,
            front_face_order: FrontFaceOrder::CounterClockwise,
            depth_test: Comparison::Always, // no depth test,
            depth_write: false,             // no depth write,
            depth_write_offset: None,
            color_blend: None,
            alpha_blend: None,
            stencil_test: None,
            color_write: (true, true, true, true),
            primitive_type: PrimitiveType::Triangles,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BufferUsage {
    Immutable,
    Dynamic,
    Stream,
}

pub enum TextureSource<'a> {
    Empty,
    Bytes(&'a [u8]),
}
