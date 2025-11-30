use std::error::Error;
use std::fmt::Display;
use std::rc::Rc;

use crate::graphics::buffer::BufferBinding;
use crate::graphics::{ColorMask, Comparison, CullFace, FrontFaceOrder, GlContext, PrimitiveType};
use crate::{BlendState, IndexBufferBinding, StencilState, TextureBinding};

use glow::HasContext;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct PipelineParams {
    pub cull_face: CullFace,
    pub front_face_order: FrontFaceOrder,
    pub depth_test: Comparison,
    pub depth_write: bool,
    pub depth_write_offset: Option<(f32, f32)>,
    pub color_blend: Option<BlendState>,
    pub alpha_blend: Option<BlendState>,
    pub stencil_test: Option<StencilState>,
    pub color_write: ColorMask,
    pub primitive_type: PrimitiveType,
}

impl Default for PipelineParams {
    fn default() -> PipelineParams {
        PipelineParams {
            cull_face: CullFace::Nothing,
            front_face_order: FrontFaceOrder::CounterClockwise,
            depth_test: Comparison::Always, // no depth test,
            depth_write: false,             // no depth write,
            depth_write_offset: None,
            color_blend: None,
            alpha_blend: None,
            stencil_test: None,
            color_write: (true, true, true, true),
            primitive_type: PrimitiveType::Triangles,
        }
    }
}

#[derive(Clone)]
pub struct Pipeline(Rc<PipelineInternal>);

impl Pipeline {
    pub fn new<S: Into<String>>(
        ctx: Rc<GlContext>,
        vertex_shader_source: &str,
        fragment_shader_source: &str,
        params: PipelineParams,
        attributes: impl IntoIterator<Item = VertexAttribute>,
        uniforms: impl IntoIterator<Item = UniformDesc>,
        image_uniforms: impl IntoIterator<Item = S>,
    ) -> Result<Pipeline, ShaderError> {
        let mut cache = ctx.cache.borrow_mut();

        let vertex = load_shader(&ctx.gl, ShaderType::Vertex, vertex_shader_source)?;
        let fragment = load_shader(&ctx.gl, ShaderType::Fragment, fragment_shader_source)?;
        let program = create_program(&ctx.gl, vertex, fragment)?;

        cache.bind_program(&ctx.gl, program);
        let attributes = get_pipeline_attributes(&ctx.gl, program, attributes)?;
        let uniforms = get_pipeline_uniforms(&ctx.gl, program, uniforms)?;
        let images = get_pipeline_images(&ctx.gl, program, image_uniforms)?;

        std::mem::drop(cache);
        let internal = PipelineInternal {
            ctx,
            gl_prog: program,
            image_uniforms: images,
            uniforms,
            attributes,
            params,
        };

        Ok(Pipeline(Rc::new(internal)))
    }

    pub(crate) fn primitive_type(&self) -> PrimitiveType {
        self.0.params.primitive_type
    }

    pub(crate) fn apply(
        &self,
        vertex_buffers: &[BufferBinding],
        index_buffer: IndexBufferBinding,
        textures: &[TextureBinding],
        uniform_data: &[u8],
    ) {
        let gl = &self.0.ctx.gl;
        let mut cache = self.0.ctx.cache.borrow_mut();
        cache.bind_program(gl, self.0.gl_prog);
        std::mem::drop(cache);

        self.apply_parameters(&self.0.params);
        self.apply_uniforms(uniform_data);
        self.apply_bindings(vertex_buffers, index_buffer, textures);
    }

