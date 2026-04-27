//! Just draws a static triangle with different vertex colors assigned
//! to each corner:
//! * left -- red
//! * right -- green
//! * top -- blue

use glam::vec2;
use mimiq::util::{BasicVertex, GeometryBatcher};
use mimiq::*;
use std::rc::Rc;
use winit::event::WindowEvent;
use winit::window::Window;

fn main() {
    mimiq::run::<(), App>(Conf::default(), ());
}

struct App {
    total_time: Duration,
    pipeline: Pipeline<Meta>,
    batcher: GeometryBatcher<BasicVertex>,
    ctx: Rc<GlContext>,
}

impl EventHandler<()> for App {
    fn update(&mut self, dt: Duration) {
        self.total_time += dt;
    }

    fn window_event(&mut self, event: WindowEvent, _window: &Window) {
        match event {
            WindowEvent::RedrawRequested => self.draw(),
            _ => (),
        }
    }

    fn init(ctx: Rc<GlContext>, _fs: Rc<dyn FsServer>, _init: ()) -> App {
        let batcher = GeometryBatcher::new_from_size(&ctx, 50, 50);
        let pipeline = ctx.new_pipeline();

        App { pipeline, batcher, ctx, total_time: Duration::ZERO }
    }
}

impl App {
    pub fn draw(&mut self) {
        let t = self.total_time.as_secs_f32();
        let dr = vec2(t.cos(), t.sin());

        self.batcher.clear();

        #[rustfmt::skip]
        self.batcher.extend(
            &[
                BasicVertex { v_pos: vec2(-0.5, -0.5) + dr, v_color: Color::RED },
                BasicVertex { v_pos: vec2(0.5, -0.5) + dr, v_color: Color::GREEN },
                BasicVertex { v_pos: vec2(0.0,  0.5) + dr, v_color: Color::BLUE },
            ],
            &[0, 1, 2],
        );
        #[rustfmt::skip]
        self.batcher.extend(
            &[
                BasicVertex { v_pos: vec2(0.0, -0.5), v_color: Color::RED },
                BasicVertex { v_pos: vec2(1.0, -0.5), v_color: Color::GREEN },
                BasicVertex { v_pos: vec2(0.5 + 0.5 * t.sin(),  1.0), v_color: Color::BLUE },
            ],
            &[0, 1, 2],
        );
        #[rustfmt::skip]
        self.batcher.extend(
            &[
                BasicVertex { v_pos: vec2(-1.0, 0.0), v_color: Color::RED },
                BasicVertex { v_pos: vec2(-1.0, -1.0), v_color: Color::GREEN },
                BasicVertex { v_pos: vec2(0.0,  -1.0), v_color: Color::BLUE },
            ],
            &[0, 1, 2],
        );
        let num_elements = self.batcher.flush();

        self.ctx
            .default_pass(Clear::depth_color(Color::BLACK), |_, _| {
                self.ctx.draw(DrawCall {
                    pipeline: &self.pipeline,
                    base_element: 0,
                    num_elements,
                    vertex_buffer: &self.batcher.vertices,
                    index_buffer: &self.batcher.indicies,
                    images: &NoImages,
                    uniforms: &NoUniforms,
                });
            });
    }
}

pub struct Meta;

impl PipelineMeta for Meta {
    const VERTEX_SHADER: &str = include_str!("shaders/basic_vert.vert");
    const FRAGMENT_SHADER: &str = include_str!("shaders/basic_color.frag");

    const IMAGES_NAMES: () = ();
    type Images = NoImages;
    type Vertex = util::BasicVertex;
    type Uniforms = NoUniforms;
    const PARAMS: PipelineParams = default_pipeline_params();
}
