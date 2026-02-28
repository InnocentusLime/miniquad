//! Just draws a static triangle with different vertex colors assigned
//! to each corner:
//! * left -- red
//! * right -- green
//! * top -- blue

use bytemuck::{Pod, Zeroable};
use glam::{Vec2, vec2};
use mimiq::*;
use std::rc::Rc;
use winit::event::WindowEvent;
use winit::window::Window;

fn main() {
    mimiq::run::<App>(Conf::default());
}

struct App {
    pipeline: Pipeline<Meta>,
    vertices: VertexBuffer<TriangleVertex>,
    indicies: IndexBuffer,
    ctx: Rc<GlContext>,
}

impl EventHandler for App {
    fn update(&mut self, _dt: Duration) {}

    fn window_event(&mut self, event: WindowEvent, _window: &Window) {
        match event {
            WindowEvent::RedrawRequested => self.draw(),
            _ => (),
        }
    }

    fn init(ctx: Rc<GlContext>, _fs: FsServerHandle) -> App {
        #[rustfmt::skip]
        let vertices = ctx.new_vertex_buffer(BufferUsage::Immutable, &[
            TriangleVertex { pos: vec2(-0.5, -0.5), color: RED },
            TriangleVertex { pos: vec2(0.5, -0.5), color: GREEN },
            TriangleVertex { pos: vec2(0.0,  0.5), color: BLUE },
        ]);
        let indicies = ctx.new_index_buffer(BufferUsage::Immutable, &[0, 1, 2]);
        let pipeline = ctx.new_pipeline();

        App { pipeline, indicies, vertices, ctx }
    }
}

impl App {
    pub fn draw(&mut self) {
        self.ctx.default_pass(Clear::depth_color(BLACK), |_, _| {
            self.ctx.draw(DrawCall {
                pipeline: &self.pipeline,
                base_element: 0,
                num_elements: 3,
                vertex_buffer: &self.vertices,
                index_buffer: &self.indicies,
                images: &NoImages,
                uniforms: &NoUniforms,
            });
        });
    }
}

#[repr(C)]
#[derive(Debug, Default, Pod, Zeroable, Clone, Copy)]
pub struct TriangleVertex {
    pub pos: Vec2,
    pub color: Color,
}

impl Vertex for TriangleVertex {
    const LAYOUT: &'static [VertexField] =
        &[attribute_of!(TriangleVertex, pos), attribute_of!(TriangleVertex, color)];
}

pub struct Meta;

impl PipelineMeta for Meta {
    const VERTEX_SHADER: &str = include_str!("shaders/basic_vert.vert");
    const FRAGMENT_SHADER: &str = include_str!("shaders/basic_color.frag");

    const IMAGES_NAMES: () = ();
    type Images = NoImages;
    type Vertex = TriangleVertex;
    type Uniforms = NoUniforms;
    const PARAMS: PipelineParams = default_pipeline_params();
}
