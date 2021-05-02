pub mod syscall;
mod scheduler;
mod tick;
pub mod core;

pub struct Arch;

// re-exports
pub use crate::cortex_m::core::ArchCore;