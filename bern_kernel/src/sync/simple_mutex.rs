use core::sync::atomic::{AtomicBool, Ordering};
use core::cell::UnsafeCell;

pub struct SimpleMutex<T> {
    inner: UnsafeCell<T>,
    lock: AtomicBool,
}

impl<T> SimpleMutex<T> {
    pub const fn new(element: T) -> Self {
        SimpleMutex {
            inner: UnsafeCell::new(element),
            lock: AtomicBool::new(false),
        }
    }

    pub fn lock(&self) -> Option<&mut T> {
        match self.lock.compare_exchange(false, true, Ordering::Acquire, Ordering::Acquire) {
            Ok(_) => Some(unsafe { &mut *self.inner.get() }),
            Err(_) => None,
        }
    }

    // this is a temporary solution -> stupid because one can release the lock without having the data
    pub fn release(&self) {
        self.lock.store(false, Ordering::Release);
    }
}

unsafe impl<T> Sync for SimpleMutex<T> { }