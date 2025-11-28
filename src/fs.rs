#[derive(Debug)]
pub enum Error {
    IOError(std::io::Error),
    DownloadFailed,
    AndroidAssetLoadingError,
    /// MainBundle pathForResource returned null
    IOSAssetNoSuchFile,
    /// NSData dataWithContentsOfFile or data.bytes are null
    IOSAssetNoData,
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::IOError(e)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::IOError(e) => write!(f, "I/O error: {e}"),
            Self::DownloadFailed => write!(f, "Download failed"),
            Self::AndroidAssetLoadingError => write!(f, "[android] Failed to load asset"),
            Self::IOSAssetNoSuchFile => write!(f, "[ios] No such asset file"),
            Self::IOSAssetNoData => write!(f, "[ios] No data in asset file"),
        }
    }
}

impl std::error::Error for Error {}

pub type Response = Result<Vec<u8>, Error>;

/// Filesystem path on desktops or HTTP URL in WASM
pub fn load_file<F: Fn(Response) + 'static>(path: &str, on_loaded: F) {
    load_file_desktop(path, on_loaded);
}

fn load_file_desktop<F: Fn(Response)>(path: &str, on_loaded: F) {
    fn load_file_sync(path: &str) -> Response {
        use std::fs::File;
        use std::io::Read;

        let mut response = vec![];
        let mut file = File::open(path)?;
        file.read_to_end(&mut response)?;
        Ok(response)
    }

    let response = load_file_sync(path);

    on_loaded(response);
}
