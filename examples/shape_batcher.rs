//! Draws a bunch of shapes with the shape batcher.

use mimiq::graphics::*;
use mimiq::util::{
    BasicPipelineUniforms, BasicVertex, circle, circle_lines, line, poly_lines, polygon, rect,
    rect_lines, triangle, triangle_lines,
};
use mimiq::*;

use glam::{Mat4, vec2};
use std::rc::Rc;
use winit::{event::WindowEvent, window::Window};

fn main() {
    mimiq::run::<(), App>(Conf::default(), ());
}

struct App {
    total_time: Duration,
    pipeline: util::BasicPipeline,
    batcher: util::GeometryBatcher<BasicVertex>,
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

    fn init(ctx: Rc<GlContext>, _: Rc<audio::AlContext>, _fs: Rc<dyn FsServer>, _init: ()) -> App {
        let batcher = util::GeometryBatcher::new_from_size(&ctx, 20_000, 20_000).unwrap();
        let pipeline = util::new_basic_pipeline(&ctx).unwrap();

        App { pipeline, batcher, ctx, total_time: Duration::ZERO }
    }
}

impl App {
    pub fn draw(&mut self) {
        let t = self.total_time.as_secs_f32();

        self.batcher.clear();

        triangle(
            &mut self.batcher,
            Color::RED,
            vec2(0.0, 100.0),
            vec2(100.0, 100.0),
            vec2(0.0, 0.0),
        );

        triangle(
            &mut self.batcher,
            Color::GREEN,
            vec2(300.0, 200.0),
            vec2(200.0, 200.0),
            vec2(300.0, 300.0),
        );

        line(
            &mut self.batcher,
            Color::CYAN,
            3.0,
            vec2(50.0, 200.0),
            vec2(100.0, 300.0 + 30.0 * t.sin()),
        );

        line(
            &mut self.batcher,
            Color::CYAN,
            1.0,
            vec2(500.0 + 50.0 * (t * 2.0).cos(), 200.0),
            vec2(100.0, 500.0),
        );

        triangle_lines(
            &mut self.batcher,
            Color::CYAN,
            1.0,
            vec2(400.0, 500.0),
            vec2(460.0, 530.0),
            vec2(470.0, 510.0),
        );

        poly_lines(
            &mut self.batcher,
            Color::GREEN,
            2.0,
            vec2(600.0, 600.0),
            t / (0.5 * std::f32::consts::TAU),
            5,
            30.0,
        );

        polygon(
            &mut self.batcher,
            Color::GREEN,
            vec2(650.0, 600.0),
            t / (0.5 * std::f32::consts::TAU),
            6,
            30.0,
        );

        circle_lines(&mut self.batcher, Color::RED, 1.0, vec2(400.0, 600.0), 40.0);

        circle(&mut self.batcher, Color::RED, vec2(540.0, 600.0), 40.0);

        rect(
            &mut self.batcher,
            Color::PURPLE,
            vec2(-50.0, 0.0),
            vec2(100.0, 200.0),
            std::f32::consts::FRAC_PI_2,
        );

        rect(
            &mut self.batcher,
            Color::PURPLE,
            vec2(300.0, 100.0),
            vec2(100.0, 200.0),
            t / (0.5 * std::f32::consts::TAU),
        );

        rect_lines(
            &mut self.batcher,
            Color::PURPLE,
            1.0,
            vec2(100.0, 300.0),
            vec2(50.0, 100.0),
            -t / (0.5 * std::f32::consts::TAU),
        );

        self.ctx
            .default_pass(Clear::depth_color(Color::BLACK), |width, height| {
                let proj =
                    Mat4::orthographic_rh_gl(0.0, width as f32, height as f32, 0.0, 0.0, 1.0);
                let num_elements = self.batcher.flush();

                self.pipeline.draw(
                    0,
                    num_elements,
                    &self.batcher.vertices,
                    &self.batcher.indicies,
                    &NoImages,
                    &BasicPipelineUniforms { view_projection: proj },
                )
            })
            .unwrap();
    }
}
