use core::cell::UnsafeCell;

use bern_arch::ISync;
use bern_arch::arch::Arch;

pub struct CriticalMutex<T> {
    inner: UnsafeCell<T>,
}

impl<T> CriticalMutex<T> {
    pub const fn new(element: T) -> Self {
        CriticalMutex {
            inner: UnsafeCell::new(element),
        }
    }

    pub fn lock(&self) -> &mut T {
        Arch::disable_interrupts(0);
        unsafe {
            &mut *self.inner.get()
        }
    }

    // this is a temporary solution -> stupid because one can release the lock without having the data
    pub fn release(&self) {
        Arch::enable_interrupts();
    }
}

unsafe impl<T> Sync for CriticalMutex<T> { }