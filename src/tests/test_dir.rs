use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use rand::{Rng, thread_rng};
use tempfile::TempDir;

use crate::err;

const RAND_BYTES: usize = 512;

pub struct TestDir(TempDir);

impl TestDir {
    pub fn new() -> TestDir {
        let tempdir = TempDir::new().expect("Failed to create tempdir for the test dir");
        TestDir(tempdir)
    }

    pub fn close(self) {
        self.0.close().unwrap_or_else(|_| panic!("Failed to close test dir"))
    }

    pub fn path(&self) -> &Path {
        self.0.path()
    }

    /// Return a path joined to the path to this directory.
    pub fn join<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        self.path().join(path)
    }

    /// Create a directory at the given path, while creating all intermediate
    /// directories as needed.
    pub fn mkdirp<P: AsRef<Path>>(&self, path: P) {
        let full = self.join(path);
        fs::create_dir_all(&full)
            .map_err(|e| err!("failed to create directory {}: {}", full.display(), e))
            .unwrap();
    }

    /// Create an empty file at the given path. All ancestor directories must
    /// already exists.
    pub fn touch<P: AsRef<Path>>(&self, path: P) {
        let full = self.join(path);
        File::create(&full)
            .map_err(|e| err!("failed to create file {}: {}", full.display(), e))
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
        let full = self.join(path);

        let mut open_opt = OpenOptions::new();
        open_opt.create_new(true).write(true);

        let mut file = open_opt
            .open(&full)
            .map_err(|e| err!("failed to create file {}: {}", full.display(), e))
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

    pub fn copy_properly(&self, from: impl AsRef<Path>, to: impl AsRef<Path>) {
        let from = self.join(from);
        let to = self.join(to);

        crate::copy(&from, &to).unwrap();

        let mut file1 = File::open(&from).unwrap();
        let mut file2 = File::open(&to).unwrap();

        let mut buf1 = Vec::new();
        let mut buf2 = Vec::new();
        file1.read_to_end(&mut buf1).unwrap();
        file2.read_to_end(&mut buf2).unwrap();

        assert_eq!(buf1, buf2);
        assert!(from.exists());
        assert!(to.exists());
    }

    pub fn move_properly(&self, from: impl AsRef<Path>, to: impl AsRef<Path>) {
        let from = self.join(from);
        let to = self.join(to);

        crate::move_file(&from, &to).unwrap();
        assert!(!from.exists());
        assert!(to.exists());
    }

    pub fn remove_properly(&self, path: impl AsRef<Path>) {
        let path = self.join(path);
        crate::remove_file(&path).unwrap();
        assert!(!path.exists());
    }

    pub fn remove_dir_all_properly(&self, path: impl AsRef<Path>) {
        let path = self.join(path);
        crate::remove_dir_all(&path).unwrap();
        assert!(!path.exists());
    }

    pub fn create_dir_all_properly(&self, path: impl AsRef<Path>) {
        let path = self.join(path);
        crate::create_dir_all(&path).unwrap();
        assert!(path.exists());
    }

    pub fn copy_create_properly(&self, from: impl AsRef<Path>, to: impl AsRef<Path>) {
        let from = self.join(from);
        let to = self.join(to);

        crate::copy_create(&from, &to).unwrap();

        let mut file1 = File::open(&from).unwrap();
        let mut file2 = File::open(&to).unwrap();

        let mut buf1 = Vec::new();
        let mut buf2 = Vec::new();
        file1.read_to_end(&mut buf1).unwrap();
        file2.read_to_end(&mut buf2).unwrap();

        assert_eq!(buf1, buf2);
        assert!(to.parent().map(Path::exists).unwrap_or(true));
        assert!(from.exists());
        assert!(to.exists());
    }
}

pub fn random_bytes() -> [u8; RAND_BYTES] {
    let mut rng = thread_rng();
    let mut buf = [0u8; RAND_BYTES];
    rng.fill(&mut buf);
    buf
}
