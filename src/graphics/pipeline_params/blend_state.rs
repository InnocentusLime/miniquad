#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Blending {
    None,
    All(BlendFunc),
    Separate { color: BlendFunc, alpha: BlendFunc },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct BlendFunc {
    pub equation: BlendEquation,
    pub source: BlendFactor,
    pub dest: BlendFactor,
}

/// Specifies how incoming RGBA values (source) and the RGBA in framebuffer (destination)
/// are combined.
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum BlendEquation {
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

/// Blend factors.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BlendFactor {
    Zero,
    One,
    Value(BlendValue),
    OneMinusValue(BlendValue),
    SourceAlphaSaturate,
}

/// Blend values.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BlendValue {
    SrcColor,
    SrcAlpha,
    DestColor,
    DestAlpha,
}

impl From<BlendFactor> for u32 {
    fn from(factor: BlendFactor) -> u32 {
        match factor {
            BlendFactor::Zero => glow::ZERO,
            BlendFactor::One => glow::ONE,
            BlendFactor::Value(BlendValue::SrcColor) => glow::SRC_COLOR,
            BlendFactor::Value(BlendValue::SrcAlpha) => glow::SRC_ALPHA,
            BlendFactor::Value(BlendValue::DestColor) => glow::DST_COLOR,
            BlendFactor::Value(BlendValue::DestAlpha) => glow::DST_ALPHA,
            BlendFactor::OneMinusValue(BlendValue::SrcColor) => glow::ONE_MINUS_SRC_COLOR,
            BlendFactor::OneMinusValue(BlendValue::SrcAlpha) => glow::ONE_MINUS_SRC_ALPHA,
            BlendFactor::OneMinusValue(BlendValue::DestColor) => glow::ONE_MINUS_DST_COLOR,
            BlendFactor::OneMinusValue(BlendValue::DestAlpha) => glow::ONE_MINUS_DST_ALPHA,
            BlendFactor::SourceAlphaSaturate => glow::SRC_ALPHA_SATURATE,
        }
    }
}

impl From<BlendEquation> for u32 {
    fn from(eq: BlendEquation) -> Self {
        match eq {
            BlendEquation::Add => glow::FUNC_ADD,
            BlendEquation::Subtract => glow::FUNC_SUBTRACT,
            BlendEquation::ReverseSubtract => glow::FUNC_REVERSE_SUBTRACT,
        }
    }
}
