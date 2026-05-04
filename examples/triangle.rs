//! Just draws a static triangle with different vertex colors assigned
//! to each corner:
//! * left -- red
//! * right -- green
//! * top -- blue

use mimiq::graphics::*;
use mimiq::*;

use bytemuck::{Pod, Zeroable};
use glam::{Vec2, vec2};
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
        let vertices = ctx.new_vertex_buffer(BufferUsage::Immutable, &[
            TriangleVertex { v_pos: vec2(-0.5, -0.5), v_color: Color::RED },
            TriangleVertex { v_pos: vec2(0.5, -0.5), v_color: Color::GREEN },
            TriangleVertex { v_pos: vec2(0.0,  0.5), v_color: Color::BLUE },
        ]).unwrap();
        let indicies = ctx
            .new_index_buffer(BufferUsage::Immutable, &[0, 1, 2])
            .unwrap();
        let pipeline = ctx
            .new_pipeline(
                include_str!("shaders/basic_vert.vert"),
                include_str!("shaders/basic_color.frag"),
                default_pipeline_params(),
            )
            .unwrap();

        App { pipeline, indicies, vertices, ctx }
    }
}

impl App {
    pub fn draw(&mut self) {
        self.ctx
            .default_pass(Clear::depth_color(Color::BLACK), |_, _| {
                self.pipeline
                    .draw(0, 3, &self.vertices, &self.indicies, &NoImages, &NoUniforms)
            })
            .unwrap()
    }
}

#[repr(C)]
#[derive(Debug, Default, Pod, Zeroable, Clone, Copy, Vertex)]
pub struct TriangleVertex {
    pub v_pos: Vec2,
    pub v_color: Color,
}
