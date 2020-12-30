#[macro_export]
macro_rules! fs_fn {
    (
        $( #[$meta:meta] )*
        fn $test_name:ident $params:tt ($test_dir:ident) -> $return:ty $body:block
    ) => {
        $( #[$meta] )*
        fn $test_name $params -> $return {
            let $test_dir = test_dir::TestDir::new();
            let res = $block
            $test_dir.close()
            res
        }
    };
    (
        $( #[$meta:meta] )*
        fn $test_name:ident $params:tt ($test_dir:ident) $body:block
    ) => {
        $( #[$meta] )*
        fn $test_name $params {
            let $test_dir = test_dir::TestDir::new();
            $body
            $test_dir.close()
        }
    };
}

macro_rules! err {
    ($($tt:tt)*) => {
        Box::<dyn std::error::Error + Send + Sync>::from(format!($($tt)*))
    }
}

/// helper macro to call asref on all of the identifiers
macro_rules! as_ref_all {
    ( $( $var:ident ),* ) => {
        $( let $var = $var.as_ref(); )*
    };
}

#[macro_export]
macro_rules! join_all {
    ( $self:ident, $($path:ident),+ ) => {
        $(
            let $path = $self.join_check(Path::new($path));
        )+
    };
    ( $self:ident, $($path:expr),+ ) => {
        (
            $(
                $self.join_check($path)
            ),+
        )
    }
}

#[macro_export]
macro_rules! assert_file_contents_eq {
    ( $($path:expr),* ) => {
        assert_eq!(
            $({
                use std::fs::File;
                use std::io::Read;

                let path = &$path;
                let mut file = File::open(&path).unwrap_or_else(|_| panic!("Failed to open path {}", path.display()));
                let mut buf = Vec::new();
                file.read_to_end(&mut buf).unwrap_or_else(|_| panic!("Failed to read file with path {} to end", path.display()));
                buf
            }),*
        )
    };
}

#[macro_export]
macro_rules! assert_paths_exists {
    ( $($path:expr),* ) => {
        $(
            let path = $path;
            if !path.exists() {
                panic!("Assertion failed, the path {} does not exist", path.display())
            }
        )*
    };
}

macro_rules! assert_macro_testing {
    ($($boolean:expr),+) => {
        assert!(
            true $( && {
                $boolean
            })+
        )
    };
}

#[test]
fn assert_macro_test() {
    assert_macro_testing!(true, true);
}

#[test]
#[should_panic]
fn asset_macro_test_one_false() {
    assert_macro_testing!(true, true, false, true);
}
