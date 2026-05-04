//! Draws the same triangle as the `triangle` example, but
//! using the byte based colors.

use mimiq::graphics::*;
use mimiq::*;

use bytemuck::{Pod, Zeroable};
use glam::{U8Vec4, Vec2, u8vec4, vec2};
use std::rc::Rc;
use winit::event::WindowEvent;
use winit::window::Window;

fn main() {
    mimiq::run::<(), App>(Conf::default(), ());
}

struct App {
    pipeline: Pipeline<TriangleVertex>,
    vertices: VertexBuffer<TriangleVertex>,
    indicies: IndexBuffer,
    ctx: Rc<GlContext>,
}

impl EventHandler<()> for App {
    fn update(&mut self, _dt: Duration) {}

    fn window_event(&mut self, event: WindowEvent, _window: &Window) {
        match event {
            WindowEvent::RedrawRequested => self.draw(),
            _ => (),
        }
    }

    fn init(ctx: Rc<GlContext>, _fs: Rc<dyn FsServer>, _init: ()) -> App {
        #[rustfmt::skip]
        let vertices = [
            TriangleVertex { v_pos: vec2(-0.5, -0.5), v_color: u8vec4(0xFF, 0, 0, 0xFF) },
            TriangleVertex { v_pos: vec2(0.5, -0.5), v_color: u8vec4(0, 0xFF, 0, 0xFF) },
            TriangleVertex { v_pos: vec2(0.0,  0.5), v_color: u8vec4(0, 0, 0xFF, 0xFF) },
        ];
        let vertices = ctx
            .new_vertex_buffer(BufferUsage::Immutable, &vertices)
            .unwrap();

        let indicies = [0, 1, 2];
        let indicies = ctx
            .new_index_buffer(BufferUsage::Immutable, &indicies)
            .unwrap();
        let pipeline = ctx
            .new_pipeline(
                include_str!("shaders/basic_coloru8.vert"),
                include_str!("shaders/basic_color.frag"),
                default_pipeline_params(),
            )
            .unwrap();

        App { pipeline, vertices, indicies, ctx }
    }
}

impl App {
    fn draw(&mut self) {
        self.ctx
            .default_pass(Clear::depth_color(Color::BLACK), |_, _| {
                self.pipeline
                    .draw(0, 3, &self.vertices, &self.indicies, &NoImages, &NoUniforms)
            })
            .unwrap();
    }
}

#[repr(C)]
#[derive(Debug, Default, Pod, Zeroable, Clone, Copy, Vertex)]
pub struct TriangleVertex {
    pub v_pos: Vec2,
    pub v_color: U8Vec4,
}
