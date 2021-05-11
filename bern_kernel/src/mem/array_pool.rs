use core::cell::RefCell;
use core::ptr::NonNull;

use crate::mem::pool_allocator::{PoolAllocator, Error};
use crate::collection::boxed::Box;


#[derive(Debug)]
pub struct ArrayPool<T, const N: usize> {
    pool: RefCell<[Option<T>; N]>,
}

impl<T, const N: usize> ArrayPool<T, {N}>
{
    pub const fn new(array: [Option<T>; N]) -> Self {
        ArrayPool {
            pool: RefCell::new(array),
        }
    }
}

// todo: make sync safe!
unsafe impl<T, const N: usize> Sync for ArrayPool<T, {N}> {}


impl<T, const N: usize> PoolAllocator<T> for ArrayPool<T, {N}> {
    fn insert(&self, element: T) -> Result<Box<T>, Error> {
        for item in self.pool.borrow_mut().iter_mut() {
            if item.is_none() {
                *item = Some(element);
                match item {
                    Some(i) => unsafe {
                        return Ok(Box::from_raw(NonNull::new_unchecked(i as *mut _)))
                    },
                    _ => return Err(Error::Unknown),
                }
            }
        }
        return Err(Error::OutOfMemory);
    }
}