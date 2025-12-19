//! Just draws a static triangle with different vertex colors assigned
//! to each corner:
//! * left -- red
//! * right -- green
//! * top -- blue

use glam::vec2;
use miniquad::util::{BasicVertex, GeometryBatcher};
use miniquad::*;
use std::rc::Rc;
use winit::event::WindowEvent;
use winit::window::Window;

fn main() {
    miniquad::run::<Stage>(Conf::default());
}

struct Stage {
    start: Instant,
    pipeline: Pipeline<Meta>,
    batcher: GeometryBatcher<BasicVertex>,
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
        let pipeline = ctx.new_pipeline();

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
                BasicVertex { pos: vec2(-0.5, -0.5) + dr, color: RED },
                BasicVertex { pos: vec2(0.5, -0.5) + dr, color: GREEN },
                BasicVertex { pos: vec2(0.0,  0.5) + dr, color: BLUE },
            ],
            &[0, 1, 2],
        );
        #[rustfmt::skip]
        self.batcher.extend(
            &[
                BasicVertex { pos: vec2(0.0, -0.5), color: RED },
                BasicVertex { pos: vec2(1.0, -0.5), color: GREEN },
                BasicVertex { pos: vec2(0.5 + 0.5 * t.sin(),  1.0), color: BLUE },
            ],
            &[0, 1, 2],
        );
        #[rustfmt::skip]
        self.batcher.extend(
            &[
                BasicVertex { pos: vec2(-1.0, 0.0), color: RED },
                BasicVertex { pos: vec2(-1.0, -1.0), color: GREEN },
                BasicVertex { pos: vec2(0.0,  -1.0), color: BLUE },
            ],
            &[0, 1, 2],
        );
        let num_elements = self.batcher.finish();

        self.ctx.default_pass(Clear::depth_color(BLACK), |_, _| {
            self.ctx.draw(DrawCall {
                pipeline: &self.pipeline,
                base_element: 0,
                num_elements,
                vertex_buffer: &self.batcher.vertices,
                index_buffer: &self.batcher.indicies,
                images: &[],
                uniforms: &util::NoUniforms,
            });
        });
    }
}

pub struct Meta;

impl PipelineMeta for Meta {
    const VERTEX_SHADER: &str = include_str!("shaders/basic_vert.vert");
    const FRAGMENT_SHADER: &str = include_str!("shaders/basic_color.frag");

    const IMAGES_NAMES: &[&str; 0] = &[];
    type Images = [Texture2D; 0];
    type Vertex = miniquad::util::BasicVertex;
    type Uniforms = miniquad::util::NoUniforms;
    const PARAMS: PipelineParams = default_pipeline_params();
}
