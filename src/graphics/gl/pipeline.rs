use std::rc::Rc;

use crate::graphics::gl::buffer::BufferBinding;
use crate::graphics::gl::cache::VertexAttributeInternal;
use crate::graphics::gl::texture::Texture;
use crate::graphics::gl::GlContext;
use crate::graphics::{
    ColorMask, FrontFaceOrder, PipelineParams, ShaderError, ShaderMeta, ShaderType, UniformType,
    MAX_VERTEX_ATTRIBUTES,
};
use crate::{BlendState, CullFace, IndexBuffer, IndexBufferElement, PrimitiveType, StencilState};

use glow::HasContext;

#[derive(Clone)]
pub struct Pipeline(Rc<PipelineInternal>);

impl Pipeline {
    pub fn new(
        ctx: Rc<GlContext>,
        vertex: &str,
        fragment: &str,
        meta: ShaderMeta,
        params: PipelineParams,
    ) -> Result<Pipeline, ShaderError> {
        let vertex = load_shader(&ctx.gl, glow::VERTEX_SHADER, vertex)?;
        let fragment = load_shader(&ctx.gl, glow::FRAGMENT_SHADER, fragment)?;
        let program = create_program(&ctx.gl, vertex, fragment)?;

        unsafe {
            ctx.gl.use_program(Some(program));
        }
        let attributes = get_pipeline_attributes(&ctx.gl, program, &meta)?;
        let images = get_pipeline_images(&ctx.gl, program, &meta)?;
        let uniforms = get_pipeline_uniforms(&ctx.gl, program, &meta)?;
        let internal = PipelineInternal {
            ctx,
            gl_prog: program,
            images,
            uniforms,
            attributes,
            params,
        };

        Ok(Pipeline(Rc::new(internal)))
    }

    pub(crate) fn primitive_type(&self) -> PrimitiveType {
        self.0.params.primitive_type
    }

