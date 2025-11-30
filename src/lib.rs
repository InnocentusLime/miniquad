#![doc = include_str!("../README.md")]

mod embeded_assets;
pub mod fs;
pub mod graphics;
pub mod native;

pub use bytemuck::offset_of;
pub use graphics::*;

use glutin::surface::SwapInterval;
use std::num::NonZeroU32;
use std::rc::Rc;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::window::{Icon, Window, WindowAttributes};

use crate::fs::FsServerHandle;

/// Start miniquad.
pub fn start<F, Handler>(conf: Conf, f: F)
where
    F: 'static + FnOnce(Rc<GlContext>, FsServerHandle) -> Handler,
    Handler: EventHandler,
{
    native::run(conf, f);
}

/// Describes a hardware and platform-specific setup.
#[derive(Debug)]
pub struct Conf {
    pub window_attributes: WindowAttributes,

    /// Optional swap interval (vertical sync).
    ///
    /// Note that this is highly platform- and driver-dependent.
    /// There is no guarantee the FPS will match the specified `swap_interval`.
    /// In other words, `swap_interval` is only a hint to the GPU driver and
    /// not a reliable way to limit the game's FPS.
    pub swap_interval: SwapInterval,
}

impl Default for Conf {
    fn default() -> Conf {
        Conf {
            window_attributes: default_window_attributes(),
            swap_interval: SwapInterval::Wait(NonZeroU32::new(1).unwrap()),
        }
    }
}

pub fn default_window_attributes() -> WindowAttributes {
    let default_icon = Icon::from_rgba(
        embeded_assets::DEFAULT_ICON.to_vec(),
        embeded_assets::DEFAULT_ICON_WIDTH,
        embeded_assets::DEFAULT_ICON_HEIGHT,
    )
    .unwrap();

    WindowAttributes::default()
        .with_inner_size(PhysicalSize::new(800, 600))
        .with_resizable(true)
        .with_title("Miniquad window")
        .with_window_icon(Some(default_icon))
}

/// A trait defining event callbacks.
pub trait EventHandler {
    /// On most platforms update() and draw() are called each frame, sequentially,
    /// draw right after update.
    /// But on Android (and maybe some other platforms in the future) update might
    /// be called without draw.
    /// When the app is in background, Android destroys the rendering surface,
    /// while app is still alive and can do some usefull calculations.
    /// Note that in this case drawing from update may lead to crashes.
    fn update(&mut self);

    fn window_event(&mut self, event: WindowEvent, window: &Window);
}

pub mod date {
    pub fn now() -> f64 {
        use std::time::SystemTime;

        let time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_else(|e| panic!("{}", e));
        time.as_secs_f64()
    }
}
