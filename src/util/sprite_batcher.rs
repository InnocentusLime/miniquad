use std::rc::Rc;

use bytemuck::{Pod, Zeroable};
use glam::{Affine2, Mat4, UVec2, Vec2, vec2};

use crate::graphics::{
    BlendEquation, BlendFactor, BlendFunc, BlendValue, Blending, GlContext, PipelineMeta,
    PipelineParams, Texture2D, UniformBlock, UniformField, Vertex, VertexField,
    default_pipeline_params,
};
use crate::util::GeometryBatcher;
use crate::{attribute_of, uniform_of};

#[derive(Debug, Clone, Copy)]
pub struct Sprite {
    pub tex_rect_pos: UVec2,
    pub tex_rect_size: UVec2,
    pub transform: Affine2,
}

pub struct SpriteBatcher(pub GeometryBatcher<SpriteVertex>);

impl SpriteBatcher {
    pub fn new_from_size(ctx: &Rc<GlContext>, sprites: usize) -> Self {
        SpriteBatcher::new(GeometryBatcher::new_from_size(
            ctx,
            sprites * 4,
            sprites * 6,
        ))
    }

    pub fn new(batcher: GeometryBatcher<SpriteVertex>) -> Self {
        SpriteBatcher(batcher)
    }

    #[track_caller]
    pub fn add_sprite(&mut self, sprite: Sprite) {
        const INDICIES: &[u16] = &[0, 1, 2, 0, 2, 3];

        let tf = sprite.transform;

        // Verts
        let center = tf.transform_point2(Vec2::ZERO);
        let halfs = sprite.tex_rect_size.as_vec2() * vec2(0.5, 0.5);
        let horizontal = tf.transform_vector2(vec2(halfs.x, 0.0));
        let vertical = tf.transform_vector2(vec2(0.0, halfs.y));

        // Texcoords
        let tex_top_left = sprite.tex_rect_pos.as_vec2();
        let Vec2 { x: tex_width, y: tex_height } = sprite.tex_rect_size.as_vec2();

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

        let vertices = &[
            SpriteVertex { v_pos: p1, v_uv_not_normalized: t1 },
            SpriteVertex { v_pos: p2, v_uv_not_normalized: t2 },
            SpriteVertex { v_pos: p3, v_uv_not_normalized: t3 },
            SpriteVertex { v_pos: p4, v_uv_not_normalized: t4 },
        ];

        self.0.extend(vertices, INDICIES);
    }

    pub fn flush(&mut self) -> u32 {
        self.0.flush()
    }

    pub fn clear(&mut self) {
        self.0.clear()
    }
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

pub struct BasicSpritePipelineMeta;

impl PipelineMeta for BasicSpritePipelineMeta {
    const VERTEX_SHADER: &str = include_str!("shader/basic_sprite.vert");
    const FRAGMENT_SHADER: &str = include_str!("shader/basic_sprite.frag");

    type Images = Texture2D;
    const IMAGES_NAMES: &str = "tex";

    type Vertex = SpriteVertex;
    type Uniforms = BasicSpritePipelineUniforms;
    const PARAMS: PipelineParams = PipelineParams {
        blending: Blending::All(BlendFunc {
            equation: BlendEquation::Add,
            source: BlendFactor::Value(BlendValue::SrcAlpha),
            dest: BlendFactor::OneMinusValue(BlendValue::SrcAlpha),
        }),
        ..default_pipeline_params()
    };
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
