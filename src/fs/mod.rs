#[cfg(not(target_family = "wasm"))]
mod native;
#[cfg(target_family = "wasm")]
mod wasm;

use std::io;
use std::path::{Path, PathBuf};

#[cfg(not(target_family = "wasm"))]
pub(crate) use native::spawn_fs_server;
#[cfg(target_family = "wasm")]
pub(crate) use wasm::spawn_fs_server;

pub trait FsServer {
    fn load_file(&self, path: &Path);
}

#[derive(Debug)]
pub struct FileReady {
    pub path: PathBuf,
    pub bytes_result: io::Result<Vec<u8>>,
}
