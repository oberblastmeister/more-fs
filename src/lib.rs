pub mod error;
mod utils;

use std::fs;
use std::path::Path;

use error::{Error, Result};

pub fn copy(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<u64> {
    let from = from.as_ref();
    let to = to.as_ref();

    fs::copy(from, to).map_err(|e| Error::Copy {
        from: from.to_path_buf(),
        to: to.to_path_buf(),
        source: e,
    })
}

pub fn remove_file(path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();

    fs::remove_file(path).map_err(|e| Error::IoExt {
        source: e,
        path: path.to_path_buf(),
        operation: String::from("removing file"),
    })
}
