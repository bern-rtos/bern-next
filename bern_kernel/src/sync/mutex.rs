use core::sync::atomic::{AtomicBool, Ordering};
use core::marker::PhantomData;

use crate::syscall;
use core::ptr::{NonNull, null_mut};
use core::ops::{Deref, DerefMut};

pub enum Error {
    WouldBlock,
}

pub struct Mutex<T> {
    id: usize,
    _marker: PhantomData<T>,
}

impl<T> Mutex<T> {
    pub fn new(element: T) -> Self {
        let id = syscall::mutex_new(&element);
        Mutex {
            id,
            _marker: PhantomData,
        }
    }

    pub fn try_lock(&self) -> Result<MutexGuard<'_,T>, Error> {
        // Note(unsafe): the syscall will either return an error or a valid pointer
        syscall::mutex_lock(self.id)
            .map(|data| MutexGuard::new(
                self,
                unsafe { NonNull::new_unchecked(data as *mut T) }
            ))
    }
}

pub struct MutexGuard<'a,T> {
    inner: NonNull<T>,
    lock: &'a Mutex<T>,
}

impl<'a,T> MutexGuard<'a,T> {
    fn new(lock: &'a Mutex<T>, inner: NonNull<T>) -> Self {
        MutexGuard {
            inner,
            lock,
        }
    }
}

impl<'a,T> Deref for MutexGuard<'a,T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.inner.as_ref() }
    }
}

impl<'a,T> DerefMut for MutexGuard<'a,T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.inner.as_mut() }
    }
}

impl<'a,T> Drop for MutexGuard<'a,T> {
    fn drop(&mut self) {
        syscall::mutex_release(self.lock.id);
    }
}

// userland barrier ////////////////////////////////////////////////////////////

/// kernel internal mutex data
pub(crate) struct MutexInternal {
    inner: *mut usize,
    lock: AtomicBool,
}

impl MutexInternal {
    pub(crate) fn new(data: *mut usize) -> Self {
        MutexInternal {
            inner: data,
            lock: AtomicBool::new(false),
        }
    }

    pub(crate) fn try_lock(&mut self) -> *mut usize {
        if self.lock.compare_exchange(false, true,
                                      Ordering::Acquire,
                                      Ordering::Acquire).is_ok() {
            return self.inner;
        } else {
            return null_mut();
        }
    }

    pub(crate) fn release(&mut self) {
        self.lock.store(false, Ordering::Release);
    }
}
