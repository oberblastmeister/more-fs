use std::{
    path::{Path, PathBuf},
    process::Command,
    str,
};

pub fn clone_repo<P: AsRef<Path>>(url: &str, path: P) {
    let path = path.as_ref();

    println!("git cloning path {}", path.display());
    let output = Command::new("git")
        .arg("clone")
        .arg(url)
        .arg(&path)
        .output()
        .expect("failed to git clone linux");
    println!("did git clone linux...{:?}", output);
}
