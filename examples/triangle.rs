use std::rc::Rc;

use bytemuck::{Pod, Zeroable};
use glam::{vec2, vec4, Vec2, Vec4};
///! Just draws a static triangle with different vertex colors assigned
///! to each corner:
///! * left -- red
///! * right -- green
///! * top -- blue
use miniquad::*;
use winit::{event::WindowEvent, window::Window};

#[repr(C)]
#[derive(Default, Pod, Zeroable, Clone, Copy)]
struct Vertex {
    pos: Vec2,
    color: Vec4,
}

struct Stage {
    pipeline: Pipeline,
    vertices: Buffer<Vertex>,
    indicies: IndexBuffer,
    ctx: Rc<GlContext>,
}

impl Stage {
    pub fn new(ctx: Rc<GlContext>) -> Stage {
        #[rustfmt::skip]
        let vertices = [
            Vertex { pos: vec2(-0.5, -0.5), color: vec4(1., 0., 0., 1.) },
            Vertex { pos: vec2(0.5, -0.5), color: vec4(0., 1., 0., 1.) },
            Vertex { pos: vec2(0.0,  0.5), color: vec4(0., 0., 1., 1.) },
        ];
        let vertices = Buffer::new(ctx.clone(), BufferUsage::Immutable, &vertices);

        let indicies = [0, 1, 2];
        let indicies = IndexBuffer::new(ctx.clone(), BufferUsage::Immutable, &indicies);

        let pipeline = Pipeline::new(
            ctx.clone(),
            shader::VERTEX,
            shader::FRAGMENT,
            shader::meta(),
            PipelineParams::default(),
        )
        .unwrap();

        Stage {
            pipeline,
            indicies,
            vertices,
            ctx,
        }
    }
}

impl EventHandler for Stage {
    fn update(&mut self) {}
    
    fn window_event(&mut self, event: WindowEvent, _window: &Window) {
        if matches!(event, WindowEvent::RedrawRequested) {
            self.ctx
                .perform_default_render_pass(PassAction::default(), || {
                    DrawCall {
                        ctx: &self.ctx,
                        pipeline: &self.pipeline,
                        base_element: 0,
                        num_elements: 3,
                        vertex_buffers: &bind_buffers![
                            (&self.vertices) as <Vertex>::pos,
                            (&self.vertices) as <Vertex>::color,
                        ],
                        index_buffer: &self.indicies,
                        textures: &[],
                        uniform_data: &[],
                    }
                    .execute();
                });
        }
    }
}

fn main() {
    miniquad::start(conf::Conf::default(), Stage::new);
}

mod shader {
    use miniquad::*;

    pub const VERTEX: &str = r#"#version 100
    attribute vec2 in_pos;
    attribute vec4 in_color;

    varying lowp vec4 color;

    void main() {
        gl_Position = vec4(in_pos, 0, 1);
        color = in_color;
    }"#;

    pub const FRAGMENT: &str = r#"#version 100
    varying lowp vec4 color;

    void main() {
        gl_FragColor = color;
    }"#;

    pub fn meta() -> ShaderMeta {
        ShaderMeta {
            images: vec![],
            uniforms: vec![],
            attributes: vec![
                VertexAttribute::new("in_pos", VertexFormat::Float2),
                VertexAttribute::new("in_color", VertexFormat::Float4),
            ],
        }
    }
}
