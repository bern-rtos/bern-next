#![no_std] // change for module tests to #![cfg_attr(not(test), no_std)]
#![feature(unsize)]
#![feature(asm)]

pub mod error;
pub mod task;
pub mod scheduler;
pub mod boxed;