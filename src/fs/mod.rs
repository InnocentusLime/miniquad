#[cfg(not(target_family = "wasm"))]
mod native;
#[cfg(target_family = "wasm")]
mod wasm;

static TARGET_NAME: &str = "fs_loader";

#[cfg(not(target_family = "wasm"))]
pub use native::*;
#[cfg(target_family = "wasm")]
pub use wasm::*;

#[derive(Debug)]
pub struct FileReady {
    pub user_id: u64,
    pub bytes_result: anyhow::Result<Vec<u8>>,
}
