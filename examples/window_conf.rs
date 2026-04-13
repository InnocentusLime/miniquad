//! Opens a fullscreen window with green screen.
//! The title of that should be "mimiq".

use mimiq::*;
use std::rc::Rc;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;

fn main() {
    mimiq::run::<(), App>(
        Conf {
            window_attributes: default_window_attributes()
                .with_title("My custom window")
                .with_inner_size(PhysicalSize::new(1024, 768)),
            ..Default::default()
        },
        (),
    );
}

struct App {
    ctx: Rc<GlContext>,
}
impl EventHandler<()> for App {
    fn update(&mut self, _dt: Duration) {}

    fn window_event(&mut self, event: winit::event::WindowEvent, _window: &winit::window::Window) {
        if matches!(event, WindowEvent::RedrawRequested) {
            self.ctx
                .default_pass(Clear::depth_color(Color::GREEN), |_, _| {});
        }
    }

    fn init(ctx: Rc<GlContext>, _fs_server: Rc<dyn FsServer>, _init: ()) -> Self {
        App { ctx }
    }
}
