use std::fmt::Debug;
use std::rc::Rc;

use crate::graphics::vertex_buffer::VertexBufferBinding;
use crate::graphics::{ColorMask, Comparison, CullFace, FrontFaceOrder, GlContext, PrimitiveType};
use crate::{BlendState, IndexBufferBinding, StencilState, TextureBinding};

use anyhow::Context;
use glow::HasContext;

static TARGET_NAME: &str = "gl.pipeline";

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

#[derive(Debug)]
pub struct Pipeline {
    ctx: Rc<GlContext>,
    gl_prog: glow::Program,
    image_uniforms: Vec<glow::UniformLocation>,
    uniforms: Vec<ShaderUniform>,
    attributes: Vec<(u32, VertexAttribute)>,
    params: PipelineParams,
}

impl Pipeline {
    pub fn new<S: Into<String>>(
        ctx: Rc<GlContext>,
        vertex_shader_source: &str,
        fragment_shader_source: &str,
        params: PipelineParams,
        attributes: impl IntoIterator<Item = VertexAttribute>,
        uniforms: impl IntoIterator<Item = UniformDesc>,
        image_uniforms: impl IntoIterator<Item = S>,
    ) -> anyhow::Result<Pipeline> {
        let mut cache = ctx.cache.borrow_mut();

        let vertex = load_shader(&ctx.gl, glow::VERTEX_SHADER, vertex_shader_source)
            .context("load vertex shader")?;
        tracing::debug!(
            target: TARGET_NAME, 
            "compiled vertex shader: {vertex:?}",
        );
        
        let fragment = load_shader(&ctx.gl, glow::FRAGMENT_SHADER, fragment_shader_source)
            .context("load fragment shader")?;
        tracing::debug!(
            target: TARGET_NAME, 
            "compiled fragment shader: {fragment:?}",
        );
        
        let program = create_program(&ctx.gl, vertex, fragment)?;
        tracing::debug!(
            target: TARGET_NAME,
            params=?params,
            "new: {program:?} (vertex={vertex:?}, fragment={fragment:?})",
        );

        cache.bind_program(&ctx.gl, program);
        let attributes = get_pipeline_attributes(&ctx.gl, program, attributes)?;
        let uniforms = get_pipeline_uniforms(&ctx.gl, program, uniforms)?;
        let images = get_pipeline_images(&ctx.gl, program, image_uniforms)?;

        std::mem::drop(cache);
        Ok(Pipeline {
            ctx,
            gl_prog: program,
            image_uniforms: images,
            uniforms,
            attributes,
            params,
        })
    }

    pub(crate) fn primitive_type(&self) -> PrimitiveType {
        self.params.primitive_type
    }

    pub(crate) fn apply(
        &self,
        vertex_buffers: &[VertexBufferBinding],
        index_buffer: IndexBufferBinding,
        textures: &[TextureBinding],
        uniform_data: &[u8],
    ) { 
        let mut cache = self.ctx.cache.borrow_mut();
        cache.bind_program(&self.ctx.gl, self.gl_prog);
        std::mem::drop(cache);

        self.apply_parameters(&self.params);
        self.apply_uniforms(uniform_data);
        self.apply_bindings(vertex_buffers, index_buffer, textures);
    }

