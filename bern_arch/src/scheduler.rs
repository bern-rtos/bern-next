pub trait IScheduler {
    unsafe fn init_task_stack(stack_ptr: *mut usize, entry: *const usize, arg: *const usize, exit: *const usize) -> *mut usize;
    fn start_first_task(stack_ptr: *const usize) -> !;
    fn trigger_context_switch();
}