#[derive(Copy, Clone)]
pub struct Region {
    pub start: *const usize,
    pub stop: *const usize,
}

pub trait IStartup {
    fn init_static_memory();
    fn region() -> Region;
}