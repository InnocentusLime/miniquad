use std::fmt::Debug;

use crate::Texture2D;

pub trait ImagesBlock {
    type Names;
    type Borrow<'a>: Debug;

    fn as_slice<'a>(x: Self::Borrow<'a>) -> &'a [&'a Texture2D];
    fn names(x: &'static Self::Names) -> &'static [&'static str];
}

impl<const N: usize> ImagesBlock for [Texture2D; N] {
    type Names = [&'static str; N];
    type Borrow<'a> = &'a [&'a Texture2D; N];

    fn as_slice<'a>(x: Self::Borrow<'a>) -> &'a [&'a Texture2D] {
        x.as_slice()
    }

    fn names(x: &'static Self::Names) -> &'static [&'static str] {
        x.as_slice()
    }
}
