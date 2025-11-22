use crate::graphics::gl::GlContext;
use crate::graphics::{PassAction, RenderPassId, TextureId};
use crate::native::gl::*;
use crate::window;

pub struct RenderPassInternal {
    pub gl_fb: GLuint,
    pub color_textures: Vec<TextureId>,
    pub resolves: Option<Vec<(u32, TextureId)>>,
    pub depth_texture: Option<TextureId>,
}

impl GlContext {
    pub fn new_gl_render_pass_mrt(
        &mut self,
        color_img: &[TextureId],
        resolve_img: Option<&[TextureId]>,
        depth_img: Option<TextureId>,
    ) -> RenderPassId {
        if color_img.is_empty() && depth_img.is_none() {
            panic!("Render pass should have at least one non-none target");
        }
        let mut gl_fb = 0;

        let mut resolves = None;
        unsafe {
            glGenFramebuffers(1, &mut gl_fb as *mut _);
            glBindFramebuffer(GL_FRAMEBUFFER, gl_fb);
            for (i, color_img) in color_img.iter().enumerate() {
                let texture = self.textures.get(*color_img);
                glFramebufferTexture2D(
                    GL_FRAMEBUFFER,
                    GL_COLOR_ATTACHMENT0 + i as u32,
                    GL_TEXTURE_2D,
                    texture.gl_tex,
                    0,
                );
            }
            if let Some(depth_img) = depth_img {
                let texture = self.textures.get(depth_img);
                glFramebufferTexture2D(
                    GL_FRAMEBUFFER,
                    GL_DEPTH_ATTACHMENT,
                    GL_TEXTURE_2D,
                    texture.gl_tex,
                    0,
                );
            }
            let mut attachments = vec![];
            for i in 0..color_img.len() {
                attachments.push(GL_COLOR_ATTACHMENT0 + i as u32);
            }

            if color_img.len() > 1 {
                glDrawBuffers(color_img.len() as _, attachments.as_ptr() as _);
            }

            if let Some(resolve_img) = resolve_img {
                resolves = Some(vec![]);
                let resolves = resolves.as_mut().unwrap();
                for (i, resolve_img) in resolve_img.iter().enumerate() {
                    let mut resolve_fb = 0;
                    glGenFramebuffers(1, &mut resolve_fb as *mut _);
                    glBindFramebuffer(GL_FRAMEBUFFER, resolve_fb);
                    resolves.push((resolve_fb, *resolve_img));
                    let texture = self.textures.get(*resolve_img);
                    glFramebufferTexture2D(
                        GL_FRAMEBUFFER,
                        GL_COLOR_ATTACHMENT0 + i as u32,
                        GL_TEXTURE_2D,
                        texture.gl_tex,
                        0,
                    );
                    let fb_status = glCheckFramebufferStatus(GL_FRAMEBUFFER);
                    assert!(fb_status != 0);
                    glDrawBuffers(1, attachments.as_ptr() as _);
                }
            }
            glBindFramebuffer(GL_FRAMEBUFFER, self.default_framebuffer);
        }
        let pass = RenderPassInternal {
            gl_fb,
            color_textures: color_img.to_vec(),
            resolves,
            depth_texture: depth_img,
        };

        RenderPassId(self.passes.add(pass))
    }

    pub fn gl_render_pass_color_attachments(&self, render_pass: RenderPassId) -> &[TextureId] {
        &self.passes[render_pass.0].color_textures
    }

    pub fn delete_gl_render_pass(&mut self, render_pass: RenderPassId) {
        let pass_id = render_pass.0;

        let render_pass = self.passes.remove(pass_id);

        unsafe { glDeleteFramebuffers(1, &render_pass.gl_fb as *const _) }

        for color_texture in &render_pass.color_textures {
            self.delete_gl_texture(*color_texture);
        }
        if let Some(resolves) = render_pass.resolves {
            for (fb, texture) in resolves {
                unsafe { glDeleteFramebuffers(1, &fb as *const _) }
                self.delete_gl_texture(texture);
            }
        }
        if let Some(depth_texture) = render_pass.depth_texture {
            self.delete_gl_texture(depth_texture);
        }
    }

    pub fn begin_default_gl_render_pass(&mut self, action: PassAction) {
        self.begin_gl_render_pass(None, action);
    }

    pub fn begin_gl_render_pass(&mut self, pass: Option<RenderPassId>, action: PassAction) {
        self.cache.cur_pass = pass;
        let (framebuffer, w, h) = match pass {
            None => {
                let (screen_width, screen_height) = window::screen_size();

                (
                    self.default_framebuffer,
                    screen_width as i32,
                    screen_height as i32,
                )
            }
            Some(pass) => {
                let pass = &self.passes[pass.0];
                // new_render_pass will panic with both color and depth components none
                // so unwrap is safe here
                let texture = pass
                    .color_textures
                    .first()
                    .copied()
                    .or(pass.depth_texture)
                    .unwrap();
                (
                    pass.gl_fb,
                    self.textures.get(texture).params.width as i32,
                    self.textures.get(texture).params.height as i32,
                )
            }
        };
        unsafe {
            glBindFramebuffer(GL_FRAMEBUFFER, framebuffer);
            glViewport(0, 0, w, h);
            glScissor(0, 0, w, h);
        }
        match action {
            PassAction::Nothing => {}
            PassAction::Clear {
                color,
                depth,
                stencil,
            } => {
                self.gl_clear(color, depth, stencil);
            }
        }
    }

    pub fn end_gl_render_pass(&mut self) {
        unsafe {
            if let Some(pass) = self.cache.cur_pass.take() {
                let pass = &self.passes[pass.0];
                if let Some(resolves) = &pass.resolves {
                    glBindFramebuffer(GL_READ_FRAMEBUFFER, pass.gl_fb);
                    for (i, (resolve_fb, resolve_img)) in resolves.iter().enumerate() {
                        let texture = self.textures.get(*resolve_img);
                        let w = texture.params.width;
                        let h = texture.params.height;
                        glBindFramebuffer(GL_DRAW_FRAMEBUFFER, *resolve_fb);
                        glReadBuffer(GL_COLOR_ATTACHMENT0 + i as u32);
                        glBlitFramebuffer(
                            0,
                            0,
                            w as _,
                            h as _,
                            0,
                            0,
                            w as _,
                            h as _,
                            GL_COLOR_BUFFER_BIT,
                            GL_NEAREST,
                        );
                    }
                }
            }
            glBindFramebuffer(GL_FRAMEBUFFER, self.default_framebuffer);
            self.cache.bind_buffer(GL_ARRAY_BUFFER, 0, None);
            self.cache.bind_buffer(GL_ELEMENT_ARRAY_BUFFER, 0, None);
        }
    }

    pub fn gl_clear(
        &mut self,
        color: Option<(f32, f32, f32, f32)>,
        depth: Option<f32>,
        stencil: Option<i32>,
    ) {
        let mut bits = 0;
        if let Some((r, g, b, a)) = color {
            bits |= GL_COLOR_BUFFER_BIT;
            unsafe {
                glClearColor(r, g, b, a);
            }
        }

        if let Some(v) = depth {
            bits |= GL_DEPTH_BUFFER_BIT;
            unsafe {
                glClearDepthf(v);
            }
        }

        if let Some(v) = stencil {
            bits |= GL_STENCIL_BUFFER_BIT;
            unsafe {
                glClearStencil(v);
            }
        }

        if bits != 0 {
            unsafe {
                glClear(bits);
            }
        }
    }
}
