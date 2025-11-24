use std::ffi::CString;
use std::rc::Rc;

use crate::graphics::gl::buffer::{Buffer, BufferBinding};
use crate::graphics::gl::cache::{CachedAttribute, VertexAttributeInternal};
use crate::graphics::gl::texture::Texture;
use crate::graphics::gl::GlContext;
use crate::graphics::{
    ColorMask, FrontFaceOrder, PipelineParams, ShaderError, ShaderMeta, ShaderSource, ShaderType,
    UniformType, MAX_VERTEX_ATTRIBUTES,
};
use crate::native::gl::*;
use crate::{BlendState, CullFace, PrimitiveType, StencilState};

#[derive(Clone)]
pub struct Pipeline(Rc<PipelineInternal>);

impl Pipeline {
    pub fn new(
        ctx: Rc<GlContext>,
        shader: ShaderSource,
        meta: ShaderMeta,
        params: PipelineParams,
    ) -> Result<Pipeline, ShaderError> {
        let (fragment, vertex) = match shader {
            ShaderSource::Glsl { fragment, vertex } => (fragment, vertex),
            _ => panic!("Metal source on OpenGl context"),
        };
        let vertex = load_shader(GL_VERTEX_SHADER, vertex)?;
        let fragment = load_shader(GL_FRAGMENT_SHADER, fragment)?;
        let program = create_program(vertex, fragment)?;

        unsafe {
            glUseProgram(program);
        }
        let attributes = get_pipeline_attributes(program, &meta)?;
        let images = get_pipeline_images(program, &meta)?;
        let uniforms = get_pipeline_uniforms(program, &meta)?;
        let internal = PipelineInternal {
            ctx,
            program,
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

    pub(crate) fn apply(
        &self,
        vertex_buffers: &[BufferBinding],
        index_buffer: &Buffer,
        textures: &[&Texture],
        uniform_data: &[u8],
    ) {
        unsafe {
            glUseProgram(self.0.program);
        }

        self.apply_parameters(&self.0.params);
        self.apply_uniforms(uniform_data);
        self.apply_bindings(vertex_buffers, index_buffer, textures);
    }

    fn apply_uniforms(&self, uniform_data: &[u8]) {
        let mut offset = 0;
        for uniform in self.0.uniforms.iter() {
            let location = uniform.gl_loc;
            let sz = uniform.uniform_type.size() * (uniform.array_count as usize);
            let data = &uniform_data[offset..(offset + sz)];

            match uniform.uniform_type {
                UniformType::Float1 => unsafe {
                    let value = data.as_ptr() as *const f32;
                    glUniform1fv(location, uniform.array_count, value);
                },
                UniformType::Float2 => unsafe {
                    let value = data.as_ptr() as *const f32;
                    glUniform2fv(location, uniform.array_count, value);
                },
                UniformType::Float3 => unsafe {
                    let value = data.as_ptr() as *const f32;
                    glUniform3fv(location, uniform.array_count, value);
                },
                UniformType::Float4 => unsafe {
                    let value = data.as_ptr() as *const f32;
                    glUniform4fv(location, uniform.array_count, value);
                },
                UniformType::Int1 => unsafe {
                    let value = data.as_ptr() as *const i32;
                    glUniform1iv(location, uniform.array_count, value);
                },
                UniformType::Int2 => unsafe {
                    let value = data.as_ptr() as *const i32;
                    glUniform2iv(location, uniform.array_count, value);
                },
                UniformType::Int3 => unsafe {
                    let value = data.as_ptr() as *const i32;
                    glUniform3iv(location, uniform.array_count, value);
                },
                UniformType::Int4 => unsafe {
                    let value = data.as_ptr() as *const i32;
                    glUniform4iv(location, uniform.array_count, value);
                },
                UniformType::Mat4 => unsafe {
                    let value = data.as_ptr() as *const f32;
                    glUniformMatrix4fv(location, uniform.array_count, 0, value);
                },
            }
            offset += sz;
        }
    }

    fn apply_bindings(
        &self,
        vertex_buffers: &[BufferBinding],
        index_buffer: &Buffer,
        textures: &[&Texture],
    ) {
        let mut cache = self.0.ctx.cache.borrow_mut();

        for (n, (shader_image, texture)) in self.0.images.iter().zip(textures).enumerate() {
            let gl_loc = shader_image.gl_loc;
            unsafe {
                cache.bind_texture(n, GL_TEXTURE_2D, texture.gl_tex());
                glUniform1i(gl_loc, n as i32);
            }
        }

        cache.bind_buffer(
            GL_ELEMENT_ARRAY_BUFFER,
            index_buffer.gl_buf(),
            index_buffer.index_type(),
        );

        for attr_index in 0..MAX_VERTEX_ATTRIBUTES {
            let cached_attr = &mut cache.attributes[attr_index];
            let Some(attribute) = self.0.attributes.get(attr_index) else {
                unsafe {
                    glDisableVertexAttribArray(attr_index as GLuint);
                }
                *cached_attr = None;
                continue;
            };
            let vb = vertex_buffers.get(attr_index).unwrap();

            if cached_attr.map_or(true, |cached_attr| {
                *attribute != cached_attr.attribute || cached_attr.gl_vbuf != vb.gl_buf()
            }) {
                cache.bind_buffer(GL_ARRAY_BUFFER, vb.gl_buf(), vb.index_type());

                unsafe {
                    match attribute.type_ {
                        GL_INT | GL_UNSIGNED_INT | GL_SHORT | GL_UNSIGNED_SHORT
                        | GL_UNSIGNED_BYTE | GL_BYTE
                            if !attribute.gl_pass_as_float =>
                        {
                            glVertexAttribIPointer(
                                attr_index as GLuint,
                                attribute.size,
                                attribute.type_,
                                vb.stride as GLsizei,
                                vb.offset as *mut _,
                            )
                        }
                        _ => glVertexAttribPointer(
                            attr_index as GLuint,
                            attribute.size,
                            attribute.type_,
                            GL_FALSE as u8,
                            vb.stride as GLsizei,
                            vb.offset as *mut _,
                        ),
                    }
                    glEnableVertexAttribArray(attr_index as GLuint);
                };

                let cached_attr = &mut cache.attributes[attr_index];
                *cached_attr = Some(CachedAttribute {
                    attribute: *attribute,
                    gl_vbuf: vb.gl_buf(),
                });
            }
        }
    }

    fn apply_parameters(&self, params: &PipelineParams) {
        if params.depth_write {
            unsafe {
                glEnable(GL_DEPTH_TEST);
                glDepthFunc(params.depth_test.into())
            }
        } else {
            unsafe {
                glDisable(GL_DEPTH_TEST);
            }
        }

        match params.front_face_order {
            FrontFaceOrder::Clockwise => unsafe {
                glFrontFace(GL_CW);
            },
            FrontFaceOrder::CounterClockwise => unsafe {
                glFrontFace(GL_CCW);
            },
        }

        self.set_cull_face(params.cull_face);
        self.set_blend(params.color_blend, params.alpha_blend);
        self.set_stencil(params.stencil_test);
        self.set_color_write(params.color_write);
    }

    fn set_blend(&self, color_blend: Option<BlendState>, alpha_blend: Option<BlendState>) {
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
                    glEnable(GL_BLEND);
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
                    glBlendFuncSeparate(
                        src_rgb.into(),
                        dst_rgb.into(),
                        src_alpha.into(),
                        dst_alpha.into(),
                    );
                    glBlendEquationSeparate(eq_rgb.into(), eq_alpha.into());
                } else {
                    glBlendFunc(src_rgb.into(), dst_rgb.into());
                    glBlendEquationSeparate(eq_rgb.into(), eq_rgb.into());
                }
            } else if cache.color_blend.is_some() {
                glDisable(GL_BLEND);
            }
        }

