use std::cell::Cell;
use std::rc::Rc;

use crate::graphics::gl::GlContext;
use crate::graphics::{BufferSource, BufferType, BufferUsage};
use crate::native::gl::*;

#[derive(Clone)]
pub struct Buffer(Rc<BufferInternal>);

impl Buffer {
    pub fn new(
        ctx: Rc<GlContext>,
        type_: BufferType,
        usage: BufferUsage,
        data: BufferSource,
    ) -> Buffer {
        let mut cache = ctx.cache.borrow_mut();

        let gl_target = gl_buffer_target(type_);
        let gl_usage = gl_usage(usage);
        let size = match &data {
            BufferSource::Slice(data) => data.size,
            BufferSource::Empty { size, .. } => *size,
        };
        let mut gl_buf: u32 = 0;

        unsafe {
            glGenBuffers(1, &mut gl_buf as *mut _);
            cache.store_buffer_binding(gl_target);
            cache.bind_buffer(gl_target, gl_buf, gl_index_type(type_));

            glBufferData(gl_target, size as _, std::ptr::null() as *const _, gl_usage);
            if let BufferSource::Slice(data) = data {
                debug_assert!(data.is_slice);
                glBufferSubData(gl_target, 0, size as _, data.ptr as _);
            }
            cache.restore_buffer_binding(gl_target);
        }

        std::mem::drop(cache);
        let buffer = BufferInternal {
            ctx,
            gl_buf,
            buffer_type: type_,
            size: Cell::new(size),
        };
        Buffer(Rc::new(buffer))
    }

    pub fn size(&self) -> usize {
        self.0.size.get()
    }

    pub fn update(&self, data: BufferSource) {
        let mut cache = self.0.ctx.cache.borrow_mut();

        let data = match data {
            BufferSource::Slice(data) => data,
            _ => panic!("buffer_update expects BufferSource::slice"),
        };
        debug_assert!(data.is_slice);

        if let BufferType::IndexBuffer(elem_size) = self.0.buffer_type {
            assert!(data.element_size as u32 == elem_size as u32);
        };

        let size = data.size;
        assert!(size <= self.size());

        let gl_target = gl_buffer_target(self.0.buffer_type);
        cache.store_buffer_binding(gl_target);
        cache.bind_buffer(gl_target, self.0.gl_buf, gl_index_type(self.0.buffer_type));
        unsafe { glBufferSubData(gl_target, 0, size as _, data.ptr as _) };
        cache.restore_buffer_binding(gl_target);
    }

    pub fn gl_buf(&self) -> GLuint {
        self.0.gl_buf
    }

    pub fn index_type(&self) -> Option<u32> {
        gl_index_type(self.0.buffer_type)
    }

    pub fn binding(&self, offset: u32, stride: u32) -> BufferBinding<'_> {
        BufferBinding {
            buffer: self,
            offset,
            stride,
        }
    }
}

#[derive(Clone)]
pub struct BufferInternal {
    ctx: Rc<GlContext>,
    gl_buf: GLuint,
    buffer_type: BufferType,
    size: Cell<usize>,
}

impl Drop for BufferInternal {
    fn drop(&mut self) {
        unsafe { glDeleteBuffers(1, &self.gl_buf as *const _) }
    }
}

fn gl_index_type(buffer_type: BufferType) -> Option<u32> {
    match buffer_type {
        BufferType::VertexBuffer => None,
        BufferType::IndexBuffer(x) => Some(x as u32),
    }
}

fn gl_buffer_target(buffer_type: BufferType) -> GLenum {
    match buffer_type {
        BufferType::VertexBuffer => GL_ARRAY_BUFFER,
        BufferType::IndexBuffer(_) => GL_ELEMENT_ARRAY_BUFFER,
    }
}

fn gl_usage(usage: BufferUsage) -> GLenum {
    match usage {
        BufferUsage::Immutable => GL_STATIC_DRAW,
        BufferUsage::Dynamic => GL_DYNAMIC_DRAW,
        BufferUsage::Stream => GL_STREAM_DRAW,
    }
}

pub struct BufferBinding<'a> {
    pub(crate) buffer: &'a Buffer,
    pub(crate) offset: u32,
    pub(crate) stride: u32,
}

impl<'a> BufferBinding<'a> {
    pub fn gl_buf(&self) -> GLuint {
        self.buffer.gl_buf()
    }

    pub fn index_type(&self) -> Option<u32> {
        self.buffer.index_type()
    }
}
