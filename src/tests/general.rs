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
        let (from, to) = join_all!(dir, "from", "to");
        dir.touch_with_contents(&from);

        crate::move_file(&from, &to).unwrap();
        assert_paths_exists!(to);
        assert!(!from.exists())
    }
}
