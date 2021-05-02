pub mod syscall;
mod context_switch;
mod tick;
pub mod core;

pub struct Arch;

// re-exports
pub use crate::cortex_m::core::ArchCore;