pub mod error;
mod utils;

use std::fs;
use std::path::Path;

use error::{Error, Result};

pub fn move_file(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<u64> {
    let from = from.as_ref();
    let to = to.as_ref();

    let amount = copy_create(from, to)?;
    remove_file(from)?;
    Ok(amount)
}

pub fn copy_create(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<u64> {
    let from = from.as_ref();
    let to = to.as_ref();

    if let Some(parent) = to.parent() {
        if !parent.exists() {
            create_dir_all(parent)?;
        }
    }

    copy(from, to)
}

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

pub fn remove_dir_all(path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();

    fs::remove_dir_all(path).map_err(|e| Error::IoExt {
        source: e,
        path: path.to_path_buf(),
        operation: String::from("removing all contents of directory"),
    })
}

pub fn create_dir_all(path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();

    fs::create_dir_all(path).map_err(|e| Error::IoExt {
        source: e,
        path: path.to_path_buf(),
        operation: String::from("creating all directories"),
    })
}
