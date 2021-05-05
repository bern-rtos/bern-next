#![cfg_attr(target_os = "none", no_std)]
#![feature(unsize)]
#![feature(asm)]
#![feature(naked_functions)]

pub mod error;
pub mod task;
pub mod scheduler;
pub mod syscall;
pub mod time;
pub mod stack;
mod collection;
mod sync;

pub use crate::syscall::*;
#[allow(unused_imports)]
use bern_arch::arch as _;
