mod basic_pipeline;
mod geometry_batcher;
mod input;
mod shape_batcher;
mod sprite_batcher;

pub use basic_pipeline::*;
pub use geometry_batcher::*;
pub use input::*;
pub use shape_batcher::*;
pub use sprite_batcher::*;

use bytemuck::{Pod, Zeroable};
use glam::Vec2;

use crate::attribute_of;
use crate::graphics::{Color, Vertex, VertexField};

#[derive(Debug, Default, Pod, Zeroable, Clone, Copy)]
#[repr(C)]
pub struct BasicVertex {
    pub v_pos: Vec2,
    pub v_color: Color,
}

impl Vertex for BasicVertex {
    const LAYOUT: &'static [VertexField] =
        &[attribute_of!(BasicVertex, v_pos), attribute_of!(BasicVertex, v_color)];
}
