use std::rc::Rc;

use bytemuck::{Pod, Zeroable};
use glam::Mat4;

use crate::{
    GlContext, Pipeline, PipelineParams, UniformDesc, UniformType, VertexAttribute, VertexFormat,
};

pub fn make_basic_pipeline(ctx: &Rc<GlContext>) -> Pipeline {
    ctx.new_pipeline::<&'static str>(
        BASIC_PIPELINE_VERTEX,
        BASIC_PIPELINE_FRAGMENT,
        PipelineParams::default(),
        [
            VertexAttribute::new("in_pos", VertexFormat::F32x2),
            VertexAttribute::new("in_color", VertexFormat::F32x4),
        ],
        [UniformDesc::new_scalar("view_proj", UniformType::F32x4x4)],
        [],
    )
    .unwrap()
}

#[repr(C)]
#[derive(Pod, Zeroable, Clone, Copy)]
pub struct BasicPipelineUniform {
    pub view_projection: Mat4,
}

pub const BASIC_PIPELINE_VERTEX: &str = r#"#version 100
uniform mat4 view_proj;

attribute vec2 in_pos;
attribute vec4 in_color;

varying lowp vec4 color;

void main() {
    gl_Position = view_proj * vec4(in_pos, 0, 1);
    color = in_color;
}"#;

pub const BASIC_PIPELINE_FRAGMENT: &str = r#"#version 100
varying lowp vec4 color;

void main() {
    gl_FragColor = color;
}"#;
