use std::rc::Rc;

use bytemuck::{Pod, Zeroable};
use glam::{Affine2, Mat4, UVec2, Vec2, vec2};

use crate::{
    BlendEquation, BlendFactor, BlendFunc, BlendValue, Blending, Color, DrawCall, GlContext,
    Pipeline, PipelineMeta, PipelineParams, Texture2D, UniformBlock, UniformField, Vertex,
    VertexField, attribute_of, default_pipeline_params, uniform_of, util::GeometryBatcher,
};

#[derive(Debug, Clone, Copy)]
pub struct Sprite {
    pub tex_rect_pos: UVec2,
    pub tex_rect_size: UVec2,
    pub color: Color,
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

        let color = sprite.color;
        let tf = sprite.transform;

        // Verts
        let center = tf.transform_point2(Vec2::ZERO);
        let halfs = sprite.tex_rect_size.as_vec2() * vec2(0.5, 0.5);
        let horizontal = tf.transform_vector2(vec2(halfs.x, 0.0));
        let vertical = tf.transform_vector2(vec2(0.0, halfs.y));

        // Texcoords
        let tex_top_left = sprite.tex_rect_pos.as_vec2();
        let Vec2 {
            x: tex_width,
            y: tex_height,
        } = sprite.tex_rect_size.as_vec2();

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
            SpriteVertex {
                pos: p1,
                texcoord: t1,
                color,
            },
            SpriteVertex {
                pos: p2,
                texcoord: t2,
                color,
            },
            SpriteVertex {
                pos: p3,
                texcoord: t3,
                color,
            },
            SpriteVertex {
                pos: p4,
                texcoord: t4,
                color,
            },
        ];

        self.0.extend(vertices, INDICIES);
    }

    #[track_caller]
    pub fn draw(
        &mut self,
        ctx: &GlContext,
        view_projection: Mat4,
        pipeline: &Pipeline<BasicSpritePipelineMeta>,
        texture: &Texture2D,
    ) {
        let num_elements = self.0.finish();
        ctx.draw(DrawCall {
            pipeline,
            base_element: 0,
            num_elements,
            vertex_buffer: &self.0.vertices,
            index_buffer: &self.0.indicies,
            images: texture,
            uniforms: &BasicSpritePipelineUniforms {
                view_projection,
                width_height: texture.size().as_vec2(),
            },
        });
    }
}

#[derive(Debug, Default, Pod, Zeroable, Clone, Copy)]
#[repr(C)]
pub struct SpriteVertex {
    pub pos: Vec2,
    pub texcoord: Vec2,
    pub color: Color,
}

impl Vertex for SpriteVertex {
    const LAYOUT: &'static [VertexField] = &[
        attribute_of!(SpriteVertex, pos),
        attribute_of!(SpriteVertex, texcoord),
        attribute_of!(SpriteVertex, color),
    ];
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
