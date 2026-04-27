use std::fmt::Debug;

use crate::graphics::Texture2D;

#[derive(Debug)]
pub struct NoImages;

impl ImagesBlock for NoImages {
    type Names = ();
    type Borrow<'a> = &'a Self;

    fn as_slice<'a, 'b>(_x: &'b Self::Borrow<'a>) -> &'b [&'a Texture2D] {
        &[]
    }

    fn names(_x: &()) -> &'static [&'static str] {
        &[]
    }
}

impl ImagesBlock for Texture2D {
    type Names = &'static str;
    type Borrow<'a> = &'a Texture2D;

    fn as_slice<'a, 'b>(x: &'b Self::Borrow<'a>) -> &'b [&'a Texture2D] {
        std::slice::from_ref(x)
    }

    fn names<'a>(x: &'a &'static str) -> &'a [&'static str] {
        std::slice::from_ref(x)
    }
}

impl<const N: usize> ImagesBlock for [Texture2D; N] {
    type Names = &'static [&'static str; N];
    type Borrow<'a> = &'a [&'a Texture2D; N];

    fn as_slice<'a, 'b>(x: &'b Self::Borrow<'a>) -> &'b [&'a Texture2D] {
        x.as_slice()
    }

    fn names(x: &Self::Names) -> &[&'static str] {
        x.as_slice()
    }
}

pub trait ImagesBlock {
    type Names;
    type Borrow<'a>: Debug;

    fn as_slice<'a, 'b>(x: &'b Self::Borrow<'a>) -> &'b [&'a Texture2D];
    fn names(x: &Self::Names) -> &[&'static str];
}
