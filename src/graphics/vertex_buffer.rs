use std::marker::PhantomData;
use std::rc::Rc;

use glow::HasContext;

use crate::check_gl;
use crate::graphics::{BufferUsage, Error, GlContext, Result, Vertex};

static TARGET_NAME: &str = "gl.vertex_buffer";

#[derive(Debug)]
pub struct VertexBuffer<T: Vertex> {
    ctx: Rc<GlContext>,
    pub(crate) gl_buf: glow::Buffer,
    size: usize,
    _phantom: PhantomData<&'static [T]>,
}

impl<T: Vertex> VertexBuffer<T> {
    pub fn new_empty(
        ctx: Rc<GlContext>,
        usage: BufferUsage,
        size: usize,
    ) -> Result<VertexBuffer<T>> {
        let size = size * std::mem::size_of::<T>();
        let gl_buf = create_and_bind_buffer(&ctx, std::any::type_name::<T>())?;
        unsafe {
            ctx.gl
                .buffer_data_size(glow::ARRAY_BUFFER, size as i32, super::gl_usage(usage));
        }
        check_gl!(&ctx.gl, Error::BufferInit);
        Ok(VertexBuffer { ctx, gl_buf, size, _phantom: PhantomData })
    }

    pub fn new(ctx: Rc<GlContext>, usage: BufferUsage, data: &[T]) -> Result<VertexBuffer<T>> {
        let data: &[u8] = bytemuck::cast_slice(data);
        let gl_buf = create_and_bind_buffer(&ctx, std::any::type_name::<T>())?;
        unsafe {
            ctx.gl
                .buffer_data_u8_slice(glow::ARRAY_BUFFER, data, super::gl_usage(usage));
        }
        check_gl!(&ctx.gl, Error::BufferInit);
        Ok(VertexBuffer { ctx, gl_buf, size: data.len(), _phantom: PhantomData })
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
}

impl<T: Vertex> Drop for VertexBuffer<T> {
    fn drop(&mut self) {
        tracing::debug!(
            target: TARGET_NAME,
            vertex_ty = std::any::type_name::<T>(),
            "dropping: {:?}",
            self.gl_buf,
        );
        unsafe {
            self.ctx.gl.delete_buffer(self.gl_buf);
        }
    }
}

fn create_and_bind_buffer(ctx: &GlContext, ty_name: &'static str) -> Result<glow::Buffer> {
    let mut cache = ctx.cache.borrow_mut();
    let Ok(gl_buf) = (unsafe { ctx.gl.create_buffer() }) else {
        return Err(Error::BufferAlloc);
    };
    tracing::debug!(
        target: TARGET_NAME,
        vertex_ty = ty_name,
        "new: {gl_buf:?}",
    );
    cache.bind_buffer(&ctx.gl, gl_buf);
    Ok(gl_buf)
}