    fn apply_uniforms(&self, uniform_data: &[u8]) {
        let mut offset = 0;
        for uniform in self.uniforms.iter() {
            let location = uniform.gl_loc;
            let sz = uniform.uniform_type.size() * (uniform.array_count as usize);
            let data = &uniform_data[offset..(offset + sz)];

            match uniform.uniform_type {
                UniformType::F32 => unsafe {
                    let value = self.get_uniform_value::<_, f32>(data, uniform);
                    self.ctx.gl.uniform_1_f32_slice(Some(&location), value);
                },
                UniformType::F32x2 => unsafe {
                    let value = self.get_uniform_value::<_, [f32; 2]>(data, uniform);
                    self.ctx.gl.uniform_2_f32_slice(Some(&location), value);
                },
                UniformType::F32x3 => unsafe {
                    let value = self.get_uniform_value::<_, [f32; 3]>(data, uniform);
                    self.ctx.gl.uniform_3_f32_slice(Some(&location), value);
                },
                UniformType::F32x4 => unsafe {
                    let value = self.get_uniform_value::<_, [f32; 4]>(data, uniform);
                    self.ctx.gl.uniform_4_f32_slice(Some(&location), value);
                },
                UniformType::I32 => unsafe {
                    let value = self.get_uniform_value::<_, i32>(data, uniform);
                    self.ctx.gl.uniform_1_i32_slice(Some(&location), value);
                },
                UniformType::I32x2 => unsafe {
                    let value = self.get_uniform_value::<_, [i32; 2]>(data, uniform);
                    self.ctx.gl.uniform_2_i32_slice(Some(&location), value);
                },
                UniformType::I32x3 => unsafe {
                    let value = self.get_uniform_value::<_, [i32; 3]>(data, uniform);
                    self.ctx.gl.uniform_3_i32_slice(Some(&location), value);
                },
                UniformType::I32x4 => unsafe {
                    let value = self.get_uniform_value::<_, [i32; 4]>(data, uniform);
                    self.ctx.gl.uniform_4_i32_slice(Some(&location), value);
                },
                UniformType::F32x4x4 => unsafe {
                    let value = self.get_uniform_value::<_, [[f32; 4]; 4]>(data, uniform);
                    self.ctx.gl.uniform_matrix_4_f32_slice(Some(&location), false, value);
                },
            }
            offset += sz;
        }
    }

