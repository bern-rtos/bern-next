use core::mem::size_of;
use core::mem::size_of_val;
use super::task::Task;

/*
pub fn spawn<F>(closure: F)
    where F: 'static + Sync + FnMut() {
    //let size = size_of::<F>();
    let size = size_of_val(&closure);
    unsafe { asm!(
        "mov r1, r4",
        "mov r2, r5",
        "mov r0, 42", // service id
        "svc 0",
        in("r4") &closure as *const _ as usize,
        in("r5") size,
    )};
}*/

pub fn spawn(task: Task) {
    unsafe { asm!(
        "mov r1, r4",
        "svc 1",
        in("r4") &task as *const _ as usize,
    )};
}

pub fn delay(ms: u32) {
    unsafe { asm!(
        "mov r1, r4",
        "svc 2",
        in("r4") ms,
    )};
}

pub fn task_exit() {
    cortex_m::asm::bkpt();
}