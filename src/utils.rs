use std::path::{Path, PathBuf};

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

pub fn change_parent(path: impl AsRef<Path>, new_dir: impl AsRef<Path>) -> Option<PathBuf> {
    let filename = path.as_ref().file_name()?;
    Some(new_dir.as_ref().join(filename))
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

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

    const PATH_RE: &str = r"(/\w)+";
    const SEG_RE: &str = r"[^/\.]";

    proptest! {
        #[test]
        fn test_in_same_dir_prop(s in PATH_RE, seg in SEG_RE) {
            let path = PathBuf::from(&s);
            let parent = path.parent().unwrap();
            let path2 = parent.join(seg);
            prop_assert!(in_same_dir(path, path2))
        }
    }

    proptest! {
        #[test]
        fn not_in_same_dir_prop(s in PATH_RE, seg in SEG_RE) {
            let path = PathBuf::from(&s);
            let parent = path.parent().unwrap();
            let path2 = PathBuf::from(seg).join(parent);
            prop_assert!(!in_same_dir(path, path2))
        }
    }
}
