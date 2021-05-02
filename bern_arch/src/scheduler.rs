pub trait IScheduler {
    fn trigger_context_switch();
    fn start_first_task(stack_ptr: *const usize) -> !;
}