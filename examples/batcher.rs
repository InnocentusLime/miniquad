use bytemuck::{Pod, Zeroable};
use glam::{Vec2, Vec4, vec2, vec4};
use miniquad::{util::GeometryBatcher, *};
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
    start: Instant,
    pipeline: Pipeline<util::NoUniforms>,
    batcher: GeometryBatcher<Vertex>,
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
        let batcher = GeometryBatcher::new_from_size(&ctx, 50, 50);
        let pipeline = ctx
            .new_pipeline(
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
            batcher,
            ctx,
            start: Instant::now(),
        }
    }
}

impl Stage {
    pub fn draw(&mut self) {
        let t = Instant::now().duration_since(self.start).as_secs_f32();
        let dr = vec2(t.cos(), t.sin());

        #[rustfmt::skip]
        self.batcher.extend(
            &[
                Vertex { pos: vec2(-0.5, -0.5) + dr, color: vec4(1., 0., 0., 1.) },
                Vertex { pos: vec2(0.5, -0.5) + dr, color: vec4(0., 1., 0., 1.) },
                Vertex { pos: vec2(0.0,  0.5) + dr, color: vec4(0., 0., 1., 1.) },
            ],
            &[0, 1, 2],
        );
        #[rustfmt::skip]
        self.batcher.extend(
            &[
                Vertex { pos: vec2(0.0, -0.5), color: vec4(1., 0., 0., 1.) },
                Vertex { pos: vec2(1.0, -0.5), color: vec4(0., 1., 0., 1.) },
                Vertex { pos: vec2(0.5 + 0.5 * t.sin(),  1.0), color: vec4(0., 0., 1., 1.) },
            ],
            &[0, 1, 2],
        );
        #[rustfmt::skip]
        self.batcher.extend(
            &[
                Vertex { pos: vec2(-1.0, 0.0), color: vec4(1., 0., 0., 1.) },
                Vertex { pos: vec2(-1.0, -1.0), color: vec4(0., 1., 0., 1.) },
                Vertex { pos: vec2(0.0,  -1.0), color: vec4(0., 0., 1., 1.) },
            ],
            &[0, 1, 2],
        );
        let num_elements = self.batcher.finish();

        self.ctx
            .perform_default_render_pass(Clear::depth_color(BLACK), |_, _| {
                self.ctx.submit_drawcall(DrawCall {
                    pipeline: &self.pipeline,
                    base_element: 0,
                    num_elements,
                    vertex_buffers: &bind_vertex_buffers![
                        (&self.batcher.vertices) as <Vertex>::pos,
                        (&self.batcher.vertices) as <Vertex>::color,
                    ],
                    index_buffer: self.batcher.indicies.bind(),
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