    fn apply_uniforms(&self, uniform_data: &[u8]) {
        let gl = &self.0.ctx.gl;
        let mut offset = 0;
        for uniform in self.0.uniforms.iter() {
            let location = uniform.gl_loc;
            let sz = uniform.uniform_type.size() * (uniform.array_count as usize);
            let data = &uniform_data[offset..(offset + sz)];

            match uniform.uniform_type {
                UniformType::F32 => unsafe {
                    let value = bytemuck::cast_slice(data);
                    gl.uniform_1_f32_slice(Some(&location), value);
                },
                UniformType::F32x2 => unsafe {
                    let value = bytemuck::cast_slice(data);
                    gl.uniform_2_f32_slice(Some(&location), value);
                },
                UniformType::F32x3 => unsafe {
                    let value = bytemuck::cast_slice(data);
                    gl.uniform_3_f32_slice(Some(&location), value);
                },
                UniformType::F32x4 => unsafe {
                    let value = bytemuck::cast_slice(data);
                    gl.uniform_4_f32_slice(Some(&location), value);
                },
                UniformType::I32 => unsafe {
                    let value = bytemuck::cast_slice(data);
                    gl.uniform_1_i32_slice(Some(&location), value);
                },
                UniformType::I32x2 => unsafe {
                    let value = bytemuck::cast_slice(data);
                    gl.uniform_2_i32_slice(Some(&location), value);
                },
                UniformType::I32x3 => unsafe {
                    let value = bytemuck::cast_slice(data);
                    gl.uniform_3_i32_slice(Some(&location), value);
                },
                UniformType::I32x4 => unsafe {
                    let value = bytemuck::cast_slice(data);
                    gl.uniform_4_i32_slice(Some(&location), value);
                },
                UniformType::F32x4x4 => unsafe {
                    let value = bytemuck::cast_slice(data);
                    gl.uniform_matrix_4_f32_slice(Some(&location), false, value);
                },
            }
            offset += sz;
        }
    }

    fn apply_bindings(
        &self,
        vertex_buffers: &[BufferBinding],
        index_buffer: IndexBufferBinding,
        textures: &[TextureBinding],
    ) {
        let gl = &self.0.ctx.gl;
        let mut cache = self.0.ctx.cache.borrow_mut();

        for (n, (image_loc, texture)) in self.0.image_uniforms.iter().zip(textures).enumerate() {
            unsafe {
                cache.bind_texture(gl, n as u32, glow::TEXTURE_2D, texture.gl_tex);
                gl.uniform_1_i32(Some(image_loc), n as i32);
            }
        }

        cache.bind_index_buffer(gl, index_buffer.gl_buf);
        for attr_index in 0..MAX_VERTEX_ATTRIBUTES {
            unsafe {
                gl.disable_vertex_attrib_array(attr_index as u32);
            }
        }

        for ((attr_index, attr), vb) in self.0.attributes.iter().zip(vertex_buffers) {
            let (gl_type, component_count) = attr.format.gl_info();
            let is_integer = matches!(
                gl_type,
                glow::INT
                    | glow::SHORT
                    | glow::BYTE
                    | glow::UNSIGNED_INT
                    | glow::UNSIGNED_SHORT
                    | glow::UNSIGNED_BYTE
            );

            cache.bind_buffer(gl, vb.gl_buf);
            unsafe {
                if is_integer {
                    gl.vertex_attrib_pointer_i32(
                        *attr_index,
                        component_count,
                        gl_type,
                        vb.stride as i32,
                        vb.offset as i32,
                    )
                } else {
                    gl.vertex_attrib_pointer_f32(
                        *attr_index,
                        component_count,
                        gl_type,
                        false,
                        vb.stride as i32,
                        vb.offset as i32,
                    )
                }
                gl.enable_vertex_attrib_array(*attr_index);
            }
        }
    }

    fn apply_parameters(&self, params: &PipelineParams) {
        let gl = &self.0.ctx.gl;

        if params.depth_write {
            unsafe {
                gl.enable(glow::DEPTH_TEST);
                gl.depth_func(params.depth_test.into())
            }
        } else {
            unsafe {
                gl.disable(glow::DEPTH_TEST);
            }
        }

        match params.front_face_order {
            FrontFaceOrder::Clockwise => unsafe {
                gl.front_face(glow::CW);
            },
            FrontFaceOrder::CounterClockwise => unsafe {
                gl.front_face(glow::CCW);
            },
        }

        self.set_cull_face(params.cull_face);
        self.set_blend(params.color_blend, params.alpha_blend);
        self.set_stencil(params.stencil_test);
        self.set_color_write(params.color_write);
    }

