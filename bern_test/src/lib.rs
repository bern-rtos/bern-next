#![no_std]

pub mod serial;
pub mod console;
pub mod runall;

pub use bern_test_macros::tests;

#[cfg(feature = "serial")]
#[macro_export]
macro_rules! println {
    ($($args:tt)*) => {
        {
            $crate::sprintln!($($args)*);
        }
    }
}

#[cfg(feature = "serial")]
#[macro_export]
macro_rules! print {
    ($($args:tt)*) => {
        {
            $crate::sprint!($($args)*);
        }
    }
}

pub fn get_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub fn is_autorun_enabled() -> bool {
    #[cfg(feature = "autorun")]
    return true;
    #[cfg(not(feature = "autorun"))]
    return false;
}