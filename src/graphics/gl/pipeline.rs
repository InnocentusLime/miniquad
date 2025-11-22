use std::ffi::CString;

use crate::native::gl::*;
use crate::graphics::{ShaderId, PipelineParams, UniformType, ShaderSource, PipelineId, VertexAttribute, VertexFormat, VertexStep, FrontFaceOrder, BufferId, ShaderMeta, ShaderError, ShaderType, BufferLayout, TextureId, MAX_VERTEX_ATTRIBUTES};
use crate::graphics::gl::cache::{CachedAttribute, VertexAttributeInternal};
use crate::graphics::gl::GlContext;

pub struct PipelineInternal {
    pub layout: Vec<Option<VertexAttributeInternal>>,
    pub shader: ShaderId,
    pub params: PipelineParams,
}

pub struct ShaderInternal {
    pub program: GLuint,
    pub images: Vec<ShaderImage>,
    pub uniforms: Vec<ShaderUniform>,
}

#[derive(Debug)]
pub struct ShaderUniform {
    pub gl_loc: UniformLocation,
    pub uniform_type: UniformType,
    pub array_count: i32,
}

type UniformLocation = GLint;

pub struct ShaderImage {
    gl_loc: UniformLocation,
}

fn get_uniform_location(program: GLuint, name: &str) -> Option<i32> {
    let cname = CString::new(name).unwrap_or_else(|e| panic!("{}", e));
    let location = unsafe { glGetUniformLocation(program, cname.as_ptr()) };

    if location == -1 {
        return None;
    }

    Some(location)
}

fn load_shader_internal(
    vertex_shader: &str,
    fragment_shader: &str,
    meta: ShaderMeta,
) -> Result<ShaderInternal, ShaderError> {
    unsafe {
        let vertex_shader = load_shader(GL_VERTEX_SHADER, vertex_shader)?;
        let fragment_shader = load_shader(GL_FRAGMENT_SHADER, fragment_shader)?;

        let program = glCreateProgram();
        glAttachShader(program, vertex_shader);
        glAttachShader(program, fragment_shader);
        glLinkProgram(program);

        // delete no longer used shaders
        glDetachShader(program, vertex_shader);
        glDeleteShader(vertex_shader);
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
            assert!(max_length >= 1);
            let error_message =
                std::string::String::from_utf8_lossy(&error_message[0..max_length as usize - 1]);
            return Err(ShaderError::LinkError(error_message.to_string()));
        }

        glUseProgram(program);

        let images = meta
            .images
            .into_iter()
            .map(|name| {
                Ok(ShaderImage {
                    gl_loc: get_uniform_location(program, &name)
                        .ok_or(ShaderError::MissingUniform { uniform: name })?,
                })
            })
            .collect::<Result<Vec<_>, ShaderError>>()?;

        let uniforms = meta
            .uniforms
            .uniforms
            .into_iter()
            .map(|uniform| {
                Ok(ShaderUniform {
                    gl_loc: get_uniform_location(program, &uniform.name).ok_or(
                        ShaderError::MissingUniform {
                            uniform: uniform.name,
                        },
                    )?,
                    uniform_type: uniform.uniform_type,
                    array_count: uniform.array_count as _,
                })
            })
            .collect::<Result<Vec<_>, ShaderError>>()?;

        Ok(ShaderInternal {
            program,
            images,
            uniforms,
        })
    }
}

