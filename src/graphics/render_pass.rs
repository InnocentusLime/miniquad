use std::rc::Rc;

use glow::HasContext;

use crate::graphics::GlContext;
use crate::graphics::texture::Texture;

#[derive(Debug)]
pub struct RenderPass {
    ctx: Rc<GlContext>,
    pub(crate) gl_fb: glow::Framebuffer,
    color_textures: Vec<Texture>,
    depth_texture: Option<Texture>,
}

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
                    Some(color_img.gl_tex),
                    0,
                );
            }
            if let Some(depth_img) = &depth_img {
                ctx.gl.framebuffer_texture_2d(
                    glow::FRAMEBUFFER,
                    glow::DEPTH_ATTACHMENT,
                    glow::TEXTURE_2D,
                    Some(depth_img.gl_tex),
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

        RenderPass {
            ctx,
            gl_fb,
            color_textures: color_img,
            depth_texture: depth_img,
        }
    }

    pub fn color_attachments(&self) -> &[Texture] {
        &self.color_textures
    }

    pub fn perform(&self, pass_action: PassAction, code: impl FnOnce()) {
        // new_render_pass will panic with both color and depth components none
        // so unwrap is safe here
        let texture = self
            .color_textures
            .first()
            .or(self.depth_texture.as_ref())
            .unwrap();
        let (framebuffer, w, h) = (
            self.gl_fb,
            texture.width() as i32,
            texture.height() as i32,
        );

        unsafe {
            self.ctx.gl.bind_framebuffer(glow::FRAMEBUFFER, Some(framebuffer));
            self.ctx.gl.viewport(0, 0, w, h);
            self.ctx.gl.scissor(0, 0, w, h);
        }
        gl_clear(&self.ctx.gl, pass_action);

        code();
    }
}

impl Drop for RenderPass {
    fn drop(&mut self) {
        unsafe { self.ctx.gl.delete_framebuffer(self.gl_fb) }
    }
}

impl GlContext {
    pub fn perform_default_render_pass(&self, pass_action: PassAction, code: impl FnOnce()) {
        let (screen_width, screen_height) = self.screen_size();
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

#[derive(Clone, Copy)]
pub struct PassAction {
    color: Option<(f32, f32, f32, f32)>,
    depth: Option<f32>,
    stencil: Option<i32>,
}

impl PassAction {
    pub const fn clear_depth_color(r: f32, g: f32, b: f32, a: f32) -> PassAction {
        PassAction {
            color: Some((r, g, b, a)),
            depth: Some(1.),
            stencil: None,
        }
    }
}