        cache.color_blend = color_blend;
        cache.alpha_blend = alpha_blend;
    }

    fn set_stencil(&self, stencil_test: Option<StencilState>) {
        let mut cache = self.0.ctx.cache.borrow_mut();

        if cache.stencil == stencil_test {
            return;
        }
        unsafe {
            if let Some(stencil) = stencil_test {
                if cache.stencil.is_none() {
                    glEnable(GL_STENCIL_TEST);
                }

                let front = &stencil.front;
                glStencilOpSeparate(
                    GL_FRONT,
                    front.fail_op.into(),
                    front.depth_fail_op.into(),
                    front.pass_op.into(),
                );
                glStencilFuncSeparate(
                    GL_FRONT,
                    front.test_func.into(),
                    front.test_ref,
                    front.test_mask,
                );
                glStencilMaskSeparate(GL_FRONT, front.write_mask);

                let back = &stencil.back;
                glStencilOpSeparate(
                    GL_BACK,
                    back.fail_op.into(),
                    back.depth_fail_op.into(),
                    back.pass_op.into(),
                );
                glStencilFuncSeparate(
                    GL_BACK,
                    back.test_func.into(),
                    back.test_ref,
                    back.test_mask,
                );
                glStencilMaskSeparate(GL_BACK, back.write_mask);
            } else if cache.stencil.is_some() {
                glDisable(GL_STENCIL_TEST);
            }
        }

        cache.stencil = stencil_test;
    }

    fn set_cull_face(&self, cull_face: CullFace) {
        let mut cache = self.0.ctx.cache.borrow_mut();

        if cache.cull_face == cull_face {
            return;
        }

        match cull_face {
            CullFace::Nothing => unsafe {
                glDisable(GL_CULL_FACE);
            },
            CullFace::Front => unsafe {
                glEnable(GL_CULL_FACE);
                glCullFace(GL_FRONT);
            },
            CullFace::Back => unsafe {
                glEnable(GL_CULL_FACE);
                glCullFace(GL_BACK);
            },
        }
        cache.cull_face = cull_face;
    }

    fn set_color_write(&self, color_write: ColorMask) {
        let mut cache = self.0.ctx.cache.borrow_mut();

        if cache.color_write == color_write {
            return;
        }
        let (r, g, b, a) = color_write;
        unsafe { glColorMask(r as _, g as _, b as _, a as _) }
        cache.color_write = color_write;
    }
}

