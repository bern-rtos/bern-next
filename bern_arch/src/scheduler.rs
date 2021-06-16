//! Scheduler.

/// Scheduler.
pub trait IScheduler {
    /// Init the stack of task.
    ///
    /// # Safety
    /// The stack must be large enough for the initial stack frame.
    unsafe fn init_task_stack(stack_ptr: *mut usize, entry: *const usize, arg: *const usize, exit: *const usize) -> *mut usize;
    /// Start the first task.
    fn start_first_task(stack_ptr: *const usize) -> !;
    /// Trigger context switch exception.
    fn trigger_context_switch();
}