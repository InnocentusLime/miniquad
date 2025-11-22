use crate::ResourceManager;

mod buffer;
mod cache;
mod pipeline;
mod render_pass;
mod texture;

use super::*;
use cache::*;

/// Raw OpenGL bindings
/// Highly unsafe, some of the functions could be missing due to incompatible GL version
/// or all of them might be missing alltogether if rendering context is not a GL one.
pub mod raw_gl {
    use super::*;

    #[doc(inline)]
    pub use crate::native::gl::*;

    pub fn texture_format_into_gl(format: TextureFormat) -> (GLenum, GLenum, GLenum) {
        format.into()
    }
}

pub struct GlContext {
    shaders: ResourceManager<pipeline::ShaderInternal>,
    pipelines: ResourceManager<pipeline::PipelineInternal>,
    passes: ResourceManager<render_pass::RenderPassInternal>,
    buffers: ResourceManager<buffer::Buffer>,
    textures: texture::Textures,
    default_framebuffer: GLuint,
    pub(crate) cache: GlCache,
    pub(crate) info: ContextInfo,
}

impl Default for GlContext {
    fn default() -> Self {
        Self::new()
    }
}

impl GlContext {
    pub fn new() -> GlContext {
        unsafe {
            let mut default_framebuffer: GLuint = 0;
            glGetIntegerv(
                GL_FRAMEBUFFER_BINDING,
                &mut default_framebuffer as *mut _ as *mut _,
            );
            let mut vao = 0;

            glGenVertexArrays(1, &mut vao as *mut _);
            glBindVertexArray(vao);
            let info = gl_info();
            GlContext {
                default_framebuffer,
                shaders: ResourceManager::default(),
                pipelines: ResourceManager::default(),
                passes: ResourceManager::default(),
                buffers: ResourceManager::default(),
                textures: texture::Textures::default(),
                info,
                cache: GlCache {
                    stored_index_buffer: 0,
                    stored_index_type: None,
                    stored_vertex_buffer: 0,
                    index_buffer: 0,
                    index_type: None,
                    vertex_buffer: 0,
                    cur_pipeline: None,
                    cur_pass: None,
                    color_blend: None,
                    alpha_blend: None,
                    stencil: None,
                    color_write: (true, true, true, true),
                    cull_face: CullFace::Nothing,
                    stored_texture: 0,
                    stored_target: 0,
                    textures: [CachedTexture {
                        target: 0,
                        texture: 0,
                    }; MAX_SHADERSTAGE_IMAGES],
                    attributes: [None; MAX_VERTEX_ATTRIBUTES],
                },
            }
        }
    }

    pub fn features(&self) -> &Features {
        &self.info.features
    }