fn load_shader(shader_type: GLenum, source: &str) -> Result<GLuint, ShaderError> {
    unsafe {
        let shader = glCreateShader(shader_type);
        assert!(shader != 0);

        let cstring = CString::new(source)?;
        let csource = [cstring];
        glShaderSource(shader, 1, csource.as_ptr() as *const _, std::ptr::null());
        glCompileShader(shader);

        let mut is_compiled = 0;
        glGetShaderiv(shader, GL_COMPILE_STATUS, &mut is_compiled as *mut _);
        if is_compiled == 0 {
            let mut max_length: i32 = 0;
            glGetShaderiv(shader, GL_INFO_LOG_LENGTH, &mut max_length as *mut _);

            let mut error_message = vec![0u8; max_length as usize + 1];
            glGetShaderInfoLog(
                shader,
                max_length,
                &mut max_length as *mut _,
                error_message.as_mut_ptr() as *mut _,
            );

            assert!(max_length >= 1);
            let mut error_message =
                std::string::String::from_utf8_lossy(&error_message[0..max_length as usize - 1])
                    .into_owned();

            // On Wasm + Chrome, for unknown reason, string with zero-terminator is returned. On Firefox there is no zero-terminators in JavaScript string.
            if error_message.ends_with('\0') {
                error_message.pop();
            }

            return Err(ShaderError::CompilationError {
                shader_type: match shader_type {
                    GL_VERTEX_SHADER => ShaderType::Vertex,
                    GL_FRAGMENT_SHADER => ShaderType::Fragment,
                    _ => unreachable!(),
                },
                error_message,
            });
        }

        Ok(shader)
    }
}

