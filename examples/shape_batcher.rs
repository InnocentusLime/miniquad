use glam::{Mat4, vec2, vec4};
use miniquad::{
    util::{ShapeBatcher, make_basic_pipeline},
    *,
};
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
    pipeline: Pipeline<util::BasicPipelineUniform>,
    batcher: ShapeBatcher,
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
        let batcher = ShapeBatcher::new_from_size(&ctx, 20_000, 20_000);
        let pipeline = make_basic_pipeline(&ctx);

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

        self.batcher.triangle(
            vec4(1.0, 0.0, 0.0, 1.0),
            vec2(0.0, 100.0),
            vec2(100.0, 100.0),
            vec2(0.0, 0.0),
        );

        self.batcher.triangle(
            vec4(0.0, 1.0, 0.0, 1.0),
            vec2(300.0, 200.0),
            vec2(200.0, 200.0),
            vec2(300.0, 300.0),
        );

        self.batcher.line(
            vec4(0.0, 1.0, 1.0, 1.0),
            3.0,
            vec2(50.0, 200.0),
            vec2(100.0, 300.0 + 30.0 * t.sin()),
        );

        self.batcher.line(
            vec4(0.0, 1.0, 1.0, 1.0),
            1.0,
            vec2(500.0 + 50.0 * (t * 2.0).cos(), 200.0),
            vec2(100.0, 500.0),
        );

        self.batcher.triangle_lines(
            vec4(0.0, 1.0, 1.0, 1.0),
            1.0,
            vec2(400.0, 500.0),
            vec2(460.0, 530.0),
            vec2(470.0, 510.0),
        );

        self.batcher.poly_lines(
            vec4(0.0, 1.0, 0.0, 1.0),
            2.0,
            vec2(600.0, 600.0),
            t / (0.5 * std::f32::consts::TAU),
            5,
            30.0,
        );

        self.batcher
            .circle_lines(vec4(1.0, 0.0, 0.0, 1.0), 1.0, vec2(400.0, 600.0), 40.0);

        self.batcher.rect(
            vec4(1.0, 0.0, 1.0, 1.0),
            vec2(-50.0, 0.0),
            vec2(100.0, 200.0),
            std::f32::consts::FRAC_PI_2,
        );

        self.batcher.rect(
            vec4(1.0, 0.0, 1.0, 1.0),
            vec2(300.0, 100.0),
            vec2(100.0, 200.0),
            t / (0.5 * std::f32::consts::TAU),
        );

        self.batcher.rect_lines(
            vec4(1.0, 0.0, 1.0, 1.0),
            1.0,
            vec2(100.0, 300.0),
            vec2(50.0, 100.0),
            -t / (0.5 * std::f32::consts::TAU),
        );

        self.ctx.perform_default_render_pass(
            Clear::clear_depth_color(0.0, 0.0, 0.0, 1.0),
            |width, height| {
                let proj =
                    Mat4::orthographic_rh_gl(0.0, width as f32, height as f32, 0.0, 0.0, 1.0);
                self.batcher.basic_draw(&self.ctx, proj, &self.pipeline);
            },
        );
    }
}
