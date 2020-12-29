use std::{
    fs::{self, File, OpenOptions},
    io::{Write},
    path::{Path, PathBuf},
};

use rand::{thread_rng, Rng};
use tempfile::TempDir;

use crate::{as_ref_all, err};

#[macro_export]
macro_rules! join_all {
    ( $self:ident, $($path:ident),+ ) => {
        $(
            let $path = $self.join_check($path);
        )+
    };
    ( $self:ident, $($path:expr),+ ) => {
        (
            $(
                $self.join_check($path)
            ),+
        )
    }
}

#[macro_export]
macro_rules! assert_file_contents_eq {
    ( $($path:expr),* ) => {
        assert_eq!(
            $({
                use std::fs::File;
                use std::io::Read;

                let path = &$path;
                let mut file = File::open(&path).unwrap_or_else(|_| panic!("Failed to open path {}", path.display()));
                let mut buf = Vec::new();
                file.read_to_end(&mut buf).unwrap_or_else(|_| panic!("Failed to read file with path {} to end", path.display()));
                buf
            }),*
        )
    };
}

#[macro_export]
macro_rules! assert_paths_exists {
    ( $($path:expr),* ) => {
        assert!(
            true $( && {
                let path = &$path;
                path.exists()
            })*
        )
    };
}

macro_rules! assert_macro_testing {
    ($($boolean:expr),+) => {
        assert!(
            true $( && {
                $boolean
            })+
        )
    };
}

#[test]
fn assert_macro_test() {
    assert_macro_testing!(true, true);
}

#[test]
#[should_panic]
fn asset_macro_test_one_false() {
    assert_macro_testing!(true, true, false, true);
}

const RAND_BYTES: usize = 512;

pub struct TestDir(TempDir);

impl TestDir {
    pub fn new() -> TestDir {
        let tempdir = TempDir::new().expect("Failed to create tempdir for the test dir");
        TestDir(tempdir)
    }

    pub fn close(self) {
        self.0
            .close()
            .unwrap_or_else(|_| panic!("Failed to close test dir"))
    }

    pub fn path(&self) -> &Path {
        self.0.path()
    }

    /// Return a path joined to the path to this directory.
    pub fn join<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        self.path().join(path)
    }

    /// Return a path joined to the path to this directory. Panics if it is already there
    pub fn join_check<'a, P: AsRef<Path> + 'a>(&self, path: P) -> PathBuf {
        let path = path.as_ref();

        if !path.starts_with(self.path()) {
            self.path().join(path)
        } else {
            panic!(
                "The path {} cannot be join to the tempdir with path {}",
                path.display(),
                self.path().display()
            );
        }
    }

    /// Create a directory at the given path, while creating all intermediate
    /// directories as needed.
    pub fn mkdirp<P: AsRef<Path>>(&self, path: P) {
        as_ref_all!(path);

        fs::create_dir_all(&path)
            .map_err(|e| err!("failed to create directory {}: {}", path.display(), e))
            .unwrap();
    }

    /// Create an empty file at the given path. All ancestor directories must
    /// already exists.
    pub fn touch<P: AsRef<Path>>(&self, path: P) {
        as_ref_all!(path);

        File::create(&path)
            .map_err(|e| err!("failed to create file {}: {}", path.display(), e))
            .unwrap();
    }

    /// Create empty files at the given paths. All ancestor directories must
    /// already exists.
    pub fn touch_all<P: AsRef<Path>>(&self, paths: &[P]) {
        for p in paths {
            self.touch(p);
        }
    }

    pub fn touch_with_contents<P: AsRef<Path>>(&self, path: P) {
        as_ref_all!(path);

        let mut open_opt = OpenOptions::new();
        open_opt.create_new(true).write(true);

        let mut file = open_opt
            .open(&path)
            .map_err(|e| err!("failed to create file {}: {}", path.display(), e))
            .unwrap();

        file.write_all(&random_bytes())
            .expect("Failed to write random bytes to the file");
        file.sync_all()
            .expect("Failed to sync file before dropping");
    }

    /// Create empty files at the given paths. All ancestor directories must
    /// already exists.
    pub fn touch_all_with_contents<P: AsRef<Path>>(&self, paths: &[P]) {
        for p in paths {
            self.touch_with_contents(p);
        }
    }
}

pub fn random_bytes() -> [u8; RAND_BYTES] {
    let mut rng = thread_rng();
    let mut buf = [0u8; RAND_BYTES];
    rng.fill(&mut buf);
    buf
}
