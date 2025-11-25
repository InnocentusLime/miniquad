use std::cell::Cell;
use std::marker::PhantomData;
use std::rc::Rc;

use bytemuck::{Pod, Zeroable};

use crate::graphics::gl::GlContext;
use crate::graphics::BufferUsage;
use crate::native::gl::*;

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

#[derive(Clone)]
pub struct Buffer<T: Default + Zeroable + Pod + 'static> {
    internal: Rc<BufferInternal>,
    _phantom: PhantomData<&'static [T]>,
}

impl<T: Default + Zeroable + Pod + 'static> Buffer<T> {
    pub fn new_empty(ctx: Rc<GlContext>, usage: BufferUsage, size: usize) -> Buffer<T> {
        assert_eq!(size % std::mem::size_of::<T>(), 0, "size must be aligned");

        let mut cache = ctx.cache.borrow_mut();
        let mut gl_buf: u32 = 0;

        unsafe {
            glGenBuffers(1, &mut gl_buf as *mut _);
        }
        cache.store_buffer_binding();
        cache.bind_buffer(gl_buf);
        unsafe {
            glBufferData(
                GL_ARRAY_BUFFER,
                size as GLsizei,
                std::ptr::null(),
                gl_usage(usage),
            );
        }
        cache.restore_buffer_binding();

        std::mem::drop(cache);
        let buffer = BufferInternal {
            ctx,
            gl_buf,
            size: Cell::new(size),
        };
        Buffer {
            internal: Rc::new(buffer),
            _phantom: PhantomData,
        }
    }

    pub fn new(ctx: Rc<GlContext>, usage: BufferUsage, data: &[T]) -> Buffer<T> {
        let mut cache = ctx.cache.borrow_mut();
        let data: &[u8] = bytemuck::cast_slice(data);
        let size = data.len();
        let mut gl_buf: u32 = 0;

        unsafe {
            glGenBuffers(1, &mut gl_buf as *mut _);
        }
        cache.store_buffer_binding();
        cache.bind_buffer(gl_buf);
        unsafe {
            glBufferData(
                GL_ARRAY_BUFFER,
                size as GLsizei,
                data.as_ptr() as *const GLvoid,
                gl_usage(usage),
            );
        }
        cache.restore_buffer_binding();

        std::mem::drop(cache);
        let buffer = BufferInternal {
            ctx,
            gl_buf,
            size: Cell::new(size),
        };
        Buffer {
            internal: Rc::new(buffer),
            _phantom: PhantomData,
        }
    }

    pub fn size(&self) -> usize {
        self.internal.size.get()
    }

    pub fn update(&self, data: &[T]) {
        let mut cache = self.internal.ctx.cache.borrow_mut();
        let data: &[u8] = bytemuck::cast_slice(data);
        let size = data.len();
        assert!(size <= self.size());

        cache.store_buffer_binding();
        cache.bind_buffer(self.internal.gl_buf);
        unsafe { glBufferSubData(GL_ARRAY_BUFFER, 0, size as _, data.as_ptr() as _) };
        cache.restore_buffer_binding();
    }

    pub fn gl_buf(&self) -> GLuint {
        self.internal.gl_buf
    }

    pub fn binding(&self, offset: u32) -> BufferBinding {
        BufferBinding {
            gl_buf: self.gl_buf(),
            offset,
            stride: std::mem::size_of::<T>() as u32,
        }
    }
}

#[derive(Clone)]
pub struct BufferInternal {
    ctx: Rc<GlContext>,
    gl_buf: GLuint,
    size: Cell<usize>,
}

impl Drop for BufferInternal {
    fn drop(&mut self) {
        unsafe { glDeleteBuffers(1, &self.gl_buf as *const _) }
    }
}

fn gl_usage(usage: BufferUsage) -> GLenum {
    match usage {
        BufferUsage::Immutable => GL_STATIC_DRAW,
        BufferUsage::Dynamic => GL_DYNAMIC_DRAW,
        BufferUsage::Stream => GL_STREAM_DRAW,
    }
}

#[derive(Clone, Copy)]
pub struct BufferBinding {
    pub(crate) gl_buf: GLuint,
    pub(crate) offset: u32,
    pub(crate) stride: u32,
}