    pub(crate) fn apply<I: IndexBufferElement>(
        &self,
        vertex_buffers: &[BufferBinding],
        index_buffer: &IndexBuffer<I>,
        textures: &[&Texture],
        uniform_data: &[u8],
    ) {
        unsafe {
            self.0.ctx.gl.use_program(Some(self.0.gl_prog));
        }

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
                UniformType::Float1 => unsafe {
                    let value = bytemuck::cast_slice(data);
                    gl.uniform_1_f32_slice(Some(&location), value);
                },
                UniformType::Float2 => unsafe {
                    let value = bytemuck::cast_slice(data);
                    gl.uniform_2_f32_slice(Some(&location), value);
                },
                UniformType::Float3 => unsafe {
                    let value = bytemuck::cast_slice(data);
                    gl.uniform_3_f32_slice(Some(&location), value);
                },
                UniformType::Float4 => unsafe {
                    let value = bytemuck::cast_slice(data);
                    gl.uniform_4_f32_slice(Some(&location), value);
                },
                UniformType::Int1 => unsafe {
                    let value = bytemuck::cast_slice(data);
                    gl.uniform_1_i32_slice(Some(&location), value);
                },
                UniformType::Int2 => unsafe {
                    let value = bytemuck::cast_slice(data);
                    gl.uniform_2_i32_slice(Some(&location), value);
                },
                UniformType::Int3 => unsafe {
                    let value = bytemuck::cast_slice(data);
                    gl.uniform_3_i32_slice(Some(&location), value);
                },
                UniformType::Int4 => unsafe {
                    let value = bytemuck::cast_slice(data);
                    gl.uniform_4_i32_slice(Some(&location), value);
                },
                UniformType::Mat4 => unsafe {
                    let value = bytemuck::cast_slice(data);
                    gl.uniform_matrix_4_f32_slice(Some(&location), false, value);
                },
            }
            offset += sz;
        }
    }

    fn apply_bindings<I: IndexBufferElement>(
        &self,
        vertex_buffers: &[BufferBinding],
        index_buffer: &IndexBuffer<I>,
        textures: &[&Texture],
    ) {
        let gl = &self.0.ctx.gl;
        let mut cache = self.0.ctx.cache.borrow_mut();

        for (n, (shader_image, texture)) in self.0.images.iter().zip(textures).enumerate() {
            unsafe {
                cache.bind_texture(gl, n as u32, glow::TEXTURE_2D, texture.gl_tex());
                gl.uniform_1_i32(Some(&shader_image.gl_loc), n as i32);
            }
        }

        cache.bind_index_buffer(gl, index_buffer.gl_buf());

        for attr_index in 0..MAX_VERTEX_ATTRIBUTES {
            let Some(attribute) = self.0.attributes.get(attr_index) else {
                unsafe {
                    gl.disable_vertex_attrib_array(attr_index as u32);
                }
                continue;
            };

            let vb = vertex_buffers.get(attr_index).unwrap();
            cache.bind_buffer(gl, vb.gl_buf);
            unsafe {
                gl.enable_vertex_attrib_array(attr_index as u32);
                match attribute.type_ {
                    glow::INT
                    | glow::UNSIGNED_INT
                    | glow::SHORT
                    | glow::UNSIGNED_SHORT
                    | glow::UNSIGNED_BYTE
                    | glow::BYTE
                        if !attribute.gl_pass_as_float =>
                    {
                        gl.vertex_attrib_pointer_i32(
                            attr_index as u32,
                            attribute.size,
                            attribute.type_,
                            vb.stride as i32,
                            vb.offset as i32,
                        )
                    }
                    _ => gl.vertex_attrib_pointer_f32(
                        attr_index as u32,
                        attribute.size,
                        attribute.type_,
                        false,
                        vb.stride as i32,
                        vb.offset as i32,
                    ),
                }
                gl.enable_vertex_attrib_array(attr_index as u32);
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
    shader_type: u32,
    source: &str,
) -> Result<glow::Shader, ShaderError> {
    unsafe {
        let shader = gl.create_shader(shader_type).unwrap();
        gl.shader_source(shader, source);
        gl.compile_shader(shader);

        if !gl.get_shader_compile_status(shader) {
            let error_message = gl.get_shader_info_log(shader);
            return Err(ShaderError::CompilationError {
                shader_type: match shader_type {
                    glow::VERTEX_SHADER => ShaderType::Vertex,
                    glow::FRAGMENT_SHADER => ShaderType::Fragment,
                    _ => unreachable!(),
                },
                error_message,
            });
        }

        Ok(shader)
    }
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

        // delete no longer used shaders
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
    meta: &ShaderMeta,
) -> Result<Vec<VertexAttributeInternal>, ShaderError> {
    let mut vertex_layout = Vec::new();
    for attr in meta.attributes.iter() {
        let Some(attr_loc) = (unsafe { gl.get_attrib_location(program, attr.name) }) else {
            return Err(ShaderError::MissingAttribute {
                attribute: attr.name.to_string(),
            });
        };
        vertex_layout.push(VertexAttributeInternal {
            attr_loc: attr_loc as u32,
            size: attr.format.components(),
            type_: attr.format.type_(),
            gl_pass_as_float: attr.gl_pass_as_float,
        });
    }

    Ok(vertex_layout)
}

fn get_pipeline_images(
    gl: &glow::Context,
    program: glow::Program,
    meta: &ShaderMeta,
) -> Result<Vec<ShaderImage>, ShaderError> {
    meta.images
        .iter()
        .map(|name| {
            Ok(ShaderImage {
                gl_loc: get_uniform_location(gl, program, &name)?,
            })
        })
        .collect::<Result<Vec<_>, ShaderError>>()
}

fn get_pipeline_uniforms(
    gl: &glow::Context,
    program: glow::Program,
    meta: &ShaderMeta,
) -> Result<Vec<ShaderUniform>, ShaderError> {
    meta.uniforms
        .iter()
        .map(|uniform| {
            Ok(ShaderUniform {
                gl_loc: get_uniform_location(gl, program, &uniform.name)?,
                uniform_type: uniform.uniform_type,
                array_count: uniform.array_count as _,
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

pub struct PipelineInternal {
    ctx: Rc<GlContext>,
    gl_prog: glow::Program,
    images: Vec<ShaderImage>,
    uniforms: Vec<ShaderUniform>,
    attributes: Vec<VertexAttributeInternal>,
    params: PipelineParams,
}

impl Drop for PipelineInternal {
    fn drop(&mut self) {
        let mut cache = self.ctx.cache.borrow_mut();

        unsafe { self.ctx.gl.delete_program(self.gl_prog) };
        cache.cur_pipeline = None;
    }
}

#[derive(Debug)]
struct ShaderUniform {
    gl_loc: glow::UniformLocation,
    uniform_type: UniformType,
    array_count: i32,
}

struct ShaderImage {
    gl_loc: glow::UniformLocation,
}
