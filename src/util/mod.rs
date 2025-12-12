mod basic_pipeline;
mod geometry_batcher;
mod input;
mod shape_batcher;

pub use basic_pipeline::*;
pub use geometry_batcher::*;
pub use input::*;
pub use shape_batcher::*;

use bytemuck::{Pod, Zeroable};
use glam::Vec2;

use crate::Color;

#[derive(Debug, Default, Pod, Zeroable, Clone, Copy)]
#[repr(C)]
pub struct BasicVertex {
    pub pos: Vec2,
    pub color: Color,
}

#[derive(Debug, Pod, Zeroable, Clone, Copy)]
#[repr(C)]
pub struct NoUniforms;
