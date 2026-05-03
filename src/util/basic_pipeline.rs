use std::rc::Rc;

use bytemuck::{Pod, Zeroable};
use glam::Mat4;

use crate::graphics::{GlContext, Pipeline, UniformBlock, UniformField, default_pipeline_params};
use crate::uniform_of;
use crate::util::BasicVertex;

pub type BasicPipeline = Pipeline<BasicVertex, BasicPipelineUniforms>;

pub fn new_basic_pipeline(ctx: &Rc<GlContext>) -> BasicPipeline {
    ctx.new_pipeline(
        include_str!("shader/basic_pipeline.vert"),
        include_str!("shader/basic_pipeline.frag"),
        default_pipeline_params(),
    )
}

#[repr(C)]
#[derive(Debug, Pod, Zeroable, Clone, Copy)]
pub struct BasicPipelineUniforms {
    pub view_projection: Mat4,
}

impl UniformBlock for BasicPipelineUniforms {
    const FIELDS: &'static [UniformField] = &[uniform_of!(BasicPipelineUniforms, view_projection)];
}
