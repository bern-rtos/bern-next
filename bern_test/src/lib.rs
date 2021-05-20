#![no_std]

pub mod serial;
pub mod console;
pub mod run_all;

pub use bern_test_macros::tests;

#[cfg(feature = "rtt")]
pub use rtt_target;

use core::panic::PanicInfo;

pub fn test_succeeded() {
    println!(term_green!("ok"));
    run_all::test_succeeded();
}

pub fn test_failed(message: &str) {
    println!(term_red!("FAILED"));
    println!("{}", message);
}

pub fn test_panicked(info: &PanicInfo) {
    println!(term_red!("FAILED"));
    println!(" └─ stdout:\n{}", info);
}


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

#[cfg(feature = "rtt")]
#[macro_export]
macro_rules! println {
    ($($args:tt)*) => {
        {
            rtt_target::rprintln!($($args)*);
        }
    }
}

#[cfg(feature = "rtt")]
#[macro_export]
macro_rules! print {
    ($($args:tt)*) => {
        {
            rtt_target::rprint!($($args)*);
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