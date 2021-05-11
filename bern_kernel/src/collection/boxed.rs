#![allow(unused)]

use core::ptr::NonNull;
use core::ops::{Deref, DerefMut};

#[derive(Debug)]
pub struct Box<T> {
    value: NonNull<T>,
}

impl<T> Box<T> {

    // todo: add ref to allocator to create and drop memory
    pub fn from_raw(pointer: NonNull<T>) -> Self {
        Box {
            value: pointer,
        }
    }

    pub fn into_nonnull(self) -> NonNull<T> {
        self.value
    }
}

impl<T> Deref for Box<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.value.as_ref() }
    }
}

impl<T> DerefMut for Box<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.value.as_mut() }
    }
}