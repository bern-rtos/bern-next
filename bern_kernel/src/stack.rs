use core::ops::{DerefMut, Deref};

#[derive(Copy, Clone)] // todo: remove
#[repr(C)]
pub struct Stack {
    top: *mut u8,
    len: usize,
    pub ptr: *mut usize,
}

impl Stack {
    pub fn new(stack: &mut [u8]) -> Self {
        Stack {
            top: stack.as_mut_ptr(),
            len: stack.len(),
            ptr: unsafe { stack.as_mut_ptr().offset(stack.len() as isize) } as *mut usize,
        }
    }

    pub fn top_ptr(&self) -> *mut u8 {
        self.top
    }

    pub fn len(&self) -> usize {
        self.len
    }
}


// based on https://github.com/japaric/aligned/blob/master/src/lib.rs
#[repr(align(512))]
pub struct A256B;

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
    ($size:expr) => {
        {
            #[link_section = ".task_stack"]
            static mut STACK: $crate::stack::Aligned<$crate::stack::A256B, [u8; $size]> =
                $crate::stack::Aligned([0; $size]);
            //static mut STACK: [u8; $size] = [0; $size]; // will not be initialized -> linker script
            /*unsafe{ // stack pattern for debugging
                for byte in *STACK.iter_mut() {
                    *byte = 0xAA;
                }
            }*/
            unsafe { $crate::stack::Stack::new(&mut *STACK) }
        }
    };
}