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
    blend: Blending,
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
            blend: Blending::None,
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

    pub fn set_blend(&mut self, gl: &glow::Context, blend: Blending) {
        if self.blend == blend {
            return;
        }
        match blend {
            Blending::None => unsafe {
                gl.disable(glow::BLEND);
            },
            Blending::All(all) => unsafe {
                if self.blend == Blending::None {
                    gl.enable(glow::BLEND);
                }
                gl.blend_func(all.source.into(), all.dest.into());
                gl.blend_equation(all.equation.into());
            },
            Blending::Separate { color, alpha } => unsafe {
                if self.blend == Blending::None {
                    gl.enable(glow::BLEND);
                }
                gl.blend_func_separate(
                    color.source.into(),
                    color.dest.into(),
                    alpha.source.into(),
                    alpha.dest.into(),
                );
                gl.blend_equation_separate(color.equation.into(), alpha.equation.into());
            },
        }
        self.blend = blend;
    }

    pub fn set_stencil(&mut self, gl: &glow::Context, stencil_test: Option<StencilState>) {
        if self.stencil == stencil_test {
            return;
        }
        let Some(stencil) = stencil_test else {
            unsafe {
                gl.disable(glow::STENCIL_TEST);
            }
            return;
        };
        unsafe {
            if self.stencil.is_none() {
                gl.enable(glow::STENCIL_TEST);
            }

            gl.stencil_op_separate(
                glow::FRONT,
                stencil.front.fail_op.into(),
                stencil.front.depth_fail_op.into(),
                stencil.front.pass_op.into(),
            );
            gl.stencil_func_separate(
                glow::FRONT,
                stencil.front.test_func.into(),
                stencil.front.test_ref,
                stencil.front.test_mask,
            );
            gl.stencil_mask_separate(glow::FRONT, stencil.front.write_mask);

            gl.stencil_op_separate(
                glow::BACK,
                stencil.back.fail_op.into(),
                stencil.back.depth_fail_op.into(),
                stencil.back.pass_op.into(),
            );
            gl.stencil_func_separate(
                glow::BACK,
                stencil.back.test_func.into(),
                stencil.back.test_ref,
                stencil.back.test_mask,
            );
            gl.stencil_mask_separate(glow::BACK, stencil.back.write_mask);
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
            (Some(_), Some(cmp)) => unsafe { gl.depth_func(cmp.into()) },
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
        unsafe {
            gl.color_mask(r as _, g as _, b as _, a as _);
        }
        self.color_write = color_write;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CachedTexture {
    target: u32,
    texture: glow::Texture,
}

const MAX_SHADERSTAGE_IMAGES: usize = 12;
