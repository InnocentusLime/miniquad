use bytemuck::Pod;
use std::fmt::Debug;

pub trait VertexIndex: Pod + Debug {
    const GL_TYPE: u32;

    fn offset_by(self, off: usize) -> Self;
}

impl VertexIndex for u8 {
    const GL_TYPE: u32 = glow::UNSIGNED_BYTE;

    fn offset_by(self, off: usize) -> Self {
        self + off as u8
    }
}

impl VertexIndex for u16 {
    const GL_TYPE: u32 = glow::UNSIGNED_SHORT;

    fn offset_by(self, off: usize) -> Self {
        self + off as u16
    }
}

impl VertexIndex for u32 {
    const GL_TYPE: u32 = glow::UNSIGNED_SHORT;

    fn offset_by(self, off: usize) -> Self {
        self + off as u32
    }
}
