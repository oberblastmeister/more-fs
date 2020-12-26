use std::{fs::File, path::PathBuf};

use rand::{thread_rng, Rng};

pub fn get_asset_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/assets")
}

pub fn random_bytes() -> [u8; 1024] {
    let mut rng = thread_rng();
    let mut buf = [0u8; 1024];
    rng.fill(&mut buf);
    buf
}
