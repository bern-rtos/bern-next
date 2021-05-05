#![allow(unused)]
use core::mem;
use core::ptr;
use core::ops::Deref;

use crate::scheduler;
use crate::syscall;
use crate::time;
use crate::stack::Stack;


#[derive(Debug)]
pub struct TaskError;

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
pub type RunnableResult = (); // todo: replace with '!' when possible

pub struct TaskBuilder {
    stack: Option<Stack>,
}

impl TaskBuilder {
    pub fn static_stack(&mut self, stack: Stack) -> &mut Self {
        self.stack = Some(stack);
        self
    }

    // todo: return result
    ///
    /// A task cannot access another tasks stack, thus all stack initialization
    /// must be handled via syscalls
    pub fn spawn<F>(&mut self, closure: F)
        where F: 'static + Sync + FnMut() -> RunnableResult
    {
        syscall::move_closure_to_stack(closure, self);
        let mut stack = match self.stack.as_mut() {
            Some(stack) => stack,
            None => panic!("todo: allocate stack"),
        };

        // create trait object pointing to closure on stack
        let mut runnable: &mut (dyn FnMut() -> RunnableResult);
        unsafe {
            runnable = &mut *(stack.ptr as *mut F);
        }

        syscall::task_spawn(self, &runnable);
    }

    // userland barrier ////////////////////////////////////////////////////////

    pub(crate) fn move_closure_to_stack(&mut self, closure: *const u8, size_bytes: usize) {
        // todo: check stack size otherwise this is pretty insecure
        let mut stack = match self.stack.as_mut() {
            Some(stack) => stack,
            None => panic!("todo: return error"),
        };

        unsafe {
            let mut ptr = stack.ptr as *mut u8;
            ptr = ptr.offset(-(size_bytes as isize));
            ptr::copy_nonoverlapping(closure, ptr, size_bytes);
            stack.ptr = ptr as *mut usize;
        }
    }

    pub(crate) fn build(&mut self, runnable: &&mut (dyn FnMut() -> RunnableResult)) {
        // todo: check stack size otherwise this is pretty insecure
        let mut stack = match self.stack.as_mut() {
            Some(stack) => stack,
            None => panic!("todo: return error"),
        };
        let mut ptr = stack.ptr as *mut u8;

        // copy runnable trait object to stack
        let runnable_len = mem::size_of_val(runnable);
        unsafe {
            ptr = ptr.offset(-(runnable_len as isize));
            ptr::write(ptr as *mut _, runnable.deref());
        }
        let runnable_ptr = ptr as *mut usize;

        // align task stack to double word
        let mut alignment = ptr as usize % 8;
        unsafe {
            ptr = ptr.offset(-(alignment as isize));
        }
        stack.ptr = ptr as *mut usize;

        let mut task = Task {
            transition: Transition::None,
            runnable_ptr,
            next_wut: 0,
            stack: self.stack.take().unwrap(),
        };
        scheduler::add(task)
    }
}


// todo: manage lifetime of stack & runnable
#[derive(Copy, Clone)]
pub struct Task {
    transition: Transition,
    runnable_ptr: *mut usize,
    next_wut: u64,
    stack: Stack,
}

impl Task {
    pub fn new() -> TaskBuilder {
        TaskBuilder {
            stack: None,
        }
    }

    // userland barrier ////////////////////////////////////////////////////////

    pub fn runnable_ptr(&self) -> *const usize {
        self.runnable_ptr
    }

    pub fn stack_ptr(&self) -> *mut usize {
        self.stack.ptr
    }
    pub fn set_stack_ptr(&mut self, psp: *mut usize) {
        self.stack.ptr = psp;
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

/// *Note* don't be fooled by the `&mut &mut` the first one is a reference
/// and second one is part of the trait object type
pub fn entry(runnable: &mut &mut (dyn FnMut() -> RunnableResult)) {
    (runnable)();
}

#[cfg(all(test, not(target_os = "none")))]
mod tests {
    use super::*;
    use bern_arch::mock::MockArch;

    #[test]
    fn example_test() {
        let mut call_index = 0;
        let ctx = MockArch::syscall_context();
        ctx.expect()
            .times(2)
            .returning(move |id, arg0, arg1, arg2| {
                match call_index {
                    0 => {
                        assert_eq!(id, syscall::Service::MoveClosureToStack.service_id());
                    },
                    1 => {
                        assert_eq!(id, syscall::Service::TaskSpawn.service_id());
                    },
                    _ => (),
                }
                call_index += 1;
                0
            });

        Task::new()
            .static_stack(crate::alloc_static_stack!(512))
            .spawn(move || {
                loop { }
            });

        ctx.checkpoint();
    }
}