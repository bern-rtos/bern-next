
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

// todo: enforce alignment and size restrictions
#[macro_export]
macro_rules! alloc_static_stack {
    ($size:expr) => {
        {
            #[link_section = ".taskstack"]
            static mut STACK: [u8; $size] = [0; $size]; // will not be initialized -> linker script
            unsafe{ // stack pattern for debugging
                for byte in STACK.iter_mut() {
                    *byte = 0xAA;
                }
            }
            unsafe { $crate::stack::Stack::new(STACK.as_mut()) }
        }
    };
}