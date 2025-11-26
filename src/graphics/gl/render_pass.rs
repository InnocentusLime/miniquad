use std::rc::Rc;

use glow::HasContext;

use crate::graphics::gl::texture::Texture;
use crate::graphics::gl::GlContext;
use crate::graphics::PassAction;
use crate::window;

#[derive(Clone)]
pub struct RenderPass(Rc<RenderPassInternal>);

impl RenderPass {
    pub fn new(
        ctx: Rc<GlContext>,
        color_img: Vec<Texture>,
        depth_img: Option<Texture>,
    ) -> RenderPass {
        if color_img.is_empty() && depth_img.is_none() {
            panic!("Render pass should have at least one non-none target");
        }

        let gl_fb = unsafe { ctx.gl.create_framebuffer().unwrap() };

        unsafe {
            ctx.gl.bind_framebuffer(glow::FRAMEBUFFER, Some(gl_fb));
            for (i, color_img) in color_img.iter().enumerate() {
                ctx.gl.framebuffer_texture_2d(
                    glow::FRAMEBUFFER,
                    glow::COLOR_ATTACHMENT0 + i as u32,
                    glow::TEXTURE_2D,
                    Some(color_img.gl_tex()),
                    0,
                );
            }
            if let Some(depth_img) = &depth_img {
                ctx.gl.framebuffer_texture_2d(
                    glow::FRAMEBUFFER,
                    glow::DEPTH_ATTACHMENT,
                    glow::TEXTURE_2D,
                    Some(depth_img.gl_tex()),
                    0,
                );
            }
            let mut attachments = vec![];
            for i in 0..color_img.len() {
                attachments.push(glow::COLOR_ATTACHMENT0 + i as u32);
            }

            if color_img.len() > 1 {
                ctx.gl.draw_buffers(&attachments);
            }
        }

        let pass = RenderPassInternal {
            ctx,
            gl_fb,
            color_textures: color_img.to_vec(),
            depth_texture: depth_img,
        };
        RenderPass(Rc::new(pass))
    }

    pub fn color_attachments(&self) -> &[Texture] {
        &self.0.color_textures
    }

    pub fn perform(&self, pass_action: PassAction, code: impl FnOnce()) {
        let gl = &self.0.ctx.gl;
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
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(framebuffer));
            gl.viewport(0, 0, w, h);
            gl.scissor(0, 0, w, h);
        }
        gl_clear(gl, pass_action);

        code();

        // TODO: add "resolves"
    }
}

impl GlContext {
    pub fn perform_default_render_pass(&self, pass_action: PassAction, code: impl FnOnce()) {
        let (screen_width, screen_height) = window::screen_size();
        let (w, h) = (screen_width as i32, screen_height as i32);
        unsafe {
            self.gl.bind_framebuffer(glow::FRAMEBUFFER, None);
            self.gl.viewport(0, 0, w, h);
            self.gl.scissor(0, 0, w, h);
        }
        gl_clear(&self.gl, pass_action);

        code();
    }
}

struct RenderPassInternal {
    ctx: Rc<GlContext>,
    gl_fb: glow::Framebuffer,
    color_textures: Vec<Texture>,
    depth_texture: Option<Texture>,
}

impl Drop for RenderPassInternal {
    fn drop(&mut self) {
        unsafe { self.ctx.gl.delete_framebuffer(self.gl_fb) }
    }
}

fn gl_clear(gl: &glow::Context, pass_action: PassAction) {
    let mut bits = 0;
    if let Some((r, g, b, a)) = pass_action.color {
        bits |= glow::COLOR_BUFFER_BIT;
        unsafe {
            gl.clear_color(r, g, b, a);
        }
    }

    if let Some(v) = pass_action.depth {
        bits |= glow::DEPTH_BUFFER_BIT;
        unsafe {
            gl.clear_depth_f32(v);
        }
    }

    if let Some(v) = pass_action.stencil {
        bits |= glow::STENCIL_BUFFER_BIT;
        unsafe {
            gl.clear_stencil(v);
        }
    }

    if bits != 0 {
        unsafe {
            gl.clear(bits);
        }
    }
}
