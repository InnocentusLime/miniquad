use bytemuck::{Pod, Zeroable};
use glam::{Vec2, Vec4, vec2, vec4};
use miniquad::*;
///! Just draws a static triangle with different vertex colors assigned
///! to each corner:
///! * left -- red
///! * right -- green
///! * top -- blue
use std::rc::Rc;
use winit::{event::WindowEvent, window::Window};

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
            Vertex { pos: vec2(-0.5, -0.5), color: vec4(1., 0., 0., 1.) },
            Vertex { pos: vec2(0.5, -0.5), color: vec4(0., 1., 0., 1.) },
            Vertex { pos: vec2(0.0,  0.5), color: vec4(0., 0., 1., 1.) },
        ];
        let vertices = ctx.new_vertex_buffer(BufferUsage::Immutable, &vertices);

        let indicies = [0, 1, 2];
        let indicies = ctx.new_index_buffer(BufferUsage::Immutable, &indicies);

        let pipeline = ctx
            .new_pipeline::<&'static str, _>(
                shader::VERTEX,
                shader::FRAGMENT,
                PipelineParams::default(),
                [
                    VertexAttribute::new("in_pos", VertexFormat::F32x2),
                    VertexAttribute::new("in_color", VertexFormat::F32x4),
                ],
                [],
                [],
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

impl Stage {
    pub fn draw(&mut self) {
        self.ctx.perform_default_render_pass(
            PassAction::clear_depth_color(0.0, 0.0, 0.0, 1.0),
            |_, _| {
                self.ctx.submit_drawcall(DrawCall {
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
            },
        );
    }
}

#[repr(C)]
#[derive(Default, Pod, Zeroable, Clone, Copy)]
struct Vertex {
    pos: Vec2,
    color: Vec4,
}

mod shader {
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
}
