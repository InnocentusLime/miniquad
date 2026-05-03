use std::fmt::Debug;

#[derive(Debug)]
pub struct NoImages;

impl ImagesUniformBlock for NoImages {
    const FIELDS: &[ImageUniformField] = &[];

    type Borrow<'a> = &'a Self;

    fn bind(_raw: Self::Borrow<'_>) { /* NOOP */
    }
}

#[macro_export]
macro_rules! image_uniform_of {
    ($Type:path, $field:tt) => {{
        $crate::graphics::ImageUniformField {
            name: stringify!($field),
            gl_type: $crate::graphics::gl_type_of_image_uniform_val::<$Type>(),
        }
    }};
}

pub trait ImagesUniformBlock {
    const FIELDS: &'static [ImageUniformField];

    type Borrow<'a>: Debug + Copy;

    fn bind(raw: Self::Borrow<'_>);
}

#[derive(Debug, Clone, Copy)]
pub struct ImageUniformField {
    pub name: &'static str,
    pub gl_type: u32,
}

pub trait ImageUniformVal: Debug {
    const GL_TYPE: u32;

    fn bind(&self, slot: u32);
}

pub const fn gl_type_of_image_uniform_val<T: ImageUniformVal>() -> u32 {
    T::GL_TYPE
}
