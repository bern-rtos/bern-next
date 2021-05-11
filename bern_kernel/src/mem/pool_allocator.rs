use crate::collection::boxed::Box;

pub enum Error {
    OutOfMemory,
    Unknown,
}

pub trait PoolAllocator<T> {
    fn insert(&self, element: T) -> Result<Box<T>, Error>;
    // todo: drop
}