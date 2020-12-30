use super::utils::clone_repo;
use test_dir::{assert_file_contents_eq, assert_paths_exists, fs_fn, join_all};

fs_fn! {
    #[test]
    fn copy()(dir) {
        let (from, to) = join_all!(dir, "from", "to");

        dir.touch_with_contents(&from);

        crate::copy(&from, &to).unwrap();

        assert_file_contents_eq!(&from, &to);
        assert_paths_exists!(from, to);
    }
}

fs_fn! {
    #[test]
    #[should_panic]
    fn copy_test_not_in_same_dir()(dir) {
        let (from, to) = join_all!(dir, "from", "a_dir/another_dir/to");
        dir.touch_with_contents(&from);

        crate::copy(&from, &to).unwrap();

        assert_file_contents_eq!(from, to);
        assert_paths_exists!(from, to);
    }
}

fs_fn! {
    #[test]
    fn copy_create()(dir) {
        let (from, to) = join_all!(dir, "from", "a_dir/another_dir/to");
        dir.touch_with_contents(&from);

        crate::copy_create(&from, &to).unwrap();

        assert_file_contents_eq!(from, to);
        assert_paths_exists!(from, to);
    }
}

fs_fn! {
    #[test]
    fn move_file()(dir) {
        let (from, to) = join_all!(dir, "from", "moved");
        dir.touch_with_contents(&from);

        crate::move_file(&from, &to).unwrap();
        assert_paths_exists!(to);
        assert!(!from.exists())
    }
}

fs_fn! {
    #[test]
    /// This shouldn't panic, creating a dir that already exists should be okay
    fn create_dir_all_already_exists()(dir) {
        let (already_exists) = join_all!(dir, "already_exists_dir/another_dir");
        let create = already_exists.clone();
        dir.mkdirp(already_exists);
        crate::create_dir_all(create).unwrap();
    }
}

fs_fn! {
    #[test]
    fn copy_dir_all()(dir) {
        let (create_dir, create_file, from, to) = join_all!(dir, "a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p", "a/b/c/d/e/a_file.txt", "a", "moved");
        dir.mkdirp(create_dir);
        dir.touch(create_file);

        crate::copy_dir_all(&from, &to).unwrap();
        assert_paths_exists!(from, to);
    }
}

#[cfg(feature = "rayon")]
fs_fn! {
    #[test]
    /// these will block for some reason, currently parallel doesn't work
    fn copy_dir_all_par()(dir) {
        let (create_dir, create_file, from, to) = join_all!(dir, "from/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p", "from/b/c/d/hello.txt", "from", "to");
        dir.mkdirp(create_dir);
        dir.touch_with_contents(create_file);

        println!("copy_dir_all_par");
        crate::copy_dir_all_par(&from, &to).unwrap();
        assert_paths_exists!(from, to);
    }
}

fs_fn! {
    #[test]
    fn copy_dir_all_fd()(dir) {
        let (from, to) = join_all!(dir, "fd_from", "fd_to");
        clone_repo("https://github.com/sharkdp/fd.git", &from);

        assert!(from.exists());
        assert!(!to.exists());
        crate::copy_dir_all(&from, &to).unwrap();
        assert_paths_exists!(from, to);
    }
}

#[cfg(feature = "rayon")]
fs_fn! {
    #[test]
    fn copy_dir_all_par_fd()(dir) {
        let (from, to) = join_all!(dir, "fd_from_par", "fd_to");
        clone_repo("https://github.com/sharkdp/fd.git", &from);

        assert!(from.exists());
        assert!(!to.exists());
        crate::copy_dir_all_par(&from, &to).unwrap();
        assert_paths_exists!(from, to);
    }
}
