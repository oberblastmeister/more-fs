use std::path::Path;

use crate::error::{Error, Result};

fn try_in_same_dir(path1: impl AsRef<Path>, path2: impl AsRef<Path>) -> Option<bool> {
    let path1 = path1.as_ref();
    let path2 = path2.as_ref();

    let parent1 = path1.parent()?;
    let parent2 = path2.parent()?;

    Some(parent1 == parent2)
}

pub fn in_same_dir(path1: impl AsRef<Path>, path2: impl AsRef<Path>) -> bool {
    try_in_same_dir(path1, path2).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_same_dir() {
        let path1 = "/home/person/dir/hello.txt";
        let path2 = "/home/person/dir/goodbye.txt";

        assert!(in_same_dir(path1, path2));
    }

    #[test]
    fn in_same_dir_root_test() {
        let path1 = "/";
        let path2 = "/";

        assert!(!in_same_dir(path1, path2));
    }

    #[test]
    fn in_same_dir_same_test() {
        let path1 = "/home/person";
        let path2 = "/home/person";

        assert!(in_same_dir(path1, path2));
    }
}
