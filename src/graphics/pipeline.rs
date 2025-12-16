use std::fmt::Debug;
use std::marker::PhantomData;
use std::rc::Rc;

use crate::graphics::{GlContext, PipelineParams, PrimitiveType};
use crate::{
    ImagesBlock, IndexBufferBinding, Texture2DBinding, UniformBlock, UniformField, Vertex,
    VertexBuffer, VertexField, apply_attributes_impl, apply_uniforms_impl,
};

use glow::HasContext;

static TARGET_NAME: &str = "gl.pipeline";

#[derive(Debug)]
pub struct Pipeline<M: PipelineMeta> {
    raw: PipelineRaw,
    _phantom: PhantomData<M>,
}

impl<M: PipelineMeta> Pipeline<M> {
    #[track_caller]
    pub fn new(ctx: Rc<GlContext>) -> Pipeline<M> {
        let raw = PipelineRaw::new(
            ctx,
            M::VERTEX_SHADER,
            M::FRAGMENT_SHADER,
            M::PARAMS,
            M::IMAGES_NAMES,
            M::Vertex::LAYOUT,
            M::Uniforms::FIELDS,
        );
        Pipeline {
            raw,
            _phantom: PhantomData,
        }
    }

    pub(crate) fn primitive_type(&self) -> PrimitiveType {
        self.raw.params.primitive_type
    }

    pub(crate) fn apply<'a>(
        &'a self,
        vertex_buffer: &'a VertexBuffer<M::Vertex>,
        index_buffer: IndexBufferBinding,
        images: &'a M::Images<'a>,
        uniforms: &'a M::Uniforms,
    ) {
        #[cfg(debug_assertions)]
        tracing::trace!(
            target: TARGET_NAME,
            gl_prog = ?self.raw.gl_prog,
            vertex_buffer = ?vertex_buffer,
            index_buffers = ?index_buffer,
            images = ?images,
            "applying bindings",
        );
        self.raw.apply(
            vertex_buffer.gl_buf,
            index_buffer.gl_buf,
            images.as_slice(),
            bytemuck::bytes_of(uniforms),
            M::Uniforms::FIELDS,
            std::mem::size_of::<M::Vertex>(),
            M::Vertex::LAYOUT,
        );
    }
}

pub trait PipelineMeta: 'static {
    const VERTEX_SHADER: &'static str;
    const FRAGMENT_SHADER: &'static str;

    const IMAGES_NAMES: &'static [&'static str];
    type Images<'a>: ImagesBlock<'a>;
    type Vertex: Vertex;
    type Uniforms: UniformBlock;
    const PARAMS: PipelineParams;
}

#[derive(Debug)]
struct PipelineRaw {
    ctx: Rc<GlContext>,
    gl_prog: glow::Program,
    image_uniform_locs: Vec<glow::UniformLocation>,
    uniform_locs: Vec<glow::UniformLocation>,
    params: PipelineParams,
}

impl PipelineRaw {
    #[track_caller]
    fn new(
        ctx: Rc<GlContext>,
        vertex: &str,
        fragment: &str,
        params: PipelineParams,
        image_names: &[&str],
        attributes: &[VertexField],
        uniforms: &[UniformField],
    ) -> PipelineRaw {
        let vertex = compile_shader(&ctx.gl, glow::VERTEX_SHADER, "vertex shader", vertex);
        tracing::debug!(target: TARGET_NAME, "compiled vertex shader: {vertex:?}");

        let fragment = compile_shader(&ctx.gl, glow::FRAGMENT_SHADER, "fragment shader", fragment);
        tracing::debug!(target: TARGET_NAME, "compiled fragment shader: {fragment:?}");

        let gl_prog = create_program(&ctx.gl, vertex, fragment);
        tracing::debug!(target: TARGET_NAME, "new: {gl_prog:?}");

        let mut cache = ctx.cache.borrow_mut();
        cache.bind_program(&ctx.gl, gl_prog);
        std::mem::drop(cache);

        check_pipeline_attributes(&ctx.gl, gl_prog, attributes);
        let mut image_uniform_locs = Vec::new();
        for image_name in image_names {
            image_uniform_locs.push(get_uniform_location(&ctx.gl, gl_prog, image_name));
        }
        let mut uniform_locs = Vec::new();
        for uniform in uniforms {
            uniform_locs.push(get_uniform_location(&ctx.gl, gl_prog, uniform.name));
        }
        ctx.check_no_gl_error();

        PipelineRaw {
            ctx,
            gl_prog,
            image_uniform_locs,
            uniform_locs,
            params,
        }
    }

