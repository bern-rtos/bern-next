use crate::task::Task;

pub trait Syscall {
    fn spawn(task: Task);
    fn sleep(ms: u32);
    fn task_exit();
}