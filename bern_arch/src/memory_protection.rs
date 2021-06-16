//! Memory Protection.

use crate::arch::memory_protection::MemoryRegion;
/// Memory Protection.
pub trait IMemoryProtection {
    /// Size of a memory region
    type Size;
    /// Precalculated memory region configuration
    type MemoryRegion;

    /// Enable memory protection hardware.
    fn enable_memory_protection();
    /// Disable memory protection hardware.
    fn disable_memory_protection();
    /// Setup and enable one memory region.
    fn enable_memory_region(region: u8, config: Config<Self::Size>);
    /// Disable one memory region.
    fn disable_memory_region(region: u8);
    /// Compile register values from configuration and store in `MemoryRegion`.
    ///
    /// Same as [`Self::enable_memory_region()`] but return the register configuration
    /// instead of applying it to the actual registers.
    fn prepare_memory_region(region: u8, config: Config<Self::Size>) -> Self::MemoryRegion;
    /// Compile register values for an unused memory region.
    fn prepare_unused_region(region: u8) -> Self::MemoryRegion;
    /// Apply 3 precompiled memory regions.
    fn apply_regions(memory_regions: &[MemoryRegion; 3]);
}

/// Access Permission
pub enum Permission {
    /// Access not permitted
    NoAccess,
    /// Can only be read
    ReadOnly,
    /// Full access, can be read and written
    ReadWrite,
}

/// Access configuration
pub struct Access {
    /// Permission in user mode (i.e. tasks)
    pub user: Permission,
    /// Permission in system mode (i.e. ISR, kernel)
    pub system: Permission,
}

/// Type of memory
pub enum Type {
    /// SRAM in the microcontroller
    SramInternal,
    /// SRAM attach to the microcontroller externally
    SramExternal,
    /// Internal flash memory
    Flash,
    /// Microcontroller peripherals
    Peripheral,
}

/// Memory region configuration
pub struct Config<S> {
    /// Region base address
    pub addr: *const usize,
    /// Memory type
    pub memory: Type,
    /// Size of region
    pub size: S,
    /// Permissions
    pub access: Access,
    /// Memory region can be used to fetch instructions
    pub executable: bool,
}