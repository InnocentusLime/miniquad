use std::rc::Rc;

///! Opens a fullscreen window with green screen.
///! The title of that should be "miniquad".
use miniquad::*;
use winit::{dpi::PhysicalSize, event::WindowEvent};

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
            self.ctx.perform_default_render_pass(
                Clear::clear_depth_color(0., 1.0, 0.0, 1.0),
                |_, _| {},
            );
        }
    }

    fn init(ctx: Rc<GlContext>, _fs_server: FsServerHandle) -> Self {
        Stage { ctx }
    }
}
