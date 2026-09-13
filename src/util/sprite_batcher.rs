use std::rc::Rc;

use bytemuck::{Pod, Zeroable};
use glam::{Affine2, Mat4, UVec2, Vec2, vec2};

use super::BasicTexImages;
use crate::graphics::{
    BlendEquation, BlendFactor, BlendFunc, BlendValue, Blending, GlContext, Pipeline,
    PipelineParams, Result, UniformBlock, UniformField, Vertex, VertexField, VertexIndex,
    default_pipeline_params,
};
use crate::{attribute_of, uniform_of};

type GeometryBatcher<I> = super::GeometryBatcher<SpriteVertex, I>;

pub fn sprite<I: VertexIndex>(
    batcher: &mut GeometryBatcher<I>,
    tex_rect_pos: UVec2,
    tex_rect_size: UVec2,
    tf: Affine2,
) {
    let off = batcher.index_offset();

    // Verts
    let center = tf.transform_point2(Vec2::ZERO);
    let halfs = tex_rect_size.as_vec2() * vec2(0.5, 0.5);
    let horizontal = tf.transform_vector2(vec2(halfs.x, 0.0));
    let vertical = tf.transform_vector2(vec2(0.0, halfs.y));

    // Texcoords
    let tex_top_left = tex_rect_pos.as_vec2();
    let Vec2 { x: tex_width, y: tex_height } = tex_rect_size.as_vec2();

    // (-1.0, 1.0)
    let p1 = center - horizontal + vertical;
    let t1 = tex_top_left + vec2(0.0, tex_height);
    // (1.0,-1.0)
    let p2 = center + horizontal + vertical;
    let t2 = tex_top_left + vec2(tex_width, tex_height);
    // (1.0, -1.0)
    let p3 = center + horizontal - vertical;
    let t3 = tex_top_left + vec2(tex_width, 0.0);
    // (-1.0, -1.0)
    let p4 = center - horizontal - vertical;
    let t4 = tex_top_left;

    batcher.extend(
        &[
            SpriteVertex { v_pos: p1, v_uv_not_normalized: t1 },
            SpriteVertex { v_pos: p2, v_uv_not_normalized: t2 },
            SpriteVertex { v_pos: p3, v_uv_not_normalized: t3 },
            SpriteVertex { v_pos: p4, v_uv_not_normalized: t4 },
        ],
        &[
            I::from(0u8).offset_by(off),
            I::from(1u8).offset_by(off),
            I::from(2u8).offset_by(off),
            I::from(0u8).offset_by(off),
            I::from(2u8).offset_by(off),
            I::from(3u8).offset_by(off),
        ],
    );
}

#[derive(Debug, Default, Pod, Zeroable, Clone, Copy)]
#[repr(C)]
pub struct SpriteVertex {
    pub v_pos: Vec2,
    pub v_uv_not_normalized: Vec2,
}

impl Vertex for SpriteVertex {
    const LAYOUT: &'static [VertexField] =
        &[attribute_of!(SpriteVertex, v_pos), attribute_of!(SpriteVertex, v_uv_not_normalized)];
}

pub type BasicSpritePipeline =
    Pipeline<SpriteVertex, BasicSpritePipelineUniforms, BasicTexImages<'static>>;

pub fn new_basic_sprite_pipeline(ctx: &Rc<GlContext>) -> Result<BasicSpritePipeline> {
    ctx.new_pipeline(
        include_str!("shader/basic_sprite.vert"),
        include_str!("shader/basic_sprite.frag"),
        PipelineParams {
            blending: Blending::All(BlendFunc {
                equation: BlendEquation::Add,
                source: BlendFactor::Value(BlendValue::SrcAlpha),
                dest: BlendFactor::OneMinusValue(BlendValue::SrcAlpha),
            }),
            ..default_pipeline_params()
        },
    )
}

#[repr(C)]
#[derive(Debug, Pod, Zeroable, Clone, Copy)]
pub struct BasicSpritePipelineUniforms {
    pub view_projection: Mat4,
    pub width_height: Vec2,
}

impl UniformBlock for BasicSpritePipelineUniforms {
    const FIELDS: &[UniformField] = &[
        uniform_of!(BasicSpritePipelineUniforms, view_projection),
        uniform_of!(BasicSpritePipelineUniforms, width_height),
    ];
}
