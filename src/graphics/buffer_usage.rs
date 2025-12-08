#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BufferUsage {
    Immutable,
    Dynamic,
    Stream,
}

pub(crate) fn gl_usage(usage: BufferUsage) -> u32 {
    match usage {
        BufferUsage::Immutable => glow::STATIC_DRAW,
        BufferUsage::Dynamic => glow::DYNAMIC_DRAW,
        BufferUsage::Stream => glow::STREAM_DRAW,
    }
}
