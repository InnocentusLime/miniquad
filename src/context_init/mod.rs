#[cfg(not(target_family = "wasm"))]
mod native;
#[cfg(target_family = "wasm")]
mod wasm;

#[cfg(not(target_family = "wasm"))]
pub use native::*;
#[cfg(target_family = "wasm")]
pub use wasm::*;

use winit::dpi::PhysicalSize;

#[derive(Debug, Clone, Copy)]
pub struct NewSize(pub PhysicalSize<u32>);
