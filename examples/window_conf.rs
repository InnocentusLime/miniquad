use std::rc::Rc;

///! Opens a fullscreen window with green screen.
///! The title of that should be "miniquad".
use miniquad::*;
use winit::event::WindowEvent;

struct Stage {
    ctx: Rc<GlContext>,
}
impl EventHandler for Stage {
    fn update(&mut self) {}
    
    fn window_event(&mut self, event: winit::event::WindowEvent, _window: &winit::window::Window) {
        if matches!(event, WindowEvent::RedrawRequested) {
            self.ctx
                .perform_default_render_pass(PassAction::clear_color(0., 1.0, 0.0, 1.0), || {});
        }
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
        |ctx| Stage { ctx },
    );
}
