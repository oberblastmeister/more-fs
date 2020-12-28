mod general;
mod test_dir;
mod utils;

use std::{error, result};

#[macro_export]
macro_rules! err {
    ($($tt:tt)*) => {
        Box::<dyn std::error::Error + Send + Sync>::from(format!($($tt)*))
    }
}

/// A convenient result type alias.
pub type Result<T> = result::Result<T, Box<dyn error::Error + Send + Sync>>;
