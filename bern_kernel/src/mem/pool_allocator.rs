use crate::mem::boxed::Box;

#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    OutOfMemory,
    Unknown,
}

pub trait PoolAllocator<T> {
    fn insert(&self, element: T) -> Result<Box<T>, Error>;
    // todo: drop
}