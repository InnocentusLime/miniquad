//! Demonstrates how to handle raw winit events.
//! Press some keys and use your mouse to see some logs!

use mimiq::*;
use std::rc::Rc;
use tracing::info;
use winit::event::WindowEvent;

fn main() {
    mimiq::run::<App>(Conf::default());
}

struct App {
    ctx: Rc<GlContext>,
}
impl EventHandler for App {
    fn update(&mut self) {}

    fn window_event(&mut self, event: winit::event::WindowEvent, _window: &winit::window::Window) {
        match event {
            WindowEvent::RedrawRequested => {
                self.ctx.default_pass(Clear::depth_color(GREEN), |_, _| {})
            }
            WindowEvent::KeyboardInput { event, .. } => info!(
                loc=?event.location,
                phys_code=?event.physical_key,
                virt_code=?event.logical_key,
                state=?event.state,
                "key input",
            ),
            WindowEvent::MouseInput { state, button, .. } => info!(
                state=?state,
                button=?button,
                "mouse input",
            ),
            _ => (),
        }
    }

    fn init(ctx: Rc<GlContext>, _fs_server: FsServerHandle) -> Self {
        App { ctx }
    }
}
