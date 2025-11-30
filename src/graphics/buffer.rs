use std::marker::PhantomData;
use std::rc::Rc;

use bytemuck::Pod;
use glow::HasContext;

use crate::graphics::{BufferUsage, GlContext};

#[macro_export]
macro_rules! bind_buffers {
    (
        $(
            ($buf:expr) as <$Type:path>::$field:tt
        ),+
        $(,)?
    ) => {
        [$(
            $crate::bind_buffer!($buf, $Type, $field)
        ),+]
    };
    () => { [] }
}

#[macro_export]
macro_rules! bind_buffer {
    ($buf:expr, $Type:path, $field:tt) => {{
        let local: &Buffer<$Type> = $buf;
        local.binding($crate::offset_of!($Type, $field) as u32)
    }};
}

#[derive(Debug, Clone)]
pub struct Buffer<T: Pod + Default> {
    internal: Rc<BufferInternal>,
    pub(crate) gl_buf: glow::Buffer,
    _phantom: PhantomData<&'static [T]>,
}

impl<T: Pod + Default> Buffer<T> {
    pub fn new_empty(ctx: Rc<GlContext>, usage: BufferUsage, size: usize) -> Buffer<T> {
        assert_eq!(size % std::mem::size_of::<T>(), 0, "size must be aligned");

        let mut cache = ctx.cache.borrow_mut();
        let gl_buf = unsafe { ctx.gl.create_buffer().unwrap() };
        cache.bind_buffer(&ctx.gl, gl_buf);
        unsafe {
            ctx.gl
                .buffer_data_size(glow::ARRAY_BUFFER, size as i32, super::gl_usage(usage));
        }

        std::mem::drop(cache);
        let internal = BufferInternal { ctx, gl_buf, size };
        Buffer {
            gl_buf,
            internal: Rc::new(internal),
            _phantom: PhantomData,
        }
    }

    pub fn new(ctx: Rc<GlContext>, usage: BufferUsage, data: &[T]) -> Buffer<T> {
        let mut cache = ctx.cache.borrow_mut();
        let data: &[u8] = bytemuck::cast_slice(data);
        let size = data.len();
        let gl_buf = unsafe { ctx.gl.create_buffer().unwrap() };
        cache.bind_buffer(&ctx.gl, gl_buf);
        unsafe {
            ctx.gl
                .buffer_data_u8_slice(glow::ARRAY_BUFFER, data, super::gl_usage(usage));
        }

        std::mem::drop(cache);
        let internal = BufferInternal { ctx, gl_buf, size };
        Buffer {
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

        cache.bind_buffer(&self.internal.ctx.gl, self.gl_buf());
        unsafe {
            self.internal
                .ctx
                .gl
                .buffer_sub_data_u8_slice(glow::ARRAY_BUFFER, 0, data)
        };
    }

    pub fn gl_buf(&self) -> glow::Buffer {
        self.gl_buf
    }

    pub fn binding(&self, offset: u32) -> BufferBinding<'_> {
        BufferBinding {
            gl_buf: self.gl_buf(),
            offset,
            stride: std::mem::size_of::<T>() as u32,
            _phantom: PhantomData,
        }
    }
}

#[derive(Debug)]
struct BufferInternal {
    ctx: Rc<GlContext>,
    gl_buf: glow::Buffer,
    size: usize,
}

impl Drop for BufferInternal {
    fn drop(&mut self) {
        unsafe {
            self.ctx.gl.delete_buffer(self.gl_buf);
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BufferBinding<'a> {
    pub(crate) gl_buf: glow::Buffer,
    pub(crate) offset: u32,
    pub(crate) stride: u32,
    _phantom: PhantomData<&'a BufferInternal>,
}
