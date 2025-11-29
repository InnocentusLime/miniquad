//! Context creation configuration
//!
//! A [`Conf`] struct is used to describe a hardware and platform specific setup,
//! mostly video display settings.
//!

use std::num::NonZeroU32;

use glutin::surface::SwapInterval;

/// Describes a hardware and platform-specific setup.
#[derive(Debug)]
pub struct Conf {
    /// Window title. Defaults to an empty string.
    pub window_title: String,

    /// Preferred window width (ignored on WASM/Android).
    /// Defaults to `800`.
    pub window_width: i32,

    /// Preferred window height (ignored on WASM/Android).
    /// Defaults to `600`.
    pub window_height: i32,

    /// If `true`, create the window in fullscreen mode (ignored on WASM/Android).
    /// Defaults to `false`.
    pub fullscreen: bool,

    /// If `true`, the user can resize the window.
    pub window_resizable: bool,

    /// Optional icon data used by the OS where applicable:
    /// - On Windows, taskbar/title bar icon
    /// - On macOS, Dock/title bar icon
    /// - TODO: Favicon on HTML5
    /// - TODO: Taskbar/title bar icon on Linux (depends on WM)
    /// - Note: on gnome, icon is determined using `WM_CLASS` (can be set under [`Platform`]) and
    ///   an external `.desktop` file
    pub icon: Option<Icon>,

    /// Optional swap interval (vertical sync).
    ///
    /// Note that this is highly platform- and driver-dependent.
    /// There is no guarantee the FPS will match the specified `swap_interval`.
    /// In other words, `swap_interval` is only a hint to the GPU driver and
    /// not a reliable way to limit the game's FPS.
    pub swap_interval: SwapInterval,

}

/// Icon image in three levels of detail.
#[derive(Clone)]
pub struct Icon {
    /// 16 * 16 image of RGBA pixels (each 4 * u8) in row-major order.
    pub small: [u8; 16 * 16 * 4],
    /// 32 x 32 image of RGBA pixels (each 4 * u8) in row-major order.
    pub medium: [u8; 32 * 32 * 4],
    /// 64 x 64 image of RGBA pixels (each 4 * u8) in row-major order.
    pub big: [u8; 64 * 64 * 4],
}

impl Icon {
    pub fn miniquad_logo() -> Icon {
        Icon {
            small: crate::default_icon::SMALL,
            medium: crate::default_icon::MEDIUM,
            big: crate::default_icon::BIG,
        }
    }
}
// Printing 64x64 array with a default formatter is not meaningfull,
// so debug will skip the data fields of an Icon
impl std::fmt::Debug for Icon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Icon").finish()
    }
}

impl Default for Conf {
    fn default() -> Conf {
        Conf {
            window_title: "".to_owned(),
            window_width: 800,
            window_height: 600,
            fullscreen: false,
            window_resizable: true,
            icon: Some(Icon::miniquad_logo()),
            swap_interval: SwapInterval::Wait(NonZeroU32::new(1).unwrap()),
        }
    }
}
