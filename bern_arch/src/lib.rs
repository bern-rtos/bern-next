#![cfg_attr(target_os = "none", no_std)]
#![feature(asm)]
#![feature(naked_functions)]
#![feature(never_type)] // for mock only

#![allow(unused)]

pub mod syscall;
pub mod core;
pub mod scheduler;
pub mod sync;
pub mod startup;
pub mod memory_protection;

// re-exports
pub use crate::scheduler::IScheduler;
pub use crate::syscall::ISyscall;
pub use crate::core::ICore;
pub use crate::sync::ISync;
pub use crate::startup::IStartup;
pub use crate::memory_protection::IMemoryProtection;

// select architecture support
#[cfg(not(target_os = "none"))]
pub mod mock;
#[cfg(not(target_os = "none"))]
pub use crate::mock as arch;

#[cfg(all(target_arch = "arm", target_os = "none"))]
pub mod cortex_m;

#[cfg(all(target_arch = "arm", target_os = "none"))]
pub use crate::cortex_m as arch;