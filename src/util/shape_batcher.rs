use std::rc::Rc;

use super::{BasicVertex, GeometryBatcher};
use crate::{
    BufferUsage, Color, DrawCall, GlContext, IndexBuffer, Pipeline, VertexBuffer,
    bind_vertex_buffers, util::BasicPipelineUniform,
};

use glam::{Affine2, Mat4, Vec2, vec2};

#[derive(Debug)]
pub struct ShapeBatcher(pub GeometryBatcher<BasicVertex>);

impl ShapeBatcher {
    pub fn new_from_size(ctx: &Rc<GlContext>, vertices_size: usize, indicies_size: usize) -> Self {
        Self::new(
            ctx.new_empty_vertex_buffer(BufferUsage::Stream, vertices_size),
            ctx.new_empty_index_buffer(BufferUsage::Stream, indicies_size),
        )
    }

    pub fn new(vertices: VertexBuffer<BasicVertex>, indicies: IndexBuffer) -> Self {
        ShapeBatcher(GeometryBatcher::new(vertices, indicies))
    }

    pub fn element_count(&self) -> u32 {
        self.0.element_count()
    }

    pub fn finish(&mut self) -> u32 {
        self.0.finish()
    }

    pub fn basic_draw(
        &mut self,
        ctx: &GlContext,
        view_projection: Mat4,
        basic_pipeline: &Pipeline<BasicPipelineUniform>,
    ) {
        let num_elements = self.finish();
        ctx.submit_drawcall(DrawCall {
            pipeline: basic_pipeline,
            base_element: 0,
            num_elements,
            vertex_buffers: &bind_vertex_buffers!(
                (&self.0.vertices) as <BasicVertex>::pos,
                (&self.0.vertices) as <BasicVertex>::color,
            ),
            index_buffer: self.0.indicies.bind(),
            textures: &[],
            uniforms: &BasicPipelineUniform { view_projection },
        });
    }

    pub fn triangle(&mut self, color: Color, p1: Vec2, p2: Vec2, p3: Vec2) {
        let vertices = [
            BasicVertex { pos: p1, color },
            BasicVertex { pos: p2, color },
            BasicVertex { pos: p3, color },
        ];
        self.0.extend(&vertices, &[0, 1, 2]);
    }

    pub fn rect(&mut self, color: Color, center: Vec2, size: Vec2, rotation: f32) {
        let tf = Affine2::from_angle_translation(rotation, center);
        let v1 = tf.transform_point2(vec2(-size.x, size.y) * 0.5);
        let v2 = tf.transform_point2(vec2(-size.x, -size.y) * 0.5);
        let v3 = tf.transform_point2(vec2(size.x, size.y) * 0.5);
        let v4 = tf.transform_point2(vec2(size.x, -size.y) * 0.5);
        self.0.extend(
            &[
                BasicVertex { pos: v1, color },
                BasicVertex { pos: v2, color },
                BasicVertex { pos: v3, color },
                BasicVertex { pos: v4, color },
            ],
            &[0, 1, 2, 2, 1, 3],
        );
    }

    pub fn line(&mut self, color: Color, thickness: f32, p1: Vec2, p2: Vec2) {
        let dn = (p2 - p1).perp().normalize_or_zero() * (0.5 * thickness);
        let v1 = p1 + dn;
        let v2 = p1 - dn;
        let v3 = p2 + dn;
        let v4 = p2 - dn;
        self.0.extend(
            &[
                BasicVertex { pos: v1, color },
                BasicVertex { pos: v2, color },
                BasicVertex { pos: v3, color },
                BasicVertex { pos: v4, color },
            ],
            &[0, 1, 2, 2, 1, 3],
        );
    }

    pub fn triangle_lines(&mut self, color: Color, thickness: f32, p1: Vec2, p2: Vec2, p3: Vec2) {
        self.line(color, thickness, p1, p2);
        self.line(color, thickness, p2, p3);
        self.line(color, thickness, p3, p1);
    }

    pub fn poly_lines(
        &mut self,
        color: Color,
        thickness: f32,
        center: Vec2,
        rotation: f32,
        vertex_count: usize,
        radius: f32,
    ) {
        let angle_increment = std::f32::consts::TAU / (vertex_count as f32);
        for idx in 0..vertex_count {
            let angle_1 = angle_increment * idx as f32 + rotation;
            let angle_2 = angle_increment * (idx + 1) as f32 + rotation;
            let p1 = Vec2::from_angle(angle_1) * radius + center;
            let p2 = Vec2::from_angle(angle_2) * radius + center;
            self.line(color, thickness, p1, p2);
        }
    }

    pub fn circle_lines(&mut self, color: Color, thickness: f32, center: Vec2, radius: f32) {
        self.poly_lines(color, thickness, center, 0.0, 50, radius);
    }

    pub fn rect_lines(
        &mut self,
        color: Color,
        thickness: f32,
        center: Vec2,
        size: Vec2,
        rotation: f32,
    ) {
        let tf = Affine2::from_angle_translation(rotation, center);
        let p1 = tf.transform_point2(vec2(-size.x, size.y) * 0.5);
        let p2 = tf.transform_point2(vec2(-size.x, -size.y) * 0.5);
        let p3 = tf.transform_point2(vec2(size.x, size.y) * 0.5);
        let p4 = tf.transform_point2(vec2(size.x, -size.y) * 0.5);
        self.line(color, thickness, p1, p2);
        self.line(color, thickness, p2, p4);
        self.line(color, thickness, p4, p3);
        self.line(color, thickness, p3, p1);
    }
}
