use super::BasicVertex;
use crate::graphics::{Color, VertexIndex};

use glam::{Affine2, Vec2, vec2};

type GeometryBatcher<I> = super::GeometryBatcher<BasicVertex, I>;

pub fn triangle<I: VertexIndex>(
    batcher: &mut GeometryBatcher<I>,
    color: Color,
    p1: Vec2,
    p2: Vec2,
    p3: Vec2,
) {
    let off = batcher.index_offset();

    batcher.extend(
        &[
            BasicVertex { v_pos: p1, v_color: color },
            BasicVertex { v_pos: p2, v_color: color },
            BasicVertex { v_pos: p3, v_color: color },
        ],
        &[I::from(0u8).offset_by(off), I::from(1u8).offset_by(off), I::from(2u8).offset_by(off)],
    );
}

pub fn rect<I: VertexIndex>(
    batcher: &mut GeometryBatcher<I>,
    color: Color,
    center: Vec2,
    size: Vec2,
    rotation: f32,
) {
    let off = batcher.index_offset();

    let tf = Affine2::from_angle_translation(rotation, center);
    let v1 = tf.transform_point2(vec2(-size.x, size.y) * 0.5);
    let v2 = tf.transform_point2(vec2(-size.x, -size.y) * 0.5);
    let v3 = tf.transform_point2(vec2(size.x, size.y) * 0.5);
    let v4 = tf.transform_point2(vec2(size.x, -size.y) * 0.5);

    batcher.extend(
        &[
            BasicVertex { v_pos: v1, v_color: color },
            BasicVertex { v_pos: v2, v_color: color },
            BasicVertex { v_pos: v3, v_color: color },
            BasicVertex { v_pos: v4, v_color: color },
        ],
        &[
            I::from(0u8).offset_by(off),
            I::from(1u8).offset_by(off),
            I::from(2u8).offset_by(off),
            I::from(2u8).offset_by(off),
            I::from(1u8).offset_by(off),
            I::from(3u8).offset_by(off),
        ],
    );
}

pub fn line<I: VertexIndex>(
    batcher: &mut GeometryBatcher<I>,
    color: Color,
    thickness: f32,
    p1: Vec2,
    p2: Vec2,
) {
    let off = batcher.index_offset();

    let dn = (p2 - p1).perp().normalize_or_zero() * (0.5 * thickness);
    let v1 = p1 + dn;
    let v2 = p1 - dn;
    let v3 = p2 + dn;
    let v4 = p2 - dn;

    batcher.extend(
        &[
            BasicVertex { v_pos: v1, v_color: color },
            BasicVertex { v_pos: v2, v_color: color },
            BasicVertex { v_pos: v3, v_color: color },
            BasicVertex { v_pos: v4, v_color: color },
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

pub fn triangle_lines<I: VertexIndex>(
    batcher: &mut GeometryBatcher<I>,
    color: Color,
    thickness: f32,
    p1: Vec2,
    p2: Vec2,
    p3: Vec2,
) {
    line(batcher, color, thickness, p1, p2);
    line(batcher, color, thickness, p2, p3);
    line(batcher, color, thickness, p3, p1);
}

pub fn poly_lines<I: VertexIndex>(
    batcher: &mut GeometryBatcher<I>,
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

        line(batcher, color, thickness, p1, p2);
    }
}

#[inline]
pub fn circle_lines<I: VertexIndex>(
    batcher: &mut GeometryBatcher<I>,
    color: Color,
    thickness: f32,
    center: Vec2,
    radius: f32,
) {
    poly_lines(batcher, color, thickness, center, 0.0, 50, radius);
}

pub fn polygon<I: VertexIndex>(
    batcher: &mut GeometryBatcher<I>,
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

        triangle(batcher, color, p1, p2, center);
    }
}

#[inline]
pub fn circle<I: VertexIndex>(
    batcher: &mut GeometryBatcher<I>,
    color: Color,
    center: Vec2,
    radius: f32,
) {
    polygon(batcher, color, center, 0.0, 50, radius);
}

pub fn rect_lines<I: VertexIndex>(
    batcher: &mut GeometryBatcher<I>,
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

    line(batcher, color, thickness, p1, p2);
    line(batcher, color, thickness, p2, p4);
    line(batcher, color, thickness, p4, p3);
    line(batcher, color, thickness, p3, p1);
}
