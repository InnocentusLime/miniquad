///! Opens a fullscreen window with green screen.
///! The title of that should be "miniquad".
use miniquad::*;

struct Stage {
    ctx: GlContext,
}
impl EventHandler for Stage {
    fn update(&mut self) {}

    fn draw(&mut self) {
        self.ctx
            .perform_default_render_pass(PassAction::clear_color(0., 1.0, 0.0, 1.0), || {});
    }
}

fn main() {
    miniquad::start(
        conf::Conf {
            window_title: "Miniquad".to_string(),
            window_width: 1024,
            window_height: 768,
            fullscreen: true,
            ..Default::default()
        },
        || {
            Box::new(Stage {
                ctx: GlContext::new(),
            })
        },
    );
}
