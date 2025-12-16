use std::fmt::Debug;

use crate::Texture2DBinding;

pub trait ImagesBlock<'a>: Debug + 'a {
    const LEN: usize;

    fn as_slice(&'a self) -> &'a [Texture2DBinding<'a>];
}

impl<'a, const N: usize> ImagesBlock<'a> for [Texture2DBinding<'a>; N] {
    const LEN: usize = N;

    fn as_slice(&'a self) -> &'a [Texture2DBinding<'a>] {
        self.as_slice()
    }
}
