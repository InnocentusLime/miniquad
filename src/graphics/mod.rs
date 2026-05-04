mod buffer;
mod cache;
mod error;
mod pipeline;
mod pipeline_params;
mod render_pass;
mod texture;
mod ty;

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;

use cache::GlCache;
use glow::HasContext;
use image::DynamicImage;

pub use buffer::*;
pub use error::*;
pub use pipeline::*;
pub use pipeline_params::*;
pub use render_pass::*;
pub use texture::*;
pub use ty::*;

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
    ) -> Result<VertexBuffer<T>> {
        VertexBuffer::new_empty(self.clone(), usage, size)
    }

    pub fn new_vertex_buffer<T: Vertex>(
        self: &Rc<Self>,
        usage: BufferUsage,
        data: &[T],
    ) -> Result<VertexBuffer<T>> {
        VertexBuffer::new(self.clone(), usage, data)
    }

    pub fn new_empty_index_buffer<I: VertexIndex>(
        self: &Rc<Self>,
        usage: BufferUsage,
        size: usize,
    ) -> Result<IndexBuffer<I>> {
        IndexBuffer::new_empty(self.clone(), usage, size)
    }

    pub fn new_index_buffer<I: VertexIndex>(
        self: &Rc<Self>,
        usage: BufferUsage,
        data: &[I],
    ) -> Result<IndexBuffer<I>> {
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
    ) -> Result<Texture2D> {
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
    ) -> Result<Texture2D> {
        Texture2D::new(self.clone(), source, wrap, min_filter, mag_filter)
    }

    #[track_caller]
    pub fn new_pipeline<V: Vertex, U: UniformBlock, I: ImagesUniformBlock>(
        self: &Rc<Self>,
        vert_shader: &str,
        frag_shader: &str,
        params: PipelineParams,
    ) -> Result<Pipeline<V, U, I>> {
        Pipeline::new(self.clone(), vert_shader, frag_shader, params)
    }

    pub fn new_render_pass(
        self: &Rc<Self>,
        color_img: Vec<Texture2D>,
        depth_img: Option<Texture2D>,
    ) -> Result<RenderPass> {
        RenderPass::new(self.clone(), color_img, depth_img)
    }
}

impl Drop for GlContext {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_vertex_array(self.vao);
        }
    }
}
