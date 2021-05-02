pub mod syscall;
pub mod core;
mod scheduler;
mod tick;
mod register;

pub struct Arch;

// re-exports
pub use crate::cortex_m::core::ArchCore;