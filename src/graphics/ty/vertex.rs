use std::fmt::Debug;

use bytemuck::Pod;
use glam::{U8Vec4, Vec2, Vec3, Vec4};
use glow::HasContext;

#[macro_export]
macro_rules! attribute_of {
    ($Type:path, $field:tt) => {{
        let sample = $crate::zeroed::<$Type>();
        let (gl_type, component_count) = $crate::attribute_info_of_val(sample.$field);
        $crate::VertexField {
            name: stringify!($field),
            gl_type,
            component_count,
            off: std::mem::offset_of!($Type, $field),
        }
    }};
}
pub(crate) fn apply_attributes_impl(gl: &glow::Context, size: usize, layout: &[VertexField]) {
    // This is a limitation coming from WebGL. Since WebGL is a valid target,
    // this limiation is enforced on all platforms.
    //
    // REF: https://registry.khronos.org/webgl/specs/latest/1.0/#VERTEX_STRIDE
    assert!(size <= 255, "maximum supported stride is 255");
    assert!(layout.len() <= MAX_VERTEX_ATTRIBUTES, "too many attributes");

    for id in 0..MAX_VERTEX_ATTRIBUTES {
        unsafe {
            gl.disable_vertex_attrib_array(id as u32);
        }
    }

    for (id, attr) in layout.iter().enumerate() {
        unsafe {
            gl.enable_vertex_attrib_array(id as u32);
        }

        unsafe {
            if gl_type_is_integer(attr.gl_type) {
                gl.vertex_attrib_pointer_i32(
                    id as u32,
                    attr.component_count as i32,
                    attr.gl_type,
                    size as i32,
                    attr.off as i32,
                )
            } else {
                gl.vertex_attrib_pointer_f32(
                    id as u32,
                    attr.component_count as i32,
                    attr.gl_type,
                    false,
                    size as i32,
                    attr.off as i32,
                )
            }
        }
    }
}

pub trait Vertex: Pod + Debug {
    const LAYOUT: &'static [VertexField];
}

#[derive(Debug, Clone, Copy)]
pub struct VertexField {
    pub name: &'static str,
    pub gl_type: u32,
    pub component_count: usize,
    pub off: usize,
}

impl VertexFieldTy for f32 {
    const GL_TYPE: u32 = glow::FLOAT;
    const COMPONENT_COUNT: usize = 1;
}

impl VertexFieldTy for Vec2 {
    const GL_TYPE: u32 = glow::FLOAT;
    const COMPONENT_COUNT: usize = 2;
}

impl VertexFieldTy for Vec3 {
    const GL_TYPE: u32 = glow::FLOAT;
    const COMPONENT_COUNT: usize = 3;
}

impl VertexFieldTy for Vec4 {
    const GL_TYPE: u32 = glow::FLOAT;
    const COMPONENT_COUNT: usize = 4;
}

impl VertexFieldTy for U8Vec4 {
    const GL_TYPE: u32 = glow::UNSIGNED_BYTE;
    const COMPONENT_COUNT: usize = 4;
}

impl VertexFieldTy for super::Color {
    const GL_TYPE: u32 = glow::FLOAT;
    const COMPONENT_COUNT: usize = 4;
}

pub trait VertexFieldTy: Pod + Debug {
    const GL_TYPE: u32;
    const COMPONENT_COUNT: usize;
}

pub trait Test {
    const N: usize;
}

pub const fn attribute_info_of_val<T: VertexFieldTy>(_x: T) -> (u32, usize) {
    (T::GL_TYPE, T::COMPONENT_COUNT)
}

const fn gl_type_is_integer(gl_type: u32) -> bool {
    matches!(
        gl_type,
        glow::INT
            | glow::SHORT
            | glow::BYTE
            | glow::UNSIGNED_INT
            | glow::UNSIGNED_SHORT
            | glow::UNSIGNED_BYTE
    )
}

const MAX_VERTEX_ATTRIBUTES: usize = 16;