    fn set_blend(&self, color_blend: Option<BlendState>, alpha_blend: Option<BlendState>) {
        let gl = &self.0.ctx.gl;
        let mut cache = self.0.ctx.cache.borrow_mut();

        if color_blend.is_none() && alpha_blend.is_some() {
            panic!("AlphaBlend without ColorBlend");
        }
        if cache.color_blend == color_blend && cache.alpha_blend == alpha_blend {
            return;
        }

        unsafe {
            if let Some(color_blend) = color_blend {
                if cache.color_blend.is_none() {
                    gl.enable(glow::BLEND);
                }

                let BlendState {
                    equation: eq_rgb,
                    sfactor: src_rgb,
                    dfactor: dst_rgb,
                } = color_blend;

                if let Some(BlendState {
                    equation: eq_alpha,
                    sfactor: src_alpha,
                    dfactor: dst_alpha,
                }) = alpha_blend
                {
                    gl.blend_func_separate(
                        src_rgb.into(),
                        dst_rgb.into(),
                        src_alpha.into(),
                        dst_alpha.into(),
                    );
                    gl.blend_equation_separate(eq_rgb.into(), eq_alpha.into());
                } else {
                    gl.blend_func(src_rgb.into(), dst_rgb.into());
                    gl.blend_equation_separate(eq_rgb.into(), eq_rgb.into());
                }
            } else if cache.color_blend.is_some() {
                gl.disable(glow::BLEND);
            }
        }

        cache.color_blend = color_blend;
        cache.alpha_blend = alpha_blend;
    }

    fn set_stencil(&self, stencil_test: Option<StencilState>) {
        let gl = &self.0.ctx.gl;
        let mut cache = self.0.ctx.cache.borrow_mut();

        if cache.stencil == stencil_test {
            return;
        }
        unsafe {
            if let Some(stencil) = stencil_test {
                if cache.stencil.is_none() {
                    gl.enable(glow::STENCIL_TEST);
                }

                let front = &stencil.front;
                gl.stencil_op_separate(
                    glow::FRONT,
                    front.fail_op.into(),
                    front.depth_fail_op.into(),
                    front.pass_op.into(),
                );
                gl.stencil_func_separate(
                    glow::FRONT,
                    front.test_func.into(),
                    front.test_ref,
                    front.test_mask,
                );
                gl.stencil_mask_separate(glow::FRONT, front.write_mask);

                let back = &stencil.back;
                gl.stencil_op_separate(
                    glow::BACK,
                    back.fail_op.into(),
                    back.depth_fail_op.into(),
                    back.pass_op.into(),
                );
                gl.stencil_func_separate(
                    glow::BACK,
                    back.test_func.into(),
                    back.test_ref,
                    back.test_mask,
                );
                gl.stencil_mask_separate(glow::BACK, back.write_mask);
            } else if cache.stencil.is_some() {
                gl.disable(glow::STENCIL_TEST);
            }
        }

        cache.stencil = stencil_test;
    }

    fn set_cull_face(&self, cull_face: CullFace) {
        let gl = &self.0.ctx.gl;
        let mut cache = self.0.ctx.cache.borrow_mut();

        if cache.cull_face == cull_face {
            return;
        }

        match cull_face {
            CullFace::Nothing => unsafe {
                gl.disable(glow::CULL_FACE);
            },
            CullFace::Front => unsafe {
                gl.enable(glow::CULL_FACE);
                gl.cull_face(glow::FRONT);
            },
            CullFace::Back => unsafe {
                gl.enable(glow::CULL_FACE);
                gl.cull_face(glow::BACK);
            },
        }
        cache.cull_face = cull_face;
    }

    fn set_color_write(&self, color_write: ColorMask) {
        let gl = &self.0.ctx.gl;
        let mut cache = self.0.ctx.cache.borrow_mut();

        if cache.color_write == color_write {
            return;
        }
        let (r, g, b, a) = color_write;
        unsafe { gl.color_mask(r as _, g as _, b as _, a as _) }
        cache.color_write = color_write;
    }
}

