use std::path::StripPrefixError;
use std::{io, path::PathBuf};

use thiserror::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{operation} with path {path} failed with io error: {source}")]
    IoExt {
        source: io::Error,
        path: PathBuf,
        operation: String,
    },

    #[error("The path {path} does not exist")]
    DoesNotExist { path: PathBuf },

    #[error("The path {path} is not a directory")]
    NotDirectory { path: PathBuf },

    #[error("{operation} Tried to recover but it failed: {recovery}")]
    Recover {
        operation: Box<Error>,
        recovery: Box<Error>,
    },

    #[error("Failed to copy from {from} to {to}: {source}")]
    Copy {
        from: PathBuf,
        to: PathBuf,
        source: io::Error,
    },

    #[error("Failed to strip the prefix of {target} with {strip}: {source}")]
    StripPrefix {
        target: PathBuf,
        strip: PathBuf,
        source: StripPrefixError,
    },

    #[error("Walk directory error: {source}")]
    Walkdir {
        #[from]
        source: jwalk::Error,
    },
}

impl Error {
    fn recover(self, recover_fn: impl Fn() -> Result<()>) -> Error {
        let res = recover_fn();
        match res {
            Ok(()) => self,
            Err(e) => Error::Recover {
                operation: Box::new(self),
                recovery: Box::new(e),
            },
        }
    }
}
