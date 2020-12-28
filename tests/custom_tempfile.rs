use std::{ffi::{OsStr, OsString}, fs::{self, OpenOptions}, io::{self, Write}, path::PathBuf, str};

use eyre::{bail, Result, WrapErr};
use once_cell::sync::Lazy;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use tempfile::{tempdir, TempDir};

pub static TMP_DIR: Lazy<TempDir> = Lazy::new(|| tempdir().expect("Failed to initialize tempdir"));

const NUM_RETRIES: u32 = 1 << 31;
const NUM_RAND_CHARS: usize = 6;

pub fn random_bytes() -> [u8; 1024] {
    let mut rng = thread_rng();
    let mut buf = [0u8; 1024];
    rng.fill(&mut buf);
    buf
}

pub fn create_test_file(prefix: impl AsRef<OsStr>) -> Result<PathBuf> {
    let prefix = prefix.as_ref();

    let name = tmpname(prefix)?;

    let mut open_opt = OpenOptions::new();
    open_opt.create_new(true).write(true);
    let mut file = open_opt
        .open(&name)
        .wrap_err_with(|| format!("Failed to open file with name {}", name.display()))?;

    file.write_all(&random_bytes())
        .wrap_err("Failed to write random bytes to file")?;

    Ok(name)
}

/// Finds a tempname with the prefix that is non conflicting with other files
pub fn tmpname(prefix: impl AsRef<OsStr>) -> Result<PathBuf> {
    let prefix = prefix.as_ref();
    let path = TMP_DIR.path().join(get_random_name(prefix, ".tmp".as_ref(), NUM_RAND_CHARS));

    Ok(path)
}

fn tmpname_non_conflicting(prefix: &OsStr, suffix: &OsStr, random_len: usize) -> Result<PathBuf> {
    let tmp_dir_contents = fs::read_dir(TMP_DIR.path())
        .wrap_err_with(|| format!("Failed to readir with path {}", TMP_DIR.path().display()))?
        .map(|res| res.map(|d| d.path()))
        .collect::<io::Result<Vec<_>>>()?;

    for _ in 0..NUM_RETRIES {
        let path = TMP_DIR
            .path()
            .join(get_random_name(prefix, suffix, random_len));
        if !tmp_dir_contents.contains(&path) {
            return Ok(path);
        }
    }

    bail!("Too many retries to find a correct tmp file name")
}

/// copied from tempfile rust
fn get_random_name(prefix: &OsStr, suffix: &OsStr, rand_len: usize) -> OsString {
    let mut buf = OsString::with_capacity(prefix.len() + suffix.len() + rand_len);
    buf.push(prefix);

    // Push each character in one-by-one. Unfortunately, this is the only
    // safe(ish) simple way to do this without allocating a temporary
    // String/Vec.
    unsafe {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(rand_len)
            .for_each(|b| buf.push(str::from_utf8_unchecked(&[b as u8])))
    }
    buf.push(suffix);
    buf
}
