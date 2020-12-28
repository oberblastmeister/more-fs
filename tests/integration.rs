mod common;
mod custom_tempfile;

use std::fs::{File, OpenOptions};
use std::{
    io::{Read, Write},
    path::Path,
};

use custom_tempfile::{create_test_file, random_bytes, tmpname, TMP_DIR};
use eyre::{Result, WrapErr};
use function_name::named;

use common::{asset_dir, clone_repo};

/// Will make sure that there are not name clashes between functions because each function has to
/// have a separate name
macro_rules! name {
    ($s:expr) => {
        concat!(function_name!(), "::", $s)
    };
    ($s:expr, $sep:expr) => {
        concat!(function_name!(), $sep, $s)
    };
}

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

fn copy_test_helper(from: impl AsRef<Path>, to: impl AsRef<Path>, bytes: &[u8]) -> Result<()> {
    let mut opt = OpenOptions::new();
    opt.read(true).write(true).create(true).truncate(true);

    let mut file1 = opt
        .open(&from)
        .wrap_err_with(|| format!("Failed to open file with name {}", &from.as_ref().display()))?;
    file1.write_all(&bytes).unwrap();

    copy_properly(&from, &to);

    remove_properly(from);
    remove_properly(to);
    Ok(())
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
#[named]
fn copy_test() -> Result<()> {
    let from = create_test_file(name!("from"))?;
    let to = tmpname(name!("to"))?;

    copy_properly(from, to);

    Ok(())
}

#[test]
#[named]
#[ignore]
#[should_panic]
fn copy_test_not_in_same_dir() {
    let from = create_test_file(name!("from")).unwrap();
    let to = TMP_DIR
        .path()
        .join(concat!(name!("first_dir"), "/", name!("second_dir")))
        .join(tmpname(name!("to")).unwrap().file_name().unwrap());

    println!("{}", to.display());

    copy_properly(from, to);
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

#[ignore]
#[test]
fn copy_dir_all_test() {
    let dir = clone_repo("https://github.com/sharkdp/fd.git", "fd_test");
    more_fs::copy_dir_all(&dir, dir.parent().unwrap().join("copied_fd")).unwrap();
}
