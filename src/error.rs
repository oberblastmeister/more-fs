use std::error;
use std::fmt;
use std::path::StripPrefixError;
use std::{io, path::PathBuf};

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug)]
pub enum Error {
    IoExt {
        source: io::Error,
        path: PathBuf,
        operation: Operation,
    },

    IoExtMulti {
        source: io::Error,
        from: PathBuf,
        to: PathBuf,
        operation: Operation,
    },

    Recover {
        operation: Box<Error>,
        recovery: Box<Error>,
    },

    StripPrefix {
        target: PathBuf,
        strip: PathBuf,
        source: StripPrefixError,
    },

    NotDirectory {
        path: PathBuf,
    },

    WalkDir {
        source: walkdir::Error,
    },
}

#[derive(Debug)]
pub enum Operation {
    Remove,
    RemoveDirAll,
    Create,
    CreatePathAll,
    Move,
    MoveDirAll,
    Copy,
    CopyDirAll,
}

impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operation::Remove => write!(f, "remove"),
            Operation::RemoveDirAll => write!(f, "remove dir all"),
            Operation::Create => write!(f, "create"),
            Operation::CreatePathAll => write!(f, "create path all"),
            Operation::Move => write!(f, "move"),
            Operation::MoveDirAll => write!(f, "move dir all"),
            Operation::Copy => write!(f, "copy"),
            Operation::CopyDirAll => write!(f, "copy dir all"),
        }
    }
}

// impl fmt::Display for MultiOperation {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         match self {
//             MultiOperation::Move => write!(f, "move"),
//             MultiOperation::MoveDirAll => write!(f, "move dir all"),
//             MultiOperation::Copy => write!(f, "copy"),
//             MultiOperation::CopyDirAll => write!(f, "copy dir all"),
//         }
//     }
// }

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::IoExt {
                source,
                path,
                operation,
            } => write!(
                f,
                "A {} on path {} failed: {}",
                source,
                path.display(),
                operation
            ),
            Error::IoExtMulti {
                source,
                from,
                to,
                operation,
            } => write!(
                f,
                "A {} from path {} to path {} failed: {}",
                source,
                from.display(),
                to.display(),
                operation
            ),
            Error::WalkDir { source } => write!(f, "Error walking directory: {}", source),
            Error::StripPrefix {
                target,
                strip,
                source,
            } => write!(
                f,
                "Failed to strip prefix of {} with {}: {}",
                target.display(),
                strip.display(),
                source
            ),
            Error::Recover {
                operation,
                recovery,
            } => write!(
                f,
                "{} Tried to recover but it failed: {}",
                operation, recovery
            ),
            Error::NotDirectory { path } => write!(f, "{} is not a directory", path.display()),
        }
    }
}

impl Error {
    pub fn io_error(&self) -> Option<&io::Error> {
        match self {
            Error::NotDirectory { .. } => None,
            Error::IoExt { source, .. } => Some(source),
            Error::IoExtMulti { source, .. } => Some(source),
            Error::StripPrefix { .. } => None,
            Error::Recover { recovery, .. } => recovery.io_error(),
            Error::WalkDir { source } => source.io_error(),
        }
    }

    pub fn into_io_error(self) -> Option<io::Error> {
        match self {
            Error::NotDirectory { .. } => None,
            Error::IoExt { source, .. } => Some(source),
            Error::IoExtMulti { source, .. } => Some(source),
            Error::StripPrefix { .. } => None,
            Error::Recover { recovery, .. } => recovery.into_io_error(),
            Error::WalkDir { source } => source.into_io_error(),
        }
    }

    pub fn io_error_kind(&self) -> io::ErrorKind {
        let io_error = self.io_error();
        match self.io_error() {
            Some(io_error) => io_error.kind(),
            None => io::ErrorKind::Other,
        }
    }

    pub fn recover(self, recover_fn: impl FnOnce() -> Result<()>) -> Error {
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

impl error::Error for Error {
    fn cause(&self) -> Option<&dyn error::Error> {
        self.source()
    }

    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::NotDirectory { .. } => None,
            Error::IoExt { source, .. } => Some(source),
            Error::IoExtMulti { source, .. } => Some(source),
            Error::StripPrefix { .. } => None,
            Error::Recover { recovery, .. } => recovery.source(),
            Error::WalkDir { source } => Some(source),
        }
    }
}

impl From<walkdir::Error> for Error {
    fn from(walk_dir_err: walkdir::Error) -> Self {
        Error::WalkDir {
            source: walk_dir_err,
        }
    }
}

impl From<Error> for io::Error {
    fn from(more_fs_err: Error) -> io::Error {
        io::Error::new(more_fs_err.io_error_kind(), more_fs_err)
    }
}
