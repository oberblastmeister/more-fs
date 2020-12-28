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
        dir.touch_with_contents("from");
        dir.copy_properly("from", "to");
    }
}

fs_test! {
    #[test]
    #[should_panic]
    fn copy_test_not_in_same_dir(dir) {
        dir.touch_with_contents("from");
        dir.copy_properly("from", "a_dir/another_dir/to");
    }
}

fs_test! {
    #[test]
    fn copy_create(dir) {
        dir.touch_with_contents("from");
        dir.copy_create_properly("from", "a_dir/another_dir/to");
    }
}

fs_test! {
    #[test]
    fn move_file(dir) {
        dir.touch_with_contents("from");
        dir.move_properly("from", "to");
    }
}

// #[ignore]
// #[test]
// #[named]
// fn copy_dir_all_test() -> Result<()> {
//     let dir = clone_repo("https://github.com/sharkdp/fd.git", "fd_test");
//     crate::copy_dir_all(&dir, dir.parent().unwrap().join("copied_fd")).unwrap();

//     Ok(())
// }
