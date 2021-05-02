#![cfg_attr(all(not(test), not(target_arch = "thumb")), no_std)]
#![feature(asm)]
#![feature(naked_functions)]

pub mod cortex_m;
pub mod syscall;
pub mod core;
pub mod context_switch;

// re-exports
pub use crate::context_switch::ContextSwitch;
pub use crate::syscall::Syscall;
pub use crate::core::Core;



// select architecture support
pub use crate::cortex_m as arch;