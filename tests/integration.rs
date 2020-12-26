mod common;

use std::{io::{Read, Write}, path::Path};
use std::{
    fs::{self, File, OpenOptions},
    ops::Range,
};

// use proptest::prelude::*;
use eyre::{WrapErr, Result};

use common::{random_bytes, get_asset_dir};

fn remove_properly(path: impl AsRef<Path>) {
    let path = path.as_ref();
    more_fs::remove_file(path).unwrap();
    assert!(!path.exists());
}

fn copy_properly(path1: impl AsRef<Path>, path2: impl AsRef<Path>) {
    let path1 = path1.as_ref();
    let path2 = path2.as_ref();

    more_fs::copy(&path1, &path2).unwrap();

    let mut file1 = File::open(&path1).unwrap();
    let mut file2 = File::open(&path2).unwrap();

    let mut buf1 = Vec::new();
    let mut buf2 = Vec::new();
    file1.read_to_end(&mut buf1).unwrap();
    file2.read_to_end(&mut buf2).unwrap();

    assert_eq!(buf1, buf2);
}

fn copy_test_helper(path1: impl AsRef<Path>, path2: impl AsRef<Path>, bytes: &[u8]) {
    let mut opt = OpenOptions::new();
    opt
        .read(true)
        .write(true)
        .create(true)
        .truncate(true);

    let mut file1 = opt.open(&path1).unwrap();
    file1.write_all(&bytes).unwrap();

    copy_properly(&path1, &path2);

    remove_properly(path1);
    remove_properly(path2);
}

#[test]
fn copy_test() {
    let bytes = random_bytes();
    let path1 = get_asset_dir().join("test_file");
    let path2 = path1.parent().unwrap().join("copied_test_file");
    copy_test_helper(path1, path2, &bytes);
}

#[test]
#[should_panic]
fn copy_test_not_in_same_dir() {
    let bytes = random_bytes();
    let path1 = get_asset_dir().join("copy_not_in_same_dir_test_file");
    let path2 = path1.parent().unwrap().join("another_dir/copy_not_in_same_dir_test_file");
    copy_test_helper(path1, path2, &bytes);
}
