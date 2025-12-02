use std::rc::Rc;

///! Opens a fullscreen window with green screen.
///! The title of that should be "miniquad".
use miniquad::*;
use tracing::info;
use winit::event::WindowEvent;

fn main() {
    miniquad::run::<Stage>(Conf::default());
}

struct Stage {
    ctx: Rc<GlContext>,
}
impl EventHandler for Stage {
    fn update(&mut self) {}

    fn window_event(&mut self, event: winit::event::WindowEvent, _window: &winit::window::Window) {
        match event {
            WindowEvent::RedrawRequested => self.ctx.perform_default_render_pass(
                PassAction::clear_depth_color(0., 1.0, 0.0, 1.0),
                || {},
            ),
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
        Stage { ctx }
    }
}
