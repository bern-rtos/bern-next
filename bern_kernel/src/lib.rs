#![cfg_attr(target_os = "none", no_std)]
#![feature(unsize)]
#![feature(asm)]
#![feature(naked_functions)]

pub mod error;
pub mod task;
pub mod sched;
pub mod syscall;
pub mod time;
pub mod stack;
pub mod sync;
pub mod mem;

pub use crate::syscall::*;
pub use bern_kernel_macros::*;

#[allow(unused_imports)]
use bern_arch::arch as _;
pub use bern_arch;
