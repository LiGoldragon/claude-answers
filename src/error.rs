use std::path::PathBuf;

use thiserror::Error;

/// Everything that can go wrong resolving a query or reading a transcript.
#[derive(Debug, Error)]
pub enum Error {
    #[error("i/o: {0}")]
    Io(#[from] std::io::Error),

    #[error("HOME environment variable is not set")]
    HomeNotSet,

    #[error("no transcripts found under {}", .0.display())]
    NoTranscripts(PathBuf),

    #[error("no session matching {fragment:?} under {}", .directory.display())]
    SessionNotFound {
        fragment: String,
        directory: PathBuf,
    },

    #[error("invalid NOTA argument: {0}")]
    Argument(#[from] nota::NotaDecodeError),
}

pub type Result<T> = std::result::Result<T, Error>;
