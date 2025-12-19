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
    pipeline: Pipeline<Meta>,
    vertices: VertexBuffer<TriangleVertex>,
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
            TriangleVertex { pos: vec2(-0.5, -0.5), color: u8vec4(0xFF, 0, 0, 0xFF) },
            TriangleVertex { pos: vec2(0.5, -0.5), color: u8vec4(0, 0xFF, 0, 0xFF) },
            TriangleVertex { pos: vec2(0.0,  0.5), color: u8vec4(0, 0, 0xFF, 0xFF) },
        ];
        let vertices = ctx.new_vertex_buffer(BufferUsage::Immutable, &vertices);

        let indicies = [0, 1, 2];
        let indicies = ctx.new_index_buffer(BufferUsage::Immutable, &indicies);
        let pipeline = ctx.new_pipeline();

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
    pub color: U8Vec4,
}

impl Vertex for TriangleVertex {
    const LAYOUT: &'static [VertexField] = &[
        attribute_of!(TriangleVertex, pos),
        attribute_of!(TriangleVertex, color),
    ];
}

pub struct Meta;

impl PipelineMeta for Meta {
    const VERTEX_SHADER: &str = include_str!("shaders/basic_coloru8.vert");
    const FRAGMENT_SHADER: &str = include_str!("shaders/basic_color.frag");

    const IMAGES_NAMES: () = ();
    type Images = NoImages;
    type Vertex = TriangleVertex;
    type Uniforms = NoUniforms;
    const PARAMS: PipelineParams = default_pipeline_params();
}
