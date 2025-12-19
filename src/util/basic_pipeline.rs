use bytemuck::{Pod, Zeroable};
use glam::Mat4;

use crate::{
    default_pipeline_params, uniform_of, util::BasicVertex, PipelineMeta, PipelineParams, Texture2D, UniformBlock, UniformField
};

pub struct BasicPipelineMeta;

impl PipelineMeta for BasicPipelineMeta {
    const VERTEX_SHADER: &str = include_str!("shader/basic_pipeline.vert");
    const FRAGMENT_SHADER: &str = include_str!("shader/basic_pipeline.frag");

    const IMAGES_NAMES: &[&str; 0] = &[];
    type Images = [Texture2D; 0];
    type Vertex = BasicVertex;
    type Uniforms = BasicPipelineUniforms;
    const PARAMS: PipelineParams = default_pipeline_params();
}

#[repr(C)]
#[derive(Debug, Pod, Zeroable, Clone, Copy)]
pub struct BasicPipelineUniforms {
    pub view_projection: Mat4,
}

impl UniformBlock for BasicPipelineUniforms {
    const FIELDS: &'static [UniformField] = &[uniform_of!(BasicPipelineUniforms, view_projection)];
}
