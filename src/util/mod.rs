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
use crate::graphics::{
    Color, ImageUniformField, ImageUniformVal, ImagesUniformBlock, Texture2D, Vertex, VertexField,
};
use crate::image_uniform_of;

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

#[derive(Debug, Clone, Copy)]
pub struct BasicTexImages<'a> {
    pub tex: &'a Texture2D,
}

impl ImagesUniformBlock for BasicTexImages<'_> {
    const FIELDS: &'static [ImageUniformField] = &[image_uniform_of!(Texture2D, tex)];

    type Borrow<'a> = &'a BasicTexImages<'a>;

    fn bind(raw: Self::Borrow<'_>) {
        raw.tex.bind(0);
    }
}
