use bytemuck::{Pod, Zeroable};
use glam::Mat4;

use crate::graphics::{
    NoImages, PipelineMeta, PipelineParams, UniformBlock, UniformField, default_pipeline_params,
};
use crate::uniform_of;
use crate::util::BasicVertex;

pub struct BasicPipelineMeta;

impl PipelineMeta for BasicPipelineMeta {
    const VERTEX_SHADER: &str = include_str!("shader/basic_pipeline.vert");
    const FRAGMENT_SHADER: &str = include_str!("shader/basic_pipeline.frag");

    const IMAGES_NAMES: () = ();
    type Images = NoImages;
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
