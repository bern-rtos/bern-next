//! CPU core peripherals.

/// CPU core peripherals.
pub trait ICore {
    /// Setup core peripherals and return core object
    fn new() -> Self;
    /// Start peripherals used by kernel
    fn start(&mut self);
    /// Break point instruction
    fn bkpt();
}