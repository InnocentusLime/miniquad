//! Demonstrates how to handle raw winit events.
//! Press some keys and use your mouse to see some logs!

use mimiq::audio::*;
use mimiq::graphics::*;
use mimiq::*;

use std::path::Path;
use std::rc::Rc;
use winit::event::{ElementState, MouseButton, WindowEvent};

fn main() {
    mimiq::run::<(), App>(Conf { fs_root: "examples/".into(), ..Conf::default() }, ());
}

struct App {
    playback1: Option<Playback>,
    playback2: Option<Playback>,
    al_ctx: Rc<AlContext>,
    gl_ctx: Rc<GlContext>,
}
impl EventHandler<()> for App {
    fn file_ready(&mut self, event: FileReady) {
        let res = event.bytes_result.unwrap();
        let reader = std::io::Cursor::new(&res);
        let wav = hound::WavReader::new(reader).unwrap();
        let buff = self.al_ctx.new_buffer(wav).unwrap();
        let play = self.al_ctx.new_playback(&buff, false, 1.0).unwrap();

        if event.path.as_path() == "assets/hitHurt.wav" {
            self.playback1 = Some(play);
        } else if event.path.as_path() == "assets/boom.wav" {
            self.playback2 = Some(play);
        }
    }

    fn update(&mut self, _dt: Duration) {}

    fn window_event(&mut self, event: winit::event::WindowEvent, _window: &winit::window::Window) {
        match event {
            WindowEvent::RedrawRequested => self
                .gl_ctx
                .default_pass(Clear::depth_color(Color::GREEN), |_, _| Ok(()))
                .unwrap(),
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                if let Some(play) = &self.playback1 {
                    play.restart();
                }
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Right,
                ..
            } => {
                if let Some(play) = &self.playback2 {
                    play.restart();
                }
            }
            _ => (),
        }
    }

    fn init(
        gl_ctx: Rc<GlContext>,
        al_ctx: Rc<audio::AlContext>,
        fs_server: Rc<dyn FsServer>,
        _init: (),
    ) -> Self {
        fs_server.load_file(Path::new("assets/hitHurt.wav"));
        fs_server.load_file(Path::new("assets/boom.wav"));
        App { gl_ctx, al_ctx, playback1: None, playback2: None }
    }
}