fn load_shader(
    gl: &glow::Context,
    shader_type: ShaderType,
    source: &str,
) -> Result<glow::Shader, ShaderError> {
    let (shader_type_name, gl_type) = match shader_type {
        ShaderType::Vertex => ("Vertex", glow::VERTEX_SHADER),
        ShaderType::Fragment => ("Fragment", glow::FRAGMENT_SHADER),
    };

    unsafe {
        let shader = gl.create_shader(gl_type).unwrap();
        gl.shader_source(shader, source);
        gl.compile_shader(shader);

        if !gl.get_shader_compile_status(shader) {
            let error_message = gl.get_shader_info_log(shader);
            return Err(ShaderError::CompilationError {
                shader_type_name,
                error_message,
            });
        }

        Ok(shader)
    }
}

#[derive(Clone, Copy)]
enum ShaderType {
    Vertex,
    Fragment,
}

fn create_program(
    gl: &glow::Context,
    vertex_shader: glow::Shader,
    fragment_shader: glow::Shader,
) -> Result<glow::Program, ShaderError> {
    unsafe {
        let program = gl.create_program().unwrap();
        gl.attach_shader(program, vertex_shader);
        gl.attach_shader(program, fragment_shader);
        gl.link_program(program);

        gl.detach_shader(program, vertex_shader);
        gl.delete_shader(vertex_shader);
        gl.detach_shader(program, fragment_shader);
        gl.delete_shader(fragment_shader);

        if !gl.get_program_link_status(program) {
            let error_message = gl.get_program_info_log(program);
            return Err(ShaderError::LinkError(error_message));
        }

        Ok(program)
    }
}

fn get_pipeline_attributes(
    gl: &glow::Context,
    program: glow::Program,
    attributes: impl IntoIterator<Item = VertexAttribute>,
) -> Result<Vec<(u32, VertexAttribute)>, ShaderError> {
    let mut vertex_layout = Vec::new();
    for attr in attributes.into_iter() {
        let Some(attr_loc) = (unsafe { gl.get_attrib_location(program, attr.name) }) else {
            return Err(ShaderError::MissingAttribute {
                attribute: attr.name.to_string(),
            });
        };
        vertex_layout.push((attr_loc, attr));
    }

    Ok(vertex_layout)
}

fn get_pipeline_images<S: Into<String>>(
    gl: &glow::Context,
    program: glow::Program,
    image_uniforms: impl IntoIterator<Item = S>,
) -> Result<Vec<glow::UniformLocation>, ShaderError> {
    image_uniforms
        .into_iter()
        .map(|x| x.into())
        .map(|name| get_uniform_location(gl, program, &name))
        .collect::<Result<Vec<_>, ShaderError>>()
}

fn get_pipeline_uniforms(
    gl: &glow::Context,
    program: glow::Program,
    uniforms: impl IntoIterator<Item = UniformDesc>,
) -> Result<Vec<ShaderUniform>, ShaderError> {
    uniforms
        .into_iter()
        .map(|uniform| {
            Ok(ShaderUniform {
                gl_loc: get_uniform_location(gl, program, &uniform.name)?,
                uniform_type: uniform.uniform_type,
                array_count: uniform.array_len as _,
            })
        })
        .collect::<Result<Vec<_>, ShaderError>>()
}

fn get_uniform_location(
    gl: &glow::Context,
    program: glow::Program,
    name: &str,
) -> Result<glow::UniformLocation, ShaderError> {
    unsafe { gl.get_uniform_location(program, name) }.ok_or_else(|| ShaderError::MissingUniform {
        uniform: name.to_string(),
    })
}

#[derive(Debug, Clone)]
pub struct UniformDesc {
    pub name: String,
    pub uniform_type: UniformType,
    pub array_len: usize,
}

impl UniformDesc {
    pub fn new_scalar(name: &str, uniform_type: UniformType) -> UniformDesc {
        UniformDesc {
            name: name.to_string(),
            uniform_type,
            array_len: 1,
        }
    }

