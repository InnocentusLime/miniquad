mod blend_state;
mod stencil_state;

pub use blend_state::*;
pub use stencil_state::*;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct PipelineParams {
    pub cull_face: CullFace,
    pub front_face_order: FrontFaceOrder,
    pub depth_test: Option<Comparison>,
    pub depth_write_offset: Option<(f32, f32)>,
    pub blending: Blending,
    pub stencil_test: Option<StencilState>,
    pub color_write: ColorMask,
    pub primitive_type: PrimitiveType,
}

pub const fn default_pipeline_params() -> PipelineParams {
    PipelineParams {
        cull_face: CullFace::Nothing,
        front_face_order: FrontFaceOrder::CounterClockwise,
        depth_test: None, // no depth test,
        depth_write_offset: None,
        blending: Blending::None,
        stencil_test: None,
        color_write: (true, true, true, true),
        primitive_type: PrimitiveType::Triangles,
    }
}

impl Default for PipelineParams {
    fn default() -> PipelineParams {
        default_pipeline_params()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CullFace {
    Nothing,
    Front,
    Back,
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

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PrimitiveType {
    Triangles,
    Lines,
    Points,
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

pub type ColorMask = (bool, bool, bool, bool);
