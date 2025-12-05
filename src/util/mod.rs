mod geometry_batcher;
mod shape_batcher;
mod basic_pipeline;

pub use basic_pipeline::*;
pub use geometry_batcher::*;
pub use shape_batcher::*;

use bytemuck::{Pod, Zeroable};
use glam::{Vec2, Vec4};

#[derive(Debug, Default, Pod, Zeroable, Clone, Copy)]
#[repr(C)]
pub struct BasicVertex {
    pub pos: Vec2,
    pub color: Vec4,
}
