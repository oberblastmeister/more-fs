mod common;

use std::{
    fs::{self, File, OpenOptions},
    ops::Range,
};
use std::{
    io::{Read, Write},
    path::Path,
};

use eyre::{Result, WrapErr};

use common::{asset_dir, random_bytes, clone_repo};

fn move_properly(from: impl AsRef<Path>, to: impl AsRef<Path>) {
    let from = from.as_ref();
    let to = to.as_ref();

    more_fs::move_file(from, to).unwrap();
    assert!(!from.exists());
    assert!(to.exists());
}

fn move_test_helper(from: impl AsRef<Path>, to: impl AsRef<Path>, bytes: &[u8]) {
    let mut opt = OpenOptions::new();
    opt.read(true).write(true).create(true).truncate(true);

    let mut file1 = opt.open(&from).unwrap();
    file1.write_all(&bytes).unwrap();

    move_properly(&from, &to);

    remove_properly(to);
}

fn remove_properly(path: impl AsRef<Path>) {
    let path = path.as_ref();
    more_fs::remove_file(path).unwrap();
    assert!(!path.exists());
}

fn remove_dir_all_properly(path: impl AsRef<Path>) {
    let path = path.as_ref();
    more_fs::remove_dir_all(path).unwrap();
    assert!(!path.exists());
}

fn copy_properly(from: impl AsRef<Path>, to: impl AsRef<Path>) {
    let from = from.as_ref();
    let to = to.as_ref();

    more_fs::copy(&from, &to).unwrap();

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

fn create_dir_all_properly(path: impl AsRef<Path>) {
    let path = path.as_ref();
    more_fs::create_dir_all(path).unwrap();
    assert!(path.exists());
}

fn copy_test_helper(from: impl AsRef<Path>, to: impl AsRef<Path>, bytes: &[u8]) {
    let mut opt = OpenOptions::new();
    opt.read(true).write(true).create(true).truncate(true);

    let mut file1 = opt.open(&from).unwrap();
    file1.write_all(&bytes).unwrap();

    copy_properly(&from, &to);

    remove_properly(from);
    remove_properly(to);
}

fn copy_create_properly(from: impl AsRef<Path>, to: impl AsRef<Path>) {
    let from = from.as_ref();
    let to = to.as_ref();

    more_fs::copy_create(from, to).unwrap();

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

fn copy_create_test_helper(from: impl AsRef<Path>, to: impl AsRef<Path>, bytes: &[u8]) {
    let mut opt = OpenOptions::new();
    opt.read(true).write(true).create(true).truncate(true);

    let mut file1 = opt.open(&from).unwrap();
    file1.write_all(&bytes).unwrap();

    copy_create_properly(&from, &to);

    remove_properly(from);
    remove_properly(to);
}

#[test]
fn copy_test() {
    let bytes = random_bytes();
    let path1 = asset_dir().join("test_file");
    let path2 = path1.parent().unwrap().join("copied_test_file");
    copy_test_helper(path1, path2, &bytes);
}

#[test]
#[should_panic]
fn copy_test_not_in_same_dir() {
    let bytes = random_bytes();
    let path1 = asset_dir().join("copy_not_in_same_dir_test_file");
    let path2 = path1
        .parent()
        .unwrap()
        .join("another_dir/copy_not_in_same_dir_test_file");
    copy_test_helper(path1, path2, &bytes);
}

#[test]
fn copy_create() {
    let bytes = random_bytes();
    let path1 = asset_dir().join("copy_create_test_file");
    let path2 = asset_dir().join("a_dir/another_dir/new_file");
    copy_create_test_helper(path1, path2, &bytes);
    remove_dir_all_properly(asset_dir().join("a_dir"));
}

#[test]
fn move_test() {
    let bytes = random_bytes();
    let path1 = asset_dir().join("move_test_file");
    let path2 = asset_dir().join("moved_file");
    move_test_helper(path1, path2, &bytes);
}

#[test]
fn copy_dir_all_test() {
    let dir = clone_repo("https://github.com/sharkdp/fd.git", "fd_test");
    more_fs::copy_dir_all(&dir, dir.parent().unwrap().join("copied_fd")).unwrap();
}
