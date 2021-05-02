#![allow(unused)]

use super::scheduler;
use core::mem::{size_of, size_of_val};
use core::ptr;
use crate::syscall;
use crate::time;

#[derive(Debug)]
pub struct TaskError;

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
            unsafe { STACK.as_mut() }
        }
    };
}

#[derive(Copy, Clone)]
#[repr(u8)]
pub enum Transition {
    None,
    Suspending,
    Resuming,
    Terminating,
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
#[derive(Copy, Clone)]
pub struct Task
{
    transition: Transition,
    runnable_ptr: *mut usize,
    next_wut: u64,
    stack_top_ptr: *mut usize,
    stack_ptr: *mut usize,
}

impl Task
{
    pub fn runnable_ptr(&self) -> *const usize {
        self.runnable_ptr
    }

    pub fn stack_ptr(&self) -> *mut usize {
        self.stack_ptr
    }
    pub fn set_stack_ptr(&mut self, psp: *mut usize) {
        self.stack_ptr = psp;
    }

    pub fn next_wut(&self) -> u64 {
        self.next_wut
    }
    pub fn sleep(&mut self, ms: u32) {
        self.next_wut = time::tick() + u64::from(ms);
    }

    pub fn transition(&self) -> &Transition {
        &self.transition
    }
    pub fn set_transition(&mut self, transition: Transition) {
        self.transition = transition;
    }
}

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
    unsafe {
        ptr::write(stack.as_mut_ptr().offset(closure_pos as isize) as *mut _, closure);
        // create trait object pointing to closure on stack
        let mut closure_stacked = stack.as_mut_ptr().offset(closure_pos as isize) as *mut F;
        runnable = &mut (*closure_stacked);
    }

    // copy runnable trait object to stack
    let runnable_len = size_of_val(&runnable);
    let runnable_pos = stack_len - closure_len - runnable_len;
    unsafe {
        ptr::write(stack.as_mut_ptr().offset(runnable_pos as isize) as *mut _, runnable);
    }

    // set task stack pointer
    let mut alignment = unsafe { stack.as_mut_ptr().offset(runnable_pos as isize) as usize} % 8;
    let task_stack_pos = runnable_pos - alignment; // align to double word (ARM recommendation)
    let mut task_sp = unsafe { stack.as_ptr().offset(task_stack_pos as isize)} as *mut usize;

    let mut task = Task {
        transition: Transition::None,
        runnable_ptr: unsafe { stack.as_mut_ptr().offset(runnable_pos as isize) as *mut usize },
        next_wut: 0,
        stack_top_ptr: stack.as_mut_ptr() as *mut usize, // todo: replace with stack object
        stack_ptr: task_sp,

    };

    syscall::spawn(task);
    // todo: task handle?
}

/// *Note* don't be fooled by the `&mut &mut` the first one is a reference
/// and second one is part of the trait object type
pub fn entry(runnable: &mut &mut (dyn FnMut() -> RunnableResult)) {
    (runnable)();
}