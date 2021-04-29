#![cfg_attr(all(not(test), not(target_arch = "thumb")), no_std)]
#![feature(unsize)]
#![feature(asm)]
#![feature(naked_functions)]

pub mod error;
pub mod task;
pub mod scheduler;
pub mod api;
mod collection;
mod sync;
pub mod syscall;
pub mod time;

pub use crate::syscall::ArmCortexM as Core;
pub use crate::api::syscall::Syscall;
