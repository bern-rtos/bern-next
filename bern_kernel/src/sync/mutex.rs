//! Atomic mutual exclusion.
//!
//!
//!
//! # Examples

use core::sync::atomic::{AtomicBool, Ordering};
use core::ops::{Deref, DerefMut};
use core::cell::UnsafeCell;

use crate::syscall;
use crate::sched::event;
use super::Error;

///
/// (similar to [`std::sync::Mutex`](https://doc.rust-lang.org/std/sync/struct.Mutex.html))
///
/// For multiple tasks to access a mutex it must be placed in shared memory
/// section. If the data is placed in a shared section the lock can be placed
/// there as well. Malicious software can corrupt any shared memory section it
/// has access to.
pub struct Mutex<T> {
    id: UnsafeCell<usize>,
    inner: UnsafeCell<T>,
    lock: AtomicBool,
}

impl<T> Mutex<T> {
    pub const fn new(element: T) -> Self {
        Mutex {
            id: UnsafeCell::new(0),
            inner: UnsafeCell::new(element),
            lock: AtomicBool::new(false),
        }
    }

    /// Allocate an event ot the mutex.
    ///
    /// **Note:** The kernel must be initialized before calling this method.
    pub fn register(&self) -> Result<(),Error> {
        let id = syscall::event_register();
        if id == 0 {
            Err(Error::OutOfMemory)
        } else {
            // NOTE(unsafe): only called before the mutex is in use
            unsafe { self.id.get().write(id); }
            Ok(())
        }
    }

    /// Try to lock the mutex (non-blocking). Returns a [`MutexGuard`] or an
    /// error if the mutex is not available or poisoned.
    pub fn try_lock(&self) -> Result<MutexGuard<'_,T>, Error> {
        if self.raw_lock().is_ok() {
            Ok(MutexGuard::new(&self))
        } else {
            Err(Error::WouldBlock)
        }
    }

    /// Try to lock the mutex (blocking).Returns a [`MutexGuard`] or an
    /// error if the request timed out or the mutex was poisoned.
    ///
    /// **Note:** The timeout function is not implemented yet.
    pub fn lock(&self, timeout: u32) ->  Result<MutexGuard<'_,T>, Error> {
        if self.raw_lock().is_ok() {
            return Ok(MutexGuard::new(&self));
        } else {
            let id = unsafe { *self.id.get() };
            match syscall::event_await(id, timeout) {
                Ok(_) => {
                    self.raw_lock().ok();
                    Ok(MutexGuard::new(&self))
                },
                Err(event::Error::TimeOut) => Err(Error::TimeOut),
                Err(_) => Err(Error::Poisoned),
            }
        }
    }

    fn raw_lock(&self) -> Result<bool,bool> {
        self.lock.compare_exchange(false, true,
                                   Ordering::Acquire,
                                   Ordering::Acquire)
    }

    fn raw_unlock(&self) {
        self.lock.store(false, Ordering::Release);
        // NOTE(unsafe): `id` is not changed after startup
        syscall::event_fire(unsafe { *self.id.get() });
    }
}

unsafe impl<T> Sync for Mutex<T> {}

/// Scoped mutex (similar to [`std::sync::MutexGuard`](https://doc.rust-lang.org/std/sync/struct.MutexGuard.html))
pub struct MutexGuard<'a,T> {
    lock: &'a Mutex<T>,
}

impl<'a,T> MutexGuard<'a,T> {
    fn new(lock: &'a Mutex<T>) -> Self {
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
