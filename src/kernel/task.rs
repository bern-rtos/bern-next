use super::scheduler;
use super::scheduler::Scheduler;
use super::boxed::Box;
use core::ptr::NonNull;
use core::mem::{size_of, size_of_val, transmute_copy};
use core::ptr;
use core::borrow::BorrowMut;

#[derive(Debug)]
pub struct TaskError;

// todo: enforce alignment and size restrictions
// todo: add a stack section to memory
#[macro_export]
macro_rules! alloc_static_stack {
    ($size:expr) => {
        {
            #[link_section = ".taskstack"]
            static mut STACK: [u8; $size] = [0; $size]; // will not be initialized -> linker scritp
            unsafe{ // stack pattern for debugging
                for byte in STACK.iter_mut() {
                    *byte = 0xAA;
                }
            }
            unsafe { STACK.as_mut() }
        }
    };
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
    runnable_stack_ptr: *mut usize,
    next_wut: u64,
    stack_ptr: *mut usize,
    reg_psp: *mut usize,
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
        if alignment == 0 { // ensure that stack pointer is a least decreased by one
            alignment = 4; // todo: check that again
        } else {
            alignment += 4;
        }
        let proc_stack_pos = runnable_pos - alignment; // align to double word (ARM recommendation)
        let mut proc_sp = unsafe { stack.as_ptr().offset(proc_stack_pos as isize)} as *mut usize;

        let mut task = Task {
            runnable,
            runnable_stack_ptr: unsafe { stack.as_mut_ptr().offset(runnable_pos as isize) as *mut usize },
            next_wut: 0,
            stack_ptr: stack.as_mut_ptr() as *mut usize, // todo: replace with stack object
            reg_psp: proc_sp,

        };

        // create initial stack frame
        task.bootstrap_stack();

        Scheduler::add(task);
        // todo: task handle?
    }

    /// We need to set up the process stack before we can use it
    fn bootstrap_stack(&mut self) {
        unsafe {
            // todo: set exit function in LR
            *self.reg_psp = 0x01000000; // xPSR
            *self.reg_psp.offset(-1) = Self::entry as usize; // PC
            *self.reg_psp.offset(-7) = self.runnable_stack_ptr as usize; // R0 -> runnable_ptr of entry()
            self.reg_psp =  self.reg_psp.offset(-15); // exception frame (8 regs) + r4-r11 (8 regs)
        }
    }

    /// *Note* don't be fooled by the `&mut &mut` the first one is a reference
    /// and second one is part of the trait object type
    fn entry(runnable: &mut &mut (dyn FnMut() -> RunnableResult)) {
        (runnable)();
    }

    pub fn get_psp(&self) -> *mut usize {
        self.reg_psp
    }
    pub fn set_psp(&mut self, psp: *mut usize) {
        self.reg_psp = psp;
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