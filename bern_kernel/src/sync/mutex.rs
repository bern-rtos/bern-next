use core::sync::atomic::{AtomicBool, Ordering};

use crate::syscall;
use core::ptr::{null_mut};
use core::ops::{Deref, DerefMut};
use core::cell::UnsafeCell;
use crate::mem::linked_list::LinkedList;

pub enum Error {
    WouldBlock,
}

/// For multiple tasks to access a mutex it must be placed in shared memory
/// section. If the data is placed in a shared section the lock can be placed
/// there as well. Malicious software can corrupt any shared memory section it
/// has access to.
///
pub struct Mutex<T> {
    inner: UnsafeCell<T>,
    lock: AtomicBool,
}

impl<T> Mutex<T> {
    pub const fn new(element: T) -> Self {
        Mutex {
            inner: UnsafeCell::new(element),
            lock: AtomicBool::new(false),
        }
    }

    pub fn try_lock(&self) -> Result<MutexGuard<'_,T>, Error> {
        if self.raw_lock().is_ok() {
            return Ok(MutexGuard::new(&self));
        } else {
            return Err(Error::WouldBlock);
        }
    }

    fn raw_lock(&self) -> Result<bool,bool> {
        self.lock.compare_exchange(false, true,
                                   Ordering::Acquire,
                                   Ordering::Acquire)
    }

    fn raw_unlock(&self) {
        self.lock.store(false, Ordering::Release);
    }
}

unsafe impl<T> Sync for Mutex<T> {}

pub struct MutexGuard<'a,T> {
    lock: &'a Mutex<T>,
}

impl<'a,T> MutexGuard<'a,T> {
    fn new(lock: &'a Mutex<T>,) -> Self {
        MutexGuard {
            lock,
        }
    }
}

impl<'a,T> Deref for MutexGuard<'a,T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.inner.get() }
    }
}

impl<'a,T> DerefMut for MutexGuard<'a,T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.lock.inner.get() }
    }
}

impl<'a,T> Drop for MutexGuard<'a,T> {
    fn drop(&mut self) {
        self.lock.raw_unlock();
    }
}

// userland barrier ////////////////////////////////////////////////////////////

/// kernel internal mutex data
pub(crate) struct MutexInternal {
    inner: *mut usize,
}

impl MutexInternal {
    pub(crate) fn new(data: *mut usize) -> Self {
        MutexInternal {
            inner: data,
        }
    }

    pub(crate) fn try_lock(&mut self) {

    }

    pub(crate) fn release(&mut self) {
    }
}
