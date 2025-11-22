use crate::graphics::gl::GlContext;
use crate::graphics::{BufferId, BufferSource, BufferType, BufferUsage};
use crate::native::gl::*;

#[derive(Clone, Copy, Debug)]
pub struct Buffer {
    pub gl_buf: GLuint,
    pub buffer_type: BufferType,
    pub size: usize,
    // Dimension of the indices for this buffer,
    // used only as a type argument for glDrawElements and can be
    // 1, 2 or 4
    pub index_type: Option<u32>,
}

impl GlContext {
    pub fn new_gl_buffer(
        &mut self,
        type_: BufferType,
        usage: BufferUsage,
        data: BufferSource,
    ) -> BufferId {
        let gl_target = gl_buffer_target(&type_);
        let gl_usage = gl_usage(&usage);
        let (size, element_size) = match &data {
            BufferSource::Slice(data) => (data.size, data.element_size),
            BufferSource::Empty { size, element_size } => (*size, *element_size),
        };
        let index_type = match type_ {
            BufferType::IndexBuffer
                if element_size == 1 || element_size == 2 || element_size == 4 =>
            {
                Some(element_size as u32)
            }
            BufferType::IndexBuffer => panic!("unsupported index buffer dimension"),
            BufferType::VertexBuffer => None,
        };
        let mut gl_buf: u32 = 0;

        unsafe {
            glGenBuffers(1, &mut gl_buf as *mut _);
            self.cache.store_buffer_binding(gl_target);
            self.cache.bind_buffer(gl_target, gl_buf, index_type);

            glBufferData(gl_target, size as _, std::ptr::null() as *const _, gl_usage);
            if let BufferSource::Slice(data) = data {
                debug_assert!(data.is_slice);
                glBufferSubData(gl_target, 0, size as _, data.ptr as _);
            }
            self.cache.restore_buffer_binding(gl_target);
        }

        let buffer = Buffer {
            gl_buf,
            buffer_type: type_,
            size,
            index_type,
        };

        BufferId(self.buffers.add(buffer))
    }

    pub fn gl_buffer_update(&mut self, buffer: BufferId, data: BufferSource) {
        let data = match data {
            BufferSource::Slice(data) => data,
            _ => panic!("buffer_update expects BufferSource::slice"),
        };
        debug_assert!(data.is_slice);
        let buffer = &self.buffers[buffer.0];

        if matches!(buffer.buffer_type, BufferType::IndexBuffer) {
            assert!(buffer.index_type.is_some());
            assert!(data.element_size as u32 == buffer.index_type.unwrap());
        };

        let size = data.size;

        assert!(size <= buffer.size);

        let gl_target = gl_buffer_target(&buffer.buffer_type);
        self.cache.store_buffer_binding(gl_target);
        self.cache
            .bind_buffer(gl_target, buffer.gl_buf, buffer.index_type);
        unsafe { glBufferSubData(gl_target, 0, size as _, data.ptr as _) };
        self.cache.restore_buffer_binding(gl_target);
    }

    pub fn gl_buffer_size(&mut self, buffer: BufferId) -> usize {
        self.buffers[buffer.0].size
    }

    pub fn delete_gl_buffer(&mut self, buffer: BufferId) {
        unsafe { glDeleteBuffers(1, &self.buffers[buffer.0].gl_buf as *const _) }
        self.cache.clear_buffer_bindings();
        self.cache.clear_vertex_attributes();
        self.buffers.remove(buffer.0);
    }
}

fn gl_buffer_target(buffer_type: &BufferType) -> GLenum {
    match buffer_type {
        BufferType::VertexBuffer => GL_ARRAY_BUFFER,
        BufferType::IndexBuffer => GL_ELEMENT_ARRAY_BUFFER,
    }
}

fn gl_usage(usage: &BufferUsage) -> GLenum {
    match usage {
        BufferUsage::Immutable => GL_STATIC_DRAW,
        BufferUsage::Dynamic => GL_DYNAMIC_DRAW,
        BufferUsage::Stream => GL_STREAM_DRAW,
    }
}
