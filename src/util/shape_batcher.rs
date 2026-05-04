use std::rc::Rc;

use super::{BasicVertex, GeometryBatcher};
use crate::graphics::{Color, GlContext, IndexBuffer, Result, VertexBuffer};

use glam::{Affine2, Vec2, vec2};

#[derive(Debug)]
pub struct ShapeBatcher(pub GeometryBatcher<BasicVertex>);

impl ShapeBatcher {
    pub fn new_from_size(
        ctx: &Rc<GlContext>,
        vertices_size: usize,
        indicies_size: usize,
    ) -> Result<Self> {
        let inner = GeometryBatcher::new_from_size(ctx, vertices_size, indicies_size)?;
        Ok(ShapeBatcher(inner))
    }

    pub fn new(vertices: VertexBuffer<BasicVertex>, indicies: IndexBuffer) -> Self {
        ShapeBatcher(GeometryBatcher::new(vertices, indicies))
    }

    pub fn element_count(&self) -> u32 {
        self.0.element_count()
    }

    pub fn flush(&mut self) -> u32 {
        self.0.flush()
    }

    pub fn clear(&mut self) {
        self.0.clear()
    }

    #[track_caller]
    pub fn triangle(&mut self, color: Color, p1: Vec2, p2: Vec2, p3: Vec2) {
        let vertices = [
            BasicVertex { v_pos: p1, v_color: color },
            BasicVertex { v_pos: p2, v_color: color },
            BasicVertex { v_pos: p3, v_color: color },
        ];
        self.0.extend(&vertices, &[0, 1, 2]);
    }

    #[track_caller]
    pub fn rect(&mut self, color: Color, center: Vec2, size: Vec2, rotation: f32) {
        let tf = Affine2::from_angle_translation(rotation, center);
        let v1 = tf.transform_point2(vec2(-size.x, size.y) * 0.5);
        let v2 = tf.transform_point2(vec2(-size.x, -size.y) * 0.5);
        let v3 = tf.transform_point2(vec2(size.x, size.y) * 0.5);
        let v4 = tf.transform_point2(vec2(size.x, -size.y) * 0.5);
        self.0.extend(
            &[
                BasicVertex { v_pos: v1, v_color: color },
                BasicVertex { v_pos: v2, v_color: color },
                BasicVertex { v_pos: v3, v_color: color },
                BasicVertex { v_pos: v4, v_color: color },
            ],
            &[0, 1, 2, 2, 1, 3],
        );
    }

    #[track_caller]
    pub fn line(&mut self, color: Color, thickness: f32, p1: Vec2, p2: Vec2) {
        let dn = (p2 - p1).perp().normalize_or_zero() * (0.5 * thickness);
        let v1 = p1 + dn;
        let v2 = p1 - dn;
        let v3 = p2 + dn;
        let v4 = p2 - dn;
        self.0.extend(
            &[
                BasicVertex { v_pos: v1, v_color: color },
                BasicVertex { v_pos: v2, v_color: color },
                BasicVertex { v_pos: v3, v_color: color },
                BasicVertex { v_pos: v4, v_color: color },
            ],
            &[0, 1, 2, 2, 1, 3],
        );
    }

    #[track_caller]
    pub fn triangle_lines(&mut self, color: Color, thickness: f32, p1: Vec2, p2: Vec2, p3: Vec2) {
        self.line(color, thickness, p1, p2);
        self.line(color, thickness, p2, p3);
        self.line(color, thickness, p3, p1);
    }

    #[track_caller]
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

    #[track_caller]
    pub fn polygon(
        &mut self,
        color: Color,
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
            self.triangle(color, center, p1, p2);
        }
    }

    #[track_caller]
    pub fn circle_lines(&mut self, color: Color, thickness: f32, center: Vec2, radius: f32) {
        self.poly_lines(color, thickness, center, 0.0, 50, radius);
    }

    #[track_caller]
    pub fn circle(&mut self, color: Color, center: Vec2, radius: f32) {
        self.polygon(color, center, 0.0, 50, radius);
    }

    #[track_caller]
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