    pub fn new_array(name: &str, uniform_type: UniformType, array_len: usize) -> UniformDesc {
        UniformDesc {
            name: name.to_string(),
            uniform_type,
            array_len,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum UniformType {
    F32,
    F32x2,
    F32x3,
    F32x4,
    I32,
    I32x2,
    I32x3,
    I32x4,
    F32x4x4,
}

impl UniformType {
    /// Byte size for a given UniformType
    pub fn size(&self) -> usize {
        match self {
            UniformType::F32 => 4,
            UniformType::F32x2 => 8,
            UniformType::F32x3 => 12,
            UniformType::F32x4 => 16,
            UniformType::I32 => 4,
            UniformType::I32x2 => 8,
            UniformType::I32x3 => 12,
            UniformType::I32x4 => 16,
            UniformType::F32x4x4 => 64,
        }
    }
}

#[derive(Clone, Debug)]
pub struct VertexAttribute {
    pub name: &'static str,
    pub format: VertexFormat,
}

impl VertexAttribute {
    pub const fn new(name: &'static str, format: VertexFormat) -> VertexAttribute {
        VertexAttribute { name, format }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum VertexFormat {
    F32,
    F32x2,
    F32x3,
    F32x4,
    U8,
    U8x2,
    U8x3,
    U8x4,
    U16,
    U16x2,
    U16x3,
    U16x4,
    U32,
    U32x2,
    U32x3,
    U32x4,
}

impl VertexFormat {
    // (GL_TYPE, components)
    pub fn gl_info(self) -> (u32, i32) {
        match self {
            VertexFormat::F32 => (glow::FLOAT, 1),
            VertexFormat::F32x2 => (glow::FLOAT, 2),
            VertexFormat::F32x3 => (glow::FLOAT, 3),
            VertexFormat::F32x4 => (glow::FLOAT, 4),
            VertexFormat::U8 => (glow::UNSIGNED_BYTE, 1),
            VertexFormat::U8x2 => (glow::UNSIGNED_BYTE, 2),
            VertexFormat::U8x3 => (glow::UNSIGNED_BYTE, 3),
            VertexFormat::U8x4 => (glow::UNSIGNED_BYTE, 4),
            VertexFormat::U16 => (glow::UNSIGNED_SHORT, 1),
            VertexFormat::U16x2 => (glow::UNSIGNED_SHORT, 2),
            VertexFormat::U16x3 => (glow::UNSIGNED_SHORT, 3),
            VertexFormat::U16x4 => (glow::UNSIGNED_SHORT, 4),
            VertexFormat::U32 => (glow::UNSIGNED_INT, 1),
            VertexFormat::U32x2 => (glow::UNSIGNED_INT, 2),
            VertexFormat::U32x3 => (glow::UNSIGNED_INT, 3),
            VertexFormat::U32x4 => (glow::UNSIGNED_INT, 4),
        }
    }
}

struct PipelineInternal {
    ctx: Rc<GlContext>,
    gl_prog: glow::Program,
    image_uniforms: Vec<glow::UniformLocation>,
    uniforms: Vec<ShaderUniform>,
    attributes: Vec<(u32, VertexAttribute)>,
    params: PipelineParams,
}

impl Drop for PipelineInternal {
    fn drop(&mut self) {
        unsafe { self.ctx.gl.delete_program(self.gl_prog) };
    }
}

#[derive(Debug)]
struct ShaderUniform {
    gl_loc: glow::UniformLocation,
    uniform_type: UniformType,
    array_count: i32,
}

const MAX_VERTEX_ATTRIBUTES: usize = 16;

#[derive(Clone, Debug)]
pub enum ShaderError {
    MissingUniform {
        uniform: String,
    },
    MissingAttribute {
        attribute: String,
    },
    CompilationError {
        shader_type_name: &'static str,
        error_message: String,
    },
    LinkError(String),
}

impl Display for ShaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingUniform { uniform } => write!(f, "No such uniform:{uniform}"),
            Self::MissingAttribute { attribute } => write!(f, "No such attribute:{attribute}"),
            Self::CompilationError {
                shader_type_name,
                error_message,
            } => write!(f, "{shader_type_name} shader error:\n{error_message}"),
            Self::LinkError(msg) => write!(f, "Link shader error:\n{msg}"),
        }
    }
}

impl Error for ShaderError {}
