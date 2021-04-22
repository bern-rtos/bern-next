#![cfg_attr(all(not(test), not(target_arch = "thumb")), no_std)]
#![feature(unsize)]
#![feature(asm)]

pub mod error;
pub mod task;
pub mod scheduler;
mod linked_list;
mod boxed;