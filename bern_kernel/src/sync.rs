#[allow(dead_code)]

pub(crate) mod critical_mutex;
pub(crate) mod critical_section;
pub mod mutex;
pub mod semaphore;

pub enum Error {
    WouldBlock,
    TimeOut,
    Poisoned,
    OutOfMemory,
}