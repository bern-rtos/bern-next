use core::ops::{DerefMut, Deref};
use crate::bern_arch::arch::memory_protection::Size;

#[derive(Copy, Clone)] // todo: remove
#[repr(C)]
pub struct Stack {
    bottom: *mut u8,
    pub ptr: *mut usize,
    size: Size,
}

impl Stack {
    pub fn new(stack: &mut [u8], size: Size) -> Self {
        Stack {
            bottom: stack.as_mut_ptr(),
            ptr: unsafe { stack.as_mut_ptr().offset(stack.len() as isize) } as *mut usize,
            size
        }
    }

    pub fn bottom_ptr(&self) -> *mut u8 {
        self.bottom
    }

    pub fn size(&self) -> Size {
        self.size
    }
}


// based on https://github.com/japaric/aligned/blob/master/src/lib.rs
#[repr(C)]
pub struct Aligned<A, T>
    where
        T: ?Sized,
{
    _alignment: [A; 0],
    value: T,
}

#[allow(non_snake_case)]
pub const fn Aligned<A, T>(value: T) -> Aligned<A, T> {
    Aligned {
        _alignment: [],
        value,
    }
}

impl<A, T> Deref for Aligned<A, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
impl<A, T> DerefMut for Aligned<A, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

// todo: enforce alignment and size restrictions
#[macro_export]
macro_rules! alloc_static_stack {
    ($size:tt) => {
        {
            #[link_section = ".task_stack"]
            static mut STACK: $crate::stack::Aligned<$crate::bern_arch::alignment_from_size!($size), [u8; $size]> =
                $crate::stack::Aligned([0; $size]);

            // this is unsound, because the same stack can 'allocated' multiple times
            unsafe { $crate::stack::Stack::new(&mut *STACK, $crate::bern_arch::size_from_raw!($size)) }
        }
    };
}