    fn get_uniform_value<'a, T, Inter>(&self, data: &'a [u8], uniform: &ShaderUniform) -> &'a [T]
    where 
        T: bytemuck::AnyBitPattern + Debug + 'static,
        Inter: bytemuck::AnyBitPattern + Debug + 'static,
    {
        let value = bytemuck::cast_slice(data);
        let interpreted = bytemuck::cast_slice::<_, Inter>(data);
        tracing::trace!(
            target: TARGET_NAME,
            uniform_name = uniform.name,
            gl_prog = ?self.gl_prog,
            interpreted = ?interpreted,
            "set uniform",
        );
        value
    }

    fn apply_bindings(
        &self,
        vertex_buffers: &[VertexBufferBinding],
        index_buffer: IndexBufferBinding,
        textures: &[TextureBinding],
    ) {
        tracing::trace!(
            target: TARGET_NAME,
            gl_prog = ?self.gl_prog,
            vertex_buffers = ?vertex_buffers,
            index_buffers = ?index_buffer,
            textures = ?textures,
            "applying bindings",
        );
        let mut cache = self.ctx.cache.borrow_mut();

        for (n, (image_loc, texture)) in self.image_uniforms.iter().zip(textures).enumerate() {
            unsafe {
                cache.bind_texture(&self.ctx.gl, n as u32, glow::TEXTURE_2D, texture.gl_tex);
                self.ctx.gl.uniform_1_i32(Some(image_loc), n as i32);
            }
        }

        cache.bind_index_buffer(&self.ctx.gl, index_buffer.gl_buf);
        for attr_index in 0..MAX_VERTEX_ATTRIBUTES {
            unsafe {
                self.ctx.gl.disable_vertex_attrib_array(attr_index as u32);
            }
        }

        for ((attr_index, attr), vb) in self.attributes.iter().zip(vertex_buffers) {
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

            cache.bind_buffer(&self.ctx.gl, vb.gl_buf);
            unsafe {
                if is_integer {
                    self.ctx.gl.vertex_attrib_pointer_i32(
                        *attr_index,
                        component_count,
                        gl_type,
                        vb.stride as i32,
                        vb.offset as i32,
                    )
                } else {
                    self.ctx.gl.vertex_attrib_pointer_f32(
                        *attr_index,
                        component_count,
                        gl_type,
                        false,
                        vb.stride as i32,
                        vb.offset as i32,
                    )
                }
                self.ctx.gl.enable_vertex_attrib_array(*attr_index);
            }
        }
    }

    fn apply_parameters(&self, params: &PipelineParams) {
        if params.depth_write {
            unsafe {
                self.ctx.gl.enable(glow::DEPTH_TEST);
                self.ctx.gl.depth_func(params.depth_test.into())
            }
        } else {
            unsafe {
                self.ctx.gl.disable(glow::DEPTH_TEST);
            }
        }

        match params.front_face_order {
            FrontFaceOrder::Clockwise => unsafe {
                self.ctx.gl.front_face(glow::CW);
            },
            FrontFaceOrder::CounterClockwise => unsafe {
                self.ctx.gl.front_face(glow::CCW);
            },
        }

        self.set_cull_face(params.cull_face);
        self.set_blend(params.color_blend, params.alpha_blend);
        self.set_stencil(params.stencil_test);
        self.set_color_write(params.color_write);
    }

    fn set_blend(&self, color_blend: Option<BlendState>, alpha_blend: Option<BlendState>) {
        let mut cache = self.ctx.cache.borrow_mut();

        if color_blend.is_none() && alpha_blend.is_some() {
            panic!("AlphaBlend without ColorBlend");
        }
        if cache.color_blend == color_blend && cache.alpha_blend == alpha_blend {
            return;
        }

        unsafe {
            if let Some(color_blend) = color_blend {
                if cache.color_blend.is_none() {
                    self.ctx.gl.enable(glow::BLEND);
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
                    self.ctx.gl.blend_func_separate(
                        src_rgb.into(),
                        dst_rgb.into(),
                        src_alpha.into(),
                        dst_alpha.into(),
                    );
                    self.ctx.gl.blend_equation_separate(eq_rgb.into(), eq_alpha.into());
                } else {
                    self.ctx.gl.blend_func(src_rgb.into(), dst_rgb.into());
                    self.ctx.gl.blend_equation_separate(eq_rgb.into(), eq_rgb.into());
                }
            } else if cache.color_blend.is_some() {
                self.ctx.gl.disable(glow::BLEND);
            }
        }

        cache.color_blend = color_blend;
        cache.alpha_blend = alpha_blend;
    }

    fn set_stencil(&self, stencil_test: Option<StencilState>) {
        let mut cache = self.ctx.cache.borrow_mut();
        if cache.stencil == stencil_test {
            return;
        }
        unsafe {
            if let Some(stencil) = stencil_test {
                if cache.stencil.is_none() {
                    self.ctx.gl.enable(glow::STENCIL_TEST);
                }

                let front = &stencil.front;
                self.ctx.gl.stencil_op_separate(
                    glow::FRONT,
                    front.fail_op.into(),
                    front.depth_fail_op.into(),
                    front.pass_op.into(),
                );
                self.ctx.gl.stencil_func_separate(
                    glow::FRONT,
                    front.test_func.into(),
                    front.test_ref,
                    front.test_mask,
                );
                self.ctx.gl.stencil_mask_separate(glow::FRONT, front.write_mask);

                let back = &stencil.back;
                self.ctx.gl.stencil_op_separate(
                    glow::BACK,
                    back.fail_op.into(),
                    back.depth_fail_op.into(),
                    back.pass_op.into(),
                );
                self.ctx.gl.stencil_func_separate(
                    glow::BACK,
                    back.test_func.into(),
                    back.test_ref,
                    back.test_mask,
                );
                self.ctx.gl.stencil_mask_separate(glow::BACK, back.write_mask);
            } else if cache.stencil.is_some() {
                self.ctx.gl.disable(glow::STENCIL_TEST);
            }
        }

        cache.stencil = stencil_test;
    }

    fn set_cull_face(&self, cull_face: CullFace) {
        let mut cache = self.ctx.cache.borrow_mut();
        if cache.cull_face == cull_face {
            return;
        }

        match cull_face {
            CullFace::Nothing => unsafe {
                self.ctx.gl.disable(glow::CULL_FACE);
            },
            CullFace::Front => unsafe {
                self.ctx.gl.enable(glow::CULL_FACE);
                self.ctx.gl.cull_face(glow::FRONT);
            },
            CullFace::Back => unsafe {
                self.ctx.gl.enable(glow::CULL_FACE);
                self.ctx.gl.cull_face(glow::BACK);
            },
        }
        cache.cull_face = cull_face;
    }

    fn set_color_write(&self, color_write: ColorMask) {
        let mut cache = self.ctx.cache.borrow_mut();
        if cache.color_write == color_write {
            return;
        }
        let (r, g, b, a) = color_write;
        unsafe { self.ctx.gl.color_mask(r as _, g as _, b as _, a as _) }
        cache.color_write = color_write;
    }
}

