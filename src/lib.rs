mod error;
#[cfg(test)]
mod tests;
mod utils;

use std::{fs, path::PathBuf, result};
use std::{io, path::Path};

use error::Operation;
use jwalk::WalkDir;
use rayon::prelude::*;

pub use error::{Error, Result};
use utils::change_dir;

/// helper macro to call asref on all of the identifiers
macro_rules! as_ref_all {
    ( $( $var:ident ),* ) => {
        $( let $var = $var.as_ref(); )*
    };
}

/// Moves a directory from one place to another recursively. Currently is a wrapper around `copy_dir_all` but removes the
/// `from` directory
pub fn move_dir_all(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<u64> {
    as_ref_all!(from, to);

    let copied = copy_dir_all(from, to)?;
    remove_dir_all(from)?;

    Ok(copied)
}

/// Moves a file from one place to another. Currently is a wrapper around `copy` but removes the
/// `from` argument
pub fn move_file(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<u64> {
    as_ref_all!(from, to);

    let amount = copy_create(from, to)?;
    remove_file(from)?;
    Ok(amount)
}

fn check_path_copy_dir_all(path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();

    if !path.exists() {
        return Err(Error::IoExt {
            source: io::Error::new(io::ErrorKind::NotFound, ""),
            path: path.to_path_buf(),
            operation: Operation::CopyDirAll,
        });
    }

    if !path.is_dir() {
        return Err(Error::NotDirectory {
            path: path.to_path_buf(),
        });
    }

    Ok(())
}

/// Recursively copies all contents of the directory to another directory. Will create the new
/// directory if it does not exist
pub fn copy_dir_all(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<u64> {
    as_ref_all!(from, to);

    check_path_copy_dir_all(from)?;

    let walkdir = WalkDir::new(from).skip_hidden(false);

    let mut copied = 0;
    for entry in walkdir {
        let entry = entry?;
        let path = entry.path();
        let new_path = change_dir(from, to, &path)?;

        let file_type = entry.file_type();
        if file_type.is_dir() {
            create_dir(new_path)?;
        } else {
            // the iterator will always iterate over parent directories first so we don't need to
            // use copy_create
            copied += copy(path, new_path)?;
        }
    }

    Ok(copied)
}

// /// par bridge is a little bit wierd, don't use this yet, will deadlock
// pub fn copy_dir_all_par(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<u64> {
//     as_ref_all!(from, to);

//     check_path_copy_dir_all(from)?;

//     WalkDir::new(from)
//         .skip_hidden(false)
//         .into_iter()
//         .par_bridge()
//         .try_for_each(|entry| -> Result<()> {
//             let entry = entry?;
//             let path = entry.path();
//             let new_path = change_dir(from, to, &path)?;

//             let file_type = entry.file_type();
//             if file_type.is_dir() {
//                 println!("creating new dir all {}", new_path.display());
//                 create_dir_all(new_path)?; // if it already exists, will not fail
//             } else {
//                 println!("copying from {} to {}", path.display(), new_path.display());
//                 copy_create(path, new_path)?;
//             }
//             Ok(())
//         })?;

//     Ok(0)
// }

pub fn copy_dir_all_par(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<()> {
    as_ref_all!(from, to);

    check_path_copy_dir_all(from)?;

    let entries = WalkDir::new(from)
        .skip_hidden(false)
        .into_iter()
        .collect::<result::Result<Vec<_>, jwalk::Error>>()
        .map_err(|e| Error::WalkDir { source: e })?; // would like to use par bridge once it doesn't deadlock

    entries
        .into_par_iter()
        .try_for_each(|entry| -> Result<()> {
            let path = entry.path();
            let new_path = change_dir(from, to, &path)?;
            let file_type = entry.file_type();

            if file_type.is_dir() {
                create_dir_all(new_path)?;
            } else {
                // the iterator will always iterate over parent directories first so we don't need to
                // use copy_create
                copy_create(path, new_path)?;
            }
            Ok(())
        })?;
    Ok(())
}

/// A wrapper around `copy` that will also create the parent directories of the file if they do not
/// exist
pub fn copy_create(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<u64> {
    as_ref_all!(from, to);

    if let Some(parent) = to.parent() {
        if !parent.exists() {
            create_dir_all(parent)?;
        }
    }

    copy(from, to)
}

/// A wrapper for the standard library's `copy`. Will fail with a custom error that
/// includes the source error, path, and operation
pub fn copy(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<u64> {
    as_ref_all!(from, to);

    fs::copy(from, to).map_err(|e| Error::IoExtMulti {
        source: e,
        from: from.to_path_buf(),
        to: to.to_path_buf(),
        operation: Operation::Copy,
    })
}

/// A wrapper for the standard library's `remove_file`. Will fail with a custom error that
/// includes the source error, path, and operation
pub fn remove_file(path: impl AsRef<Path>) -> Result<()> {
    as_ref_all!(path);

    fs::remove_file(path).map_err(|e| Error::IoExt {
        source: e,
        path: path.to_path_buf(),
        operation: Operation::Remove,
    })
}

/// A wrapper for the standard library's `remove_dir_all`. Will fail with a custom error that
/// includes the source error, path, and operation
pub fn remove_dir_all(path: impl AsRef<Path>) -> Result<()> {
    as_ref_all!(path);

    fs::remove_dir_all(path).map_err(|e| Error::IoExt {
        source: e,
        path: path.to_path_buf(),
        operation: Operation::RemoveDirAll,
    })
}

/// A wrapper for the standard library's `create_dir_all`. Will fail with a custom error that
/// includes the source error, path, and operation
pub fn create_dir_all(path: impl AsRef<Path>) -> Result<()> {
    as_ref_all!(path);

    fs::create_dir_all(path).map_err(|e| Error::IoExt {
        source: e,
        path: path.to_path_buf(),
        operation: Operation::CreatePathAll,
    })
}

pub fn create_dir(path: impl AsRef<Path>) -> Result<()> {
    as_ref_all!(path);

    fs::create_dir(path).map_err(|e| Error::IoExt {
        source: e,
        path: path.to_path_buf(),
        operation: Operation::Create,
    })
}
