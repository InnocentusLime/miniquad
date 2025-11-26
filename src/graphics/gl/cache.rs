use glow::HasContext;

use crate::graphics::*;

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct VertexAttributeInternal {
    pub attr_loc: u32,
    pub size: i32,
    pub type_: u32,
    pub gl_pass_as_float: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct CachedTexture {
    pub target: u32,
    pub texture: glow::Texture,
}

pub struct GlCache {
    pub index_buffer: Option<glow::Buffer>,
    pub vertex_buffer: Option<glow::Buffer>,
    pub textures: [Option<CachedTexture>; MAX_SHADERSTAGE_IMAGES],
    pub cur_pipeline: Option<PipelineId>,

    pub color_blend: Option<BlendState>,
    pub alpha_blend: Option<BlendState>,
    pub stencil: Option<StencilState>,
    pub color_write: ColorMask,
    pub cull_face: CullFace,
}

impl GlCache {
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
                gl.active_texture(slot_index);
                gl.bind_texture(target, Some(texture));
            }
            self.textures[slot_index as usize] = Some(store);
        }
    }
}
