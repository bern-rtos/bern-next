pub trait IMemoryProtection {
    fn enable_memory_protection();
    fn disable_memory_protection();
    fn protect_memory_region(region: u8, addr: *const usize, size: usize, mode: usize);
}