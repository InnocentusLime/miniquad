use std::rc::Rc;

use crate::graphics::gl::texture::Texture;
use crate::graphics::gl::GlContext;
use crate::graphics::PassAction;
use crate::native::gl::*;
use crate::window;

#[derive(Clone)]
pub struct RenderPass(Rc<RenderPassInternal>);

impl RenderPass {
    pub fn new(
        ctx: Rc<GlContext>,
        color_img: Vec<Texture>,
        resolve_img: Option<Vec<Texture>>,
        depth_img: Option<Texture>,
    ) -> RenderPass {
        if color_img.is_empty() && depth_img.is_none() {
            panic!("Render pass should have at least one non-none target");
        }
        let mut gl_fb = 0;

        let mut resolves = None;
        unsafe {
            glGenFramebuffers(1, &mut gl_fb as *mut _);
            glBindFramebuffer(GL_FRAMEBUFFER, gl_fb);
            for (i, color_img) in color_img.iter().enumerate() {
                glFramebufferTexture2D(
                    GL_FRAMEBUFFER,
                    GL_COLOR_ATTACHMENT0 + i as u32,
                    GL_TEXTURE_2D,
                    color_img.gl_tex(),
                    0,
                );
            }
            if let Some(depth_img) = &depth_img {
                glFramebufferTexture2D(
                    GL_FRAMEBUFFER,
                    GL_DEPTH_ATTACHMENT,
                    GL_TEXTURE_2D,
                    depth_img.gl_tex(),
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
                for (i, resolve_img) in resolve_img.into_iter().enumerate() {
                    let mut resolve_fb = 0;
                    glGenFramebuffers(1, &mut resolve_fb as *mut _);
                    glBindFramebuffer(GL_FRAMEBUFFER, resolve_fb);
                    glFramebufferTexture2D(
                        GL_FRAMEBUFFER,
                        GL_COLOR_ATTACHMENT0 + i as u32,
                        GL_TEXTURE_2D,
                        resolve_img.gl_tex(),
                        0,
                    );
                    let fb_status = glCheckFramebufferStatus(GL_FRAMEBUFFER);
                    assert!(fb_status != 0);
                    glDrawBuffers(1, attachments.as_ptr() as _);

                    resolves.push((resolve_fb, resolve_img));
                }
            }
            glBindFramebuffer(GL_FRAMEBUFFER, 0);
        }

        let pass = RenderPassInternal {
            ctx,
            gl_fb,
            color_textures: color_img.to_vec(),
            resolves,
            depth_texture: depth_img,
        };
        RenderPass(Rc::new(pass))
    }

    pub fn color_attachments(&self) -> &[Texture] {
        &self.0.color_textures
    }

    pub fn perform(&self, pass_action: PassAction, code: impl FnOnce()) {
        // new_render_pass will panic with both color and depth components none
        // so unwrap is safe here
        let texture = self
            .0
            .color_textures
            .first()
            .or(self.0.depth_texture.as_ref())
            .unwrap();
        let (framebuffer, w, h) = (
            self.0.gl_fb,
            texture.width() as i32,
            texture.height() as i32,
        );

        unsafe {
            glBindFramebuffer(GL_FRAMEBUFFER, framebuffer);
            glViewport(0, 0, w, h);
            glScissor(0, 0, w, h);
        }
        gl_clear(pass_action);

        code();

        unsafe {
            if let Some(resolves) = &self.0.resolves {
                glBindFramebuffer(GL_READ_FRAMEBUFFER, self.0.gl_fb);
                for (i, (resolve_fb, resolve_img)) in resolves.iter().enumerate() {
                    let w = resolve_img.width();
                    let h = resolve_img.height();
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
            glBindFramebuffer(GL_FRAMEBUFFER, 0);
        }

        let mut cache = self.0.ctx.cache.borrow_mut();
        cache.bind_buffer(GL_ARRAY_BUFFER, 0, None);
        cache.bind_buffer(GL_ELEMENT_ARRAY_BUFFER, 0, None);
    }
}

impl GlContext {
    pub fn perform_default_render_pass(&self, pass_action: PassAction, code: impl FnOnce()) {
        let (screen_width, screen_height) = window::screen_size();
        let (w, h) = (screen_width as i32, screen_height as i32);
        unsafe {
            glBindFramebuffer(GL_FRAMEBUFFER, 0);
            glViewport(0, 0, w, h);
            glScissor(0, 0, w, h);
        }
        gl_clear(pass_action);

        code();

        let mut cache = self.cache.borrow_mut();
        cache.bind_buffer(GL_ARRAY_BUFFER, 0, None);
        cache.bind_buffer(GL_ELEMENT_ARRAY_BUFFER, 0, None);
    }
}

struct RenderPassInternal {
    ctx: Rc<GlContext>,
    gl_fb: GLuint,
    color_textures: Vec<Texture>,
    resolves: Option<Vec<(u32, Texture)>>,
    depth_texture: Option<Texture>,
}

impl Drop for RenderPassInternal {
    fn drop(&mut self) {
        unsafe { glDeleteFramebuffers(1, &self.gl_fb as *const _) }

        if let Some(resolves) = &self.resolves {
            for (fb, _) in resolves {
                unsafe { glDeleteFramebuffers(1, fb as *const _) }
            }
        }
    }
}

fn gl_clear(pass_action: PassAction) {
    let mut bits = 0;
    if let Some((r, g, b, a)) = pass_action.color {
        bits |= GL_COLOR_BUFFER_BIT;
        unsafe {
            glClearColor(r, g, b, a);
        }
    }

    if let Some(v) = pass_action.depth {
        bits |= GL_DEPTH_BUFFER_BIT;
        unsafe {
            glClearDepthf(v);
        }
    }

    if let Some(v) = pass_action.stencil {
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
