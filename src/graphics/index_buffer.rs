use std::rc::Rc;
use std::{fmt::Debug, marker::PhantomData};

use bytemuck::Pod;
use glow::HasContext;

use crate::check_gl;
use crate::graphics::{BufferUsage, Error, GlContext, Result};

static TARGET_NAME: &str = "gl.index_buffer";

#[derive(Debug)]
pub struct IndexBuffer<T: VertexIndex = u16> {
    ctx: Rc<GlContext>,
    pub(crate) gl_buf: glow::Buffer,
    size: usize,
    _phantom: PhantomData<&'static [T]>,
}

impl<T: VertexIndex> IndexBuffer<T> {
    pub fn new_empty(
        ctx: Rc<GlContext>,
        usage: BufferUsage,
        size: usize,
    ) -> Result<IndexBuffer<T>> {
        let size = size * std::mem::size_of::<T>();
        let gl_buf = create_and_bind_buffer(&ctx, std::any::type_name::<T>())?;
        unsafe {
            ctx.gl.buffer_data_size(
                glow::ELEMENT_ARRAY_BUFFER,
                size as i32,
                super::gl_usage(usage),
            );
        }
        check_gl!(&ctx.gl, Error::BufferInit);
        Ok(IndexBuffer { ctx, gl_buf, size, _phantom: PhantomData })
    }

    pub fn new(ctx: Rc<GlContext>, usage: BufferUsage, data: &[T]) -> Result<IndexBuffer<T>> {
        let data: &[u8] = bytemuck::cast_slice(data);
        let gl_buf = create_and_bind_buffer(&ctx, std::any::type_name::<T>())?;
        unsafe {
            ctx.gl
                .buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, data, super::gl_usage(usage));
        }
        check_gl!(&ctx.gl, Error::BufferInit);
        Ok(IndexBuffer { ctx, gl_buf, size: data.len(), _phantom: PhantomData })
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn update(&self, data: &[T]) {
        let data: &[u8] = bytemuck::cast_slice(data);
        assert!(data.len() <= self.size());

        let mut cache = self.ctx.cache.borrow_mut();
        cache.bind_index_buffer(&self.ctx.gl, self.gl_buf);
        unsafe {
            self.ctx
                .gl
                .buffer_sub_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, 0, data)
        };
    }
}

impl<T: VertexIndex> Drop for IndexBuffer<T> {
    fn drop(&mut self) {
        tracing::debug!(
            target: TARGET_NAME,
            index_ty = std::any::type_name::<T>(),
            "dropping: {:?}",
            self.gl_buf,
        );
        unsafe { self.ctx.gl.delete_buffer(self.gl_buf) }
    }
}

pub trait VertexIndex: Pod + Debug {
    const GL_TYPE: u32;

    fn offset_by(self, off: usize) -> Self;
}

impl VertexIndex for u8 {
    const GL_TYPE: u32 = glow::UNSIGNED_BYTE;

    fn offset_by(self, off: usize) -> Self {
        self + off as u8
    }
}

impl VertexIndex for u16 {
    const GL_TYPE: u32 = glow::UNSIGNED_SHORT;

    fn offset_by(self, off: usize) -> Self {
        self + off as u16
    }
}

impl VertexIndex for u32 {
    const GL_TYPE: u32 = glow::UNSIGNED_SHORT;

    fn offset_by(self, off: usize) -> Self {
        self + off as u32
    }
}

fn create_and_bind_buffer(ctx: &GlContext, ty_name: &'static str) -> Result<glow::Buffer> {
    let mut cache = ctx.cache.borrow_mut();
    let Ok(gl_buf) = (unsafe { ctx.gl.create_buffer() }) else {
        return Err(Error::BufferAlloc);
    };
    tracing::debug!(
        target: TARGET_NAME,
        index_ty = ty_name,
        "new: {gl_buf:?}",
    );
    cache.bind_index_buffer(&ctx.gl, gl_buf);
    Ok(gl_buf)
}