pub fn load_shader(shader_type: GLenum, source: &str) -> Result<GLuint, ShaderError> {
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

impl GlContext {
    pub fn new_gl_shader(
        &mut self,
        shader: ShaderSource,
        meta: ShaderMeta,
    ) -> Result<ShaderId, ShaderError> {
        let (fragment, vertex) = match shader {
            ShaderSource::Glsl { fragment, vertex } => (fragment, vertex),
            _ => panic!("Metal source on OpenGl context"),
        };
        let shader = load_shader_internal(vertex, fragment, meta)?;
        Ok(ShaderId(self.shaders.add(shader)))
    }
    
    pub fn new_gl_pipeline(
        &mut self,
        buffer_layout: &[BufferLayout],
        attributes: &[VertexAttribute],
        shader: ShaderId,
        params: PipelineParams,
    ) -> PipelineId {
        #[derive(Clone, Copy, Default)]
        struct BufferCacheData {
            stride: i32,
            offset: i64,
        }

        let mut buffer_cache: Vec<BufferCacheData> =
            vec![BufferCacheData::default(); buffer_layout.len()];

        for VertexAttribute {
            format,
            buffer_index,
            ..
        } in attributes
        {
            let layout = buffer_layout.get(*buffer_index).unwrap_or_else(|| panic!());
            let cache = buffer_cache
                .get_mut(*buffer_index)
                .unwrap_or_else(|| panic!());

            if layout.stride == 0 {
                cache.stride += format.size_bytes();
            } else {
                cache.stride = layout.stride;
            }
            // WebGL 1 limitation
            assert!(cache.stride <= 255);
        }

        let program = self.shaders[shader.0].program;

        let attributes_len = attributes
            .iter()
            .map(|layout| match layout.format {
                VertexFormat::Mat4 => 4,
                _ => 1,
            })
            .sum();

        let mut vertex_layout: Vec<Option<VertexAttributeInternal>> = vec![None; attributes_len];

        for VertexAttribute {
            name,
            format,
            buffer_index,
            gl_pass_as_float,
        } in attributes
        {
            let buffer_data = &mut buffer_cache
                .get_mut(*buffer_index)
                .unwrap_or_else(|| panic!());
            let layout = buffer_layout.get(*buffer_index).unwrap_or_else(|| panic!());

            let cname = CString::new(*name).unwrap_or_else(|e| panic!("{}", e));
            let attr_loc = unsafe { glGetAttribLocation(program, cname.as_ptr() as *const _) };
            let attr_loc = if attr_loc == -1 { None } else { Some(attr_loc) };
            let divisor = if layout.step_func == VertexStep::PerVertex {
                0
            } else {
                layout.step_rate
            };

            let mut attributes_count: usize = 1;
            let mut format = *format;

            if format == VertexFormat::Mat4 {
                format = VertexFormat::Float4;
                attributes_count = 4;
            }
            for i in 0..attributes_count {
                if let Some(attr_loc) = attr_loc {
                    let attr_loc = attr_loc as GLuint + i as GLuint;

                    let attr = VertexAttributeInternal {
                        attr_loc,
                        size: format.components(),
                        type_: format.type_(),
                        offset: buffer_data.offset,
                        stride: buffer_data.stride,
                        buffer_index: *buffer_index,
                        divisor,
                        gl_pass_as_float: *gl_pass_as_float,
                    };

                    assert!(
                        attr_loc < vertex_layout.len() as u32,
                        "attribute: {} outside of allocated attributes array len: {}",
                        name,
                        vertex_layout.len()
                    );
                    vertex_layout[attr_loc as usize] = Some(attr);
                }
                buffer_data.offset += format.size_bytes() as i64
            }
        }

        let pipeline = PipelineInternal {
            layout: vertex_layout,
            shader,
            params,
        };

        PipelineId(self.pipelines.add(pipeline))
    }
    
    pub fn delete_gl_shader(&mut self, program: ShaderId) {
        unsafe { glDeleteProgram(self.shaders[program.0].program) };
        self.shaders.remove(program.0);
        self.cache.cur_pipeline = None;
    }

    pub fn delete_gl_pipeline(&mut self, pipeline: PipelineId) {
        self.pipelines.remove(pipeline.0);
    }

    pub fn apply_gl_pipeline(&mut self, pipeline: &PipelineId) {
        self.cache.cur_pipeline = Some(*pipeline);

        {
            let pipeline = &self.pipelines[pipeline.0];
            let shader = &self.shaders[pipeline.shader.0];
            unsafe {
                glUseProgram(shader.program);
            }

            unsafe {
                glEnable(GL_SCISSOR_TEST);
            }

            if pipeline.params.depth_write {
                unsafe {
                    glEnable(GL_DEPTH_TEST);
                    glDepthFunc(pipeline.params.depth_test.into())
                }
            } else {
                unsafe {
                    glDisable(GL_DEPTH_TEST);
                }
            }

            match pipeline.params.front_face_order {
                FrontFaceOrder::Clockwise => unsafe {
                    glFrontFace(GL_CW);
                },
                FrontFaceOrder::CounterClockwise => unsafe {
                    glFrontFace(GL_CCW);
                },
            }
        }

        self.set_cull_face(self.pipelines[pipeline.0].params.cull_face);
        self.set_blend(
            self.pipelines[pipeline.0].params.color_blend,
            self.pipelines[pipeline.0].params.alpha_blend,
        );

        self.set_stencil(self.pipelines[pipeline.0].params.stencil_test);
        self.set_color_write(self.pipelines[pipeline.0].params.color_write);
    }
    
    pub fn gl_apply_bindings_from_slice(
        &mut self,
        vertex_buffers: &[BufferId],
        index_buffer: BufferId,
        textures: &[TextureId],
    ) {
        let pip = &self.pipelines[self.cache.cur_pipeline.unwrap().0];
        let shader = &self.shaders[pip.shader.0];

        for (n, shader_image) in shader.images.iter().enumerate() {
            let bindings_image = textures
                .get(n)
                .unwrap_or_else(|| panic!("Image count in bindings and shader did not match!"));
            let gl_loc = shader_image.gl_loc;
            let texture = self.textures.get(*bindings_image);
            let raw = match texture.raw {
                super::texture::TextureOrRenderbuffer::Texture(id) => id,
                super::texture::TextureOrRenderbuffer::Renderbuffer(id) => id,
            };
            unsafe {
                self.cache.bind_texture(n, texture.params.kind.into(), raw);
                glUniform1i(gl_loc, n as i32);
            }
        }

        self.cache.bind_buffer(
            GL_ELEMENT_ARRAY_BUFFER,
            self.buffers[index_buffer.0].gl_buf,
            self.buffers[index_buffer.0].index_type,
        );

        let pip = &self.pipelines[self.cache.cur_pipeline.unwrap().0];

        for attr_index in 0..MAX_VERTEX_ATTRIBUTES {
            let cached_attr = &mut self.cache.attributes[attr_index];

            let pip_attribute = pip.layout.get(attr_index).copied();

            if let Some(Some(attribute)) = pip_attribute {
                assert!(
                    attribute.buffer_index < vertex_buffers.len(),
                    "Attribute index outside of vertex_buffers length"
                );
                let vb = vertex_buffers[attribute.buffer_index];
                let vb = self.buffers[vb.0];

                if cached_attr.map_or(true, |cached_attr| {
                    attribute != cached_attr.attribute || cached_attr.gl_vbuf != vb.gl_buf
                }) {
                    self.cache
                        .bind_buffer(GL_ARRAY_BUFFER, vb.gl_buf, vb.index_type);

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
                                    attribute.stride,
                                    attribute.offset as *mut _,
                                )
                            }
                            _ => glVertexAttribPointer(
                                attr_index as GLuint,
                                attribute.size,
                                attribute.type_,
                                GL_FALSE as u8,
                                attribute.stride,
                                attribute.offset as *mut _,
                            ),
                        }
                        if self.info.features.instancing {
                            glVertexAttribDivisor(attr_index as GLuint, attribute.divisor as u32);
                        }
                        glEnableVertexAttribArray(attr_index as GLuint);
                    };

                    let cached_attr = &mut self.cache.attributes[attr_index];
                    *cached_attr = Some(CachedAttribute {
                        attribute,
                        gl_vbuf: vb.gl_buf,
                    });
                }
            } else if cached_attr.is_some() {
                unsafe {
                    glDisableVertexAttribArray(attr_index as GLuint);
                }
                *cached_attr = None;
            }
        }
    }

    pub fn gl_apply_uniforms_from_bytes(&mut self, uniform_ptr: *const u8, size: usize) {
        let pip = &self.pipelines[self.cache.cur_pipeline.unwrap().0];
        let shader = &self.shaders[pip.shader.0];

        let mut offset = 0;

        for uniform in shader.uniforms.iter() {
            use UniformType::*;

            assert!(
                offset as i32 <= size as i32 - uniform.uniform_type.size() as i32 / 4,
                "Uniforms struct does not match shader uniforms layout"
            );

            unsafe {
                let data = (uniform_ptr as *const f32).add(offset);
                let data_int = (uniform_ptr as *const i32).add(offset);
                let gl_loc = uniform.gl_loc;

                match uniform.uniform_type {
                    Float1 => {
                        glUniform1fv(gl_loc, uniform.array_count, data);
                    }
                    Float2 => {
                        glUniform2fv(gl_loc, uniform.array_count, data);
                    }
                    Float3 => {
                        glUniform3fv(gl_loc, uniform.array_count, data);
                    }
                    Float4 => {
                        glUniform4fv(gl_loc, uniform.array_count, data);
                    }
                    Int1 => {
                        glUniform1iv(gl_loc, uniform.array_count, data_int);
                    }
                    Int2 => {
                        glUniform2iv(gl_loc, uniform.array_count, data_int);
                    }
                    Int3 => {
                        glUniform3iv(gl_loc, uniform.array_count, data_int);
                    }
                    Int4 => {
                        glUniform4iv(gl_loc, uniform.array_count, data_int);
                    }
                    Mat4 => {
                        glUniformMatrix4fv(gl_loc, uniform.array_count, 0, data);
                    }
                }
            }
            offset += uniform.uniform_type.size() / 4 * uniform.array_count as usize;
        }
    }
}
