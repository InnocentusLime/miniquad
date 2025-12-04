use glow::HasContext;

use crate::graphics::*;

#[derive(Debug)]
pub struct GlCache {
    index_buffer: Option<glow::Buffer>,
    vertex_buffer: Option<glow::Buffer>,
    textures: [Option<CachedTexture>; MAX_SHADERSTAGE_IMAGES],
    program: Option<glow::Program>,

    front_face_order: FrontFaceOrder,
    depth_test: Option<Comparison>,
    color_blend: Option<BlendState>,
    alpha_blend: Option<BlendState>,
    stencil: Option<StencilState>,
    color_write: ColorMask,
    cull_face: CullFace,
}

impl GlCache {
    pub fn new() -> GlCache {
        GlCache {
            index_buffer: None,
            vertex_buffer: None,
            textures: [None; MAX_SHADERSTAGE_IMAGES],
            program: None,

            front_face_order: FrontFaceOrder::CounterClockwise,
            depth_test: None,
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

    pub fn set_blend(&mut self, gl: &glow::Context, color_blend: Option<BlendState>, alpha_blend: Option<BlendState>) {
        // TODO: this looks annoying. Better enforce it on type-level
        if color_blend.is_none() && alpha_blend.is_some() {
            panic!("AlphaBlend without ColorBlend");
        }
        if self.color_blend == color_blend && self.alpha_blend == alpha_blend {
            return;
        }

        unsafe {
            if let Some(color_blend) = color_blend {
                if self.color_blend.is_none() {
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
            } else if self.color_blend.is_some() {
                gl.disable(glow::BLEND);
            }
        }

        self.color_blend = color_blend;
        self.alpha_blend = alpha_blend;
    }

    pub fn set_stencil(&mut self, gl: &glow::Context, stencil_test: Option<StencilState>) {
        if self.stencil == stencil_test {
            return;
        }
        unsafe {
            if let Some(stencil) = stencil_test {
                if self.stencil.is_none() {
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
            } else if self.stencil.is_some() {
                gl.disable(glow::STENCIL_TEST);
            }
        }

        self.stencil = stencil_test;
    }

    pub fn set_depth_test(&mut self, gl: &glow::Context, depth_test: Option<Comparison>) {
        if self.depth_test == depth_test {
            return;
        }

        match (self.depth_test, depth_test) {
            (None, Some(cmp)) => unsafe {
                gl.enable(glow::DEPTH_TEST);
                gl.depth_func(cmp.into())
            },
            (Some(_), None) => unsafe {
                gl.disable(glow::DEPTH_TEST);
            },
            (Some(_), Some(cmp)) => unsafe {
                gl.depth_func(cmp.into())
            }
            _ => (),
        }
    }

    pub fn set_front_face_order(&mut self, gl: &glow::Context, front_face_order: FrontFaceOrder) {
        if self.front_face_order == front_face_order {
            return;
        }

        match front_face_order {
            FrontFaceOrder::Clockwise => unsafe {
                gl.front_face(glow::CW);
            },
            FrontFaceOrder::CounterClockwise => unsafe {
                gl.front_face(glow::CCW);
            },
        }
    }

    pub fn set_cull_face(&mut self, gl: &glow::Context, cull_face: CullFace) {
        if self.cull_face == cull_face {
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
        self.cull_face = cull_face;
    }

    pub fn set_color_write(&mut self, gl: &glow::Context, color_write: ColorMask) {
        if self.color_write == color_write {
            return;
        }
        let (r, g, b, a) = color_write;
        unsafe { gl.color_mask(r as _, g as _, b as _, a as _); }
        self.color_write = color_write;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CachedTexture {
    target: u32,
    texture: glow::Texture,
}

const MAX_SHADERSTAGE_IMAGES: usize = 12;
