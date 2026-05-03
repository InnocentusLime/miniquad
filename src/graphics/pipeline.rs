use std::fmt::Debug;
use std::marker::PhantomData;
use std::rc::Rc;

use crate::GLSL_VERSION;
use crate::graphics::{
    GlContext, ImageUniformField, ImagesUniformBlock, IndexBuffer, NoImages, NoUniforms,
    PipelineParams, PrimitiveType, UniformBlock, UniformField, Vertex, VertexBuffer, VertexField,
    VertexIndex, apply_attributes_impl, apply_uniforms_impl,
};

use glow::HasContext;

static TARGET_NAME: &str = "gl.pipeline";

#[derive(Debug)]
pub struct Pipeline<V, U = NoUniforms, I = NoImages> {
    raw: PipelineRaw,
    _phantom: PhantomData<fn(&V, &U, &I)>,
}

impl<Vert, Uni, Img> Pipeline<Vert, Uni, Img>
where
    Vert: Vertex,
    Uni: UniformBlock,
    Img: ImagesUniformBlock,
{
    #[track_caller]
    pub fn new(
        ctx: Rc<GlContext>,
        vert_shader: &str,
        frag_shader: &str,
        params: PipelineParams,
    ) -> Pipeline<Vert, Uni, Img> {
        debug_assert_eq!(
            Self::sz_vert_fields(),
            std::mem::size_of::<Vert>(),
            "vertex layout mismatch",
        );
        debug_assert_eq!(
            Self::sz_uni_fields(),
            std::mem::size_of::<Uni>(),
            "uniform layout mismatch",
        );

        let raw = PipelineRaw::new(
            ctx,
            vert_shader,
            frag_shader,
            params,
            Img::FIELDS,
            Vert::LAYOUT,
            Uni::FIELDS,
        );
        Pipeline { raw, _phantom: PhantomData }
    }

    pub(crate) fn primitive_type(&self) -> PrimitiveType {
        self.raw.params.primitive_type
    }

    pub fn draw<'a, Idx: VertexIndex>(
        &'a self,
        base_element: u32,
        num_elements: u32,
        vertex_buffer: &'a VertexBuffer<Vert>,
        index_buffer: &'a IndexBuffer<Idx>,
        images: Img::Borrow<'a>,
        uniforms: &'a Uni,
    ) {
        self.apply(vertex_buffer, index_buffer, images, uniforms);

        let sz_elem = std::mem::size_of::<Idx>() as i32;
        let offset = sz_elem * base_element as i32;
        let mode = match self.primitive_type() {
            PrimitiveType::Triangles => glow::TRIANGLES,
            PrimitiveType::Lines => glow::LINES,
            PrimitiveType::Points => glow::POINTS,
        };

        unsafe {
            self.raw.ctx.gl.draw_elements_instanced(
                mode,
                num_elements as i32,
                Idx::GL_TYPE,
                offset,
                1,
            );
        }

        self.raw.ctx.check_no_gl_error();
    }

    pub(crate) fn apply<'a, Idx: VertexIndex>(
        &'a self,
        vertex_buffer: &'a VertexBuffer<Vert>,
        index_buffer: &'a IndexBuffer<Idx>,
        images: Img::Borrow<'a>,
        uniforms: &'a Uni,
    ) {
        tracing::trace!(
            target: TARGET_NAME,
            gl_prog = ?self.raw.gl_prog,
            vertex_buffer = ?vertex_buffer,
            index_buffers = ?index_buffer,
            images = ?images,
            "applying bindings",
        );
        Img::bind(images);
        self.raw.apply(
            vertex_buffer.gl_buf,
            index_buffer.gl_buf,
            bytemuck::bytes_of(uniforms),
            Uni::FIELDS,
            std::mem::size_of::<Vert>(),
            Vert::LAYOUT,
        );
    }

    const fn sz_vert_fields() -> usize {
        let mut res = 0;
        let mut idx = 0;
        while idx < Vert::LAYOUT.len() {
            res += Vert::LAYOUT[idx].sz;
            idx += 1;
        }
        res
    }

    const fn sz_uni_fields() -> usize {
        let mut res = 0;
        let mut idx = 0;
        while idx < Uni::FIELDS.len() {
            res += Uni::FIELDS[idx].sz;
            idx += 1;
        }
        res
    }
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
        images: &[ImageUniformField],
        attributes: &[VertexField],
        uniforms: &[UniformField],
    ) -> PipelineRaw {
        let vertex = format!("{GLSL_VERSION}\n{vertex}");
        let fragment = format!("{GLSL_VERSION}\n{fragment}");

        let vertex = compile_shader(&ctx.gl, glow::VERTEX_SHADER, "vertex shader", &vertex);
        tracing::debug!(target: TARGET_NAME, "compiled vertex shader: {vertex:?}");

        let fragment = compile_shader(&ctx.gl, glow::FRAGMENT_SHADER, "fragment shader", &fragment);
        tracing::debug!(target: TARGET_NAME, "compiled fragment shader: {fragment:?}");

        let gl_prog = create_program(&ctx.gl, vertex, fragment);
        tracing::debug!(target: TARGET_NAME, "new: {gl_prog:?}");

        let mut cache = ctx.cache.borrow_mut();
        cache.bind_program(&ctx.gl, gl_prog);
        std::mem::drop(cache);

        check_pipeline_attributes(&ctx.gl, gl_prog, attributes);
        let mut image_uniform_locs = Vec::new();
        for image in images {
            image_uniform_locs.push(get_uniform_location(&ctx.gl, gl_prog, image.name));
        }
        let mut uniform_locs = Vec::new();
        for uniform in uniforms {
            uniform_locs.push(get_uniform_location(&ctx.gl, gl_prog, uniform.name));
        }
        ctx.check_no_gl_error();

        PipelineRaw { ctx, gl_prog, image_uniform_locs, uniform_locs, params }
    }

    fn apply(
        &self,
        vertex_buffer: glow::Buffer,
        index_buffer: glow::Buffer,
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

        for (n, image_loc) in self.image_uniform_locs.iter().enumerate() {
            unsafe {
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
