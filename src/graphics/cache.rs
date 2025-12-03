use glow::HasContext;

use crate::graphics::*;

#[derive(Debug)]
pub struct GlCache {
    index_buffer: Option<glow::Buffer>,
    vertex_buffer: Option<glow::Buffer>,
    textures: [Option<CachedTexture>; MAX_SHADERSTAGE_IMAGES],
    program: Option<glow::Program>,

    pub color_blend: Option<BlendState>,
    pub alpha_blend: Option<BlendState>,
    pub stencil: Option<StencilState>,
    pub color_write: ColorMask,
    pub cull_face: CullFace,
}

impl GlCache {
    pub fn new() -> GlCache {
        GlCache {
            index_buffer: None,
            vertex_buffer: None,
            textures: [None; MAX_SHADERSTAGE_IMAGES],
            program: None,

            color_blend: None,
            alpha_blend: None,
            stencil: None,
            color_write: (true, true, true, true),
            cull_face: CullFace::Nothing,
        }
    }

    pub fn bind_buffer(&mut self, gl: &glow::Context, buffer: glow::Buffer) {
        if self.vertex_buffer != Some(buffer) {
            self.vertex_buffer = Some(buffer);
            unsafe {
                gl.bind_buffer(glow::ARRAY_BUFFER, Some(buffer));
            }
        }
    }

    pub fn bind_index_buffer(&mut self, gl: &glow::Context, buffer: glow::Buffer) {
        if self.index_buffer != Some(buffer) {
            self.index_buffer = Some(buffer);
            unsafe {
                gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(buffer));
            }
        }
    }

    pub fn bind_texture(
        &mut self,
        gl: &glow::Context,
        slot_index: u32,
        target: u32,
        texture: glow::Texture,
    ) {
        let store = CachedTexture { target, texture };
        if self.textures[slot_index as usize] != Some(store) {
            unsafe {
                gl.active_texture(slot_index + glow::TEXTURE0);
                gl.bind_texture(target, Some(texture));
            }
            self.textures[slot_index as usize] = Some(store);
        }
    }

    pub fn bind_program(&mut self, gl: &glow::Context, program: glow::Program) {
        if self.program != Some(program) {
            self.program = Some(program);
            unsafe {
                gl.use_program(Some(program));
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CachedTexture {
    target: u32,
    texture: glow::Texture,
}

const MAX_SHADERSTAGE_IMAGES: usize = 12;
