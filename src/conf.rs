//! Context creation configuration
//!
//! A [`Conf`] struct is used to describe a hardware and platform specific setup,
//! mostly video display settings.
//!

use std::num::NonZeroU32;

use glutin::surface::SwapInterval;
use winit::{dpi::PhysicalSize, window::{Icon, WindowAttributes}};

use crate::default_icon;

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
        default_icon::BIG.to_vec(), 
        64, 
        64,
    ).unwrap();

    WindowAttributes::default()
        .with_inner_size(PhysicalSize::new(800, 600))
        .with_resizable(true)
        .with_title("Miniquad window")
        .with_window_icon(Some(default_icon))
}
