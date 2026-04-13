//! Opens a fullscreen window with green screen.
//! The title of that should be "mimiq".

use mimiq::*;
use std::rc::Rc;
use winit::{
    event::{MouseButton, WindowEvent},
    keyboard::KeyCode,
};

fn main() {
    mimiq::run::<(), App>(Conf::default(), ());
}

struct App {
    ctx: Rc<GlContext>,
    input: util::InputTracker,
}
impl EventHandler<()> for App {
    fn update(&mut self, _dt: Duration) {
        let key = KeyCode::KeyA;
        let button = MouseButton::Left;

        if self.input.is_key_pressed(key) {
            tracing::info!("pressed A");
        }

        if self.input.is_key_held(key) {
            tracing::info!("Held A");
        }

        if self.input.is_key_released(key) {
            tracing::info!("released A");
        }

        if self.input.is_button_pressed(button) {
            tracing::info!("pressed Left Button");
        }

        if self.input.is_button_held(button) {
            tracing::info!("Held Left Button");
        }

        if self.input.is_button_released(button) {
            tracing::info!("released Left Button");
        }

        self.input.update();
    }

    fn window_event(&mut self, event: winit::event::WindowEvent, _window: &winit::window::Window) {
        self.input.handle_event(&event);

        match event {
            WindowEvent::RedrawRequested => self
                .ctx
                .default_pass(Clear::depth_color(Color::BLACK), |_, _| {}),
            _ => (),
        }
    }

    fn init(ctx: Rc<GlContext>, _fs_server: Rc<dyn FsServer>, _init: ()) -> Self {
        App { ctx, input: util::InputTracker::new() }
    }
}
