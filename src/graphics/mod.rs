mod buffer_usage;
mod cache;
mod index_buffer;
mod pipeline;
mod pipeline_params;
mod render_pass;
mod texture;
mod ty;
mod vertex_buffer;

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;

use cache::GlCache;
use glow::HasContext;
use image::DynamicImage;

pub use buffer_usage::*;
pub use index_buffer::*;
pub use pipeline::*;
pub use pipeline_params::*;
pub use render_pass::*;
pub use texture::*;
pub use ty::*;
pub use vertex_buffer::*;

#[derive(Debug)]
pub struct GlContext {
    pub(crate) client_area_size: Cell<(u32, u32)>,
    pub(crate) gl: Arc<glow::Context>,
    pub(crate) vao: glow::VertexArray,
    pub(crate) cache: RefCell<GlCache>,
}

impl GlContext {
    pub fn new(gl: Arc<glow::Context>, client_area_size: (u32, u32)) -> GlContext {
        let vao = unsafe { gl.create_vertex_array().unwrap() };
        unsafe {
            gl.bind_vertex_array(Some(vao));
        }
        GlContext {
            cache: RefCell::new(GlCache::new()),
            client_area_size: Cell::new(client_area_size),
            gl,
            vao,
        }
    }

    pub(crate) fn recapture_gl(&self) {
        let mut cache = self.cache.borrow_mut();
        cache.reset(&self.gl);
        unsafe {
            self.gl.bind_vertex_array(Some(self.vao));
        }
    }

    pub fn screen_size(&self) -> (u32, u32) {
        self.client_area_size.get()
    }

    pub fn new_empty_vertex_buffer<T: Vertex>(
        self: &Rc<Self>,
        usage: BufferUsage,
        size: usize,
    ) -> VertexBuffer<T> {
        VertexBuffer::new_empty(self.clone(), usage, size)
    }

    pub fn new_vertex_buffer<T: Vertex>(
        self: &Rc<Self>,
        usage: BufferUsage,
        data: &[T],
    ) -> VertexBuffer<T> {
        VertexBuffer::new(self.clone(), usage, data)
    }

    pub fn new_empty_index_buffer<I: VertexIndex>(
        self: &Rc<Self>,
        usage: BufferUsage,
        size: usize,
    ) -> IndexBuffer<I> {
        IndexBuffer::new_empty(self.clone(), usage, size)
    }

    pub fn new_index_buffer<I: VertexIndex>(
        self: &Rc<Self>,
        usage: BufferUsage,
        data: &[I],
    ) -> IndexBuffer<I> {
        IndexBuffer::new(self.clone(), usage, data)
    }

    pub fn new_empty_texture(
        self: &Rc<Self>,
        format: Texture2DFormat,
        width: u32,
        height: u32,
        wrap: TextureWrap,
        min_filter: FilterMode,
        mag_filter: FilterMode,
    ) -> Texture2D {
        Texture2D::new_empty(
            self.clone(),
            format,
            width,
            height,
            wrap,
            min_filter,
            mag_filter,
        )
    }

    pub fn new_texture(
        self: &Rc<Self>,
        source: impl Into<DynamicImage>,
        wrap: TextureWrap,
        min_filter: FilterMode,
        mag_filter: FilterMode,
    ) -> Texture2D {
        Texture2D::new(self.clone(), source, wrap, min_filter, mag_filter)
    }

    #[track_caller]
    pub fn new_pipeline<V: Vertex, U: UniformBlock, I: ImagesUniformBlock>(
        self: &Rc<Self>,
        vert_shader: &str,
        frag_shader: &str,
        params: PipelineParams,
    ) -> Pipeline<V, U, I> {
        Pipeline::new(self.clone(), vert_shader, frag_shader, params)
    }

    pub fn new_render_pass(
        self: &Rc<Self>,
        color_img: Vec<Texture2D>,
        depth_img: Option<Texture2D>,
    ) -> RenderPass {
        RenderPass::new(self.clone(), color_img, depth_img)
    }

    #[cfg(debug_assertions)]
    #[track_caller]
    pub(crate) fn check_no_gl_error(&self) {
        let err = unsafe { self.gl.get_error() };
        if err == glow::NO_ERROR {
            return;
        }
        panic!("detected error: {err:#x}");
    }

    #[cfg(not(debug_assertions))]
    pub(crate) fn check_no_gl_error(&self) { /* NOOP */
    }
}

impl Drop for GlContext {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_vertex_array(self.vao);
        }
    }
}