impl Drop for Pipeline {
    fn drop(&mut self) {
        tracing::debug!(
            target: TARGET_NAME, 
            "dropping: {:?}",
            self.gl_prog,
        );
        unsafe { self.ctx.gl.delete_program(self.gl_prog) };
    }
}

fn load_shader(
    gl: &glow::Context,
    shader_type: u32,
    source: &str,
) -> anyhow::Result<glow::Shader> {
    unsafe {
        let shader = gl.create_shader(shader_type).unwrap();
        gl.shader_source(shader, source);
        gl.compile_shader(shader);

        if !gl.get_shader_compile_status(shader) {
            let error_message = gl.get_shader_info_log(shader);
            anyhow::bail!("compilation error: {error_message}");
        }

        Ok(shader)
    }
}

fn create_program(
    gl: &glow::Context,
    vertex_shader: glow::Shader,
    fragment_shader: glow::Shader,
) -> anyhow::Result<glow::Program> {
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
            anyhow::bail!("link error: {error_message}");
        }

        Ok(program)
    }
}

fn get_pipeline_attributes(
    gl: &glow::Context,
    program: glow::Program,
    attributes: impl IntoIterator<Item = VertexAttribute>,
) -> anyhow::Result<Vec<(u32, VertexAttribute)>> {
    let mut vertex_layout = Vec::new();
    for attr in attributes.into_iter() {
        let Some(attr_loc) = (unsafe { gl.get_attrib_location(program, attr.name) }) else {
            anyhow::bail!("attribute {:?} not found", attr.name);
        };
        vertex_layout.push((attr_loc, attr));
    }

    Ok(vertex_layout)
}

fn get_pipeline_images<S: Into<String>>(
    gl: &glow::Context,
    program: glow::Program,
    image_uniforms: impl IntoIterator<Item = S>,
) -> anyhow::Result<Vec<glow::UniformLocation>> {
    image_uniforms
        .into_iter()
        .map(|x| x.into())
        .map(|name| get_uniform_location(gl, program, &name))
        .collect()
}

fn get_pipeline_uniforms(
    gl: &glow::Context,
    program: glow::Program,
    uniforms: impl IntoIterator<Item = UniformDesc>,
) -> anyhow::Result<Vec<ShaderUniform>> {
    uniforms
        .into_iter()
        .map(|uniform| {
            Ok(ShaderUniform {
                gl_loc: get_uniform_location(gl, program, &uniform.name)?,
                name: uniform.name,
                uniform_type: uniform.uniform_type,
                array_count: uniform.array_len as _,
            })
        })
        .collect()
}

fn get_uniform_location(
    gl: &glow::Context,
    program: glow::Program,
    name: &str,
) -> anyhow::Result<glow::UniformLocation> {
    unsafe { gl.get_uniform_location(program, name) }
        .ok_or_else(|| anyhow::anyhow!("uniform {name:?} not found"))
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

#[derive(Debug)]
struct ShaderUniform {
    #[allow(dead_code)]
    name: String,
    gl_loc: glow::UniformLocation,
    uniform_type: UniformType,
    array_count: i32,
}

const MAX_VERTEX_ATTRIBUTES: usize = 16;
