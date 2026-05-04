use std::error::Error as StdError;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    AudioBackendUnavailable,
    OutOfSoundBufferCapacity,
    OutOfPlaybackCapacity,
    UnexpectedBitdepth { found: u16 },
    UnexpectedChannelCount { found: u16 },
    UnexpectedWAVEOF,
    WavParsingError(hound::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::AudioBackendUnavailable => write!(f, "Audio backend is unavailable"),
            Error::OutOfSoundBufferCapacity => {
                write!(f, "Out of sound buffer capacity")
            }
            Error::OutOfPlaybackCapacity => {
                write!(f, "Out of sound playback capacity")
            }
            Error::UnexpectedBitdepth { found } => {
                write!(f, "Unexpected bit depth: expected 1 or 4, found {found}",)
            }
            Error::UnexpectedChannelCount { found } => write!(
                f,
                "Unexpected channel count: expected 1 or 2, found {found}",
            ),
            Error::UnexpectedWAVEOF => write!(f, "Unexpected WAV EOF"),
            Error::WavParsingError(_) => write!(f, "Error while parsing the WAV file"),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::WavParsingError(error) => Some(error),
            _ => None,
        }
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn StdError> {
        self.source()
    }
}