fn create_program(vertex_shader: GLuint, fragment_shader: GLuint) -> Result<GLuint, ShaderError> {
    unsafe {
        let program = glCreateProgram();
        glAttachShader(program, vertex_shader);
        glAttachShader(program, fragment_shader);
        glLinkProgram(program);

        // delete no longer used shaders
        glDetachShader(program, vertex_shader);
        glDeleteShader(vertex_shader);
        glDetachShader(program, fragment_shader);
        glDeleteShader(fragment_shader);

        let mut link_status = 0;
        glGetProgramiv(program, GL_LINK_STATUS, &mut link_status as *mut _);
        if link_status == 0 {
            let mut max_length: i32 = 0;
            glGetProgramiv(program, GL_INFO_LOG_LENGTH, &mut max_length as *mut _);

            let mut error_message = vec![0u8; max_length as usize + 1];
            glGetProgramInfoLog(
                program,
                max_length,
                &mut max_length as *mut _,
                error_message.as_mut_ptr() as *mut _,
            );
            let error_message = String::from_utf8_lossy(&error_message[0..max_length as usize - 1]);
            return Err(ShaderError::LinkError(error_message.to_string()));
        }

        Ok(program)
    }
}

fn get_pipeline_attributes(
    program: GLuint,
    meta: &ShaderMeta,
) -> Result<Vec<VertexAttributeInternal>, ShaderError> {
    let mut vertex_layout = Vec::new();
    for attr in meta.attributes.iter() {
        let cname = CString::new(attr.name).unwrap_or_else(|e| panic!("{}", e));
        let attr_loc = unsafe { glGetAttribLocation(program, cname.as_ptr() as *const _) };
        if attr_loc == -1 {
            return Err(ShaderError::MissingAttribute {
                attribute: attr.name.to_string(),
            });
        }
        vertex_layout.push(VertexAttributeInternal {
            attr_loc: attr_loc as GLuint,
            size: attr.format.components(),
            type_: attr.format.type_(),
            gl_pass_as_float: attr.gl_pass_as_float,
        });
    }

    Ok(vertex_layout)
}

fn get_pipeline_images(
    program: GLuint,
    meta: &ShaderMeta,
) -> Result<Vec<ShaderImage>, ShaderError> {
    meta.images
        .iter()
        .map(|name| {
            Ok(ShaderImage {
                gl_loc: get_uniform_location(program, &name)?,
            })
        })
        .collect::<Result<Vec<_>, ShaderError>>()
}

fn get_pipeline_uniforms(
    program: GLuint,
    meta: &ShaderMeta,
) -> Result<Vec<ShaderUniform>, ShaderError> {
    meta.uniforms
        .iter()
        .map(|uniform| {
            Ok(ShaderUniform {
                gl_loc: get_uniform_location(program, &uniform.name)?,
                uniform_type: uniform.uniform_type,
                array_count: uniform.array_count as _,
            })
        })
        .collect::<Result<Vec<_>, ShaderError>>()
}

fn get_uniform_location(program: GLuint, name: &str) -> Result<i32, ShaderError> {
    let cname = CString::new(name).unwrap();
    let location = unsafe { glGetUniformLocation(program, cname.as_ptr()) };

    if location == -1 {
        Err(ShaderError::MissingUniform {
            uniform: name.to_string(),
        })
    } else {
        Ok(location)
    }
}

pub struct PipelineInternal {
    ctx: Rc<GlContext>,
    program: GLuint,
    images: Vec<ShaderImage>,
    uniforms: Vec<ShaderUniform>,
    attributes: Vec<VertexAttributeInternal>,
    params: PipelineParams,
}

impl Drop for PipelineInternal {
    fn drop(&mut self) {
        let mut cache = self.ctx.cache.borrow_mut();

        unsafe { glDeleteProgram(self.program) };
        cache.cur_pipeline = None;
    }
}

#[derive(Debug)]
struct ShaderUniform {
    gl_loc: UniformLocation,
    uniform_type: UniformType,
    array_count: i32,
}

struct ShaderImage {
    gl_loc: UniformLocation,
}

type UniformLocation = GLint;
