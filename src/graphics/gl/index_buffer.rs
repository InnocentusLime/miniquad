use std::cell::Cell;
use std::marker::PhantomData;
use std::rc::Rc;

use bytemuck::Pod;

use crate::graphics::gl::GlContext;
use crate::graphics::BufferUsage;
use crate::native::gl::*;

#[derive(Clone)]
pub struct IndexBuffer<T: IndexBufferElement = u16> {
    internal: Rc<IndexBufferInternal>,
    _phantom: PhantomData<&'static [T]>,
}

impl<T: IndexBufferElement> IndexBuffer<T> {
    pub fn new_empty(ctx: Rc<GlContext>, usage: BufferUsage, size: usize) -> IndexBuffer<T> {
        assert_eq!(size % std::mem::size_of::<T>(), 0, "size must be aligned");

        let mut cache = ctx.cache.borrow_mut();
        let mut gl_buf: GLuint = 0;

        unsafe {
            glGenBuffers(1, &mut gl_buf as *mut _);
        }
        cache.store_index_buffer_binding();
        cache.bind_index_buffer(gl_buf, std::mem::size_of::<T>() as u32);
        unsafe {
            glBufferData(
                GL_ELEMENT_ARRAY_BUFFER,
                size as GLsizei,
                std::ptr::null(),
                gl_usage(usage),
            );
        }
        cache.restore_index_buffer_binding();

        std::mem::drop(cache);
        let buffer = IndexBufferInternal {
            ctx,
            gl_buf,
            size: Cell::new(size),
        };
        IndexBuffer {
            internal: Rc::new(buffer),
            _phantom: PhantomData,
        }
    }

    pub fn new(ctx: Rc<GlContext>, usage: BufferUsage, data: &[T]) -> IndexBuffer<T> {
        let mut cache = ctx.cache.borrow_mut();
        let data: &[u8] = bytemuck::cast_slice(data);
        let size = data.len();
        let mut gl_buf: GLuint = 0;

        unsafe {
            glGenBuffers(1, &mut gl_buf as *mut GLuint);
        }
        cache.store_index_buffer_binding();
        cache.bind_index_buffer(gl_buf, std::mem::size_of::<T>() as u32);
        unsafe {
            glBufferData(
                GL_ELEMENT_ARRAY_BUFFER,
                size as GLsizei,
                data.as_ptr() as *const GLvoid,
                gl_usage(usage),
            );
        }
        cache.restore_index_buffer_binding();

        std::mem::drop(cache);
        let buffer = IndexBufferInternal {
            ctx,
            gl_buf,
            size: Cell::new(size),
        };
        IndexBuffer {
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

        cache.store_index_buffer_binding();
        cache.bind_index_buffer(self.internal.gl_buf, std::mem::size_of::<T>() as u32);
        unsafe { glBufferSubData(GL_ELEMENT_ARRAY_BUFFER, 0, size as _, data.as_ptr() as _) };
        cache.restore_index_buffer_binding();
    }

    pub fn gl_buf(&self) -> GLuint {
        self.internal.gl_buf
    }
}

#[derive(Clone)]
pub struct IndexBufferInternal {
    ctx: Rc<GlContext>,
    gl_buf: GLuint,
    size: Cell<usize>,
}

impl Drop for IndexBufferInternal {
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

pub trait IndexBufferElement: 'static + Copy + Pod {
    const GL_TYPE: GLenum;
}

impl IndexBufferElement for u8 {
    const GL_TYPE: GLenum = GL_UNSIGNED_BYTE;
}

impl IndexBufferElement for u16 {
    const GL_TYPE: GLenum = GL_UNSIGNED_SHORT;
}

impl IndexBufferElement for u32 {
    const GL_TYPE: GLenum = GL_UNSIGNED_SHORT;
}
