#![cfg_attr(target_os = "none", no_std)]
#![feature(unsize)]
#![feature(asm)]
#![feature(naked_functions)]

pub mod conf;

pub mod error;
pub mod task;
pub mod sched;
pub mod syscall;
pub mod time;
pub mod stack;
pub mod sync;
pub mod mem;
mod startup;


pub use crate::syscall::*;
#[allow(unused_imports)]
use bern_arch::arch as _;
