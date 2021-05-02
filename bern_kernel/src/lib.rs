#![cfg_attr(all(not(test), not(target_arch = "thumb")), no_std)]
#![feature(unsize)]
#![feature(asm)]
#![feature(naked_functions)]

pub mod error;
pub mod task;
pub mod scheduler;
pub mod syscall;
pub mod time;
mod collection;
mod sync;

pub use crate::syscall::*;
#[allow(unused_imports)]
use bern_arch::arch as _;
