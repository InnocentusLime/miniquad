//! Contains assets that are bundled right inside the executable.

pub static DEFAULT_ICON: &[u8] = include_bytes!("embeded_assets/default_icon.bin");
pub const DEFAULT_ICON_WIDTH: u32 = 64;
pub const DEFAULT_ICON_HEIGHT: u32 = 64;
