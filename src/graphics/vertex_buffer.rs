use std::marker::PhantomData;
use std::rc::Rc;

use bytemuck::Pod;
use glow::HasContext;

use crate::graphics::{BufferUsage, GlContext};

#[macro_export]
macro_rules! bind_vertex_buffers {
    (
        $(
            ($buf:expr) as <$Type:path>::$field:tt
        ),+
        $(,)?
    ) => {
        [$(
            $crate::bind_vertex_buffer!($buf, $Type, $field)
        ),+]
    };
    () => { [] }
}

#[macro_export]
macro_rules! bind_vertex_buffer {
    ($buf:expr, $Type:path, $field:tt) => {{
        let local: &VertexBuffer<$Type> = $buf;
        local.binding($crate::offset_of!($Type, $field) as u32)
    }};
}

#[derive(Debug)]
pub struct VertexBuffer<T: Pod + Default> {
    ctx: Rc<GlContext>,
    pub(crate) gl_buf: glow::Buffer,
    size: usize,
    _phantom: PhantomData<&'static [T]>,
}

impl<T: Pod + Default> VertexBuffer<T> {
    pub fn new_empty(ctx: Rc<GlContext>, usage: BufferUsage, size: usize) -> VertexBuffer<T> {
        assert_eq!(size % std::mem::size_of::<T>(), 0, "size must be aligned");

        let mut cache = ctx.cache.borrow_mut();
        let gl_buf = unsafe { ctx.gl.create_buffer().unwrap() };
        cache.bind_buffer(&ctx.gl, gl_buf);
        unsafe {
            ctx.gl
                .buffer_data_size(glow::ARRAY_BUFFER, size as i32, super::gl_usage(usage));
        }

        std::mem::drop(cache);
        VertexBuffer {
            ctx,
            gl_buf,
            size,
            _phantom: PhantomData,
        }
    }

    pub fn new(ctx: Rc<GlContext>, usage: BufferUsage, data: &[T]) -> VertexBuffer<T> {
        let data: &[u8] = bytemuck::cast_slice(data);
        
        let mut cache = ctx.cache.borrow_mut();
        let gl_buf = unsafe { ctx.gl.create_buffer().unwrap() };
        cache.bind_buffer(&ctx.gl, gl_buf);
        unsafe {
            ctx.gl
                .buffer_data_u8_slice(glow::ARRAY_BUFFER, data, super::gl_usage(usage));
        }

        std::mem::drop(cache);
        VertexBuffer {
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
        cache.bind_buffer(&self.ctx.gl, self.gl_buf);
        unsafe {
            self.ctx
                .gl
                .buffer_sub_data_u8_slice(glow::ARRAY_BUFFER, 0, data)
        };
    }

    pub fn binding(&self, offset: u32) -> VertexBufferBinding<'_> {
        VertexBufferBinding {
            gl_buf: self.gl_buf,
            offset,
            stride: std::mem::size_of::<T>() as u32,
            _phantom: PhantomData,
        }
    }
}

impl<T: Pod + Default> Drop for VertexBuffer<T> {
    fn drop(&mut self) {
        unsafe {
            self.ctx.gl.delete_buffer(self.gl_buf);
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct VertexBufferBinding<'a> {
    pub(crate) gl_buf: glow::Buffer,
    pub(crate) offset: u32,
    pub(crate) stride: u32,
    _phantom: PhantomData<&'a glow::Buffer  >,
}
