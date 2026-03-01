//! Draws a bunch of shapes with the shape batcher.

use glam::{Mat4, vec2};
use mimiq::*;
use std::rc::Rc;
use winit::{event::WindowEvent, window::Window};

fn main() {
    mimiq::run::<(), App>(Conf::default(), ());
}

struct App {
    total_time: Duration,
    pipeline: Pipeline<util::BasicPipelineMeta>,
    batcher: util::ShapeBatcher,
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

    fn init(ctx: Rc<GlContext>, _fs: FsServerHandle, _init: ()) -> App {
        let batcher = util::ShapeBatcher::new_from_size(&ctx, 20_000, 20_000);
        let pipeline = ctx.new_pipeline();

        App { pipeline, batcher, ctx, total_time: Duration::ZERO }
    }
}

impl App {
    pub fn draw(&mut self) {
        let t = self.total_time.as_secs_f32();

        self.batcher
            .triangle(RED, vec2(0.0, 100.0), vec2(100.0, 100.0), vec2(0.0, 0.0));

        self.batcher.triangle(
            GREEN,
            vec2(300.0, 200.0),
            vec2(200.0, 200.0),
            vec2(300.0, 300.0),
        );

        self.batcher.line(
            CYAN,
            3.0,
            vec2(50.0, 200.0),
            vec2(100.0, 300.0 + 30.0 * t.sin()),
        );

        self.batcher.line(
            CYAN,
            1.0,
            vec2(500.0 + 50.0 * (t * 2.0).cos(), 200.0),
            vec2(100.0, 500.0),
        );

        self.batcher.triangle_lines(
            CYAN,
            1.0,
            vec2(400.0, 500.0),
            vec2(460.0, 530.0),
            vec2(470.0, 510.0),
        );

        self.batcher.poly_lines(
            GREEN,
            2.0,
            vec2(600.0, 600.0),
            t / (0.5 * std::f32::consts::TAU),
            5,
            30.0,
        );

        self.batcher.polygon(
            GREEN,
            vec2(650.0, 600.0),
            t / (0.5 * std::f32::consts::TAU),
            6,
            30.0,
        );

        self.batcher
            .circle_lines(RED, 1.0, vec2(400.0, 600.0), 40.0);

        self.batcher.circle(RED, vec2(540.0, 600.0), 40.0);

        self.batcher.rect(
            PURPLE,
            vec2(-50.0, 0.0),
            vec2(100.0, 200.0),
            std::f32::consts::FRAC_PI_2,
        );

        self.batcher.rect(
            PURPLE,
            vec2(300.0, 100.0),
            vec2(100.0, 200.0),
            t / (0.5 * std::f32::consts::TAU),
        );

        self.batcher.rect_lines(
            PURPLE,
            1.0,
            vec2(100.0, 300.0),
            vec2(50.0, 100.0),
            -t / (0.5 * std::f32::consts::TAU),
        );

        self.ctx
            .default_pass(Clear::depth_color(BLACK), |width, height| {
                let proj =
                    Mat4::orthographic_rh_gl(0.0, width as f32, height as f32, 0.0, 0.0, 1.0);
                self.batcher.basic_draw(&self.ctx, proj, &self.pipeline);
            });
    }
}
