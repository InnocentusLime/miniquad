use std::fmt::Debug;

use bytemuck::{Pod, Zeroable};
use glam::{IVec2, IVec3, IVec4, Mat4, Vec2, Vec3, Vec4};
use glow::HasContext;

#[derive(Debug, Pod, Zeroable, Clone, Copy)]
#[repr(C)]
pub struct NoUniforms;

impl UniformBlock for NoUniforms {
    const FIELDS: &'static [UniformField] = &[];
}

#[macro_export]
macro_rules! uniform_of {
    ($Type:path, $field:tt) => {{
        let sample = $crate::zeroed::<$Type>();
        $crate::UniformField {
            name: stringify!($field),
            ty: $crate::uniform_type_of_val(sample.$field),
            off: std::mem::offset_of!($Type, $field),
            sz: std::mem::size_of_val(&sample.$field),
        }
    }};
}

pub(crate) fn apply_uniforms_impl(
    gl: &glow::Context,
    uniform_data: &[u8],
    layout: &[UniformField],
    locs: &[glow::UniformLocation],
) {
    for (layout, location) in layout.iter().zip(locs) {
        let data = &uniform_data[layout.off..(layout.off + layout.sz)];
        let location = Some(location);

        match layout.ty {
            UniformType::F32 => unsafe {
                gl.uniform_1_f32_slice(location, bytemuck::cast_slice(data));
            },
            UniformType::F32x2 => unsafe {
                gl.uniform_2_f32_slice(location, bytemuck::cast_slice(data));
            },
            UniformType::F32x3 => unsafe {
                gl.uniform_3_f32_slice(location, bytemuck::cast_slice(data));
            },
            UniformType::F32x4 => unsafe {
                gl.uniform_4_f32_slice(location, bytemuck::cast_slice(data));
            },
            UniformType::F32x4x4 => unsafe {
                gl.uniform_matrix_4_f32_slice(location, false, bytemuck::cast_slice(data));
            },
            UniformType::I32 => unsafe {
                gl.uniform_1_i32_slice(location, bytemuck::cast_slice(data));
            },
            UniformType::I32x2 => unsafe {
                gl.uniform_2_i32_slice(location, bytemuck::cast_slice(data));
            },
            UniformType::I32x3 => unsafe {
                gl.uniform_3_i32_slice(location, bytemuck::cast_slice(data));
            },
            UniformType::I32x4 => unsafe {
                gl.uniform_4_i32_slice(location, bytemuck::cast_slice(data));
            },
        }
    }
}

pub trait UniformBlock: Pod + Debug {
    const FIELDS: &'static [UniformField];
}

#[derive(Debug, Clone, Copy)]
pub struct UniformField {
    pub name: &'static str,
    pub ty: UniformType,
    pub off: usize,
    pub sz: usize,
}

impl<T: UniformVal, const N: usize> UniformVal for [T; N] {
    const UNIFORM_TYPE: UniformType = T::UNIFORM_TYPE;
}

impl UniformVal for f32 {
    const UNIFORM_TYPE: UniformType = UniformType::F32;
}

impl UniformVal for Vec2 {
    const UNIFORM_TYPE: UniformType = UniformType::F32x2;
}

impl UniformVal for Vec3 {
    const UNIFORM_TYPE: UniformType = UniformType::F32x3;
}

impl UniformVal for Vec4 {
    const UNIFORM_TYPE: UniformType = UniformType::F32x4;
}

impl UniformVal for super::Color {
    const UNIFORM_TYPE: UniformType = UniformType::F32x4;
}

impl UniformVal for Mat4 {
    const UNIFORM_TYPE: UniformType = UniformType::F32x4x4;
}

impl UniformVal for i32 {
    const UNIFORM_TYPE: UniformType = UniformType::I32;
}

impl UniformVal for IVec2 {
    const UNIFORM_TYPE: UniformType = UniformType::I32x2;
}

impl UniformVal for IVec3 {
    const UNIFORM_TYPE: UniformType = UniformType::I32x3;
}

impl UniformVal for IVec4 {
    const UNIFORM_TYPE: UniformType = UniformType::I32x4;
}

pub trait UniformVal: Pod + Debug {
    const UNIFORM_TYPE: UniformType;
}

#[derive(Clone, Copy, Debug)]
pub enum UniformType {
    F32,
    F32x2,
    F32x3,
    F32x4,
    F32x4x4,
    I32,
    I32x2,
    I32x3,
    I32x4,
}

impl UniformType {
    pub fn size(self) -> usize {
        match self {
            UniformType::F32 => 4,
            UniformType::F32x2 => 8,
            UniformType::F32x3 => 12,
            UniformType::F32x4 => 16,
            UniformType::F32x4x4 => 64,
            UniformType::I32 => 4,
            UniformType::I32x2 => 8,
            UniformType::I32x3 => 12,
            UniformType::I32x4 => 16,
        }
    }
}

pub const fn uniform_type_of_val<T: UniformVal>(_x: T) -> UniformType {
    T::UNIFORM_TYPE
}
