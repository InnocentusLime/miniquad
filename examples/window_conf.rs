//! Opens a fullscreen window with green screen.
//! The title of that should be "miniquad".

use miniquad::*;
use std::rc::Rc;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;

fn main() {
    miniquad::run::<Stage>(Conf {
        window_attributes: default_window_attributes()
            .with_title("My custom window")
            .with_inner_size(PhysicalSize::new(1024, 768)),
        ..Default::default()
    });
}

struct Stage {
    ctx: Rc<GlContext>,
}
impl EventHandler for Stage {
    fn update(&mut self) {}

    fn window_event(&mut self, event: winit::event::WindowEvent, _window: &winit::window::Window) {
        if matches!(event, WindowEvent::RedrawRequested) {
            self.ctx.default_pass(Clear::depth_color(GREEN), |_, _| {});
        }
    }

    fn init(ctx: Rc<GlContext>, _fs_server: FsServerHandle) -> Self {
        Stage { ctx }
    }
}
