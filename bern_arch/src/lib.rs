#![cfg_attr(all(not(test), not(target_arch = "thumb")), no_std)]
#![feature(asm)]
#![feature(naked_functions)]

pub mod cortex_m;
pub mod syscall;
pub mod core;
pub mod scheduler;

// re-exports
pub use crate::scheduler::IScheduler;
pub use crate::syscall::ISyscall;
pub use crate::core::ICore;



// select architecture support
pub use crate::cortex_m as arch;