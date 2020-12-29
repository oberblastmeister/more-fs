use super::utils::clone_repo;
use crate::{assert_file_contents_eq, assert_paths_exists, join_all};

macro_rules! fs_test {
    (
        #[test]
        $( #[$meta:meta] )*
        fn $test_name:ident($test_dir:ident) -> $return:ty $body:block
    ) => {
        #[test]
        $( #[$meta] )*
        fn $test_name() -> $return {
            let $test_dir = super::test_dir::TestDir::new();
            $block
            $test_dir.close()
        }
    };
    (
        #[test]
        $( #[$meta:meta] )*
        fn $test_name:ident($test_dir:ident) $body:block
    ) => {
        #[test]
        $( #[$meta] )*
        fn $test_name() {
            let $test_dir = super::test_dir::TestDir::new();
            $body
            $test_dir.close()
        }
    };
}

fs_test! {
    #[test]
    fn copy(dir) {
        let (from, to) = join_all!(dir, "from", "to");

        dir.touch_with_contents(&from);

        crate::copy(&from, &to).unwrap();

        assert_file_contents_eq!(&from, &to);
        assert_paths_exists!(from, to);
    }
}

fs_test! {
    #[test]
    #[should_panic]
    fn copy_test_not_in_same_dir(dir) {
        let (from, to) = join_all!(dir, "from", "a_dir/another_dir/to");
        dir.touch_with_contents(&from);

        crate::copy(&from, &to).unwrap();

        assert_file_contents_eq!(from, to);
        assert_paths_exists!(from, to);
    }
}

fs_test! {
    #[test]
    fn copy_create(dir) {
        let (from, to) = join_all!(dir, "from", "a_dir/another_dir/to");
        dir.touch_with_contents(&from);

        crate::copy_create(&from, &to).unwrap();

        assert_file_contents_eq!(from, to);
        assert_paths_exists!(from, to);
    }
}

fs_test! {
    #[test]
    fn move_file(dir) {
        let (from, to) = join_all!(dir, "from", "moved");
        dir.touch_with_contents(&from);

        crate::move_file(&from, &to).unwrap();
        assert_paths_exists!(to);
        assert!(!from.exists())
    }
}

fs_test! {
    #[test]
    /// This shouldn't panic, creating a dir that already exists should be okay
    fn create_dir_all_already_exists(dir) {
        let (already_exists) = join_all!(dir, "already_exists_dir/another_dir");
        let create = already_exists.clone();
        dir.mkdirp(already_exists);
        crate::create_dir_all(create).unwrap();
    }
}

fs_test! {
    #[test]
    #[ignore]
    fn copy_dir_all(dir) {
        let (create_dir, create_file, from, to) = join_all!(dir, "a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p", "a/b/c/d/e/a_file.txt", "a", "moved");
        dir.mkdirp(create_dir);
        dir.touch(create_file);

        crate::copy_dir_all(&from, &to).unwrap();
        assert_paths_exists!(from, to);
    }
}

fs_test! {
    #[test]
    #[ignore]
    /// these will block for some reason
    fn copy_dir_all_par(dir) {
        // let (create_dir, from, to) = join_all!(dir, "from/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p", "from", "to");
        let (create_dir, from, to) = join_all!(dir, "from/another_dir", "from", "to");
        dir.mkdirp(create_dir);

        // crate::copy_dir_all_par(&from, &to).unwrap();
        // assert_paths_exists!(from, to);
    }
}

fs_test! {
    #[test]
    fn copy_dir_all_par_fd(dir) {
        let (from, to) = join_all!(dir, "fd_from", "fd_to");
        clone_repo("https://github.com/sharkdp/fd.git", &from);

        crate::copy_dir_all_par(&from, &to).unwrap();
        assert_paths_exists!(from, to);
    }
}
