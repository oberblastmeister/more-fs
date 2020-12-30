/*!
`more_fs` trys to be the fastest library of convenient functions for filesystem operations.
The crate tries to mirror the standard library api to make it easy to use.

To use this crate, add the more-fs dependency to you  project's `Cargo.toml`:

```ignore
[dependencies]
more_fs = "2"
```

# New functions

This crate adds some new functions that are not in the standard library.
These new functions are [`copy_dir_all`] and [`move_dir_all`].
Copying can be done concurrently using [`rayon`] with the `rayon` feature flag (enabled by default).
Enabling the flag enables the functions [`copy_dir_all_par`] and [`move_dir_all_par`] that are the 
same as the prior functions but do things concurrently

# Standard library functions

This crate also includes wrappers for the standard library functions.
They do the same thing, but have much errors with more context such as
the path that triggered the error or the operation that was done to
trigger the error. Checkout the [`Error`] type to learn more about the errors.

# Example

This code will recursively move a directory to a new directory, similar to `mv` behavior.

```no_run
use more_fs::move_dir_all;

move_dir_all("starting_directory", "moved_directory").unwrap();
```

If you want to move files, you can do this

```no_run
use more_fs::move_file;

move_file("starting file", "moved_file").unwrap();
```

Copying files or directories works the same way:

```no_run
use more_fs::copy;

copy("starting file", "copied_file").unwrap();
```

# Advanced error handling

Copying or moving a whole directory can fail in between the process.
Lets say that you have a directory called `from_directory` that has two files in it, `file1` and `file2`.
You are trying recursively copy it to `to_directory`.

```no_run
use more_fs::copy_dir_all_par;

// from_directory contains from_directory/file1
// and from_directory/file2
copy_dir_all_par("from_directory", "to_directory").unwrap()
```

more_fs will first create `to_directory` and copy `from_directory/file1` to `to_directory/file1`.
Now it will try to copy `from_directory/file2` to `to_directory/file2`.
If it fails, `to_directory/file1` will still exist.
If we want to recovery from this operation, we can do this

```no_run
use more_fs::{remove_dir_all, copy_dir_all_par};

// if copying from_directory fails, we can remove the left over artifact by using recover
copy_dir_all_par("from_directory", "to_directory").map_err(|e| {
    e.recover(|| remove_dir_all("to_directory"))
}).unwrap();
```

[`Error::recover`] takes a closure or a function that it will run to recover from the error given.
If that functions succeeds, it will return the first error, the one that recover was called on.
If the function fails, it will return a new error of type [`Error::Recover`] that will include both errors inside of a [`Box`].
Combining this with map error is very useful because this way recover will only execute when there is an error and will
have the option to map the error to a [`Error::Recover`]. For more information check out [`Error::Recover`] or [`Error::recover`].

# Performance

`more_fs` is benchmarked using the wonderful [`criterion`] library.
Compared to [`fs_extra`] all of the `more_fs` recursive functions are faster whether they are concurrent or not.
The difference between the single threaded functions from [`fs_extra`] and `more_fs` is smaller when the directories are smaller.
The concurrent [`copy_dir_all_par`] always pulls ahead. On the rust source code repo single threaded more_fs performs at an average of 600ms
while fs_extra performs at around 760ms. The more_fs is faster but not by that much. The multi-threaded outperforms
both with an average time of 370 ms. You can run `$ cargo bench` to find out more about the benchmarks and get nice plots.
*Note*: they will take a while because they run git clone on the rust github repo which can take a while.

In short, use [`copy_dir_all_par`] or [`move_dir_all_par`] whenever you can because it will be faster across the board.
If you don't want to because your directories are extremely small or you don't want to pull in many dependencies from [`rayon`],
you can turn the `rayon` feature flag off.

[`criterion`]: https://docs.rs/criterion
[`fs_extra`]: https://docs.rs/fs_extra

*/

mod error;
#[cfg(test)]
mod tests;
mod utils;

use std::{fs, path::PathBuf, result};
use std::{io, path::Path};

use error::Operation;
#[cfg(feature = "rayon")]
use rayon::prelude::*;
use walkdir::WalkDir;

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

/// Moves a directory from one place to another recursively in parallel. Currently is a wrapper around `copy_dir_all` but removes the
/// `from` directory
pub fn move_dir_all_par(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<()> {
    as_ref_all!(from, to);

    copy_dir_all_par(from, to)?;
    remove_dir_all(from)?;

    Ok(())
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

fn copy_or_create(
    file_type: fs::FileType,
    from: impl AsRef<Path>,
    to: impl AsRef<Path>,
) -> Result<u64> {
    let amount = if file_type.is_dir() {
        create_dir(to)?;
        0
    } else {
        // the iterator will always iterate over parent directories first so we don't need to
        // use copy_create
        copy(from, to)?
    };
    Ok(amount)
}

/// Recursively copies all contents of the directory to another directory. Will create the new
/// directory if it does not exist
pub fn copy_dir_all(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<u64> {
    as_ref_all!(from, to);

    check_path_copy_dir_all(from)?;

    let walkdir = WalkDir::new(from);

    let mut copied = 0;
    for entry in walkdir {
        let entry = entry?;
        let path = entry.path();
        let new_path = change_dir(from, to, &path)?;

        copied += copy_or_create(entry.file_type(), path, new_path)?;
    }

    Ok(copied)
}

#[cfg(feature = "rayon")]
fn copy_or_create_par(
    file_type: fs::FileType,
    from: impl AsRef<Path>,
    to: impl AsRef<Path>,
) -> Result<()> {
    if file_type.is_dir() {
        create_dir_all(to)?;
    } else {
        // the iterator will always iterate over parent directories first so we don't need to
        // use copy_create
        copy_create(from, to)?;
    }
    Ok(())
}

/// Recursively copies all contents of the directory to another directory in parallel. Will create the new
/// directory if it does not exist.
#[cfg(feature = "rayon")]
pub fn copy_dir_all_par(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<()> {
    as_ref_all!(from, to);

    check_path_copy_dir_all(from)?;

    WalkDir::new(from)
        .into_iter()
        .par_bridge()
        .try_for_each(|entry| -> Result<()> {
            let entry = entry?;
            let path = entry.path();
            let new_path = change_dir(from, to, &path)?;
            let file_type = entry.file_type();

            copy_or_create_par(file_type, path, new_path)?;

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

/// A wrapper for the standard library's [`fs::create_dir_all`]. Will fail with a custom error that
/// includes the source error, path, and operation. Checkout [`fs::create_dir`] to see the
/// differences between this function and [`create_dir`]
pub fn create_dir_all(path: impl AsRef<Path>) -> Result<()> {
    as_ref_all!(path);

    fs::create_dir_all(path).map_err(|e| Error::IoExt {
        source: e,
        path: path.to_path_buf(),
        operation: Operation::CreatePathAll,
    })
}

/// A wrapper for the standard library's [`fs::create_dir`]. Will fail with a custom error that
/// includes the source error, path, and operation
pub fn create_dir(path: impl AsRef<Path>) -> Result<()> {
    as_ref_all!(path);

    fs::create_dir(path).map_err(|e| Error::IoExt {
        source: e,
        path: path.to_path_buf(),
        operation: Operation::Create,
    })
}
