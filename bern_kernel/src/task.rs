use super::scheduler;
use super::scheduler::Scheduler;
use core::mem::{size_of, size_of_val};
use core::ptr;

#[derive(Debug)]
pub struct TaskError;

// todo: enforce alignment and size restrictions
// todo: add a stack section to memory
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
            unsafe { STACK.as_mut() }
        }
    };
}

/* adapted from cortex-m crate */

/// CPU registers pushed/popped by the hardware
#[repr(C)]
pub struct StackFrameException {
    /// (General purpose) Register 0
    pub r0: u32,
    /// (General purpose) Register 1
    pub r1: u32,
    /// (General purpose) Register 2
    pub r2: u32,
    /// (General purpose) Register 3
    pub r3: u32,
    /// (General purpose) Register 12
    pub r12: u32,
    /// Linker Register
    pub lr: u32,
    /// Program Counter
    pub pc: u32,
    /// Program Status Register
    pub xpsr: u32,
}

/// CPU registers the software must push/pop to/from the stack
#[repr(C)]
pub struct StackFrameExtension {
    /// (General purpose) Register 4
    pub r4: u32,
    /// (General purpose) Register 5
    pub r5: u32,
    /// (General purpose) Register 6
    pub r6: u32,
    /// (General purpose) Register 7
    pub r7: u32,
    /// (General purpose) Register 8
    pub r8: u32,
    /// (General purpose) Register 9
    pub r9: u32,
    /// (General purpose) Register 10
    pub r10: u32,
    /// (General purpose) Register 11
    pub r11: u32,
}

/// CPU registers used by the floating point unit
#[repr(C)]
pub struct StackFrameFpu {
}

/// Issue with closures and static tasks
/// ------------------------------------
/// Every closure has its own anonymous type. A closure can only be stored in a
/// generic struct. The task object stored in the task "list" (array) must all
/// have the same size -> not generic. Thus, the closure can only be referenced
/// as trait object. But need to force the closure to be static, so our
/// reference can be as well. A static closure is not possible, as every static
/// needs a specified type.
/// To overcome the issue of storing a closure into a static task we need to
/// **copy** it into a static stack. Access to the closure is provided via a
/// closure trait object, which now references a static object which cannot go
/// out of scope.

//type RunnableResult = Result<(), TaskError>;
type RunnableResult = (); // todo: replace with '!' when possible

// todo: manage lifetime of stack & runnable
pub struct Task<'a>
{
    runnable: &'a mut (dyn FnMut() -> RunnableResult + 'static), // todo: remove
    runnable_ptr: *mut usize,
    next_wut: u64,
    stack_top_ptr: *mut usize,
    stack_ptr: *mut usize,
}

impl<'a> Task<'a>
{
    // todo: replace stack with own type
    // todo: prevent a *static* task from being spawned twice (stack)
    // todo: clean up the mess
    pub fn spawn<F>(closure: F, stack: &mut [u8])
        where F: 'static + Sync + FnMut() -> RunnableResult
    {
        let stack_len = stack.len();

        // copy closure to stack
        let closure_len = size_of::<F>();
        let closure_pos = stack_len - closure_len;
        let mut runnable: &mut (dyn FnMut() -> RunnableResult + 'static);
        let mut runnable2: &mut (dyn FnMut() -> RunnableResult + 'static);
        unsafe {
            ptr::write(stack.as_mut_ptr().offset(closure_pos as isize) as *mut _, closure);
            // create trait object pointing to closure on stack
            let mut closure_stacked = stack.as_mut_ptr().offset(closure_pos as isize) as *mut F;
            runnable = &mut (*closure_stacked);
            runnable2 = &mut (*closure_stacked);
        }

        // copy runnable trait object to stack
        let runnable_len = size_of_val(&runnable);
        let runnable_pos = stack_len - closure_len - runnable_len;
        unsafe {
            ptr::write(stack.as_mut_ptr().offset(runnable_pos as isize) as *mut _, runnable2);
        }

        // set task stack pointer
        let mut alignment = unsafe { stack.as_mut_ptr().offset(runnable_pos as isize) as usize} % 8;
        let proc_stack_pos = runnable_pos - alignment; // align to double word (ARM recommendation)
        let mut proc_sp = unsafe { stack.as_ptr().offset(proc_stack_pos as isize)} as *mut usize;

        let mut task = Task {
            runnable,
            runnable_ptr: unsafe { stack.as_mut_ptr().offset(runnable_pos as isize) as *mut usize },
            next_wut: 0,
            stack_top_ptr: stack.as_mut_ptr() as *mut usize, // todo: replace with stack object
            stack_ptr: proc_sp,

        };

        task.init_stack_frame();
        Scheduler::add(task);
        // todo: task handle?
    }

    /// We need to set up the task stack before we can use it
    fn init_stack_frame(&mut self) {
        let stack_frame = StackFrameException {
            r0: self.runnable_ptr as u32,
            r1: 0,
            r2: 0,
            r3: 0,
            r12: 0,
            lr: 0xaaaaaaaa, // this will hardfault for now
            pc: Self::entry as u32,
            xpsr: 0x01000000,
        };
        let stack_frame_offset = size_of::<StackFrameException>() / size_of::<usize>();
        unsafe {
            ptr::copy_nonoverlapping(
                &stack_frame,
                self.stack_ptr.offset(-(stack_frame_offset as isize)) as *mut _,
                1
            );
            let stack_ptr_offset =
                (size_of::<StackFrameException>() + size_of::<StackFrameExtension>()) / size_of::<usize>();
            self.stack_ptr =  self.stack_ptr.offset(-(stack_ptr_offset as isize));
        }
    }

    /// *Note* don't be fooled by the `&mut &mut` the first one is a reference
    /// and second one is part of the trait object type
    fn entry(runnable: &mut &mut (dyn FnMut() -> RunnableResult)) {
        (runnable)();
    }

    pub fn get_psp(&self) -> *mut usize {
        self.stack_ptr
    }
    pub fn set_psp(&mut self, psp: *mut usize) {
        self.stack_ptr = psp;
    }

    pub fn run(&mut self) -> RunnableResult {
        (self.runnable)()
    }

    pub fn get_next_wut(&self) -> u64 {
        self.next_wut
    }

    pub fn delay(&mut self, ms: u32) {
        self.next_wut = scheduler::get_tick() + u64::from(ms);
    }
}

