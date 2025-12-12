mod buffer_usage;
mod cache;
mod color;
mod index_buffer;
mod pipeline;
mod pipeline_params;
mod render_pass;
mod texture;
mod vertex_buffer;

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;

use bytemuck::Pod;
use cache::GlCache;
use glow::HasContext;
use image::DynamicImage;

pub use buffer_usage::*;
pub use color::*;
pub use index_buffer::*;
pub use pipeline::*;
pub use pipeline_params::*;
pub use render_pass::*;
pub use texture::*;
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

    pub fn new_empty_vertex_buffer<T: Pod + Default>(
        self: &Rc<Self>,
        usage: BufferUsage,
        size: usize,
    ) -> VertexBuffer<T> {
        VertexBuffer::new_empty(self.clone(), usage, size)
    }

    pub fn new_vertex_buffer<T: Pod + Default>(
        self: &Rc<Self>,
        usage: BufferUsage,
        data: &[T],
    ) -> VertexBuffer<T> {
        VertexBuffer::new(self.clone(), usage, data)
    }

    pub fn new_empty_index_buffer<I: IndexBufferElement>(
        self: &Rc<Self>,
        usage: BufferUsage,
        size: usize,
    ) -> IndexBuffer<I> {
        IndexBuffer::new_empty(self.clone(), usage, size)
    }

    pub fn new_index_buffer<I: IndexBufferElement>(
        self: &Rc<Self>,
        usage: BufferUsage,
        data: &[I],
    ) -> IndexBuffer<I> {
        IndexBuffer::new(self.clone(), usage, data)
    }

    pub fn new_empty_texture(
        self: &Rc<Self>,
        width: u32,
        height: u32,
        params: TextureParams,
    ) -> Texture {
        Texture::new_empty(self.clone(), width, height, params)
    }

    pub fn new_texture(
        self: &Rc<Self>,
        source: impl Into<DynamicImage>,
        params: TextureParams,
    ) -> Texture {
        Texture::new(self.clone(), source, params)
    }

    pub fn new_pipeline<'a, U: Pod + 'static>(
        self: &Rc<Self>,
        vertex_shader_source: &str,
        fragment_shader_source: &str,
        params: PipelineParams,
        attributes: impl IntoIterator<Item = Attribute>,
        uniforms: impl IntoIterator<Item = UniformDesc>,
        image_uniforms: impl IntoIterator<Item = &'a str>,
    ) -> anyhow::Result<Pipeline<U>> {
        Pipeline::new(
            self.clone(),
            vertex_shader_source,
            fragment_shader_source,
            params,
            attributes,
            uniforms,
            image_uniforms,
        )
    }

    pub fn new_render_pass(
        self: &Rc<Self>,
        color_img: Vec<Texture>,
        depth_img: Option<Texture>,
    ) -> RenderPass {
        RenderPass::new(self.clone(), color_img, depth_img)
    }

    pub fn draw<U: Pod + 'static>(&self, drawcall: DrawCall<U>) {
        drawcall.pipeline.apply(
            drawcall.vertex_buffers,
            drawcall.index_buffer,
            drawcall.textures,
            drawcall.uniforms,
        );

        let offset = drawcall.index_buffer.sz_elem * (drawcall.base_element as i32);
        let mode = match drawcall.pipeline.primitive_type() {
            PrimitiveType::Triangles => glow::TRIANGLES,
            PrimitiveType::Lines => glow::LINES,
            PrimitiveType::Points => glow::POINTS,
        };

        unsafe {
            self.gl.draw_elements_instanced(
                mode,
                drawcall.num_elements as i32,
                drawcall.index_buffer.gl_type,
                offset,
                1,
            );
        }

        self.check_no_gl_error();
    }

    #[cfg(debug_assertions)]
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

pub struct DrawCall<'a, U: Pod + 'static> {
    pub pipeline: &'a Pipeline<U>,
    pub base_element: u32,
    pub num_elements: u32,
    pub vertex_buffers: &'a [VertexBufferBinding<'a>],
    pub index_buffer: IndexBufferBinding<'a>,
    pub textures: &'a [TextureBinding<'a>],
    pub uniforms: &'a U,
}