    fn apply(
        &self,
        vertex_buffer: glow::Buffer,
        index_buffer: glow::Buffer,
        images: &[Texture2DBinding],
        uniform_data: &[u8],
        uniform_layout: &[UniformField],
        attribute_size: usize,
        attribute_layout: &[VertexField],
    ) {
        let mut cache = self.ctx.cache.borrow_mut();

        cache.bind_program(&self.ctx.gl, self.gl_prog);
        cache.bind_buffer(&self.ctx.gl, vertex_buffer);
        cache.bind_index_buffer(&self.ctx.gl, index_buffer);

        cache.set_depth_test(&self.ctx.gl, self.params.depth_test);
        cache.set_front_face_order(&self.ctx.gl, self.params.front_face_order);
        cache.set_cull_face(&self.ctx.gl, self.params.cull_face);
        cache.set_blend(&self.ctx.gl, self.params.blending);
        cache.set_stencil(&self.ctx.gl, self.params.stencil_test);
        cache.set_color_write(&self.ctx.gl, self.params.color_write);
        self.ctx.check_no_gl_error();

        for (n, (image_loc, texture)) in self.image_uniform_locs.iter().zip(images).enumerate() {
            unsafe {
                cache.bind_texture(&self.ctx.gl, n as u32, glow::TEXTURE_2D, texture.gl_tex);
                self.ctx.gl.uniform_1_i32(Some(image_loc), n as i32);
            }
        }
        apply_attributes_impl(&self.ctx.gl, attribute_size, attribute_layout);
        apply_uniforms_impl(
            &self.ctx.gl,
            uniform_data,
            uniform_layout,
            &self.uniform_locs,
        );
        self.ctx.check_no_gl_error();
    }
}

impl Drop for PipelineRaw {
    fn drop(&mut self) {
        tracing::debug!(
            target: TARGET_NAME,
            "dropping: {:?}",
            self.gl_prog,
        );
        unsafe { self.ctx.gl.delete_program(self.gl_prog) };
    }
}

#[track_caller]
fn compile_shader(
    gl: &glow::Context,
    shader_type: u32,
    shader_type_name: &str,
    source: &str,
) -> glow::Shader {
    unsafe {
        let shader = gl.create_shader(shader_type).unwrap();
        gl.shader_source(shader, source);
        gl.compile_shader(shader);

        if !gl.get_shader_compile_status(shader) {
            let error_message = gl.get_shader_info_log(shader);
            panic!("compilation error ({shader_type_name}): {error_message}");
        }

        shader
    }
}

#[track_caller]
fn create_program(
    gl: &glow::Context,
    vertex: glow::Shader,
    fragment: glow::Shader,
) -> glow::Program {
    unsafe {
        let program = gl.create_program().unwrap();
        gl.attach_shader(program, vertex);
        gl.attach_shader(program, fragment);
        gl.link_program(program);

        gl.detach_shader(program, vertex);
        gl.delete_shader(vertex);
        gl.detach_shader(program, fragment);
        gl.delete_shader(fragment);

        if !gl.get_program_link_status(program) {
            let error_message = gl.get_program_info_log(program);
            panic!("link error: {error_message}");
        }

        program
    }
}

#[track_caller]
fn check_pipeline_attributes(
    gl: &glow::Context,
    gl_prog: glow::Program,
    attributes: &[VertexField],
) {
    for (id, layout) in attributes.iter().enumerate() {
        let Some(attr_loc) = (unsafe { gl.get_attrib_location(gl_prog, layout.name) }) else {
            panic!("No attribute named {:?}", layout.name)
        };
        assert_eq!(
            id, attr_loc as usize,
            "Incorrect id for {:?}. Update your GLSL code",
            layout.name
        );
    }
}

#[track_caller]
fn get_uniform_location(
    gl: &glow::Context,
    gl_prog: glow::Program,
    name: &str,
) -> glow::UniformLocation {
    let Some(uniform_loc) = (unsafe { gl.get_uniform_location(gl_prog, name) }) else {
        panic!("No uniform named: {name:?}")
    };
    uniform_loc
}
