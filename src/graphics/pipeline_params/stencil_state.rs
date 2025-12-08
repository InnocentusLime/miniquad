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
