//! Draws the same triangle as the `triangle` example, but
//! using the byte based colors.

use bytemuck::{Pod, Zeroable};
use glam::{U8Vec4, Vec2, u8vec4, vec2};
use miniquad::*;
use std::rc::Rc;
use winit::event::WindowEvent;
use winit::window::Window;

fn main() {
    miniquad::run::<Stage>(Conf::default());
}

struct Stage {
    pipeline: Pipeline<util::NoUniforms>,
    vertices: VertexBuffer<Vertex>,
    indicies: IndexBuffer,
    ctx: Rc<GlContext>,
}

impl EventHandler for Stage {
    fn update(&mut self) {}

    fn window_event(&mut self, event: WindowEvent, _window: &Window) {
        match event {
            WindowEvent::RedrawRequested => self.draw(),
            _ => (),
        }
    }

    fn init(ctx: Rc<GlContext>, _fs: FsServerHandle) -> Stage {
        #[rustfmt::skip]
        let vertices = [
            Vertex { pos: vec2(-0.5, -0.5), color: u8vec4(0xFF, 0, 0, 0xFF) },
            Vertex { pos: vec2(0.5, -0.5), color: u8vec4(0, 0xFF, 0, 0xFF) },
            Vertex { pos: vec2(0.0,  0.5), color: u8vec4(0, 0, 0xFF, 0xFF) },
        ];
        let vertices = ctx.new_vertex_buffer(BufferUsage::Immutable, &vertices);

        let indicies = [0, 1, 2];
        let indicies = ctx.new_index_buffer(BufferUsage::Immutable, &indicies);

        let pipeline = ctx
            .new_pipeline(
                shader::VERTEX,
                shader::FRAGMENT,
                PipelineParams::default(),
                [
                    Attribute::new("in_pos", VertexFormat::F32x2),
                    Attribute::new("in_color", VertexFormat::U8x4),
                ],
                [],
            )
            .unwrap();

        Stage {
            pipeline,
            vertices,
            indicies,
            ctx,
        }
    }
}

impl Stage {
    fn draw(&mut self) {
        self.ctx.default_pass(Clear::depth_color(BLACK), |_, _| {
            self.ctx.draw(DrawCall {
                pipeline: &self.pipeline,
                base_element: 0,
                num_elements: 3,
                vertex_buffers: &bind_vertex_buffers![
                    (&self.vertices) as <Vertex>::pos,
                    (&self.vertices) as <Vertex>::color,
                ],
                index_buffer: self.indicies.bind(),
                textures: &[],
                uniforms: &util::NoUniforms,
            });
        });
    }
}

#[repr(C)]
#[derive(Default, Pod, Zeroable, Clone, Copy)]
struct Vertex {
    pos: Vec2,
    color: U8Vec4,
}

mod shader {
    pub const VERTEX: &str = r#"#version 150
    in vec2 in_pos;
    in lowp uvec4 in_color;

    out lowp vec4 color;

    void main() {
        gl_Position = vec4(in_pos, 0, 1);
        color = vec4(in_color) / 255.0;
    }"#;

    pub const FRAGMENT: &str = r#"#version 150
    in lowp vec4 color;
    out vec4 frag_color;

    void main() {
        frag_color = color;
    }"#;
}
