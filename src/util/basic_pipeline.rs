use bytemuck::{Pod, Zeroable};
use glam::Mat4;

use crate::{
    PipelineMeta, PipelineParams, Texture2DBinding, UniformBlock, UniformField,
    default_pipeline_params, uniform_of, util::BasicVertex,
};

pub struct BasicPipelineMeta;

impl PipelineMeta for BasicPipelineMeta {
    const VERTEX_SHADER: &'static str = include_str!("shader/basic_pipeline.vert");
    const FRAGMENT_SHADER: &'static str = include_str!("shader/basic_pipeline.frag");

    const IMAGES_NAMES: &'static [&'static str] = &[];
    type Images<'a> = [Texture2DBinding<'a>; 0];
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
