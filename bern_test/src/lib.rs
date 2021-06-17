//! On target test framework for microcontrollers.
//!
//! Based on `defmt-test` but not dependent on architecture or serial interface.
//!
//! # Features
//! - `autorun`: Run tests without user interaction
//! - `serial`: Use serial interface for transport
//! - `rtt`: Use RTT for transport
//! - `colored`: Use terminal colors

#![no_std]

pub mod serial;
#[doc(hidden)]
pub mod console;
#[doc(hidden)]
pub mod run_all;

pub use bern_test_macros::tests;

#[cfg(feature = "rtt")]
pub use rtt_target;

use core::panic::PanicInfo;

#[doc(hidden)]
pub fn test_succeeded() {
    println!(term_green!("ok"));
    run_all::test_succeeded();
}

#[doc(hidden)]
pub fn test_failed(message: &str) {
    println!(term_red!("FAILED"));
    println!("{}", message);
}

#[doc(hidden)]
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

#[doc(hidden)]
pub fn get_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[doc(hidden)]
pub fn is_autorun_enabled() -> bool {
    #[cfg(feature = "autorun")]
    return true;
    #[cfg(not(feature = "autorun"))]
    return false;
}