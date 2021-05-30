pub trait IMemoryProtection {
    type Size;

    fn enable_memory_protection();
    fn disable_memory_protection();
    fn protect_memory_region(region: u8, config: Config<Self::Size>);
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
    Ram,
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