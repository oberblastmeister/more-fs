use std::{fs::File, path::PathBuf, process::Command};

use rand::{thread_rng, Rng};

pub fn asset_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/assets")
}

pub fn random_bytes() -> [u8; 1024] {
    let mut rng = thread_rng();
    let mut buf = [0u8; 1024];
    rng.fill(&mut buf);
    buf
}

pub fn clone_repo(url: &str, name: &str) -> PathBuf {
    let asset_dir = asset_dir();
    let dir = asset_dir.join(name);

    if !dir.exists() {
        println!("will git clone");
        let output = Command::new("git")
            .arg("clone")
            .arg(url)
            .arg(&dir)
            .output()
            .expect("failed to git clone linux");
        println!("did git clone linux...{:?}", output);
    }

    dir
}