    fn set_blend(&mut self, color_blend: Option<BlendState>, alpha_blend: Option<BlendState>) {
        if color_blend.is_none() && alpha_blend.is_some() {
            panic!("AlphaBlend without ColorBlend");
        }
        if self.cache.color_blend == color_blend && self.cache.alpha_blend == alpha_blend {
            return;
        }

        unsafe {
            if let Some(color_blend) = color_blend {
                if self.cache.color_blend.is_none() {
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
            } else if self.cache.color_blend.is_some() {
                glDisable(GL_BLEND);
            }
        }

        self.cache.color_blend = color_blend;
        self.cache.alpha_blend = alpha_blend;
    }

    fn set_stencil(&mut self, stencil_test: Option<StencilState>) {
        if self.cache.stencil == stencil_test {
            return;
        }
        unsafe {
            if let Some(stencil) = stencil_test {
                if self.cache.stencil.is_none() {
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
            } else if self.cache.stencil.is_some() {
                glDisable(GL_STENCIL_TEST);
            }
        }

        self.cache.stencil = stencil_test;
    }

    fn set_cull_face(&mut self, cull_face: CullFace) {
        if self.cache.cull_face == cull_face {
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
        self.cache.cull_face = cull_face;
    }

    fn set_color_write(&mut self, color_write: ColorMask) {
        if self.cache.color_write == color_write {
            return;
        }
        let (r, g, b, a) = color_write;
        unsafe { glColorMask(r as _, g as _, b as _, a as _) }
        self.cache.color_write = color_write;
    }
}

impl RenderingBackend for GlContext {
    fn info(&self) -> ContextInfo {
        self.info.clone()
    }

    fn new_texture(&mut self, source: TextureSource, params: TextureParams) -> TextureId {
        self.new_gl_texture(source, params)
    }

    fn delete_texture(&mut self, texture: TextureId) {
        self.delete_gl_texture(texture);
    }

    fn delete_shader(&mut self, program: ShaderId) {
        self.delete_gl_shader(program);
    }

    fn delete_pipeline(&mut self, pipeline: PipelineId) {
        self.delete_gl_pipeline(pipeline);
    }

    fn texture_set_wrap(&mut self, texture: TextureId, wrap_x: TextureWrap, wrap_y: TextureWrap) {
        self.gl_texture_set_wrap(texture, wrap_x, wrap_y);
    }

    fn texture_set_min_filter(
        &mut self,
        texture: TextureId,
        filter: FilterMode,
        mipmap_filter: MipmapFilterMode,
    ) {
        self.gl_texture_set_min_filter(texture, filter, mipmap_filter);
    }

    fn texture_set_mag_filter(&mut self, texture: TextureId, filter: FilterMode) {
        self.gl_texture_set_mag_filter(texture, filter);
    }

    fn texture_resize(
        &mut self,
        texture: TextureId,
        width: u32,
        height: u32,
        source: Option<&[u8]>,
    ) {
        self.gl_texture_resize(texture, width, height, source);
    }

    fn texture_read_pixels(&mut self, texture: TextureId, source: &mut [u8]) {
        self.gl_texture_read_pixels(texture, source);
    }

    fn texture_generate_mipmaps(&mut self, texture: TextureId) {
        self.gl_texture_generate_mipmaps(texture);
    }

    fn texture_update_part(
        &mut self,
        texture: TextureId,
        x_offset: i32,
        y_offset: i32,
        width: i32,
        height: i32,
        source: &[u8],
    ) {
        self.gl_texture_update_part(texture, x_offset, y_offset, width, height, source);
    }

    fn texture_params(&self, texture: TextureId) -> TextureParams {
        self.gl_texture_params(texture)
    }

    unsafe fn texture_raw_id(&self, texture: TextureId) -> RawId {
        unsafe { self.gl_texture_raw_id(texture) }
    }

    fn new_render_pass_mrt(
        &mut self,
        color_img: &[TextureId],
        resolve_img: Option<&[TextureId]>,
        depth_img: Option<TextureId>,
    ) -> RenderPassId {
        self.new_gl_render_pass_mrt(color_img, resolve_img, depth_img)
    }

    fn render_pass_color_attachments(&self, render_pass: RenderPassId) -> &[TextureId] {
        self.gl_render_pass_color_attachments(render_pass)
    }

    fn delete_render_pass(&mut self, render_pass: RenderPassId) {
        self.delete_gl_render_pass(render_pass);
    }

    fn new_shader(
        &mut self,
        shader: ShaderSource,
        meta: ShaderMeta,
    ) -> Result<ShaderId, ShaderError> {
        self.new_gl_shader(shader, meta)
    }

    fn new_pipeline(
        &mut self,
        buffer_layout: &[BufferLayout],
        attributes: &[VertexAttribute],
        shader: ShaderId,
        params: PipelineParams,
    ) -> PipelineId {
        self.new_gl_pipeline(buffer_layout, attributes, shader, params)
    }

    fn apply_pipeline(&mut self, pipeline: &PipelineId) {
        self.apply_gl_pipeline(pipeline);
    }

    fn apply_bindings_from_slice(
        &mut self,
        vertex_buffers: &[BufferId],
        index_buffer: BufferId,
        textures: &[TextureId],
    ) {
        self.gl_apply_bindings_from_slice(vertex_buffers, index_buffer, textures);
    }

    fn apply_uniforms_from_bytes(&mut self, uniform_ptr: *const u8, size: usize) {
        self.gl_apply_uniforms_from_bytes(uniform_ptr, size);
    }

    /// Set a new viewport rectangle.
    /// Should be applied after begin_pass.
    fn apply_viewport(&mut self, x: i32, y: i32, w: i32, h: i32) {
        unsafe {
            glViewport(x, y, w, h);
        }
    }

    /// Set a new scissor rectangle.
    /// Should be applied after begin_pass.
    fn apply_scissor_rect(&mut self, x: i32, y: i32, w: i32, h: i32) {
        unsafe {
            glScissor(x, y, w, h);
        }
    }

    fn clear(
        &mut self,
        color: Option<(f32, f32, f32, f32)>,
        depth: Option<f32>,
        stencil: Option<i32>,
    ) {
        self.gl_clear(color, depth, stencil);
    }

    fn begin_default_render_pass(&mut self, action: PassAction) {
        self.begin_default_gl_render_pass(action);
    }

    fn begin_render_pass(&mut self, pass: Option<RenderPassId>, action: PassAction) {
        self.begin_gl_render_pass(pass, action);
    }

    fn end_render_pass(&mut self) {
        self.end_gl_render_pass();
    }

    fn commit_frame(&mut self) {
        self.cache.clear_buffer_bindings();
        self.cache.clear_texture_bindings();
    }

    fn draw(&self, base_element: i32, num_elements: i32, num_instances: i32) {
        assert!(
            self.cache.cur_pipeline.is_some(),
            "Drawing without any binded pipeline"
        );

        if !self.info.features.instancing && num_instances != 1 {
            eprintln!("Instanced rendering is not supported by the GPU");
            eprintln!("Ignoring this draw call");
            return;
        }

        let pip = &self.pipelines[self.cache.cur_pipeline.unwrap().0];
        let primitive_type = pip.params.primitive_type.into();
        let index_type = self.cache.index_type.expect("Unset index buffer type");

        unsafe {
            glDrawElementsInstanced(
                primitive_type,
                num_elements,
                match index_type {
                    1 => GL_UNSIGNED_BYTE,
                    2 => GL_UNSIGNED_SHORT,
                    4 => GL_UNSIGNED_INT,
                    _ => panic!("Unsupported index buffer type!"),
                },
                (index_type as i32 * base_element) as *mut _,
                num_instances,
            );
        }
    }

    fn new_buffer(
        &mut self,
        type_: BufferType,
        usage: BufferUsage,
        data: BufferSource,
    ) -> BufferId {
        self.new_gl_buffer(type_, usage, data)
    }

    fn buffer_update(&mut self, buffer: BufferId, data: BufferSource) {
        self.gl_buffer_update(buffer, data);
    }

    fn buffer_size(&mut self, buffer: BufferId) -> usize {
        self.gl_buffer_size(buffer)
    }

    fn delete_buffer(&mut self, buffer: BufferId) {
        self.delete_gl_buffer(buffer);
    }
}

#[allow(clippy::field_reassign_with_default)]
fn gl_info() -> ContextInfo {
    let version_string = unsafe { glGetString(super::gl::GL_VERSION) };
    let gl_version_string = unsafe { std::ffi::CStr::from_ptr(version_string as _) }
        .to_str()
        .unwrap()
        .to_string();
    //let gles2 = !gles3 && gl_version_string.contains("OpenGL ES");

    let gl2 = gl_version_string.is_empty()
        || gl_version_string.starts_with("2")
        || gl_version_string.starts_with("OpenGL ES 2");
    let webgl1 = gl_version_string == "WebGL 1.0";

    let features = Features {
        instancing: !gl2,
        resolve_attachments: !webgl1 && !gl2,
    };

    let mut glsl_support = GlslSupport::default();

    // this is not quite documented,
    // but somehow even GL2.1 usually have all the compatibility extensions to support glsl100
    // It was tested on really old windows machines, virtual machines etc. glsl100 always works!
    glsl_support.v100 = true;

    // on wasm miniquad always creates webgl1 context, with the only glsl available being version 100
    #[cfg(target_arch = "wasm32")]
    {
        // on web, miniquad always loads EXT_shader_texture_lod and OES_standard_derivatives
        glsl_support.v100_ext = true;

        let webgl2 = gl_version_string.contains("WebGL 2.0");
        if webgl2 {
            glsl_support.v300es = true;
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let gles3 = gl_version_string.contains("OpenGL ES 3");

        if gles3 {
            glsl_support.v300es = true;
        }
    }

    // there is no gl3.4, so 4+ and 3.3 covers all modern OpenGL
    if gl_version_string.starts_with("3.2") {
        glsl_support.v150 = true; // MacOS is defaulting to 3.2 and GLSL 150
    } else if gl_version_string.starts_with("4") || gl_version_string.starts_with("3.3") {
        glsl_support.v330 = true;
    // gl 3.0, 3.1, 3.2 maps to 1.30, 1.40, 1.50 glsl versions
    } else if gl_version_string.starts_with("3") {
        glsl_support.v130 = true;
    }

    ContextInfo {
        backend: Backend::OpenGl,
        gl_version_string,
        glsl_support,
        features,
    }
}

impl From<Equation> for GLenum {
    fn from(eq: Equation) -> Self {
        match eq {
            Equation::Add => GL_FUNC_ADD,
            Equation::Subtract => GL_FUNC_SUBTRACT,
            Equation::ReverseSubtract => GL_FUNC_REVERSE_SUBTRACT,
        }
    }
}

impl From<BlendFactor> for GLenum {
    fn from(factor: BlendFactor) -> GLenum {
        match factor {
            BlendFactor::Zero => GL_ZERO,
            BlendFactor::One => GL_ONE,
            BlendFactor::Value(BlendValue::SourceColor) => GL_SRC_COLOR,
            BlendFactor::Value(BlendValue::SourceAlpha) => GL_SRC_ALPHA,
            BlendFactor::Value(BlendValue::DestinationColor) => GL_DST_COLOR,
            BlendFactor::Value(BlendValue::DestinationAlpha) => GL_DST_ALPHA,
            BlendFactor::OneMinusValue(BlendValue::SourceColor) => GL_ONE_MINUS_SRC_COLOR,
            BlendFactor::OneMinusValue(BlendValue::SourceAlpha) => GL_ONE_MINUS_SRC_ALPHA,
            BlendFactor::OneMinusValue(BlendValue::DestinationColor) => GL_ONE_MINUS_DST_COLOR,
            BlendFactor::OneMinusValue(BlendValue::DestinationAlpha) => GL_ONE_MINUS_DST_ALPHA,
            BlendFactor::SourceAlphaSaturate => GL_SRC_ALPHA_SATURATE,
        }
    }
}

impl From<StencilOp> for GLenum {
    fn from(op: StencilOp) -> Self {
        match op {
            StencilOp::Keep => GL_KEEP,
            StencilOp::Zero => GL_ZERO,
            StencilOp::Replace => GL_REPLACE,
            StencilOp::IncrementClamp => GL_INCR,
            StencilOp::DecrementClamp => GL_DECR,
            StencilOp::Invert => GL_INVERT,
            StencilOp::IncrementWrap => GL_INCR_WRAP,
            StencilOp::DecrementWrap => GL_DECR_WRAP,
        }
    }
}

impl From<CompareFunc> for GLenum {
    fn from(cf: CompareFunc) -> Self {
        match cf {
            CompareFunc::Always => GL_ALWAYS,
            CompareFunc::Never => GL_NEVER,
            CompareFunc::Less => GL_LESS,
            CompareFunc::Equal => GL_EQUAL,
            CompareFunc::LessOrEqual => GL_LEQUAL,
            CompareFunc::Greater => GL_GREATER,
            CompareFunc::NotEqual => GL_NOTEQUAL,
            CompareFunc::GreaterOrEqual => GL_GEQUAL,
        }
    }
}
