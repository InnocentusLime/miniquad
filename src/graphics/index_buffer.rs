use std::marker::PhantomData;
use std::rc::Rc;

use bytemuck::Pod;
use glow::HasContext;

use crate::graphics::{BufferUsage, GlContext};

#[derive(Clone)]
pub struct IndexBuffer<T: IndexBufferElement = u16> {
    internal: Rc<IndexBufferInternal>,
    pub(crate) gl_buf: glow::Buffer,
    _phantom: PhantomData<&'static [T]>,
}

impl<T: IndexBufferElement> IndexBuffer<T> {
    pub fn new_empty(ctx: Rc<GlContext>, usage: BufferUsage, size: usize) -> IndexBuffer<T> {
        assert_eq!(size % std::mem::size_of::<T>(), 0, "size must be aligned");

        let mut cache = ctx.cache.borrow_mut();
        let gl_buf = unsafe { ctx.gl.create_buffer().unwrap() };
        cache.bind_index_buffer(&ctx.gl, gl_buf);
        unsafe {
            ctx.gl.buffer_data_size(
                glow::ELEMENT_ARRAY_BUFFER,
                size as i32,
                super::gl_usage(usage),
            );
        }

        std::mem::drop(cache);
        let buffer = IndexBufferInternal { ctx, gl_buf, size };
        IndexBuffer {
            gl_buf,
            internal: Rc::new(buffer),
            _phantom: PhantomData,
        }
    }

    pub fn new(ctx: Rc<GlContext>, usage: BufferUsage, data: &[T]) -> IndexBuffer<T> {
        let mut cache = ctx.cache.borrow_mut();
        let data: &[u8] = bytemuck::cast_slice(data);
        let size = data.len();
        let gl_buf = unsafe { ctx.gl.create_buffer().unwrap() };
        cache.bind_index_buffer(&ctx.gl, gl_buf);
        unsafe {
            ctx.gl
                .buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, data, super::gl_usage(usage));
        }

        std::mem::drop(cache);
        let internal = IndexBufferInternal { ctx, gl_buf, size };
        IndexBuffer {
            gl_buf,
            internal: Rc::new(internal),
            _phantom: PhantomData,
        }
    }

    pub fn size(&self) -> usize {
        self.internal.size
    }

    pub fn update(&self, data: &[T]) {
        let mut cache = self.internal.ctx.cache.borrow_mut();
        let data: &[u8] = bytemuck::cast_slice(data);
        let size = data.len();
        assert!(size <= self.size());

        cache.bind_index_buffer(&self.internal.ctx.gl, self.gl_buf);
        unsafe {
            self.internal
                .ctx
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

#[derive(Clone)]
struct IndexBufferInternal {
    ctx: Rc<GlContext>,
    gl_buf: glow::Buffer,
    size: usize,
}

impl Drop for IndexBufferInternal {
    fn drop(&mut self) {
        unsafe { self.ctx.gl.delete_buffer(self.gl_buf) }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct IndexBufferBinding<'a> {
    pub(crate) sz_elem: i32,
    pub(crate) gl_type: u32,
    pub(crate) gl_buf: glow::Buffer,
    _phantom: PhantomData<&'a IndexBufferInternal>,
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
