use crate::arch::memory_protection::MemoryRegion;

pub trait IMemoryProtection {
    type Size;
    type MemoryRegion;

    fn enable_memory_protection();
    fn disable_memory_protection();
    fn enable_memory_region(region: u8, config: Config<Self::Size>);
    fn disable_memory_region(region: u8);
    /// Same as `enable_memory_region()` but return the register configuration
    /// instead of applying it to the actual registers.
    fn prepare_memory_region(region: u8, config: Config<Self::Size>) -> Self::MemoryRegion;
    fn prepare_unused_region(region: u8) -> Self::MemoryRegion;
    fn apply_regions(memory_regions: &[MemoryRegion; 3]);
}

pub enum Permission {
    NoAccess,
    ReadOnly,
    ReadWrite,
}

pub struct Access {
    pub user: Permission,
    pub system: Permission,
}

pub enum Type {
    SramInternal,
    SramExternal,
    Flash,
    Peripheral,
}

pub struct Config<S> {
    pub addr: *const usize,
    pub memory: Type,
    pub size: S,
    pub access: Access,
    pub executable: bool,
}