#![no_std]

pub mod serial;
pub mod console;

pub use bern_test_macros::tests;

#[cfg(feature = "serial")]
#[macro_export]
macro_rules! println {
    ($($args:tt)*) => {
        {
            sprintln!($($args)*);
        }
    }
}

#[cfg(feature = "serial")]
#[macro_export]
macro_rules! print {
    ($($args:tt)*) => {
        {
            sprint!($($args)*);
        }
    }
}

pub fn get_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}