use std::{
    path::{Path, PathBuf},
    process::Command,
    str,
};

pub fn clone_repo<P: AsRef<Path>>(url: &str, path: P) {
    let path = path.as_ref();

    println!("git cloning path {}", path.display());
    let status = Command::new("git")
        .arg("clone")
        .arg(url)
        .arg(&path)
        .status()
        .expect("Failed to get status");
    println!("Exit status {}", status);
}
