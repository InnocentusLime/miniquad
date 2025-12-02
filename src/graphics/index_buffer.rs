use std::marker::PhantomData;
use std::rc::Rc;

use bytemuck::Pod;
use glow::HasContext;

use crate::graphics::{BufferUsage, GlContext};

static TARGET_NAME: &str = "gl.index_buffer";

#[derive(Debug)]
pub struct IndexBuffer<T: IndexBufferElement = u16> {
    ctx: Rc<GlContext>,
    pub(crate) gl_buf: glow::Buffer,
    size: usize,
    _phantom: PhantomData<&'static [T]>,
}

impl<T: IndexBufferElement> IndexBuffer<T> {
    pub fn new_empty(ctx: Rc<GlContext>, usage: BufferUsage, size: usize) -> IndexBuffer<T> {
        assert_eq!(size % std::mem::size_of::<T>(), 0, "size must be aligned");
        let gl_buf = create_and_bind_buffer(&ctx, std::any::type_name::<T>());
        unsafe {
            ctx.gl.buffer_data_size(
                glow::ELEMENT_ARRAY_BUFFER,
                size as i32,
                super::gl_usage(usage),
            );
        }
        IndexBuffer {
            ctx,
            gl_buf,
            size,
            _phantom: PhantomData,
        }
    }

    pub fn new(ctx: Rc<GlContext>, usage: BufferUsage, data: &[T]) -> IndexBuffer<T> {
        let data: &[u8] = bytemuck::cast_slice(data);
        let gl_buf = create_and_bind_buffer(&ctx, std::any::type_name::<T>());
        unsafe {
            ctx.gl
                .buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, data, super::gl_usage(usage));
        }
        IndexBuffer {
            ctx,
            gl_buf,
            size: data.len(),
            _phantom: PhantomData,
        }
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

    pub fn bind(&self) -> IndexBufferBinding<'_> {
        IndexBufferBinding {
            sz_elem: std::mem::size_of::<T>() as i32,
            gl_type: T::GL_TYPE,
            gl_buf: self.gl_buf,
            _phantom: PhantomData,
        }
    }
}

impl<T: IndexBufferElement> Drop for IndexBuffer<T> {
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

#[derive(Debug, Clone, Copy)]
pub struct IndexBufferBinding<'a> {
    pub(crate) sz_elem: i32,
    pub(crate) gl_type: u32,
    pub(crate) gl_buf: glow::Buffer,
    _phantom: PhantomData<&'a glow::Buffer>,
}

pub trait IndexBufferElement: Pod {
    const GL_TYPE: u32;
}

impl IndexBufferElement for u8 {
    const GL_TYPE: u32 = glow::UNSIGNED_BYTE;
}

impl IndexBufferElement for u16 {
    const GL_TYPE: u32 = glow::UNSIGNED_SHORT;
}

impl IndexBufferElement for u32 {
    const GL_TYPE: u32 = glow::UNSIGNED_SHORT;
}

fn create_and_bind_buffer(ctx: &GlContext, ty_name: &'static str) -> glow::Buffer {
    let mut cache = ctx.cache.borrow_mut();
    let gl_buf = unsafe { ctx.gl.create_buffer().unwrap() };
    tracing::debug!(
        target: TARGET_NAME, 
        index_ty = ty_name,
        "new: {gl_buf:?}",
    );
    cache.bind_index_buffer(&ctx.gl, gl_buf);
    gl_buf
}
