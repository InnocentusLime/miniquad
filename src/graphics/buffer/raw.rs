use std::fmt::Debug;
use std::rc::Rc;

use glow::HasContext;

use crate::check_gl;
use crate::graphics::{Error, GlContext, Result};

static TARGET_NAME: &str = "gl.buffer";

#[derive(Debug)]
pub struct RawBuffer {
    target: u32,
    ctx: Rc<GlContext>,
    gl_buf: glow::Buffer,
    size: usize,
}

impl RawBuffer {
    pub fn new_empty(
        ctx: Rc<GlContext>,
        target: u32,
        usage: BufferUsage,
        size: usize,
    ) -> Result<RawBuffer> {
        let gl_buf = create_and_bind_buffer(&ctx, target)?;
        unsafe {
            ctx.gl
                .buffer_data_size(target, size as i32, usage.to_gl_usage());
        }
        check_gl!(&ctx.gl, Error::BufferInit);
        Ok(RawBuffer { ctx, gl_buf, size, target })
    }

    pub fn new(
        ctx: Rc<GlContext>,
        target: u32,
        usage: BufferUsage,
        data: &[u8],
    ) -> Result<RawBuffer> {
        let gl_buf = create_and_bind_buffer(&ctx, target)?;
        unsafe {
            ctx.gl
                .buffer_data_u8_slice(target, data, usage.to_gl_usage());
        }
        check_gl!(&ctx.gl, Error::BufferInit);
        Ok(RawBuffer { ctx, gl_buf, size: data.len(), target })
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn update(&self, data: &[u8]) {
        assert!(data.len() <= self.size());

        let mut cache = self.ctx.cache.borrow_mut();
        cache.bind_buffer(&self.ctx.gl, self.target, self.gl_buf);
        unsafe { self.ctx.gl.buffer_sub_data_u8_slice(self.target, 0, data) };
    }

    pub fn bind(&self) {
        let mut cache = self.ctx.cache.borrow_mut();
        cache.bind_buffer(&self.ctx.gl, self.target, self.gl_buf);
    }
}

impl Drop for RawBuffer {
    fn drop(&mut self) {
        tracing::debug!(
            target: TARGET_NAME,
            "dropping: {:?}",
            self.gl_buf,
        );
        unsafe { self.ctx.gl.delete_buffer(self.gl_buf) }
    }
}

fn create_and_bind_buffer(ctx: &GlContext, target: u32) -> Result<glow::Buffer> {
    let mut cache = ctx.cache.borrow_mut();
    let Ok(gl_buf) = (unsafe { ctx.gl.create_buffer() }) else {
        return Err(Error::BufferAlloc);
    };
    tracing::debug!(
        target: TARGET_NAME,
        "new: {gl_buf:?}",
    );
    cache.bind_buffer(&ctx.gl, target, gl_buf);
    Ok(gl_buf)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BufferUsage {
    Immutable,
    Dynamic,
    Stream,
}

impl BufferUsage {
    fn to_gl_usage(self) -> u32 {
        match self {
            BufferUsage::Immutable => glow::STATIC_DRAW,
            BufferUsage::Dynamic => glow::DYNAMIC_DRAW,
            BufferUsage::Stream => glow::STREAM_DRAW,
        }
    }
}
