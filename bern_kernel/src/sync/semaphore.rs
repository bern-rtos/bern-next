use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicUsize, Ordering};
use super::Error;
use crate::syscall;
use crate::sched::event;

pub struct Semaphore {
    id: UnsafeCell<usize>,
    permits: usize,
    permits_issued: AtomicUsize,
}

impl Semaphore {
    pub const fn new(permits: usize) -> Self {
        Semaphore {
            id: UnsafeCell::new(0),
            permits,
            permits_issued: AtomicUsize::new(0),
        }
    }

    pub fn register(&self) -> Result<(),Error> {
        let id = syscall::event_register();
        if id == 0 {
            Err(Error::OutOfMemory)
        } else {
            // NOTE(unsafe): only called before the semaphore is in use
            unsafe { self.id.get().write(id); }
            Ok(())
        }
    }

    pub fn try_acquire(&self) -> Result<SemaphorePermit<'_>, Error> {
        if self.raw_acquire().is_ok() {
            Ok(SemaphorePermit::new(&self))
        } else {
            Err(Error::WouldBlock)
        }
    }

    pub fn acquire(&self, timeout: u32) ->  Result<SemaphorePermit<'_>, Error> {
        if self.raw_acquire().is_ok() {
            return Ok(SemaphorePermit::new(&self));
        } else {
            let id = unsafe { *self.id.get() };
            match syscall::event_await(id, timeout) {
                Ok(_) => {
                    self.raw_acquire().ok();
                    Ok(SemaphorePermit::new(&self))
                },
                Err(event::Error::TimeOut) => Err(Error::TimeOut),
                Err(_) => Err(Error::Poisoned),
            }
        }
    }

    ///
    /// **Note:** This will return a false positive when `permits_issued` overflows
    fn raw_acquire(&self) -> Result<(), ()> {
        let permits = self.permits_issued.fetch_add(1, Ordering::Acquire);
        if permits >= self.permits {
            self.permits_issued.fetch_sub(1, Ordering::Release);
            Err(())
        } else {
            Ok(())
        }

    }

    fn raw_release(&self) {
        self.permits_issued.fetch_sub(1, Ordering::Release);
        // NOTE(unsafe): `id` is not changed after startup
        syscall::event_fire(unsafe { *self.id.get() });
    }
}

unsafe impl Sync for Semaphore {}


pub struct SemaphorePermit<'a> {
    semaphore: &'a Semaphore,
}

impl<'a> SemaphorePermit<'a> {
    fn new(semaphore: &'a Semaphore) -> Self {
        SemaphorePermit {
            semaphore,
        }
    }
}

impl<'a> Drop for SemaphorePermit<'a> {
    fn drop(&mut self) {
        self.semaphore.raw_release();
    }
}