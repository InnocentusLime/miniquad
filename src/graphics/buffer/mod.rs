mod raw;

use std::marker::PhantomData;
use std::rc::Rc;

pub use raw::*;

use crate::graphics::{GlContext, Result, Vertex, VertexIndex};

#[derive(Debug)]
pub struct VertexBuffer<T: Vertex> {
    pub(crate) raw: RawBuffer,
    _phantom: PhantomData<&'static [T]>,
}

impl<T: Vertex> VertexBuffer<T> {
    pub fn new_empty(
        ctx: Rc<GlContext>,
        usage: BufferUsage,
        len: usize,
    ) -> Result<VertexBuffer<T>> {
        let size = len * std::mem::size_of::<T>();
        let raw = RawBuffer::new_empty(ctx, glow::ARRAY_BUFFER, usage, size)?;
        Ok(VertexBuffer { raw, _phantom: PhantomData })
    }

    pub fn new(ctx: Rc<GlContext>, usage: BufferUsage, data: &[T]) -> Result<VertexBuffer<T>> {
        let data = bytemuck::cast_slice(data);
        let raw = RawBuffer::new(ctx, glow::ARRAY_BUFFER, usage, data)?;
        Ok(VertexBuffer { raw, _phantom: PhantomData })
    }

    pub fn len(&self) -> usize {
        self.size() / std::mem::size_of::<T>()
    }

    pub fn size(&self) -> usize {
        self.raw.size()
    }

    pub fn update(&self, data: &[T]) {
        self.raw.update(bytemuck::cast_slice(data));
    }
}

#[derive(Debug)]
pub struct IndexBuffer<T: VertexIndex = u16> {
    pub(crate) raw: RawBuffer,
    _phantom: PhantomData<&'static [T]>,
}

impl<T: VertexIndex> IndexBuffer<T> {
    pub fn new_empty(ctx: Rc<GlContext>, usage: BufferUsage, len: usize) -> Result<IndexBuffer<T>> {
        let size = len * std::mem::size_of::<T>();
        let raw = RawBuffer::new_empty(ctx, glow::ELEMENT_ARRAY_BUFFER, usage, size)?;
        Ok(IndexBuffer { raw, _phantom: PhantomData })
    }

    pub fn new(ctx: Rc<GlContext>, usage: BufferUsage, data: &[T]) -> Result<IndexBuffer<T>> {
        let data = bytemuck::cast_slice(data);
        let raw = RawBuffer::new(ctx, glow::ELEMENT_ARRAY_BUFFER, usage, data)?;
        Ok(IndexBuffer { raw, _phantom: PhantomData })
    }

    pub fn len(&self) -> usize {
        self.size() / std::mem::size_of::<T>()
    }

    pub fn size(&self) -> usize {
        self.raw.size()
    }

    pub fn update(&self, data: &[T]) {
        self.raw.update(bytemuck::cast_slice(data));
    }
}
